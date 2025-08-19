use crate::{
    puller::{self, FastDownPuller},
    relaunch,
};
use fast_pull::{RandPusher, multi, reqwest::Prefetch};
use spin::mutex::SpinMutex;
use std::{num::NonZero, sync::Arc, time::Duration};
use tauri::{Emitter, Listener, Manager, http::HeaderMap};
use tauri_plugin_updater::UpdaterExt;

pub async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        if let Some(main_window) = app.get_webview_window("main") {
            let _ = main_window.set_title(&format!(
                "fast-down v{} -> v{}",
                update.current_version, update.version
            ));
        }
        let client = puller::build_client(&HeaderMap::new(), &None, false, false)?;
        let info = client.prefetch(update.download_url.clone()).await?;
        let puller = FastDownPuller::new(
            update.download_url.clone(),
            HeaderMap::new(),
            None,
            false,
            false,
            false,
        )?;
        let pusher = MemPusher::new(info.size);
        let res = multi::download_multi(
            puller,
            pusher.clone(),
            multi::DownloadOptions {
                #[allow(clippy::single_range_in_vec_init)]
                download_chunks: vec![0..info.size],
                concurrent: NonZero::new(8).unwrap(),
                retry_gap: Duration::from_millis(500),
                push_queue_cap: 10240,
                min_chunk_size: NonZero::new(8 * 1024).unwrap(),
            },
        )
        .await;
        if res.join().await.is_ok() {
            let app_clone = app.clone();
            let update_clone = update.clone();
            app.listen("accept_update", move |_| {
                if update_clone.install(pusher.data.lock().clone()).is_ok() {
                    relaunch::relaunch(app_clone.clone());
                }
            });
            app.emit(
                "update",
                UpdateInfo {
                    body: update.body,
                    current_version: update.current_version,
                    version: update.version,
                    date: update.date.map(|d| d.unix_timestamp()),
                    target: update.target,
                    download_url: update.download_url.to_string(),
                    signature: update.signature,
                },
            )?;
        }
    }
    Ok(())
}

#[derive(Clone)]
struct MemPusher {
    pub data: Arc<SpinMutex<Vec<u8>>>,
}
impl RandPusher for MemPusher {
    type Error = ();
    async fn push(
        &mut self,
        range: fast_pull::ProgressEntry,
        content: bytes::Bytes,
    ) -> Result<(), Self::Error> {
        self.data.lock()[range.start as usize..range.end as usize].copy_from_slice(&content);
        Ok(())
    }
}
impl MemPusher {
    fn new(size: u64) -> Self {
        Self {
            data: Arc::new(SpinMutex::new(vec![0; size as usize])),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// Update description
    pub body: Option<String>,
    /// Version used to check for update
    pub current_version: String,
    /// Version announced
    pub version: String,
    /// Update publish date
    pub date: Option<i64>,
    /// Target
    pub target: String,
    /// Download URL announced
    pub download_url: String,
    /// Signature announced
    pub signature: String,
}
