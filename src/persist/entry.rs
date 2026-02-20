use crate::{
    fmt::{format_size, format_time},
    persist::Config,
    ui::EntryData,
};
use fast_down::{FileId, Total};
use serde::{Deserialize, Serialize};
use slint::{SharedString, VecModel};
use std::{ops::Range, path::PathBuf, rc::Rc, time::Duration};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DatabaseEntry {
    pub file_name: String,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub file_id: FileId,
    pub progress: Vec<Range<u64>>,
    pub elapsed: Duration,
    pub url: Url,
    pub config: Config,
    pub status: Status,
}

impl DatabaseEntry {
    pub fn to_entry_data(&self, gid: i32) -> EntryData {
        let downloaded: u64 = self.progress.total();
        let file_size = self.file_size;
        let elapsed = self.elapsed;
        let speed = downloaded as f64 / elapsed.as_secs_f64();
        let speed_str: SharedString = format!("{}/s", format_size(speed)).into();
        let remaining_size = file_size.saturating_sub(downloaded) as f64;
        EntryData {
            avg_speed: speed_str.clone(),
            downloaded: format_size(downloaded as f64).into(),
            filename: self.file_name.as_str().into(),
            gid,
            path: self.file_path.to_string_lossy().as_ref().into(),
            percentage: format!("{}%", downloaded as f64 / file_size as f64 * 100.).into(),
            progress: if file_size > 0 {
                Rc::new(VecModel::from_iter(self.progress.iter().map(|r| {
                    crate::ui::Progress {
                        start: r.start as f32 / file_size as f32,
                        width: r.total() as f32 / file_size as f32,
                    }
                })))
                .into()
            } else {
                Rc::new(VecModel::from_iter([])).into()
            },
            remaining_size: format_size(remaining_size).into(),
            remaining_time: format_time((remaining_size / speed) as u64).into(),
            speed: speed_str,
            status: match self.status {
                Status::Completed => crate::ui::Status::Completed,
                Status::Error => crate::ui::Status::Error,
                Status::Paused => crate::ui::Status::Paused,
            },
            time: format_time(elapsed.as_secs()).into(),
            total: format_size(file_size as f64).into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Status {
    Completed,
    Error,
    Paused,
}
