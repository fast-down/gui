use crate::{ui::MainWindow, utils::LogErr};
use slint::ComponentHandle;

pub fn wakeup_window(ui: &MainWindow) {
    let window = ui.window();
    let _ = window.show().log_err("显示窗口出错");
    window.set_minimized(false);
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use windows::Win32::{Foundation::HWND, System::Threading::*, UI::WindowsAndMessaging::*};
        let handle = window.window_handle().window_handle().map(|h| h.as_raw());
        if let Ok(RawWindowHandle::Win32(win32_handle)) = handle {
            unsafe {
                let hwnd = HWND(win32_handle.hwnd.get() as *mut std::ffi::c_void);
                let foreground_hwnd = GetForegroundWindow();
                if foreground_hwnd != hwnd {
                    let foreground_thread = GetWindowThreadProcessId(foreground_hwnd, None);
                    let current_thread = GetCurrentThreadId();
                    if foreground_thread != current_thread {
                        let _ = AttachThreadInput(current_thread, foreground_thread, true);
                    }
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                    if foreground_thread != current_thread {
                        let _ = AttachThreadInput(current_thread, foreground_thread, false);
                    }
                } else {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                }
                let flash_info = FLASHWINFO {
                    cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
                    hwnd,
                    dwFlags: FLASHW_ALL | FLASHW_TIMERNOFG,
                    uCount: 2,
                    dwTimeout: 0,
                };
                let _ = FlashWindowEx(&flash_info);
            }
        }
    }
}
