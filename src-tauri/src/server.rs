use axum::{Json, Router, extract::State, routing::post};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;

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
    match state.handle.emit("download-request", payload) {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            log::error!("Failed to emit download request: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItemId {
    file_path: String,
}

async fn pause(State(state): State<AppState>, Json(payload): Json<DownloadItemId>) -> StatusCode {
    log::info!("Received pause request: {payload:?}");
    match state.handle.emit("pause-request", payload) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("Failed to emit pause request: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
async fn resume(State(state): State<AppState>, Json(payload): Json<DownloadItemId>) -> StatusCode {
    log::info!("Received resume request: {payload:?}");
    match state.handle.emit("resume-request", payload) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("Failed to emit resume request: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
async fn remove(State(state): State<AppState>, Json(payload): Json<DownloadItemId>) -> StatusCode {
    log::info!("Received remove request: {payload:?}");
    match state.handle.emit("remove-request", payload) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("Failed to emit remove request: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
async fn pause_all(State(state): State<AppState>) -> StatusCode {
    log::info!("Received pause all request");
    match state.handle.emit("pause-all-request", ()) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("Failed to emit pause all request: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
async fn resume_all(State(state): State<AppState>) -> StatusCode {
    log::info!("Received resume all request");
    match state.handle.emit("resume-all-request", ()) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("Failed to emit resume all request: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
async fn remove_all(State(state): State<AppState>) -> StatusCode {
    log::info!("Received remove all request");
    match state.handle.emit("remove-all-request", ()) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("Failed to emit remove all request: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn start_server(handle: tauri::AppHandle) -> Result<(), axum::Error> {
    let state = AppState {
        handle: Arc::new(handle),
    };
    let app = Router::new()
        .route("/download", post(create_download))
        .route("/pause", post(pause))
        .route("/resume", post(resume))
        .route("/remove", post(remove))
        .route("/pause-all", post(pause_all))
        .route("/resume-all", post(resume_all))
        .route("/remove-all", post(remove_all))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6121").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
