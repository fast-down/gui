mod download_multi;
mod download_single;
mod event;
mod format_dir;
mod gen_unique_path;
mod prefetch;
mod puller;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        // .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init());
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
            use tauri::Manager;
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }
    builder
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            prefetch::prefetch,
            download_multi::download_multi,
            download_single::download_single,
            format_dir::format_dir,
            gen_unique_path::gen_unique_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
