/// 检查当前是否以管理员权限运行
#[cfg(target_os = "windows")]
pub fn is_admin() -> bool {
    use std::mem;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{
        GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = mem::size_of::<TOKEN_ELEVATION>() as u32;
        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        );
        let _ = CloseHandle(token);
        result.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// 检查当前是否以管理员权限运行（非 Windows 平台）
#[cfg(not(target_os = "windows"))]
pub fn is_admin() -> bool {
    false
}

/// 以管理员身份重启当前程序
#[cfg(target_os = "windows")]
pub fn restart_as_admin() -> color_eyre::Result<()> {
    use color_eyre::eyre::bail;
    use itertools::Itertools;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
    use windows::core::PCWSTR;

    let exe_path: Vec<u16> = std::env::current_exe()?
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let args = std::env::args()
        .skip(1)
        .map(|arg| format!("\"{}\"", arg.replace('\"', "\"\"")))
        .join(" ");
    let args: Vec<u16> = OsStr::new(&args).encode_wide().chain(Some(0)).collect();
    let cwd: Vec<u16> = std::env::current_dir()?
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();

    unsafe {
        let instance = ShellExecuteW(
            None,
            windows::core::w!("runas"),
            PCWSTR(exe_path.as_ptr()),
            PCWSTR(args.as_ptr()),
            PCWSTR(cwd.as_ptr()),
            SW_SHOW,
        );
        if instance.0 as usize > 32 {
            std::process::exit(0);
        } else {
            let err = std::io::Error::last_os_error();
            bail!("Failed to restart as admin (UAC denied or Error): {}", err);
        }
    }
}

/// 以管理员身份重启当前程序（非 Windows 平台暂不支持）
#[cfg(not(target_os = "windows"))]
pub fn restart_as_admin() -> color_eyre::Result<()> {
    bail!("Running as admin is not supported on this platform")
}

/// 尝试以管理员身份重启（如果需要）
pub fn try_restart_as_admin(run_as_admin: bool) -> color_eyre::Result<()> {
    #[cfg(target_os = "windows")]
    if run_as_admin && !is_admin() {
        tracing::info!("Restarting as administrator...");
        return restart_as_admin();
    }
    Ok(())
}
