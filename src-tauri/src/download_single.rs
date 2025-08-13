use crate::{event::Event, puller::FastDownPuller};
use fast_pull::{file::SeqFilePusher, single};
use std::{collections::HashMap, time::Duration};
use tauri::{http::HeaderMap, ipc::Channel};
use tokio::fs::OpenOptions;
use url::Url;

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn download_single(
    url: &str,
    file_path: &str,
    write_buffer_size: usize,
    write_queue_cap: usize,
    retry_gap: u64,
    headers: HashMap<String, String>,
    multiplexing: bool,
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
    proxy: Option<String>,
    tx: Channel<Event>,
) -> Result<Channel<()>, Error> {
    let url = Url::parse(url)?;
    let headers = headers
        .into_iter()
        .filter_map(|(k, v)| Some((k.parse().ok()?, v.parse().ok()?)))
        .collect::<HeaderMap>();
    let retry_gap = Duration::from_millis(retry_gap);
    let puller = FastDownPuller::new(
        url,
        headers,
        proxy,
        multiplexing,
        accept_invalid_certs,
        accept_invalid_hostnames,
    )?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(file_path)
        .await?;
    let pusher = SeqFilePusher::new(file, write_buffer_size);
    let res = single::download_single(
        puller,
        pusher,
        single::DownloadOptions {
            retry_gap,
            push_queue_cap: write_queue_cap,
        },
    )
    .await;
    let res_clone = res.clone();
    let stop_channel: Channel<()> = Channel::new(move |_| {
        println!("stop download");
        res_clone.abort();
        Ok(())
    });
    tokio::spawn(async move {
        while let Ok(e) = res.event_chain.recv().await {
            tx.send(e.into()).unwrap();
        }
        res.join().await.unwrap();
        tx.send(Event::AllFinished).unwrap();
    });
    Ok(stop_channel)
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
