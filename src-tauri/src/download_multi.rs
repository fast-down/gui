use crate::{
    download_error::DownloadError,
    event::{DownloadItemId, Event},
    format_progress::fmt_progress,
    puller::FastDownPuller,
};
use fast_down::{
    MergeProgress, ProgressEntry, Total,
    file::{FilePusherError, RandFilePusherMmap, RandFilePusherStd},
    multi,
};
use std::{
    collections::HashMap,
    num::NonZero,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Listener, http::HeaderMap, ipc::Channel};
use tokio::fs::OpenOptions;
use url::Url;

pub enum WriteMethod {
    Mmap,
    Std,
}

pub enum Pusher {
    Mmap(RandFilePusherMmap),
    Std(RandFilePusherStd),
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    pub url: String,
    pub file_path: String,
    pub file_size: u64,
    pub threads: usize,
    pub write_buffer_size: usize,
    pub write_queue_cap: usize,
    pub min_chunk_size: u64,
    pub retry_gap: u64,
    pub download_chunks: Vec<(u64, u64)>,
    pub headers: HashMap<String, String>,
    pub multiplexing: bool,
    pub accept_invalid_certs: bool,
    pub accept_invalid_hostnames: bool,
    pub proxy: Option<String>,
    pub write_method: String,
    pub init_progress: Vec<Vec<(u64, u64)>>,
    pub init_downloaded: u64,
}

#[tauri::command]
pub async fn download_multi(
    app: AppHandle,
    options: DownloadOptions,
    tx: Channel<Event>,
) -> Result<(), DownloadError> {
    let url = Url::parse(&options.url)?;
    let headers = options
        .headers
        .into_iter()
        .filter_map(|(k, v)| Some((k.parse().ok()?, v.parse().ok()?)))
        .collect::<HeaderMap>();
    let retry_gap = Duration::from_millis(options.retry_gap);
    let download_chunks = options
        .download_chunks
        .into_iter()
        .map(|(start, end)| start..end)
        .collect();
    let write_method = match options.write_method.as_str() {
        "mmap" => WriteMethod::Mmap,
        "std" => WriteMethod::Std,
        _ => WriteMethod::Std,
    };
    let puller = FastDownPuller::new(
        url,
        headers,
        options.proxy,
        options.multiplexing,
        options.accept_invalid_certs,
        options.accept_invalid_hostnames,
    )?;
    let pusher = match write_method {
        WriteMethod::Mmap => {
            let res = RandFilePusherMmap::new(
                &options.file_path,
                options.file_size,
                options.write_buffer_size,
            )
            .await;
            match res {
                Ok(pusher) => Pusher::Mmap(pusher),
                Err(_) => {
                    let file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(&options.file_path)
                        .await
                        .map_err(FilePusherError::from)?;
                    Pusher::Std(
                        RandFilePusherStd::new(file, options.file_size, options.write_buffer_size)
                            .await?,
                    )
                }
            }
        }
        WriteMethod::Std => {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .open(&options.file_path)
                .await
                .map_err(FilePusherError::from)?;
            Pusher::Std(
                RandFilePusherStd::new(file, options.file_size, options.write_buffer_size).await?,
            )
        }
    };
    let download_options = multi::DownloadOptions {
        download_chunks,
        concurrent: NonZero::new(options.threads).unwrap_or(NonZero::new(1).unwrap()),
        retry_gap,
        push_queue_cap: options.write_queue_cap,
        min_chunk_size: NonZero::new(options.min_chunk_size)
            .unwrap_or(NonZero::new(8 * 1024).unwrap()),
    };
    let res = match pusher {
        Pusher::Mmap(pusher) => multi::download_multi(puller, pusher, download_options).await,
        Pusher::Std(pusher) => multi::download_multi(puller, pusher, download_options).await,
    };
    let res_clone = res.clone();
    let handle = app.clone();
    let event_id = app.listen("stop-download", move |event| {
        if let Ok(payload) = serde_json::from_str::<DownloadItemId>(event.payload())
            && payload.file_path == options.file_path
        {
            res_clone.abort();
            handle.unlisten(event.id());
        }
    });
    let handle = app.clone();
    tokio::spawn(async move {
        let mut pull_progress: Vec<Vec<ProgressEntry>> = options
            .init_progress
            .iter()
            .map(|v| v.iter().map(|e| e.0..e.1).collect())
            .collect();
        if pull_progress.len() < options.threads {
            pull_progress.resize(options.threads, Vec::new());
        }
        let mut push_progress = pull_progress.clone();
        let mut last_pull_update_time = Instant::now();
        let mut last_push_update_time = Instant::now();
        let mut downloaded = options.init_downloaded;
        const UPDATE_INTERVAL: u128 = 100;
        while let Ok(e) = res.event_chain.recv().await {
            match e {
                fast_down::Event::PullProgress(id, range) => {
                    downloaded += range.total();
                    pull_progress[id].merge_progress(range);
                    if last_pull_update_time.elapsed().as_millis() > UPDATE_INTERVAL {
                        last_pull_update_time = Instant::now();
                        tx.send(Event::PullProgress(
                            fmt_progress(&pull_progress),
                            downloaded,
                        ))
                        .unwrap();
                    }
                }
                fast_down::Event::PushProgress(id, range) => {
                    push_progress[id].merge_progress(range);
                    if last_push_update_time.elapsed().as_millis() > UPDATE_INTERVAL {
                        last_push_update_time = Instant::now();
                        tx.send(Event::PushProgress(fmt_progress(&push_progress)))
                            .unwrap();
                    }
                }
                _ => tx.send(e.into()).unwrap(),
            };
        }
        tx.send(Event::PullProgress(
            fmt_progress(&pull_progress),
            downloaded,
        ))
        .unwrap();
        tx.send(Event::PushProgress(fmt_progress(&push_progress)))
            .unwrap();
        res.join().await.unwrap();
        handle.unlisten(event_id);
        tx.send(Event::AllFinished).unwrap();
    });
    Ok(())
}
