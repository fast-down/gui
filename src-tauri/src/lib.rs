use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

mod download_multi;
mod download_single;
mod event;
mod format_dir;
mod format_progress;
mod gen_unique_path;
mod prefetch;
mod puller;
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_updater::Builder::new()
                .default_version_comparator(|current, update| update.version != current)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init());
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let main_window = app.get_webview_window("main").expect("no main window");
            let _ = main_window.unminimize();
            let _ = main_window.set_focus();
        }));
    }
    builder
        .setup(|app| {
            let main_window = app.get_webview_window("main").expect("no main window");
            let _ = main_window.set_title(&format!("fast-down v{}", app.package_info().version));
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = updater::update(handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            prefetch::prefetch,
            download_multi::download_multi,
            download_single::download_single,
            format_dir::format_dir,
            gen_unique_path::gen_unique_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
