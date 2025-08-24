use axum::{Router, routing::post};

async fn create_download() -> &'static str {
    "Hello, World!"
}

pub async fn start_server(_handle: tauri::AppHandle) -> Result<(), axum::Error> {
    let app = Router::new().route("/download", post(create_download));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:6121")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
