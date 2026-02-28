use crate::persist::loader::Loader;
use dashmap::DashMap;
use fast_down_ffi::{
    WriteMethod,
    fast_down::{FileId, ProgressEntry, utils::Proxy},
};
use parking_lot::Mutex;
use serde::Deserialize;
use std::{
    collections::HashMap, net::IpAddr, path::PathBuf, sync::atomic::AtomicI32, time::Duration,
};
use url::Url;

#[derive(Deserialize, Debug)]
pub struct DownloadConfig {
    pub save_dir: PathBuf,
    pub threads: usize,
    pub proxy: Proxy<String>,
    pub headers: HashMap<String, String>,
    pub min_chunk_size: u64,
    pub write_buffer_size: usize,
    pub write_queue_cap: usize,
    pub retry_gap: Duration,
    pub pull_timeout: Duration,
    pub accept_invalid_certs: bool,
    pub accept_invalid_hostnames: bool,
    pub local_address: Vec<IpAddr>,
    pub max_speculative: usize,
    pub write_method: WriteMethod,
    pub retry_times: usize,
    pub chunk_window: u64,
}

#[derive(Deserialize, Debug)]
pub struct GeneralConfig {
    pub max_concurrency: usize,
    pub auto_start: bool,
}

impl From<DownloadConfig> for crate::persist::DownloadConfig {
    fn from(c: DownloadConfig) -> Self {
        Self {
            proxy: c.proxy,
            retry_times: c.retry_times,
            chunk_window: c.chunk_window,
            save_dir: c.save_dir,
            threads: c.threads,
            headers: c.headers,
            min_chunk_size: c.min_chunk_size,
            write_buffer_size: c.write_buffer_size,
            write_queue_cap: c.write_queue_cap,
            retry_gap: c.retry_gap,
            pull_timeout: c.pull_timeout,
            accept_invalid_certs: c.accept_invalid_certs,
            accept_invalid_hostnames: c.accept_invalid_hostnames,
            local_address: c.local_address,
            max_speculative: c.max_speculative,
            write_method: c.write_method,
        }
    }
}

impl From<GeneralConfig> for crate::persist::GeneralConfig {
    fn from(c: GeneralConfig) -> Self {
        Self {
            max_concurrency: c.max_concurrency,
            auto_start: c.auto_start,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct DatabaseEntry {
    pub file_name: String,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub file_id: FileId,
    pub progress: Vec<ProgressEntry>,
    pub elapsed: Duration,
    pub url: Url,
    pub config: DownloadConfig,
    pub status: Status,
}

impl From<DatabaseEntry> for crate::persist::DatabaseEntry {
    fn from(e: DatabaseEntry) -> Self {
        Self {
            file_name: e.file_name,
            file_path: e.file_path,
            file_size: e.file_size,
            file_id: e.file_id,
            progress: e.progress,
            elapsed: e.elapsed,
            url: e.url,
            config: e.config.into(),
            status: e.status.into(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum Status {
    Completed,
    Error,
    Paused,
}

impl From<Status> for crate::persist::Status {
    fn from(value: Status) -> Self {
        match value {
            Status::Completed => crate::persist::Status::Completed,
            Status::Error => crate::persist::Status::Error,
            Status::Paused => crate::persist::Status::Paused,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct DatabaseInner {
    pub data: DashMap<i32, DatabaseEntry>,
    pub download_config: Mutex<DownloadConfig>,
    pub general_config: Mutex<GeneralConfig>,
    pub max_gid: AtomicI32,
}

impl From<DatabaseInner> for crate::persist::DatabaseInner {
    fn from(db: DatabaseInner) -> Self {
        Self {
            data: db.data.into_iter().map(|(k, v)| (k, v.into())).collect(),
            download_config: Mutex::new(db.download_config.into_inner().into()),
            general_config: Mutex::new(db.general_config.into_inner().into()),
            max_gid: db.max_gid,
        }
    }
}

#[derive(Debug, Clone)]
pub struct V3Loader;

impl Loader for V3Loader {
    fn load(&self, bytes: &[u8]) -> Option<crate::persist::DatabaseInner> {
        let db: DatabaseInner = bitcode::deserialize(bytes).ok()?;
        Some(db.into())
    }
}
