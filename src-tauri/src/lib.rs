use tauri::Manager;
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_log::{Target, TargetKind};

// fix someone who's linking libz without making sure it exist
#[cfg(unix)]
extern crate libz_sys;

mod download_error;
mod download_multi;
mod download_single;
mod event;
mod format_dir;
mod format_progress;
mod gen_unique_path;
mod log_if_err;
mod prefetch;
mod puller;
mod relaunch;
mod server;
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, cmd| {
            log::info!("[single_instance] {argv:?} {cmd:?}");
            let main_window = app.get_webview_window("main").expect("no main window");
            log_if_err!(main_window.show(), "failed to show main window");
            log_if_err!(main_window.unminimize(), "failed to unminimize main window");
            log_if_err!(
                main_window.set_focus(),
                "failed to set focus on main window"
            );
        }));
    }
    builder = builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_updater::Builder::new()
                .default_version_comparator(|current, update| update.version != current)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init());
    builder
        .setup(|app| {
            let main_window = app.get_webview_window("main").expect("no main window");
            log_if_err!(
                main_window.set_title(&format!("fast-down v{}", app.package_info().version)),
                "set title error"
            );
            log_if_err!(
                app.deep_link().register_all(),
                "register all deep_link error"
            );
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                log_if_err!(updater::update(handle).await, "update error");
            });
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                log_if_err!(server::start_server(handle).await, "server error");
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            prefetch::prefetch,
            download_multi::download_multi,
            download_single::download_single,
            format_dir::format_dir,
            gen_unique_path::gen_unique_path,
            relaunch::relaunch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
