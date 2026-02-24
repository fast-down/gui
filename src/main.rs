#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::Clipboard;
use fast_down_gui::{
    core::{App, TaskSet, start_entry, start_new_entry},
    ipc::{check_ipc, init_ipc},
    os::{attach_console, get_font_family},
    persist::{DB_DIR, Database},
    server::start_server,
    ui::*,
    utils::{LogErr, show_task_dialog},
};
use rfd::FileDialog;
use slint::{Model, ModelRc, ToSharedString, VecModel};
use std::{collections::HashSet, process::exit, rc::Rc};
use tracing::{info, level_filters::LevelFilter};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
    attach_console();
    let _guard = init_tracing();

    let _ = check_ipc().await.log_err("检查 ipc 通道错误");
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .select()?;
    let ui = MainWindow::new()?;
    let _ = init_ipc(ui.as_weak()).await.log_err("初始化 ipc 通道错误");

    let db = Database::new().await;
    let task_set = TaskSet::new(db.inner.config.lock().max_concurrency);

    let entries = db.inner.data.iter().map(|e| e.to_entry_data(*e.key()));
    let list_model = Rc::new(VecModel::from_iter(entries));

    let app = App {
        db: db.clone(),
        task_set: task_set.clone(),
        ui: ui.as_weak(),
    };

    start_server(app.clone(), list_model.clone(), ui.as_weak()).await?;
    setup_ui_lists(&ui, list_model.clone());
    ui.set_config(db.get_ui_config());
    ui.set_version(VERSION.into());

    ui.on_exit({
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

    ui.on_browse_folder({
        let ui = ui.as_weak();
        move || {
            let ui = ui.clone();
            std::thread::spawn(move || {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    let _ = ui.upgrade_in_event_loop(move |ui| {
                        ui.invoke_set_save_dir(folder.to_string_lossy().to_shared_string());
                    });
                }
            });
        }
    });

    ui.on_config_change({
        let db = db.clone();
        let task_set = task_set.clone();
        let ui = ui.as_weak();
        move |c| {
            info!(config = ?c, "配置已更新");
            task_set.set_concurrency(c.max_concurrency as usize);
            db.set_config(&c);
            let _ = ui.upgrade_in_event_loop(move |ui| ui.set_config(c));
        }
    });

    ui.on_start_all({
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
    ui.on_start_entry({
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

    ui.on_pause_all({
        let task_set = task_set.clone();
        move |list| {
            for entry in list.iter() {
                task_set.cancel_task(&entry.gid);
            }
        }
    });
    ui.on_pause_entry({
        let task_set = task_set.clone();
        move |gid| {
            task_set.cancel_task(&gid);
        }
    });

    ui.on_remove_all({
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
    ui.on_remove_entry({
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

    ui.on_add_task({
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
                db.get_ui_config(),
                move |urls, config| {
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

    ui.on_open_entry(|path| {
        let _ = open::that(path).log_err("打开文件失败");
    });
    ui.on_open_folder_entry(showfile::show_path_in_file_manager);

    ui.on_detail_entry({
        let db = app.db.clone();
        move |gid| {
            let Some(mut entry) = db.inner.data.get(&gid).map(|e| e.clone()) else {
                return;
            };
            let db = db.clone();
            let _ = show_task_dialog(
                entry.url.to_shared_string(),
                DialogType::EditTask,
                entry.config.to_ui_config(),
                move |urls, config| {
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

    ui.on_view_log(|| {
        let _ = open::that(DB_DIR.as_os_str()).log_err("打开日志文件夹失败");
    });

    ui.set_font_family(get_font_family().into());
    ui.show()?;
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
