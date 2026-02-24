use crate::{
    core::{App, download},
    persist::{self, DatabaseEntry},
    ui::{Config, EntryData, Status},
    utils::{ForceSendExt, LogErr},
};
use fast_down::FileId;
use slint::VecModel;
use std::{path::PathBuf, time::Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use url::Url;

/// 返回 false 意味任务没有成功添加到 task_set 中
pub fn start_entry(app: &App, entry: &EntryData, list: &VecModel<EntryData>) -> bool {
    if matches!(entry.status, Status::Running | Status::Waiting) {
        return false;
    }
    let gid = entry.gid;
    let Some(db_entry) = app.db.inner.data.get(&gid).map(|e| e.clone()) else {
        return false;
    };
    let url = db_entry.url.clone();
    let config = db_entry.config.to_ui_config();
    if db_entry.status == persist::Status::Completed {
        start_new_entry(app, url, &config, list);
        return false;
    }
    let app_c = app.clone();
    let cancel_token = CancellationToken::new();
    let token = cancel_token.clone();
    let fut = async move {
        let handler = app_c.create_download_handler(gid);
        match download(url, &config, token, Some(db_entry), handler).await {
            Ok(()) => info!(gid = gid, "任务下载完成"),
            Err(e) => {
                error!(gid = gid, err = ?e, "下载任务出错");
                app_c.db.update_status(gid, persist::Status::Error);
                app_c.update_ui_row(gid, |_, data| data.status = Status::Error);
            }
        }
    }
    .force_send();
    app.task_set.add_task(gid, cancel_token, fut);
    true
}

pub fn start_new_entry(app: &App, url: Url, config: &Config, list_model: &VecModel<EntryData>) {
    let gid = app.db.next_gid();
    let entry = DatabaseEntry {
        file_name: url.to_string(),
        file_path: PathBuf::new(),
        file_size: 0,
        file_id: FileId::default(),
        progress: Vec::new(),
        elapsed: Duration::ZERO,
        url: url.clone(),
        config: config.into(),
        status: persist::Status::Paused,
    };
    let mut ui_entry = entry.to_entry_data(gid);
    ui_entry.status = Status::Waiting;
    list_model.push(ui_entry);
    let _ = app.db.init_entry(gid, entry).log_err("数据库插入条目失败");

    let app_c = app.clone();
    let cancel_token = CancellationToken::new();
    let token = cancel_token.clone();
    let config = config.clone();

    let fut = async move {
        let handler = app_c.create_download_handler(gid);
        match download(url, &config, token, None, handler).await {
            Ok(()) => info!(gid = gid, "任务下载完成"),
            Err(e) => {
                error!(gid = gid, err = ?e, "下载任务出错");
                app_c.db.update_status(gid, persist::Status::Error);
                app_c.update_ui_row(gid, |_, data| data.status = Status::Error);
            }
        }
    }
    .force_send();
    app.task_set.add_task(gid, cancel_token, fut);
}
