use axum::{Json, Router, extract::State, routing::post};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;

use crate::log_if_err;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    url: String,
    filename: Option<String>,
    accept_invalid_certs: Option<bool>,
    accept_invalid_hostnames: Option<bool>,
    headers: Option<String>,
    proxy: Option<String>,
    min_chunk_size: Option<u64>,
    multiplexing: Option<bool>,
    retry_gap: Option<u64>,
    save_dir: Option<String>,
    threads: Option<usize>,
    write_buffer_size: Option<usize>,
    write_method: Option<String>,
    write_queue_cap: Option<usize>,
}

#[derive(Clone)]
struct AppState {
    handle: Arc<tauri::AppHandle>,
}

async fn create_download(
    State(state): State<AppState>,
    Json(payload): Json<DownloadOptions>,
) -> StatusCode {
    log::info!("Received download request: {payload:?}");
    log_if_err!(
        state.handle.emit("download-request", payload),
        "Failed to emit download request"
    );
    StatusCode::CREATED
}

pub async fn start_server(handle: tauri::AppHandle) -> Result<(), axum::Error> {
    let state = AppState {
        handle: Arc::new(handle),
    };
    let app = Router::new()
        .route("/download", post(create_download))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6121").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
