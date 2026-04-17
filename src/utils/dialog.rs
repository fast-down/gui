use crate::ui::{DialogType, DownloadConfig, TaskDialog};
use crate::utils::LogErr;
use slint::{ComponentHandle, SharedString, ToSharedString};

/// 显示添加任务对话框
pub fn show_task_dialog(
    urls: SharedString,
    dialog_type: DialogType,
    config: DownloadConfig,
    show_bg_download: bool,
    on_confirm: impl FnOnce(SharedString, DownloadConfig, bool) + 'static,
) -> color_eyre::Result<()> {
    let dialog = TaskDialog::new()?;
    dialog.set_urls(urls);
    dialog.set_type(dialog_type);
    dialog.set_show_bg_download(show_bg_download);
    dialog.set_download_config(config);

    let dialog_weak = dialog.as_weak();
    let hide_dialog = move || {
        if let Some(d) = dialog_weak.upgrade() {
            let _ = d.hide().log_err("隐藏窗口失败");
        }
    };
    dialog.on_canceled(hide_dialog.clone());

    dialog.on_browse_folder({
        let dialog_weak = dialog.as_weak();
        move || {
            let dialog_weak = dialog_weak.clone();
            let _ = slint::spawn_local(async move {
                if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await
                    && let Some(d) = dialog_weak.upgrade()
                {
                    d.invoke_set_save_dir(folder.path().to_string_lossy().to_shared_string());
                }
            });
        }
    });

    let mut handle = Some(on_confirm);
    dialog.on_confirm(move |urls, config, bg_download| {
        hide_dialog();
        if let Some(h) = handle.take() {
            h(urls, config, bg_download);
        }
    });

    dialog.show()?;
    Ok(())
}
