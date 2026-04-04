use crate::{
    core::{App, start_new_entry},
    os::wakeup_window,
    ui::{DialogType, EntryData},
    utils::{LogErr, show_task_dialog},
};
use crossfire::mpsc;
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions,
    tokio::{Stream, prelude::*},
};
use serde::{Deserialize, Serialize};
use slint::{ToSharedString, VecModel};
use std::{process::exit, rc::Rc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use url::Url;

pub const NS_NAME: &str = "top.s121.fd.sock";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    pub url: Url,
    pub headers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    WakeUp,
    Download(DownloadOptions),
}

/// 这是用户正常双击运行软件时，检查是否已经有在运行的实例
pub async fn check_ipc_and_wake() -> color_eyre::Result<()> {
    let ns_name = NS_NAME.to_ns_name::<GenericNamespaced>()?;
    if let Ok(mut stream) = Stream::connect(ns_name).await {
        tracing::info!("发现已有实例，正在发送唤醒信号...");
        let msg = IpcMessage::WakeUp;
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = stream.write_all(format!("{json}\n").as_bytes()).await;
        }
        exit(0);
    }
    Ok(())
}

/// 监听其他实例（或浏览器代理进程）发来的 IPC 请求
pub async fn init_ipc(app: App, list_model: Rc<VecModel<EntryData>>) -> color_eyre::Result<()> {
    let ns_name = NS_NAME.to_ns_name::<GenericNamespaced>()?;
    let listener = ListenerOptions::new()
        .name(ns_name)
        .try_overwrite(true)
        .create_tokio()?;

    let (tx, rx) = mpsc::unbounded_async::<IpcMessage>();

    let ui_weak = app.ui.clone();
    slint::spawn_local(async move {
        while let Ok(msg) = rx.recv().await {
            match msg {
                IpcMessage::WakeUp => {
                    tracing::info!("收到唤醒信号");
                    let _ = ui_weak.upgrade_in_event_loop(|ui| wakeup_window(&ui));
                }
                IpcMessage::Download(e) => {
                    tracing::info!("收到外部下载请求: {}", e.url);
                    let mut config = app.db.get_ui_download_config();
                    if let Some(s) = e.headers {
                        config.headers = s.into();
                    }
                    if app.db.is_ask_before_download() {
                        let app = app.clone();
                        let list_model = list_model.clone();
                        let _ = show_task_dialog(
                            e.url.to_shared_string(),
                            DialogType::AddTask,
                            config,
                            true,
                            move |urls, config, bg_download| {
                                let valid_urls = urls.lines().filter_map(|s| {
                                    Url::parse(s)
                                        .ok()
                                        .filter(|u| matches!(u.scheme(), "http" | "https"))
                                });
                                for url in valid_urls {
                                    start_new_entry(&app, url, &config, &list_model);
                                }
                                if !bg_download && let Some(ui) = app.ui.upgrade() {
                                    wakeup_window(&ui);
                                }
                            },
                        )
                        .log_err("任务对话框失败");
                    } else {
                        start_new_entry(&app, e.url, &config, &list_model);
                    }
                }
            }
        }
    })
    .log_err("IPC 消息处理任务失败")?;

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok(conn) => {
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let mut reader = BufReader::new(conn);
                        let mut buffer = String::new();
                        if reader.read_line(&mut buffer).await.is_ok()
                            && let Ok(msg) = serde_json::from_str::<IpcMessage>(&buffer)
                        {
                            let _ = tx.send(msg);
                        }
                    });
                }
                Err(e) => tracing::error!(err = ?e, "监听连接出错"),
            }
        }
    });
    Ok(())
}
