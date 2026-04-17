#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::Clipboard;
use fast_down_gui::{
    addons::{CHROME_EXT_IDS, FIREFOX_EXT_ID, auto_register, handle_browser_request},
    core::{App, TaskSet, start_entry, start_new_entry},
    ipc::{check_ipc_and_wake, init_ipc},
    os::{attach_console, get_auto_start, is_admin, setup_tray, try_restart_as_admin},
    persist::{DB_DIR, Database},
    ui::*,
    utils::{LogErr, show_task_dialog},
};
use file_alloc::init_fast_alloc;
use slint::{Model, ModelRc, ToSharedString, VecModel};
use std::{collections::HashSet, rc::Rc, sync::Arc};
use tracing::{info, level_filters::LevelFilter};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn init_tracing() -> WorkerGuard {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("fd")
        .filename_suffix("log")
        .max_log_files(3)
        .build(&*DB_DIR)
        .expect("无法初始化日志写入");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer().with_writer(non_blocking).with_ansi(false);
    let std_layer = fmt::layer().pretty();
    Registry::default()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(std_layer)
        .with(file_layer)
        .init();
    _guard
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<_> = std::env::args().collect();
    if args
        .iter()
        .any(|arg| arg.contains(FIREFOX_EXT_ID) || CHROME_EXT_IDS.iter().any(|id| arg.contains(id)))
    {
        return handle_browser_request().await;
    }
    attach_console();
    let _guard = init_tracing();
    #[cfg(target_os = "linux")]
    let _gtk_timer = {
        let _ = gtk::init().log_err("初始化 gtk 错误");
        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(50),
            move || {
                while gtk::events_pending() {
                    gtk::main_iteration_do(false);
                }
            },
        );
        timer
    };

    let _ = check_ipc_and_wake().await.log_err("检查 ipc 通道错误");
    let _ = auto_register().log_err("写入浏览器扩展通信配置失败");
    let ui = MainWindow::new()?;
    let db = Database::new().await;
    let run_as_admin = db.inner.general_config.lock().run_as_admin;
    let _ = try_restart_as_admin(run_as_admin).log_err("以管理员身份重启失败");
    init_fast_alloc();
    let task_set = TaskSet::new(db.inner.general_config.lock().max_concurrency);
    let auto = get_auto_start()
        .log_err("初始化开机自启错误")
        .ok()
        .map(Arc::new);
    if db.is_auto_start()
        && let Some(auto) = &auto
    {
        let _ = auto.enable().log_err("启用开机自启失败");
    }
    let entries = db.inner.data.iter().map(|e| e.to_entry_data(*e.key()));
    let list_model = Rc::new(VecModel::from_iter(entries));
    let app = App {
        db: db.clone(),
        task_set: task_set.clone(),
        ui: ui.as_weak(),
    };
    let _ = init_ipc(app.clone(), list_model.clone())
        .await
        .log_err("初始化 ipc 通道错误");

    let _tray = setup_tray(app.clone()).log_err("初始化托盘错误");
    setup_ui_lists(&ui, list_model.clone());
    ui.set_download_config(db.get_ui_download_config());
    ui.set_general_config(db.get_ui_general_config());
    ui.set_version(VERSION.into());
    ui.set_admin(is_admin());

    ui.global::<Logic>().on_exit({
        let app = app.clone();
        move || app.exit()
    });

    ui.on_browse_folder({
        let ui = ui.as_weak();
        move || {
            let ui = ui.clone();
            slint::spawn_local(async move {
                if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await
                    && let Some(ui) = ui.upgrade()
                {
                    ui.invoke_set_save_dir(folder.path().to_string_lossy().to_shared_string());
                }
            })
            .unwrap();
        }
    });

    ui.global::<Logic>().on_config_change({
        let app = app.clone();
        let auto = auto.clone();
        move |download_config, general_config| {
            info!(download_config = ?download_config, general_config = ?general_config, "配置已更新");
            app.set_config(download_config, general_config, auto.as_deref());
        }
    });

    ui.global::<Logic>().on_start_all({
        let app = app.clone();
        let list_model = list_model.clone();
        move |list| {
            for (i, mut entry) in list.iter().enumerate() {
                if entry.status == Status::Completed {
                    continue;
                }
                let is_started = start_entry(&app, &entry, &list_model);
                if is_started {
                    entry.status = Status::Waiting;
                    list.set_row_data(i, entry);
                }
            }
        }
    });
    ui.global::<Logic>().on_start_entry({
        let app = app.clone();
        let list_model = list_model.clone();
        move |gid| {
            for i in (0..list_model.row_count()).rev() {
                let Some(mut entry) = list_model.row_data(i) else {
                    break;
                };
                if entry.gid == gid {
                    let is_started = start_entry(&app, &entry, &list_model);
                    if is_started {
                        entry.status = Status::Waiting;
                        list_model.set_row_data(i, entry);
                    }
                    break;
                }
            }
        }
    });

    ui.global::<Logic>().on_pause_all({
        let task_set = task_set.clone();
        move |list| {
            for entry in list.iter() {
                task_set.cancel_task(&entry.gid);
            }
        }
    });
    ui.global::<Logic>().on_pause_entry({
        let task_set = task_set.clone();
        move |gid| task_set.cancel_task(&gid)
    });

    ui.global::<Logic>().on_remove_all({
        let task_set = task_set.clone();
        let list_model = list_model.clone();
        let db = db.clone();
        move |list| {
            let ids_to_remove: HashSet<_> = list.iter().map(|e| e.gid).collect();
            let mut kept_items = Vec::new();
            for item in list_model.iter() {
                if !ids_to_remove.contains(&item.gid) {
                    kept_items.push(item);
                }
            }
            list_model.set_vec(kept_items);
            for gid in ids_to_remove.iter() {
                task_set.cancel_task(gid);
                let _ = db.remove_entry(*gid).log_err("数据库移除条目失败");
            }
        }
    });
    ui.global::<Logic>().on_remove_entry({
        let task_set = task_set.clone();
        let list_model = list_model.clone();
        move |gid| {
            for i in (0..list_model.row_count()).rev() {
                let Some(item) = list_model.row_data(i) else {
                    break;
                };
                if item.gid == gid {
                    list_model.remove(i);
                    break;
                }
            }
            task_set.cancel_task(&gid);
            let _ = db.remove_entry(gid).log_err("数据库移除条目失败");
        }
    });

    ui.global::<Logic>().on_add_task({
        let app = app.clone();
        let list_model = list_model.clone();
        let db = app.db.clone();
        move || {
            let url = Clipboard::new()
                .ok()
                .and_then(|mut c| c.get_text().ok())
                .filter(|s| Url::parse(s).is_ok_and(|u| matches!(u.scheme(), "http" | "https")))
                .unwrap_or_default();
            let app = app.clone();
            let list_model = list_model.clone();
            let _ = show_task_dialog(
                url.into(),
                DialogType::AddTask,
                db.get_ui_download_config(),
                false,
                move |urls, config, _| {
                    let valid_urls = urls.lines().filter_map(|s| {
                        Url::parse(s)
                            .ok()
                            .filter(|u| matches!(u.scheme(), "http" | "https"))
                    });
                    for url in valid_urls {
                        start_new_entry(&app, url, &config, &list_model);
                    }
                },
            )
            .log_err("添加任务对话框启动失败");
        }
    });

    ui.global::<Logic>().on_open_file(|path| {
        let _ = open::that(path).log_err("打开文件失败");
    });
    ui.global::<Logic>().on_locate_file(|path| {
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn();
        #[cfg(not(target_os = "macos"))]
        showfile::show_path_in_file_manager(path);
    });

    ui.global::<Logic>().on_detail_entry({
        let db = app.db.clone();
        move |gid| {
            let Some(mut entry) = db.inner.data.get(&gid).map(|e| e.clone()) else {
                return;
            };
            let db = db.clone();
            let _ = show_task_dialog(
                entry.url.to_shared_string(),
                DialogType::EditTask,
                entry.config.to_ui_download_config(),
                false,
                move |urls, config, _| {
                    let mut valid_urls = urls.lines().filter_map(|s| {
                        Url::parse(s)
                            .ok()
                            .filter(|u| matches!(u.scheme(), "http" | "https"))
                    });
                    if let Some(url) = valid_urls.next() {
                        entry.url = url;
                        entry.config = (&config).into();
                        let _ = db.init_entry(gid, entry).log_err("更新任务配置失败");
                    }
                },
            )
            .log_err("添加任务对话框启动失败");
        }
    });

    ui.global::<Logic>().on_view_log(|| {
        let _ = open::that(DB_DIR.as_os_str()).log_err("打开日志文件夹失败");
    });

    let _ = slint::spawn_local({
        let app = app.clone();
        async move {
            loop {
                app.task_set.wait_last().await;
                let Some(ui) = app.ui.upgrade() else { break };
                let visible = ui.window().is_visible();
                info!(main_window_visible = visible, "所有任务已完成");
                if !visible && app.db.is_exit_after_download() {
                    break;
                }
            }
            app.exit();
        }
    })
    .log_err("无法检测程序下载状态");

    let is_hidden = args.iter().any(|s| s == "--hidden");
    #[cfg(target_os = "linux")]
    {
        ui.show()?;
        if is_hidden {
            let ui = ui.as_weak();
            let _ = slint::spawn_local(async move {
                if let Some(ui) = ui.upgrade() {
                    let _ = ui.hide().log_err("隐藏窗口失败");
                }
            })
            .log_err("隐藏窗口失败");
        }
    }
    #[cfg(not(target_os = "linux"))]
    if !is_hidden {
        ui.show()?;
    }
    slint::run_event_loop_until_quit()?;
    Ok(())
}

/// 设置 UI 列表的各种过滤视图
fn setup_ui_lists(ui: &MainWindow, list_model: Rc<VecModel<EntryData>>) {
    ui.set_all_list(ModelRc::new(
        list_model.clone().sort_by(|a, b| b.gid.cmp(&a.gid)),
    ));
    let filter_view = |status: Status| {
        ModelRc::new(
            list_model
                .clone()
                .filter(move |e| e.status == status)
                .sort_by(|a, b| b.gid.cmp(&a.gid)),
        )
    };
    ui.set_running_list(filter_view(Status::Running));
    ui.set_waiting_list(filter_view(Status::Waiting));
    ui.set_paused_list(filter_view(Status::Paused));
    ui.set_completed_list(filter_view(Status::Completed));
    ui.set_error_list(filter_view(Status::Error));
}
