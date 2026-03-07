use crate::{core::App, os::wakeup_window};
use color_eyre::eyre::Context;
use tray_icon::{
    MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

pub fn setup_tray(app: App) -> color_eyre::Result<TrayIcon> {
    let icon = {
        let image = image::load_from_memory_with_format(
            include_bytes!("../../assets/icon.png"),
            image::ImageFormat::Png,
        )?
        .into_rgba8();
        let (width, height) = image.dimensions();
        tray_icon::Icon::from_rgba(image.into_raw(), width, height)
    }
    .context("无法加载托盘图片")?;
    let tray_menu = Menu::new();
    let show_item = MenuItem::new("显示主界面", true, None);
    let pause_all_item = MenuItem::new("全部暂停", true, None);
    let quit_item = MenuItem::new("退出", true, None);
    tray_menu
        .append_items(&[
            &show_item,
            &PredefinedMenuItem::separator(),
            &pause_all_item,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ])
        .context("无法创建托盘")?;
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("fast-down")
        .with_icon(icon)
        .with_menu(Box::new(tray_menu))
        .with_menu_on_left_click(false)
        .build()?;
    TrayIconEvent::set_event_handler(Some({
        let ui = app.ui.clone();
        move |event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                let _ = ui.upgrade_in_event_loop(|ui| {
                    wakeup_window(&ui);
                });
            }
            _ => {}
        }
    }));
    MenuEvent::set_event_handler(Some({
        let app = app.clone();
        let show_item_id = show_item.into_id();
        let pause_all_item_id = pause_all_item.into_id();
        let quit_item_id = quit_item.into_id();
        move |event: MenuEvent| {
            let id = event.id;
            if id == show_item_id {
                let _ = app.ui.upgrade_in_event_loop(|ui| {
                    wakeup_window(&ui);
                });
            } else if id == pause_all_item_id {
                app.task_set.cancel_all();
            } else if id == quit_item_id {
                app.exit();
            }
        }
    }));
    Ok(tray_icon)
}
