use crate::{ui::MainWindow, utils::LogErr};
use i_slint_backend_winit::{WinitWindowAccessor, winit::window::UserAttentionType};
use slint::ComponentHandle;

pub fn wakeup_window(ui: &MainWindow) {
    let window = ui.window();
    let _ = window.show().log_err("显示窗口出错");
    window.set_minimized(false);
    ui.window().with_winit_window(|winit_window| {
        winit_window.focus_window();
        winit_window.request_user_attention(Some(UserAttentionType::Critical));
    });
}
