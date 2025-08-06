use fast_pull::ProgressEntry;
use serde::Serialize;

pub type WorkerId = usize;

#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum Event {
    Reading(WorkerId),
    ReadError(WorkerId, String),
    ReadProgress(WorkerId, ProgressEntry),
    WriteError(WorkerId, String),
    WriteProgress(WorkerId, ProgressEntry),
    FlushError(String),
    Finished(WorkerId),
    Abort(WorkerId),
    AllFinished,
}

impl<RE: ToString, WE: ToString> From<fast_pull::Event<RE, WE>> for Event {
    fn from(value: fast_pull::Event<RE, WE>) -> Self {
        match value {
            fast_pull::Event::Reading(id) => Event::Reading(id),
            fast_pull::Event::ReadError(id, err) => Event::ReadError(id, err.to_string()),
            fast_pull::Event::ReadProgress(id, entry) => Event::ReadProgress(id, entry),
            fast_pull::Event::WriteError(id, err) => Event::WriteError(id, err.to_string()),
            fast_pull::Event::WriteProgress(id, entry) => Event::WriteProgress(id, entry),
            fast_pull::Event::FlushError(err) => Event::FlushError(err.to_string()),
            fast_pull::Event::Finished(id) => Event::Finished(id),
            fast_pull::Event::Abort(id) => Event::Abort(id),
        }
    }
}
