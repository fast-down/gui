use crate::{
    log_if_err,
    puller::{self, FastDownPuller},
    relaunch,
};
use fast_down::{mem::MemPusher, multi, reqwest::Prefetch};
use std::{num::NonZero, time::Duration};
use tauri::{Emitter, Listener, Manager, http::HeaderMap};
use tauri_plugin_updater::UpdaterExt;

pub async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        if let Some(main_window) = app.get_webview_window("main") {
            log_if_err!(
                main_window.set_title(&format!(
                    "fast-down v{} -> v{}",
                    update.current_version, update.version
                )),
                "set title error"
            );
        }
        let client = puller::build_client(&HeaderMap::new(), "", false, false)?;
        let info = client.prefetch(update.download_url.clone()).await?;
        let puller = FastDownPuller::new(
            update.download_url.clone(),
            HeaderMap::new(),
            "",
            false,
            false,
            false,
        )?;
        let pusher = MemPusher::with_capacity(info.size as usize);
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
                let bytes = pusher.receive.lock();
                if update_clone.install(&bytes[..]).is_ok() {
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
