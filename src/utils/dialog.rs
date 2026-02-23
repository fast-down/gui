use crate::{
    ui::{Config, DialogType, TaskDialog},
    utils::{LogErr, get_font_family},
};
use rfd::FileDialog;
use slint::{ComponentHandle, SharedString, ToSharedString};

/// 显示添加任务对话框
pub fn show_task_dialog(
    urls: SharedString,
    dialog_type: DialogType,
    config: Config,
    on_comfirm: impl FnOnce(SharedString, Config) + 'static,
) -> color_eyre::Result<()> {
    let dialog = TaskDialog::new()?;
    dialog.set_urls(urls);
    dialog.set_type(dialog_type);
    dialog.set_config(config);

    let dialog_weak = dialog.as_weak();
    let hide_dialog = move || {
        let _ = dialog_weak.upgrade_in_event_loop(|d| {
            let _ = d.hide().log_err("隐藏窗口失败");
        });
    };

    dialog.on_canceled(hide_dialog.clone());

    dialog.on_browse_folder({
        let dialog = dialog.as_weak();
        move || {
            let dialog = dialog.clone();
            std::thread::spawn(move || {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    let _ = dialog.upgrade_in_event_loop(move |d| {
                        d.invoke_set_save_dir(folder.to_string_lossy().to_shared_string());
                    });
                }
            });
        }
    });

    let mut handle = Some(on_comfirm);
    dialog.on_comfirm(move |urls, config| {
        hide_dialog();
        if let Some(h) = handle.take() {
            h(urls, config);
        }
    });

    dialog.set_font_family(get_font_family().into());
    dialog.show()?;
    Ok(())
}
