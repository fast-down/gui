use crate::{
    event::{Event, StopEvent},
    puller::FastDownPuller,
};
use fast_pull::{file::SeqFilePusher, single};
use std::{collections::HashMap, time::Duration};
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
    pub proxy: Option<String>,
}

#[tauri::command]
pub async fn download_single(
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
    let puller = FastDownPuller::new(
        url,
        headers,
        options.proxy,
        options.multiplexing,
        options.accept_invalid_certs,
        options.accept_invalid_hostnames,
    )?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&options.file_path)
        .await?;
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
    app.listen("stop-download", move |event| {
        if let Ok(payload) = serde_json::from_str::<StopEvent>(event.payload())
            && payload.file_path == options.file_path
        {
            println!("stop download: {}", payload.file_path);
            res_clone.abort();
        }
    });
    tokio::spawn(async move {
        while let Ok(e) = res.event_chain.recv().await {
            tx.send(e.into()).unwrap();
        }
        res.join().await.unwrap();
        tx.send(Event::AllFinished).unwrap();
    });
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
