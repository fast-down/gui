extern crate sanitize_filename;
use crate::puller;
use fast_down::http::Prefetch;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tauri::http::HeaderMap;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlInfo {
    pub size: u64,
    pub name: String,
    pub supports_range: bool,
    pub fast_download: bool,
    pub final_url: String,
    pub etag: Option<Arc<str>>,
    pub last_modified: Option<Arc<str>>,
}

impl From<fast_down::UrlInfo> for UrlInfo {
    fn from(value: fast_down::UrlInfo) -> Self {
        let sanitized = sanitize_filename::sanitize_with_options(
            value.name,
            sanitize_filename::Options {
                truncate: true,
                #[cfg(target_os = "windows")]
                windows: true,
                #[cfg(not(target_os = "windows"))]
                windows: false,
                replacement: "_",
            },
        );
        Self {
            size: value.size,
            name: sanitized,
            supports_range: value.supports_range,
            fast_download: value.fast_download,
            final_url: value.final_url.to_string(),
            etag: value.file_id.etag,
            last_modified: value.file_id.last_modified,
        }
    }
}

#[tauri::command]
pub async fn prefetch(
    url: &str,
    headers: HashMap<String, String>,
    proxy: String,
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
) -> Result<UrlInfo, String> {
    let headers = headers
        .into_iter()
        .filter_map(|(k, v)| Some((k.parse().ok()?, v.parse().ok()?)))
        .collect::<HeaderMap>();
    let client = puller::build_client(
        &headers,
        &proxy,
        accept_invalid_certs,
        accept_invalid_hostnames,
    )
    .map_err(|e| format!("{e:?}"))?;
    let url = url.parse().map_err(|e| format!("{e:?}"))?;
    let (info, _resp) = client.prefetch(url).await.map_err(|e| format!("{e:?}"))?;
    Ok(UrlInfo::from(info))
}
