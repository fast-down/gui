#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::Clipboard;
use fast_down::{FileId, Total};
use fast_down_gui::{
    core::{DownloadEvent, TaskSet, download},
    fmt::format_size,
    persist::{self, Database, DatabaseEntry},
    ui::*,
    utils::{ForceSendExt, is_url},
};
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions,
    tokio::{Stream, prelude::*},
};
use rfd::FileDialog;
use slint::{Model, ModelRc, ToSharedString, VecModel};
use std::{path::PathBuf, process::exit, rc::Rc, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, level_filters::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    };

    Registry::default()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(ErrorLayer::default())
        .init();

    let ns_name = "com.fast-down.gui.sock".to_ns_name::<GenericNamespaced>()?;
    match Stream::connect(ns_name.clone()).await {
        Ok(mut stream) => {
            tracing::info!("发现已有实例，正在发送唤醒信号...");
            stream.write_all(b"WAKE_UP\n").await?;
            return Ok(());
        }
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound
                || e.kind() == std::io::ErrorKind::ConnectionRefused =>
        {
            tracing::info!("未发现运行中实例，准备启动主程序...");
        }
        Err(e) => Err(e)?,
    }

    let main_window = MainWindow::new()?;
    let ui_weak = main_window.as_weak();
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

    let db = Database::new().await;
    let entries = db.inner.data.iter().map(|e| e.to_entry_data(*e.key()));
    let list_model = Rc::new(VecModel::from_iter(entries));
    main_window.set_version(VERSION.into());
    main_window.set_raw_list(ModelRc::new(list_model.clone()));
    main_window.set_all_list(ModelRc::new(list_model.clone().reverse()));
    main_window.set_running_list(ModelRc::new(
        list_model
            .clone()
            .filter(|e| e.status == Status::Running)
            .reverse(),
    ));
    main_window.set_waiting_list(ModelRc::new(
        list_model
            .clone()
            .filter(|e| e.status == Status::Waiting)
            .reverse(),
    ));
    main_window.set_paused_list(ModelRc::new(
        list_model
            .clone()
            .filter(|e| e.status == Status::Paused)
            .reverse(),
    ));
    main_window.set_completed_list(ModelRc::new(
        list_model
            .clone()
            .filter(|e| e.status == Status::Completed)
            .reverse(),
    ));
    main_window.set_error_list(ModelRc::new(
        list_model
            .clone()
            .filter(|e| e.status == Status::Error)
            .reverse(),
    ));

    let task_set = TaskSet::new(db.inner.config.lock().max_concurrency);
    let task_dialog = TaskDialog::new()?;
    main_window.on_add_task({
        let dialog = task_dialog.as_weak();
        let db = db.clone();
        move || {
            let config = db.get_ui_config();
            let url = Clipboard::new()
                .ok()
                .and_then(|mut c| c.get_text().ok())
                .filter(|s| is_url(s))
                .unwrap_or_default();
            let res = dialog.upgrade_in_event_loop(move |dialog| {
                dialog.set_type(DialogType::AddTask);
                dialog.set_urls(url.into());
                dialog.set_config(config);
                if let Err(e) = dialog.show() {
                    tracing::error!(err = %e, "显示添加任务对话框出错");
                }
            });
            if let Err(e) = res {
                tracing::error!(err = %e, "升级添加任务对话框出错");
            }
        }
    });
    task_dialog.on_canceled({
        let dialog = task_dialog.as_weak();
        move || {
            let _ = dialog.upgrade_in_event_loop(|dialog| {
                if let Err(e) = dialog.hide() {
                    tracing::error!(err = %e, "隐藏对话框出错");
                }
            });
        }
    });
    task_dialog.on_browse_folder({
        let dialog = task_dialog.as_weak();
        move || {
            let dialog = dialog.clone();
            std::thread::spawn(move || {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    let _ = dialog.upgrade_in_event_loop(move |dialog| {
                        let path = folder.to_string_lossy().to_string();
                        dialog.invoke_set_save_dir(path.into());
                    });
                }
            });
        }
    });
    task_dialog.on_comfirm({
        let dialog = task_dialog.as_weak();
        let db = db.clone();
        let task_set = task_set.clone();
        let ui_handle = main_window.as_weak();

        move |urls, config| {
            let _ = dialog.upgrade_in_event_loop(|dialog| {
                if let Err(e) = dialog.hide() {
                    tracing::error!(err = %e, "隐藏对话框出错");
                }
            });
            let urls = urls
                .lines()
                .filter_map(|s| {
                    Url::parse(s)
                        .ok()
                        .filter(|s| ["http", "https"].contains(&s.scheme()))
                })
                .map(|url| (db.next_gid(), url));
            for (gid, url) in urls {
                let entry = EntryData {
                    gid,
                    avg_speed: "".into(),
                    downloaded: "".into(),
                    filename: url.to_shared_string(),
                    path: "".into(),
                    percentage: "".into(),
                    progress: ModelRc::from([]),
                    remaining_size: "".into(),
                    remaining_time: "".into(),
                    speed: "".into(),
                    status: Status::Waiting,
                    time: "".into(),
                    total: "".into(),
                };
                list_model.push(entry);
                if let Err(e) = db.init_entry(
                    gid,
                    DatabaseEntry {
                        file_name: url.to_string(),
                        file_path: PathBuf::new(),
                        file_size: 0,
                        file_id: FileId::default(),
                        progress: Vec::new(),
                        elapsed: Duration::ZERO,
                        url: url.clone(),
                        config: config.clone().into(),
                        status: persist::Status::Paused,
                    },
                ) {
                    error!(gid = %gid, err = %e, "初始化下载项失败");
                }
                let cancel_token = CancellationToken::new();
                let config = config.clone();
                let token_clone = cancel_token.clone();
                let db_clone = db.clone();
                let ui_handle_clone = ui_handle.clone();
                let mut file_size = 0;
                let event_handler = move |event| match event {
                    DownloadEvent::Info(info) => {
                        file_size = info.file_size;
                        if let Err(e) = db_clone.init_entry(gid, *info.clone()) {
                            error!(gid = %gid, err = %e, "初始化下载项失败");
                        }
                        let res = ui_handle_clone.upgrade_in_event_loop(move |ui| {
                            let list_model = ui.get_raw_list();
                            if let Some(index) = (0..list_model.row_count())
                                .rev()
                                .find(|&i| list_model.row_data(i).unwrap().gid == gid)
                            {
                                let mut data = list_model.row_data(index).unwrap();
                                data.status = Status::Running;
                                data.filename = info.file_name.into();
                                data.path = info.file_path.to_string_lossy().as_ref().into();
                                data.total = format_size(info.file_size as f64).into();
                                list_model.set_row_data(index, data);
                            }
                        });
                        if let Err(e) = res {
                            error!(gid = gid, err = ?e, "更新界面失败");
                        }
                    }
                    DownloadEvent::Progress(p) => {
                        db_clone.update_entry(gid, p.progress.clone(), p.elapsed);
                        let res = ui_handle_clone.upgrade_in_event_loop(move |ui| {
                            let list_model = ui.get_raw_list();
                            if let Some(index) = (0..list_model.row_count())
                                .rev()
                                .find(|&i| list_model.row_data(i).unwrap().gid == gid)
                            {
                                let mut data = list_model.row_data(index).unwrap();
                                data.downloaded = p.downloaded;
                                data.speed = p.speed;
                                data.avg_speed = p.avg_speed;
                                data.percentage = p.percentage;
                                data.remaining_time = p.remaining_time;
                                data.remaining_size = p.remaining_size;
                                data.time = p.time;
                                if file_size > 0 {
                                    data.progress =
                                        Rc::new(VecModel::from_iter(p.progress.iter().map(|r| {
                                            Progress {
                                                start: r.start as f32 / file_size as f32,
                                                width: r.total() as f32 / file_size as f32,
                                            }
                                        })))
                                        .into();
                                }
                                list_model.set_row_data(index, data);
                            }
                        });
                        if let Err(e) = res {
                            error!(gid = gid, err = ?e, "更新界面失败");
                        }
                    }
                    DownloadEvent::End { is_cancelled } => {
                        info!(is_cancelled = is_cancelled, "下载完成");
                        db_clone.update_status(
                            gid,
                            if is_cancelled {
                                persist::Status::Paused
                            } else {
                                persist::Status::Completed
                            },
                        );
                        let res = ui_handle_clone.upgrade_in_event_loop(move |ui| {
                            let list_model = ui.get_raw_list();
                            if let Some(index) = (0..list_model.row_count())
                                .rev()
                                .find(|&i| list_model.row_data(i).unwrap().gid == gid)
                            {
                                let mut data = list_model.row_data(index).unwrap();
                                data.status = if is_cancelled {
                                    Status::Paused
                                } else {
                                    Status::Completed
                                };
                                list_model.set_row_data(index, data);
                            }
                        });
                        if let Err(e) = res {
                            error!(gid = gid, err = ?e, "更新界面失败");
                        }
                    }
                };
                let db_clone = db.clone();
                let ui_handle_clone = ui_handle.clone();
                let fut = async move {
                    let res = download(url, config, token_clone, None, event_handler).await;
                    match res {
                        Ok(()) => info!("下载完成"),
                        Err(e) => {
                            error!(err = %e, "下载出错");
                            db_clone.update_status(gid, persist::Status::Error);
                            let res = ui_handle_clone.upgrade_in_event_loop(move |ui| {
                                let list_model = ui.get_raw_list();
                                if let Some(index) = (0..list_model.row_count())
                                    .rev()
                                    .find(|&i| list_model.row_data(i).unwrap().gid == gid)
                                {
                                    let mut data = list_model.row_data(index).unwrap();
                                    data.status = Status::Error;
                                    list_model.set_row_data(index, data);
                                }
                            });
                            if let Err(e) = res {
                                error!(gid = gid, err = ?e, "更新界面失败");
                            }
                        }
                    }
                }
                .force_send();
                task_set.add_task(gid, cancel_token, fut);
            }
        }
    });

    main_window.set_config(db.get_ui_config());
    main_window.on_exit({
        let db = db.clone();
        let task_set = task_set.clone();
        move || {
            let db = db.clone();
            let fut = tokio::task::spawn_blocking(move || db.flush_force_sync());
            task_set.cancel_all();
            let task_set = task_set.clone();
            tokio::spawn(async move {
                task_set.join().await;
                fut.await.unwrap().unwrap();
                exit(0);
            });
        }
    });
    main_window.on_browse_folder({
        let ui = main_window.as_weak();
        move || {
            let ui = ui.clone();
            std::thread::spawn(move || {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    let _ = ui.upgrade_in_event_loop(move |ui| {
                        let path = folder.to_string_lossy().to_string();
                        ui.invoke_set_save_dir(path.into());
                    });
                }
            });
        }
    });
    main_window.on_config_change({
        let db = db.clone();
        let task_set = task_set.clone();
        move |c| {
            tracing::info!(config = ?c, "配置已更新");
            task_set.set_concurrency(c.max_concurrency as usize);
            db.set_config(c);
        }
    });
    main_window.show()?;
    slint::run_event_loop_until_quit()?;
    Ok(())
}

fn wakeup_window(ui: &MainWindow) {
    let window = ui.window();
    if let Err(e) = window.show() {
        tracing::error!(err = %e, "显示窗口出错");
    }
    window.set_minimized(false);
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use windows::Win32::{Foundation::HWND, System::Threading::*, UI::WindowsAndMessaging::*};
        let handle = window.window_handle().window_handle().map(|h| h.as_raw());
        if let Ok(RawWindowHandle::Win32(win32_handle)) = handle {
            unsafe {
                let hwnd = HWND(win32_handle.hwnd.get() as *mut std::ffi::c_void);
                let foreground_hwnd = GetForegroundWindow();
                if foreground_hwnd != hwnd {
                    let foreground_thread = GetWindowThreadProcessId(foreground_hwnd, None);
                    let current_thread = GetCurrentThreadId();
                    if foreground_thread != current_thread {
                        let _ = AttachThreadInput(current_thread, foreground_thread, true);
                    }
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                    if foreground_thread != current_thread {
                        let _ = AttachThreadInput(current_thread, foreground_thread, false);
                    }
                } else {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                }
                let flash_info = FLASHWINFO {
                    cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
                    hwnd,
                    dwFlags: FLASHW_ALL | FLASHW_TIMERNOFG,
                    uCount: 2,
                    dwTimeout: 0,
                };
                let _ = FlashWindowEx(&flash_info);
            }
        }
    }
}
