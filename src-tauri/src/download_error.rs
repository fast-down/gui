use fast_down::file::FilePusherError;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
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

impl serde::Serialize for DownloadError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let msg = format!("{self:?}");
        let kind = match self {
            DownloadError::Io(_) => ErrorKind::Io(msg),
            DownloadError::UrlParse(_) => ErrorKind::UrlParse(msg),
            DownloadError::Reqwest(_) => ErrorKind::Reqwest(msg),
            DownloadError::Thread(_) => ErrorKind::Thread(msg),
        };
        kind.serialize(serializer)
    }
}
