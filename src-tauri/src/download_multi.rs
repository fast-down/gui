use crate::{event::Event, reader::FastDownReader};
#[cfg(target_pointer_width = "64")]
use fast_pull::file::RandFileWriterMmap;
#[cfg(not(target_pointer_width = "64"))]
use fast_pull::file::RandFileWriterStd;
use fast_pull::multi;
use std::{collections::HashMap, num::NonZeroUsize, time::Duration};
use tauri::{http::HeaderMap, ipc::Channel};
use tokio::fs::OpenOptions;
use url::Url;

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn download_multi(
    url: &str,
    file_path: &str,
    file_size: u64,
    threads: usize,
    write_buffer_size: usize,
    write_queue_cap: usize,
    download_chunks: Vec<(u64, u64)>,
    retry_gap: u64,
    headers: HashMap<String, String>,
    proxy: Option<String>,
    tx: Channel<Event>,
) -> Result<Channel<()>, Error> {
    let url = Url::parse(url)?;
    let headers = headers
        .into_iter()
        .filter_map(|(k, v)| Some((k.parse().ok()?, v.parse().ok()?)))
        .collect::<HeaderMap>();
    let retry_gap = Duration::from_millis(retry_gap);
    let download_chunks = download_chunks
        .into_iter()
        .map(|(start, end)| start..end)
        .collect();
    let reader = FastDownReader::new(url, headers, proxy)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(file_path)
        .await?;
    if threads == 0 {
        return Err(Error::ZeroThreads);
    }
    #[cfg(target_pointer_width = "64")]
    let writer = RandFileWriterMmap::new(file, file_size, write_buffer_size).await?;
    #[cfg(not(target_pointer_width = "64"))]
    let writer = RandFileWriterStd::new(file, file_size, write_buffer_size).await?;
    let res = multi::download_multi(
        reader,
        writer,
        multi::DownloadOptions {
            download_chunks,
            concurrent: NonZeroUsize::new(threads).unwrap_or(NonZeroUsize::new(1).unwrap()),
            retry_gap,
            write_queue_cap,
        },
    )
    .await;
    let res_clone = res.clone();
    let stop_channel: Channel<()> = Channel::new(move |_| {
        println!("stop download");
        res_clone.cancel();
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
    #[error("threads must be greater than zero")]
    ZeroThreads,
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
    ZeroThreads(String),
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
            Error::ZeroThreads => ErrorKind::ZeroThreads(msg),
            Error::Reqwest(_) => ErrorKind::Reqwest(msg),
            Error::Thread(_) => ErrorKind::Thread(msg),
        };
        kind.serialize(serializer)
    }
}
