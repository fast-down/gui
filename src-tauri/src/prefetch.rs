use crate::puller;
use fast_pull::reqwest::Prefetch;
use serde::Serialize;
use std::collections::HashMap;
use tauri::http::HeaderMap;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlInfo {
    pub size: u64,
    pub name: String,
    pub supports_range: bool,
    pub fast_download: bool,
    pub final_url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl From<fast_pull::UrlInfo> for UrlInfo {
    fn from(value: fast_pull::UrlInfo) -> Self {
        Self {
            size: value.size,
            name: value.name,
            supports_range: value.supports_range,
            fast_download: value.fast_download,
            final_url: value.final_url.to_string(),
            etag: value.etag,
            last_modified: value.last_modified,
        }
    }
}

#[tauri::command]
pub async fn prefetch(
    url: &str,
    headers: HashMap<String, String>,
    proxy: Option<String>,
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
    client
        .prefetch(url)
        .await
        .map_err(|e| format!("{e:?}"))
        .map(UrlInfo::from)
}
