use crate::{
    download_error::DownloadError,
    event::{DownloadItemId, Event},
    puller::{FastDownPuller, FastDownPullerOptions},
};
use fast_down::{
    FileId, Total,
    file::{FilePusherError, SeqFilePusher},
    single,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Listener, http::HeaderMap, ipc::Channel};
use tokio::fs::OpenOptions;
use url::Url;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    pub url: String,
    pub file_path: String,
    pub write_buffer_size: usize,
    pub write_queue_cap: usize,
    pub retry_gap: u64,
    pub headers: HashMap<String, String>,
    pub multiplexing: bool,
    pub accept_invalid_certs: bool,
    pub accept_invalid_hostnames: bool,
    pub proxy: String,
    pub etag: Option<Arc<str>>,
    pub last_modified: Option<Arc<str>>,
}

#[tauri::command]
pub async fn download_single(
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
    let puller = FastDownPuller::new(FastDownPullerOptions {
        url,
        headers,
        proxy: &options.proxy,
        multiplexing: options.multiplexing,
        accept_invalid_certs: options.accept_invalid_certs,
        accept_invalid_hostnames: options.accept_invalid_hostnames,
        file_id: FileId {
            etag: options.etag,
            last_modified: options.last_modified,
        },
        resp: None,
    })?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&options.file_path)
        .await
        .map_err(FilePusherError::from)?;
    let pusher = SeqFilePusher::new(file, options.write_buffer_size);
    let res = single::download_single(
        puller,
        pusher,
        single::DownloadOptions {
            retry_gap,
            push_queue_cap: options.write_queue_cap,
        },
    )
    .await;
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
        let mut last_pull_update_time = Instant::now();
        let mut last_push_update_time = Instant::now();
        let mut downloaded = 0;
        let mut write = 0;
        const UPDATE_INTERVAL: u128 = 100;
        while let Ok(e) = res.event_chain.recv().await {
            match e {
                fast_down::Event::PullProgress(_, range) => {
                    downloaded += range.total();
                    if last_pull_update_time.elapsed().as_millis() > UPDATE_INTERVAL {
                        last_pull_update_time = Instant::now();
                        tx.send(Event::PullProgress(vec![vec![(0, downloaded)]], downloaded))
                            .unwrap();
                    }
                }
                fast_down::Event::PushProgress(_, range) => {
                    write += range.total();
                    if last_push_update_time.elapsed().as_millis() > UPDATE_INTERVAL {
                        last_push_update_time = Instant::now();
                        tx.send(Event::PushProgress(vec![vec![(0, write)]]))
                            .unwrap();
                    }
                }
                _ => tx.send(e.into()).unwrap(),
            };
        }
        tx.send(Event::PullProgress(vec![vec![(0, downloaded)]], downloaded))
            .unwrap();
        tx.send(Event::PushProgress(vec![vec![(0, write)]]))
            .unwrap();
        res.join().await.unwrap();
        handle.unlisten(event_id);
        tx.send(Event::AllFinished).unwrap();
    });
    Ok(())
}
