use crate::{AppData, log_if_err};
use tauri::{AppHandle, Manager, process};

#[tauri::command]
pub async fn relaunch(app: AppHandle) {
    let data = app.state::<AppData>();
    if let Some(sender) = data.shutdown_sender.lock().take() {
        log_if_err!(sender.send(()), "Failed to send shutdown signal");
    }
    if let Some(receiver) = data.shutdown_finished_receiver.lock().take() {
        log_if_err!(receiver.await, "Failed to receive shutdown signal");
    }
    let mut env = app.env();
    env.args_os.truncate(0);
    process::restart(&env)
}
