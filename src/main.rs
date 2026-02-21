#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::Clipboard;
use fast_down::{FileId, Total};
use fast_down_gui::{
    core::{DownloadEvent, TaskSet, download},
    fmt::format_size,
    ipc::{check_ipc, init_ipc},
    persist::{self, Database, DatabaseEntry},
    ui::*,
    utils::{ForceSendExt, attach_console},
};
use rfd::FileDialog;
use slint::{Model, ModelRc, SharedString, VecModel, Weak};
use std::{path::PathBuf, process::exit, rc::Rc, time::Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, level_filters::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn init_tracing() {
    Registry::default()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(ErrorLayer::default())
        .init();
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    attach_console();
    init_tracing();
    if let Err(e) = check_ipc().await {
        error!(err = ?e, "检查 ipc 通道错误");
    }
    let main_window = MainWindow::new()?;
    if let Err(e) = init_ipc(main_window.as_weak()).await {
        error!(err = ?e, "初始化 ipc 通道错误");
    }

    let db = Database::new().await;
    let task_set = TaskSet::new(db.inner.config.lock().max_concurrency);

    main_window.set_config(db.get_ui_config());
    main_window.set_version(VERSION.into());

    let entries = db.inner.data.iter().map(|e| e.to_entry_data(*e.key()));
    let list_model = Rc::new(VecModel::from_iter(entries));
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
    main_window.on_add_task({
        let db = db.clone();
        let ui = main_window.as_weak();
        let list_model = list_model.clone();
        move || {
            let config = db.get_ui_config();
            let url = Clipboard::new()
                .ok()
                .and_then(|mut c| c.get_text().ok())
                .filter(|s| Url::parse(s).is_ok_and(|u| ["http", "https"].contains(&u.scheme())))
                .unwrap_or_default();
            let res = show_task_dialog(
                url.into(),
                config,
                DialogType::AddTask,
                list_model.clone(),
                db.clone(),
                ui.clone(),
                task_set.clone(),
            );
            if let Err(e) = res {
                error!(err = ?e, "添加任务失败");
            }
        }
    });

    main_window.show()?;
    slint::run_event_loop_until_quit()?;
    Ok(())
}

fn download_handler(gid: i32, db: Database, ui: Weak<MainWindow>) -> impl FnMut(DownloadEvent) {
    let mut file_size = 0;
    move |event| match event {
        DownloadEvent::Info(info) => {
            file_size = info.file_size;
            if let Err(e) = db.init_entry(gid, *info.clone()) {
                error!(gid = %gid, err = %e, "初始化下载项失败");
            }
            let res = ui.upgrade_in_event_loop(move |ui| {
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
            db.update_entry(gid, p.progress.clone(), p.elapsed);
            let res = ui.upgrade_in_event_loop(move |ui| {
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
                            Rc::new(VecModel::from_iter(p.progress.iter().map(|r| Progress {
                                start: r.start as f32 / file_size as f32,
                                width: r.total() as f32 / file_size as f32,
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
            db.update_status(
                gid,
                if is_cancelled {
                    persist::Status::Paused
                } else {
                    persist::Status::Completed
                },
            );
            let res = ui.upgrade_in_event_loop(move |ui| {
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
    }
}

fn show_task_dialog(
    urls: SharedString,
    config: Config,
    dialog_type: DialogType,
    list_model: Rc<VecModel<EntryData>>,
    db: Database,
    ui: Weak<MainWindow>,
    task_set: TaskSet<i32>,
) -> color_eyre::Result<()> {
    let dialog = TaskDialog::new()?;
    dialog.set_urls(urls);
    dialog.set_config(config);
    dialog.set_type(dialog_type);
    dialog.on_canceled({
        let dialog = dialog.as_weak();
        move || {
            let _ = dialog.upgrade_in_event_loop(|dialog| {
                if let Err(e) = dialog.hide() {
                    tracing::error!(err = %e, "隐藏对话框出错");
                }
            });
        }
    });
    dialog.on_browse_folder({
        let dialog = dialog.as_weak();
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
    dialog.on_comfirm({
        let dialog = dialog.as_weak();
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
                let entry = DatabaseEntry {
                    file_name: url.to_string(),
                    file_path: PathBuf::new(),
                    file_size: 0,
                    file_id: FileId::default(),
                    progress: Vec::new(),
                    elapsed: Duration::ZERO,
                    url: url.clone(),
                    config: config.clone().into(),
                    status: persist::Status::Paused,
                };
                list_model.push(entry.to_entry_data(gid));
                if let Err(e) = db.init_entry(gid, entry) {
                    error!(gid = %gid, err = %e, "初始化下载项失败");
                }
                let handler = download_handler(gid, db.clone(), ui.clone());
                let config = config.clone();
                let cancel_token = CancellationToken::new();
                let token_clone = cancel_token.clone();
                let db = db.clone();
                let ui = ui.clone();
                let fut = async move {
                    let res = download(url, config, token_clone, None, handler).await;
                    match res {
                        Ok(()) => info!("下载完成"),
                        Err(e) => {
                            error!(err = %e, "下载出错");
                            db.update_status(gid, persist::Status::Error);
                            let res = ui.upgrade_in_event_loop(move |ui| {
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
    dialog.show()?;
    Ok(())
}
