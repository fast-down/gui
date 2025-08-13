use fast_pull::ProgressEntry;
use serde::{Deserialize, Serialize};

pub type WorkerId = usize;

#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum Event {
    Pulling(WorkerId),
    PullError(WorkerId, String),
    PullProgress(WorkerId, ProgressEntry),
    PushError(WorkerId, String),
    PushProgress(WorkerId, ProgressEntry),
    FlushError(String),
    Finished(WorkerId),
    AllFinished,
}

impl<RE: ToString, WE: ToString> From<fast_pull::Event<RE, WE>> for Event {
    fn from(value: fast_pull::Event<RE, WE>) -> Self {
        match value {
            fast_pull::Event::Pulling(id) => Event::Pulling(id),
            fast_pull::Event::PullError(id, err) => Event::PullError(id, err.to_string()),
            fast_pull::Event::PullProgress(id, entry) => Event::PullProgress(id, entry),
            fast_pull::Event::PushError(id, err) => Event::PushError(id, err.to_string()),
            fast_pull::Event::PushProgress(id, entry) => Event::PushProgress(id, entry),
            fast_pull::Event::FlushError(err) => Event::FlushError(err.to_string()),
            fast_pull::Event::Finished(id) => Event::Finished(id),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StopEvent {
    pub file_path: String,
}
