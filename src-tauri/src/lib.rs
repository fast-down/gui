mod download_multi;
mod download_single;
mod event;
mod format_dir;
mod prefetch;
mod reader;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            format_dir::format_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
