use std::fmt::Debug;

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
    PullProgress(Vec<Vec<(u64, u64)>>, u64),
    PushError(WorkerId, String),
    PushProgress(Vec<Vec<(u64, u64)>>),
    FlushError(String),
    Finished(WorkerId),
    AllFinished,
}

impl<RE: Debug, WE: Debug> From<fast_pull::Event<RE, WE>> for Event {
    fn from(value: fast_pull::Event<RE, WE>) -> Self {
        match value {
            fast_pull::Event::Pulling(id) => Event::Pulling(id),
            fast_pull::Event::PullError(id, err) => Event::PullError(id, format!("{err:?}")),
            fast_pull::Event::PushError(id, err) => Event::PushError(id, format!("{err:?}")),
            fast_pull::Event::FlushError(err) => Event::FlushError(format!("{err:?}")),
            fast_pull::Event::Finished(id) => Event::Finished(id),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StopEvent {
    pub file_path: String,
}
