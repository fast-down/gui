use tauri::{AppHandle, Manager, process};

#[tauri::command]
pub fn relaunch(app: AppHandle) {
    let mut env = app.env();
    env.args_os.truncate(0);
    process::restart(&env)
}
