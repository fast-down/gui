mod config;
mod entry;

pub use config::*;
pub use entry::*;

use color_eyre::Result;
use dashmap::DashMap;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    ops::Range,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
    time::Duration,
};
use tokio::{fs, task::JoinHandle};
use tracing::{error, info};

use crate::utils::LogErr;

lazy_static::lazy_static! {
    pub static ref DB_NAME: String = format!("fd-state-v{}-gui.fdb", DB_VERSION);
    pub static ref DB_DIR: PathBuf = {
        let db_dir = dirs::data_dir()
            .and_then(|p| soft_canonicalize::soft_canonicalize(p).ok())
            .map(|p| p.join("fast-down-gui"))
            .unwrap_or_default();
        let _ = std::fs::create_dir_all(&db_dir);
        db_dir
    };
    pub static ref DB_PATH: PathBuf = DB_DIR.join(&*DB_NAME);
}

const DB_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DatabaseInner {
    pub data: DashMap<i32, DatabaseEntry>,
    pub config: Mutex<Config>,
    pub max_gid: AtomicI32,
}

impl DatabaseInner {
    pub fn flush(&self) -> color_eyre::Result<()> {
        let content = bitcode::serialize(self)?;
        let tmp_path = DB_PATH.with_added_extension("tmp");
        let mut file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(&tmp_path)?;
        file.write_all(&content)?;
        file.sync_all()?;
        std::fs::rename(tmp_path, &*DB_PATH)?;
        Ok(())
    }

    pub fn next_gid(&self) -> i32 {
        self.max_gid.fetch_add(1, Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
pub struct Database {
    pub inner: Arc<DatabaseInner>,
    pub is_dirty: Arc<AtomicBool>,
    pub handle: Arc<JoinHandle<()>>,
}

impl Database {
    pub async fn new() -> Self {
        let inner = fs::read(&*DB_PATH)
            .await
            .ok()
            .and_then(|bytes| bitcode::deserialize::<DatabaseInner>(&bytes).ok());
        if inner.is_none() {
            let _ = tokio::fs::rename(&*DB_PATH, DB_PATH.with_added_extension("bak"))
                .await
                .log_err("数据库重命名失败");
        }
        let inner: Arc<_> = inner.unwrap_or_default().into();
        let is_dirty = Arc::new(AtomicBool::new(false));
        let handle = tokio::spawn({
            let inner = inner.clone();
            let is_dirty = is_dirty.clone();
            async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    if is_dirty.swap(false, Ordering::Relaxed) {
                        let inner = inner.clone();
                        let res = tokio::task::spawn_blocking(move || inner.flush()).await;
                        match res {
                            Ok(Ok(())) => info!("数据库保存成功"),
                            Ok(Err(e)) => {
                                error!(err = ?e, "无法保存到数据库");
                                is_dirty.store(true, Ordering::Relaxed);
                            }
                            Err(e) => {
                                error!(err = ?e, "无法保存到数据库");
                                is_dirty.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        });
        Database {
            inner,
            is_dirty,
            handle: handle.into(),
        }
    }

    pub fn get_config(&self) -> Config {
        self.inner.config.lock().clone()
    }

    pub fn get_ui_config(&self) -> crate::ui::Config {
        self.inner.config.lock().to_ui_config()
    }

    pub fn set_config(&self, config: impl Into<Config>) {
        *self.inner.config.lock() = config.into();
        self.is_dirty.store(true, Ordering::Relaxed);
    }

    pub fn next_gid(&self) -> i32 {
        self.inner.next_gid()
    }

    pub fn init_entry(&self, gid: i32, entry: DatabaseEntry) -> Result<()> {
        self.inner.data.insert(gid, entry);
        self.is_dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn update_entry(&self, gid: i32, progress: Vec<Range<u64>>, elapsed: Duration) {
        if let Some(mut e) = self.inner.data.get_mut(&gid) {
            e.progress = progress;
            e.elapsed = elapsed;
            self.is_dirty.store(true, Ordering::Relaxed);
        }
    }

    pub fn update_status(&self, gid: i32, status: Status) {
        if let Some(mut e) = self.inner.data.get_mut(&gid) {
            e.status = status;
            self.is_dirty.store(true, Ordering::Relaxed);
        }
    }

    pub fn remove_entry(&self, gid: i32) -> Result<()> {
        self.inner.data.remove(&gid);
        self.is_dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn flush_force_sync(&self) -> Result<()> {
        if self.is_dirty.swap(false, Ordering::Relaxed) {
            match self.inner.flush() {
                Ok(()) => info!("数据库保存成功"),
                Err(e) => {
                    error!(err = ?e, "无法保存到数据库");
                    self.is_dirty.store(true, Ordering::Relaxed);
                    Err(e)?
                }
            }
        }
        Ok(())
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        self.handle.abort();
        if self.is_dirty.load(Ordering::Relaxed) {
            let _ = self.inner.flush();
        }
    }
}
