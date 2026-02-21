use crate::{ui::MainWindow, utils::wakeup_window};
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions,
    tokio::{Stream, prelude::*},
};
use slint::Weak;
use std::{io::ErrorKind, process::exit};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn check_ipc() -> color_eyre::Result<()> {
    let ns_name = "com.fast-down.gui.sock".to_ns_name::<GenericNamespaced>()?;
    match Stream::connect(ns_name.clone()).await {
        Ok(mut stream) => {
            tracing::info!("发现已有实例，正在发送唤醒信号...");
            stream.write_all(b"WAKE_UP\n").await?;
            exit(0);
        }
        Err(e) if matches!(e.kind(), ErrorKind::ConnectionRefused | ErrorKind::NotFound) => {
            tracing::info!("未发现运行中实例，准备启动主程序...");
        }
        Err(e) => Err(e)?,
    }
    Ok(())
}

pub async fn init_ipc(ui_weak: Weak<MainWindow>) -> color_eyre::Result<()> {
    let ns_name = "com.fast-down.gui.sock".to_ns_name::<GenericNamespaced>()?;
    let listener = ListenerOptions::new()
        .name(ns_name)
        .try_overwrite(true)
        .create_tokio()?;
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok(conn) => {
                    let ui_weak = ui_weak.clone();
                    tokio::spawn(async move {
                        let res = async {
                            let mut reader = BufReader::new(conn);
                            let mut buffer = String::new();
                            reader.read_line(&mut buffer).await?;
                            if buffer.trim() == "WAKE_UP" {
                                tracing::info!("收到唤醒信号");
                                let _ = ui_weak.upgrade_in_event_loop(|ui| {
                                    wakeup_window(&ui);
                                });
                            }
                            Ok::<_, color_eyre::Report>(())
                        };
                        if let Err(e) = res.await {
                            tracing::error!(err = %e, "处理连接出错");
                        }
                    });
                }
                Err(e) => tracing::error!(err = %e, "监听连接出错"),
            }
        }
    });
    Ok(())
}
