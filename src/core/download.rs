use crate::{
    fmt::{format_size, format_time},
    persist::{self, DatabaseEntry, Status},
    ui::DownloadConfig,
    utils::{auto_ext, sanitize},
};
use fast_down_ffi::{Event, Total, create_channel, prefetch, unique_path::gen_unique_path};
use file_alloc::FileAlloc;
use parking_lot::Mutex;
use slint::SharedString;
use std::{
    ops::Range,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::fs::{self, OpenOptions};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use url::Url;

pub enum DownloadEvent {
    Info(Box<DatabaseEntry>),
    Progress(ProgressInfo),
    Flushing,
    FlushError(SharedString),
    End { is_cancelled: bool },
}

pub struct ProgressInfo {
    pub downloaded: SharedString,
    pub speed: SharedString,
    pub avg_speed: SharedString,
    pub time: SharedString,
    pub remaining_time: SharedString,
    pub remaining_size: SharedString,
    pub percentage: SharedString,
    pub elapsed: Duration,
    pub progress: Vec<Range<u64>>,
}

pub async fn download(
    url: Url,
    config: &DownloadConfig,
    cancel_token: CancellationToken,
    mut entry: Option<DatabaseEntry>,
    mut on_event: impl FnMut(DownloadEvent) + Send + Sync + 'static,
) -> color_eyre::Result<()> {
    info!(url = url.as_str(), config = ?config, "启动下载");
    let result = async {
        let file_exists = matches!(&entry, Some(entry) if fs::try_exists(&entry.file_path).await.unwrap_or(false));
        if !file_exists {
            entry = None
        }
        let config: persist::DownloadConfig = config.into();
        let progress = Arc::new(Mutex::new(
            entry
                .as_ref()
                .map(|e| e.progress.clone())
                .unwrap_or_default(),
        ));
        let pre_allocate = config.pre_allocate;
        let download_config = fast_down_ffi::Config {
            retry_times: config.retry_times,
            threads: config.threads,
            proxy: config.proxy.clone(),
            headers: config.headers.clone(),
            min_chunk_size: config.min_chunk_size,
            write_buffer_size: config.write_buffer_size,
            write_queue_cap: config.write_queue_cap,
            retry_gap: config.retry_gap,
            pull_timeout: config.pull_timeout,
            accept_invalid_certs: config.accept_invalid_certs,
            accept_invalid_hostnames: config.accept_invalid_hostnames,
            write_method: config.write_method.clone(),
            local_address: config.local_address.clone(),
            max_speculative: config.max_speculative,
            downloaded_chunk: progress.clone(),
            chunk_window: config.chunk_window,
        };
        let elapsed = entry.as_ref().map(|e| e.elapsed).unwrap_or_default();
        let (tx, rx) = create_channel();
        let task = prefetch(url.clone(), download_config, tx).await?;
        info!(info = ?task.info, "获取元数据成功");
        let total_size = task.info.size;
        let (save_path, entry) = if let Some(entry) = entry
            && fs::try_exists(&entry.file_path).await.unwrap_or(false)
        {
            (entry.file_path.clone(), entry)
        } else {
            let file_name = sanitize(
                auto_ext(&task.info.raw_name, task.info.content_type.as_deref()),
                248,
            );
            let save_dir = soft_canonicalize::soft_canonicalize(
                if config.save_dir.to_string_lossy().is_empty() {
                    dirs::download_dir().unwrap_or_default()
                } else {
                    config.save_dir.clone()
                },
            )?;
            let _ = fs::create_dir_all(&save_dir).await;
            let save_path = gen_unique_path(&save_dir.join(&file_name)).await?;
            let file_name = save_path.file_name().unwrap().to_string_lossy().to_string();
            (
                save_path.clone(),
                DatabaseEntry {
                    file_name,
                    file_path: save_path,
                    file_size: total_size,
                    file_id: task.info.file_id.clone(),
                    progress: Vec::new(),
                    elapsed: Duration::ZERO,
                    url,
                    config,
                    status: Status::Paused,
                },
            )
        };
        on_event(DownloadEvent::Info(Box::new(entry)));
        if pre_allocate && total_size > 1024 * 1024 && progress.lock().is_empty() {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .open(&save_path)
                .await?;
            file.allocate(total_size).await?;
        }
        Ok::<_, color_eyre::Report>((
            task,
            save_path,
            cancel_token.clone(),
            elapsed,
            total_size,
            rx,
        ))
    };
    let (task, save_path, cancel_token, elapsed, total_size, rx) = tokio::select! {
        _ = cancel_token.cancelled() => {
            on_event(DownloadEvent::End { is_cancelled: true });
            return Ok(());
        },
        res = result => res?,
    };
    tokio::pin! {
        let fut = task.start(save_path, cancel_token.clone());
    };

    let progress = &task.config.downloaded_chunk;
    let mut smoothed_speed = 0.;
    let alpha = 0.3;
    let mut last_bytes = progress.lock().total();
    let mut last_update = Instant::now();
    let mut start = last_update - elapsed;

    macro_rules! update_progress {
        ($now:expr, $elapsed:expr, $total_elapsed:expr) => {{
            let downloaded = progress.lock().total();
            let bytes_diff = downloaded - last_bytes;
            let instant_speed = bytes_diff as f64 / $elapsed;
            smoothed_speed = if smoothed_speed == 0. {
                instant_speed
            } else {
                alpha * instant_speed + (1.0 - alpha) * smoothed_speed
            };
            let avg_speed = downloaded as f64 / $total_elapsed.as_secs_f64();
            let remaining_size = total_size.saturating_sub(downloaded);
            let remaining_time = remaining_size as f64 / smoothed_speed;
            let percentage = format!("{:.2}%", downloaded as f64 / total_size as f64 * 100.0);
            on_event(DownloadEvent::Progress(ProgressInfo {
                downloaded: format_size(downloaded as f64).into(),
                speed: format!("{}/s", format_size(smoothed_speed)).into(),
                avg_speed: format!("{}/s", format_size(avg_speed)).into(),
                time: format_time($total_elapsed.as_secs()).into(),
                remaining_time: format_time(remaining_time as u64).into(),
                remaining_size: format_size(remaining_size as f64).into(),
                percentage: percentage.into(),
                elapsed: $total_elapsed,
                progress: progress.lock().clone(),
            }));
            downloaded
        }};
    }

    let mut is_first = true;
    loop {
        tokio::select! {
            res = &mut fut => {
                res?;
                break;
            }
            event = rx.recv() => {
                let e = match event {
                    Ok(e) => e,
                    Err(_) => break,
                };
                match e {
                    Event::PrefetchError(e) => error!(err = e, "获取元数据失败"),
                    Event::Pulling(id) => info!(id = id, "开始下载"),
                    Event::PullProgress(_, _) => {}
                    Event::PullError(id, e) => warn!(err = e, id = id, "下载数据出错"),
                    Event::PullTimeout(id) => warn!("拉取数据超时 {id}"),
                    Event::Pushing(_, _) => {},
                    Event::PushError(id, r, e) => error!(err = e, id = id, start = r.start, end = r.end, "写入数据出错"),
                    Event::Flushing => {
                        info!("开始刷写磁盘");
                        if is_first {
                            is_first = false;
                            let now = Instant::now();
                            let elapsed = (now - last_update).as_secs_f64();
                            let total_elapsed = now - start;
                            update_progress!(now, elapsed, total_elapsed);
                        }
                        on_event(DownloadEvent::Flushing);
                    },
                    Event::FlushError(e) => {
                        error!(err = e, "磁盘刷写失败");
                        on_event(DownloadEvent::FlushError(e.into()));
                    },
                    Event::Finished(id) => info!(id = id, "下载完成"),
                    Event::PushProgress(_, p) => {
                        if p.start == 0 {
                            smoothed_speed = 0.;
                            last_update = Instant::now();
                            start = last_update;
                            last_bytes = 0;
                        }
                        let now = Instant::now();
                        let elapsed = (now - last_update).as_secs_f64();
                        let total_elapsed = now - start;
                        if elapsed > 1. {
                            last_bytes = update_progress!(now, elapsed, total_elapsed);
                            last_update = now;
                        }
                    }
                }
            }
        }
    }
    on_event(DownloadEvent::End {
        is_cancelled: cancel_token.is_cancelled(),
    });
    Ok(())
}
