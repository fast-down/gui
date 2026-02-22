use crate::{
    fmt::{format_size, format_time},
    persist::{DatabaseEntry, Status},
    ui::Config,
    utils::{parse_header_headermap, sanitize},
};
#[cfg(target_pointer_width = "64")]
use fast_down::file::MmapFilePusher;
use fast_down::{
    BoxPusher, Event, Merge, Total,
    file::FilePusher,
    http::Prefetch,
    invert,
    multi::{self, download_multi},
    single::{self, download_single},
    utils::{FastDownPuller, FastDownPullerOptions, build_client, gen_unique_path},
};
use parking_lot::Mutex;
use slint::SharedString;
use std::{net::IpAddr, ops::Range, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    fs::{self, OpenOptions},
    time::Instant,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use url::Url;

pub enum DownloadEvent {
    Info(Box<DatabaseEntry>),
    Progress(ProgressInfo),
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
    config: &Config,
    cancel_token: CancellationToken,
    entry: Option<DatabaseEntry>,
    mut on_event: impl FnMut(DownloadEvent) + Send + Sync,
) -> color_eyre::Result<()> {
    let result = async {
        let headers = Arc::new(parse_header_headermap(&config.headers));
        let proxy = match config.proxy.trim() {
            "" => None,
            "null" => Some(""),
            proxy => Some(proxy),
        };
        let local_addr: Arc<[IpAddr]> = config
            .ips
            .lines()
            .filter_map(|ip| ip.trim().parse().ok())
            .collect();
        let accept_invalid_certs = config.accept_invalid_certs;
        let accept_invalid_hostnames = config.accept_invalid_hostnames;
        let client = build_client(
            &headers,
            proxy,
            accept_invalid_certs,
            accept_invalid_hostnames,
            local_addr.first().cloned(),
        )?;
        let mut count = 0;
        let (info, resp) = loop {
            match client.prefetch(url.clone()).await {
                Ok(t) => break t,
                Err((e, t)) => {
                    count += 1;
                    if count > 10 {
                        color_eyre::eyre::bail!("获取元数据失败: {:?}", e);
                    }
                    error!(err = ?e, "获取元数据失败");
                    tokio::time::sleep(t.unwrap_or(Duration::from_millis(500))).await;
                }
            }
        };
        let total_size = info.size;
        let (save_path, entry) = if let Some(entry) = entry
            && fs::try_exists(&entry.file_path).await.unwrap_or(false)
        {
            (entry.file_path.clone(), entry)
        } else {
            let file_name = sanitize(info.raw_name, 248);
            let save_dir = soft_canonicalize::soft_canonicalize(if config.save_dir.is_empty() {
                dirs::download_dir().unwrap_or_default()
            } else {
                PathBuf::from(&config.save_dir)
            })?;
            let _ = fs::create_dir_all(&save_dir).await;
            let save_path = gen_unique_path(&save_dir.join(&file_name)).await?;
            let file_name = save_path.file_name().unwrap().to_string_lossy().to_string();
            (
                save_path.clone(),
                DatabaseEntry {
                    file_name,
                    file_path: save_path,
                    file_size: total_size,
                    file_id: info.file_id.clone(),
                    progress: Vec::new(),
                    elapsed: Duration::ZERO,
                    url,
                    config: config.into(),
                    status: Status::Paused,
                },
            )
        };
        let progress = entry.progress.clone();
        let elapsed = entry.elapsed;
        on_event(DownloadEvent::Info(Box::new(entry)));
        let puller = FastDownPuller::new(FastDownPullerOptions {
            url: info.final_url,
            headers,
            proxy,
            available_ips: local_addr,
            accept_invalid_certs,
            accept_invalid_hostnames,
            file_id: info.file_id,
            resp: Some(Arc::new(Mutex::new(Some(resp)))),
        })?;
        let threads = if info.fast_download {
            config.threads.max(1)
        } else {
            1
        } as usize;
        let get_std_pusher = async {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .truncate(false)
                .open(&save_path)
                .await?;
            let file_pusher =
                FilePusher::new(file, total_size, config.write_buffer_size as usize).await?;
            Ok::<_, color_eyre::Report>(BoxPusher::new(file_pusher))
        };
        let retry_gap = Duration::from_millis(config.retry_gap_ms as u64);
        let result = if info.fast_download {
            #[cfg(target_pointer_width = "64")]
            let pusher = if config.write_method == 0 {
                BoxPusher::new(MmapFilePusher::new(&save_path, total_size).await?)
            } else {
                get_std_pusher.await?
            };
            #[cfg(not(target_pointer_width = "64"))]
            let pusher = get_std_pusher.await?;
            download_multi(
                puller,
                pusher,
                multi::DownloadOptions {
                    download_chunks: invert(progress.iter().cloned(), total_size, 8 * 1024),
                    retry_gap,
                    concurrent: threads,
                    pull_timeout: Duration::from_millis(config.pull_timeout_ms as u64),
                    push_queue_cap: config.write_queue_cap as usize,
                    min_chunk_size: config.min_chunk_size as u64,
                    max_speculative: config.max_speculative as usize,
                },
            )
        } else {
            let pusher = get_std_pusher.await?;
            download_single(
                puller,
                pusher,
                single::DownloadOptions {
                    retry_gap,
                    push_queue_cap: config.write_queue_cap as usize,
                },
            )
        };
        Ok::<_, color_eyre::Report>((result, total_size, progress, elapsed))
    };
    let (result, total_size, mut progress, elapsed) = tokio::select! {
        _ = cancel_token.cancelled() => {
            on_event(DownloadEvent::End { is_cancelled: true });
            return Ok(());
        },
        res = result => res?,
    };
    let cancel_handle = tokio::spawn({
        let result = result.clone();
        let cancel_token = cancel_token.clone();
        async move {
            cancel_token.cancelled().await;
            result.abort();
        }
    });

    let mut smoothed_speed = 0.;
    let alpha = 0.3;
    let mut last_bytes = progress.total();
    let mut last_update = Instant::now();
    let mut start = last_update - elapsed;

    macro_rules! update_progress {
        ($now:expr, $elapsed:expr, $total_elapsed:expr) => {{
            let downloaded = progress.total();
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
                progress: progress.clone(),
            }));
            downloaded
        }};
    }
    while let Ok(e) = result.event_chain.recv().await {
        match e {
            Event::FlushError(e) => error!(err = ?e, "磁盘刷写失败"),
            Event::PullError(id, e) => warn!(err = ?e, id = id, "下载数据出错"),
            Event::PushError(id, e) => error!(err = ?e, id = id, "写入数据出错"),
            Event::Pulling(id) => info!(id = id, "开始下载"),
            Event::PullProgress(_, _) => {}
            Event::Finished(id) => info!(id = id, "下载完成"),
            Event::PushProgress(_, p) => {
                if p.start == 0 {
                    progress.clear();
                    smoothed_speed = 0.;
                    last_update = Instant::now();
                    start = last_update;
                    last_bytes = 0;
                }
                progress.merge_progress(p);
                let now = Instant::now();
                let elapsed = (now - last_update).as_secs_f64();
                let total_elapsed = now - start;
                if elapsed > 1. {
                    last_bytes = update_progress!(now, elapsed, total_elapsed);
                    last_update = now;
                }
            }
            Event::PullTimeout(id) => warn!("拉取数据超时 {id}"),
        }
    }
    result.join().await?;
    cancel_handle.abort();
    let now = Instant::now();
    let elapsed = (now - last_update).as_secs_f64();
    let total_elapsed = now - start;
    update_progress!(now, elapsed, total_elapsed);
    on_event(DownloadEvent::End {
        is_cancelled: cancel_token.is_cancelled(),
    });
    Ok(())
}
