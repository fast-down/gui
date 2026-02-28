use crate::{
    core::{App, start_new_entry},
    os::wakeup_window,
    ui::{EntryData, MainWindow},
    utils::LogErr,
};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use crossfire::{MTx, mpsc};
use serde::{Deserialize, Serialize};
use slint::{VecModel, Weak};
use std::{rc::Rc, time::Duration};
use tracing::{error, info};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    url: Url,
    headers: Option<String>,
}

async fn download(
    State(tx): State<MTx<mpsc::List<DownloadOptions>>>,
    Json(payload): Json<DownloadOptions>,
) -> StatusCode {
    info!(payload = ?payload, "收到下载 HTTP 请求");
    match tx.send(payload) {
        Ok(()) => StatusCode::CREATED,
        Err(e) => {
            error!(err = ?e, "无法发送下载请求到主线程");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn start_server(
    app_core: App,
    list_model: Rc<VecModel<EntryData>>,
    ui: Weak<MainWindow>,
) -> color_eyre::Result<()> {
    let (tx, rx) = crossfire::mpsc::unbounded_async::<DownloadOptions>();
    slint::spawn_local(async move {
        while let Ok(e) = rx.recv().await {
            let mut config = app_core.db.get_ui_download_config();
            if let Some(s) = e.headers {
                config.headers = s.into();
            }
            start_new_entry(&app_core, e.url, &config, &list_model);
            let _ = ui.upgrade_in_event_loop(|ui| {
                wakeup_window(&ui);
            });
        }
    })
    .log_err("启动后台 HTTP 请求处理服务失败")?;
    tokio::spawn(async move {
        let app = Router::new()
            .route("/download", post(download))
            .with_state(tx);
        let listener = loop {
            let res = tokio::net::TcpListener::bind("0.0.0.0:6121").await;
            match res {
                Ok(listener) => break listener,
                Err(e) => error!(err = ?e, "Failed to bind to port 6121"),
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        };
        axum::serve(listener, app)
            .await
            .log_err("无法启动 server")
            .unwrap();
        info!("成功启动 server");
    });
    Ok(())
}
