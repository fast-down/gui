use crate::{
    event::{Event, StopEvent},
    puller::FastDownPuller,
};
use fast_pull::{
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
}

#[tauri::command]
pub async fn download_multi(
    app: AppHandle,
    options: DownloadOptions,
    tx: Channel<Event>,
) -> Result<(), Error> {
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
                Err(e) => {
                    dbg!(e);
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
                Ok(pusher) => Pusher::Mmap(pusher),
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
    let res = match pusher {
        Pusher::Mmap(pusher) => {
            multi::download_multi(
                puller,
                pusher,
                multi::DownloadOptions {
                    download_chunks,
                    concurrent: NonZero::new(options.threads).unwrap_or(NonZero::new(1).unwrap()),
                    retry_gap,
                    push_queue_cap: options.write_queue_cap,
                    min_chunk_size: NonZero::new(options.min_chunk_size)
                        .unwrap_or(NonZero::new(8 * 1024).unwrap()),
                },
            )
            .await
        }
        Pusher::Std(pusher) => {
            multi::download_multi(
                puller,
                pusher,
                multi::DownloadOptions {
                    download_chunks,
                    concurrent: NonZero::new(options.threads).unwrap_or(NonZero::new(1).unwrap()),
                    retry_gap,
                    push_queue_cap: options.write_queue_cap,
                    min_chunk_size: NonZero::new(options.min_chunk_size)
                        .unwrap_or(NonZero::new(8 * 1024).unwrap()),
                },
            )
            .await
        }
    };
    let res_clone = res.clone();
    app.listen("stop-download", move |event| {
        if let Ok(payload) = serde_json::from_str::<StopEvent>(event.payload())
            && payload.file_path == options.file_path
        {
            println!("stop download: {}", payload.file_path);
            res_clone.abort();
        }
    });
    tokio::spawn(async move {
        let mut pull_progress = vec![vec![]; options.threads];
        let mut push_progress = vec![vec![]; options.threads];
        let mut last_update_time = Instant::now();
        let mut downloaded = 0;
        while let Ok(e) = res.event_chain.recv().await {
            match e {
                fast_pull::Event::PullProgress(id, range) => {
                    downloaded += range.total();
                    pull_progress[id].merge_progress(range);
                    if last_update_time.elapsed().as_millis() > 100 {
                        last_update_time = Instant::now();
                        let pull_progress_data = pull_progress
                            .iter()
                            .map(|v| v.iter().map(|r| (r.start, r.end)).collect())
                            .collect();
                        tx.send(Event::PullProgress(pull_progress_data, downloaded))
                            .unwrap();
                        let push_progress_data = push_progress
                            .iter()
                            .map(|v| v.iter().map(|r: &ProgressEntry| (r.start, r.end)).collect())
                            .collect();
                        tx.send(Event::PushProgress(push_progress_data)).unwrap();
                    }
                }
                fast_pull::Event::PushProgress(id, range) => {
                    push_progress[id].merge_progress(range);
                    if last_update_time.elapsed().as_millis() > 100 {
                        last_update_time = Instant::now();
                        let pull_progress_data = pull_progress
                            .iter()
                            .map(|v| v.iter().map(|r| (r.start, r.end)).collect())
                            .collect();
                        tx.send(Event::PullProgress(pull_progress_data, downloaded))
                            .unwrap();
                        let push_progress_data = push_progress
                            .iter()
                            .map(|v| v.iter().map(|r: &ProgressEntry| (r.start, r.end)).collect())
                            .collect();
                        tx.send(Event::PushProgress(push_progress_data)).unwrap();
                    }
                }
                _ => tx.send(e.into()).unwrap(),
            };
        }
        let pull_progress_data = pull_progress
            .iter()
            .map(|v| v.iter().map(|r| (r.start, r.end)).collect())
            .collect();
        tx.send(Event::PullProgress(pull_progress_data, downloaded))
            .unwrap();
        let push_progress_data = push_progress
            .iter()
            .map(|v| v.iter().map(|r: &ProgressEntry| (r.start, r.end)).collect())
            .collect();
        tx.send(Event::PushProgress(push_progress_data)).unwrap();
        res.join().await.unwrap();
        tx.send(Event::AllFinished).unwrap();
    });
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] FilePusherError),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Thread(#[from] tokio::task::JoinError),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum ErrorKind {
    Io(String),
    UrlParse(String),
    Reqwest(String),
    Thread(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let msg = self.to_string();
        let kind = match self {
            Error::Io(_) => ErrorKind::Io(msg),
            Error::UrlParse(_) => ErrorKind::UrlParse(msg),
            Error::Reqwest(_) => ErrorKind::Reqwest(msg),
            Error::Thread(_) => ErrorKind::Thread(msg),
        };
        kind.serialize(serializer)
    }
}
