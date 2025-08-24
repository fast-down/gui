use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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

impl<RE: Debug, WE: Debug> From<fast_down::Event<RE, WE>> for Event {
    fn from(value: fast_down::Event<RE, WE>) -> Self {
        match value {
            fast_down::Event::Pulling(id) => Event::Pulling(id),
            fast_down::Event::PullError(id, e) => Event::PullError(id, format!("{e:?}")),
            fast_down::Event::PushError(id, e) => Event::PushError(id, format!("{e:?}")),
            fast_down::Event::FlushError(e) => Event::FlushError(format!("{e:?}")),
            fast_down::Event::Finished(id) => Event::Finished(id),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DownloadItemId {
    pub file_path: String,
}
