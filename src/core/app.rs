use crate::{
    core::{DownloadEvent, TaskSet, apply_progress_diff},
    fmt::format_size,
    persist::{self, Database},
    ui::{self, DownloadConfig, EntryData, GeneralConfig, MainWindow},
    utils::LogErr,
};
use auto_launch::AutoLaunch;
use slint::{Model, SharedString, Weak};
use std::process::exit;

#[derive(Clone)]
pub struct App {
    pub db: Database,
    pub task_set: TaskSet<i32>,
    pub ui: Weak<MainWindow>,
}

impl App {
    pub fn update_ui_row<F>(&self, gid: i32, mutator: F)
    where
        F: FnOnce(usize, &mut EntryData) + Send + 'static,
    {
        let _ = self.ui.upgrade_in_event_loop(move |ui| {
            let list_model = ui.get_all_list();
            for (row, mut data) in list_model.iter().enumerate() {
                if data.gid == gid {
                    mutator(row, &mut data);
                    list_model.set_row_data(row, data);
                    break;
                }
            }
        });
    }

    /// 创建下载过程中的事件处理器
    pub fn create_download_handler(
        &self,
        gid: i32,
    ) -> impl FnMut(DownloadEvent) + Send + Sync + 'static {
        let app = self.clone();
        let mut file_size = 0;
        move |event| match event {
            DownloadEvent::Info(info) => {
                file_size = info.file_size;
                let _ = app
                    .db
                    .init_entry(gid, *info.clone())
                    .log_err("数据库插入条目失败");
                app.update_ui_row(gid, move |_, data| {
                    data.status = ui::Status::Running;
                    data.filename = info.file_name.into();
                    data.path = info.file_path.to_string_lossy().as_ref().into();
                    data.total = format_size(info.file_size as f64).into();
                });
            }
            DownloadEvent::Progress(p) => {
                app.db.update_entry(gid, p.progress.clone(), p.elapsed);
                app.update_ui_row(gid, move |_, data| {
                    data.downloaded = p.downloaded;
                    data.speed = p.speed;
                    data.avg_speed = p.avg_speed;
                    data.percentage = p.percentage;
                    data.remaining_time = p.remaining_time;
                    data.remaining_size = p.remaining_size;
                    data.time = p.time;
                    if file_size > 0 {
                        data.progress = apply_progress_diff(&data.progress, &p.progress, file_size);
                    }
                });
            }
            DownloadEvent::Flushing => {
                app.update_ui_row(gid, move |_, data| {
                    data.error = "文件内容已可用，但请勿关机，等待落盘中".into();
                });
            }
            DownloadEvent::FlushError(e) => {
                app.update_ui_row(gid, move |_, data| {
                    data.status = ui::Status::Error;
                    data.error = e;
                });
            }
            DownloadEvent::End { is_cancelled } => {
                let db_status = if is_cancelled {
                    persist::Status::Paused
                } else {
                    persist::Status::Completed
                };
                let ui_status = if is_cancelled {
                    ui::Status::Paused
                } else {
                    ui::Status::Completed
                };
                app.db.update_status(gid, db_status);
                app.update_ui_row(gid, move |_, data| {
                    data.status = ui_status;
                    data.error = SharedString::default();
                });
            }
        }
    }

    pub fn set_config(
        &self,
        download_config: DownloadConfig,
        general_config: GeneralConfig,
        auto: Option<&AutoLaunch>,
    ) {
        self.task_set
            .set_concurrency(general_config.max_concurrency as usize);
        self.db.set_download_config(&download_config);
        self.db.set_general_config(&general_config);
        if let Some(auto) = auto {
            if general_config.auto_start {
                let _ = auto.enable().log_err("启用开机自启失败");
            } else {
                let _ = auto.disable().log_err("禁用开机自启失败");
            }
        }
        let _ = self.ui.upgrade_in_event_loop(move |ui| {
            ui.set_download_config(download_config);
            ui.set_general_config(general_config);
        });
    }

    pub fn exit(&self) {
        let db = self.db.clone();
        let fut = tokio::task::spawn_blocking(move || db.flush_force_sync());
        let task_set = self.task_set.clone();
        task_set.cancel_all();
        tokio::spawn(async move {
            task_set.join().await;
            fut.await.unwrap().unwrap();
            exit(0);
        });
    }
}
