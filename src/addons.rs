use crate::ipc::{IpcMessage, NS_NAME};
use interprocess::local_socket::{
    GenericNamespaced,
    tokio::{Stream, prelude::*},
};
use std::{
    io::{ErrorKind, Read, Write},
    process::{Command, Stdio},
};
use tokio::io::AsyncWriteExt;

// pub const CHROME_EXT_ID: &str = "bcfnnnjblfknledeialnibeiflklcefk";
pub const CHROME_EXT_ID: &str = "mmblnfdpbgoeicbbnomdemlocdacecdf";
pub const FIREFOX_EXT_ID: &str = "fast-down@s121.top";

/// 全平台自动注册 Native Messaging
pub fn auto_register() -> color_eyre::Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    // 基础通用的配置
    let manifest_base = serde_json::json!({
        "name": "top.s121.fd",
        "description": "fast-down native messaging host",
        "path": exe_path_str,
        "type": "stdio"
    });

    // 1. 生成 Chrome/Edge 专属版本 (使用 allowed_origins)
    let mut manifest_chrome = manifest_base.clone();
    manifest_chrome["allowed_origins"] =
        serde_json::json!([format!("chrome-extension://{}/", CHROME_EXT_ID),]);
    let chrome_json = serde_json::to_string_pretty(&manifest_chrome)?;

    // 2. 生成 Firefox 专属版本 (使用 allowed_extensions)
    let mut manifest_firefox = manifest_base;
    manifest_firefox["allowed_extensions"] = serde_json::json!([FIREFOX_EXT_ID]);
    let firefox_json = serde_json::to_string_pretty(&manifest_firefox)?;

    #[cfg(target_os = "windows")]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        // Windows 需要分别写入两个文件
        let chrome_manifest_path = crate::persist::DB_DIR.join("fd_nm_manifest_chrome.json");
        let firefox_manifest_path = crate::persist::DB_DIR.join("fd_nm_manifest_firefox.json");

        std::fs::write(&chrome_manifest_path, &chrome_json)?;
        std::fs::write(&firefox_manifest_path, &firefox_json)?;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // 为 Chrome 和 Edge 写入 Chrome 版本的 manifest 路径
        let chrome_paths = [
            "Software\\Google\\Chrome\\NativeMessagingHosts\\top.s121.fd",
            "Software\\Microsoft\\Edge\\NativeMessagingHosts\\top.s121.fd",
        ];
        for path in chrome_paths {
            if let Ok((key, _)) = hkcu.create_subkey(path) {
                let _ = key.set_value("", &chrome_manifest_path.to_string_lossy().as_ref());
            }
        }

        // 为 Firefox 写入 Firefox 版本的 manifest 路径
        let firefox_path = "Software\\Mozilla\\NativeMessagingHosts\\top.s121.fd";
        if let Ok((key, _)) = hkcu.create_subkey(firefox_path) {
            let _ = key.set_value("", &firefox_manifest_path.to_string_lossy().as_ref());
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let base = home.join("Library/Application Support");

            // 写入 Chrome / Edge 目录
            let chrome_paths = [
                base.join("Google/Chrome/NativeMessagingHosts"),
                base.join("Microsoft Edge/NativeMessagingHosts"),
            ];
            write_manifests_to_dirs(&chrome_paths, &chrome_json);

            // 写入 Firefox 目录
            let firefox_paths = [base.join("Mozilla/NativeMessagingHosts")];
            write_manifests_to_dirs(&firefox_paths, &firefox_json);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            let config = dirs::config_dir().unwrap_or_else(|| home.join(".config"));

            // 写入 Chrome / Edge 目录
            let chrome_paths = [
                config.join("google-chrome/NativeMessagingHosts"),
                config.join("chromium/NativeMessagingHosts"),
                config.join("microsoft-edge/NativeMessagingHosts"),
            ];
            write_manifests_to_dirs(&chrome_paths, &chrome_json);

            // 写入 Firefox 目录
            let firefox_paths = [home.join(".mozilla/native-messaging-hosts")];
            write_manifests_to_dirs(&firefox_paths, &firefox_json);
        }
    }

    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn write_manifests_to_dirs(dirs: &[std::path::PathBuf], json: &str) {
    for dir in dirs {
        if std::fs::create_dir_all(dir).is_ok() {
            let file_path = dir.join("top.s121.fd.json");
            let _ = std::fs::write(file_path, json);
        }
    }
}

/// 读取浏览器从 stdin 传来的数据
fn read_native_message() -> Option<crate::ipc::IpcMessage> {
    let mut stdin = std::io::stdin().lock();
    let mut len_bytes = [0u8; 4];
    stdin.read_exact(&mut len_bytes).ok()?;
    let len = u32::from_ne_bytes(len_bytes) as usize;
    let mut buffer = vec![0u8; len];
    stdin.read_exact(&mut buffer).ok()?;
    serde_json::from_slice(&buffer).ok()
}

/// 给浏览器回复成功标识
fn write_native_message<T: serde::Serialize>(msg: &T) {
    if let Ok(json) = serde_json::to_string(msg) {
        let len = json.len() as u32;
        let mut stdout = std::io::stdout().lock();
        let _ = stdout.write_all(&len.to_ne_bytes());
        let _ = stdout.write_all(json.as_bytes());
        let _ = stdout.flush();
    }
}

/// 作为代理进程，接管浏览器的请求并转交给主程序
pub async fn handle_browser_request() -> color_eyre::Result<()> {
    let payload = read_native_message().unwrap_or(IpcMessage::WakeUp);
    let msg = serde_json::to_string(&payload)?;
    let ns_name = NS_NAME.to_ns_name::<GenericNamespaced>()?;

    let mut retries = 0;
    let mut stream = loop {
        match Stream::connect(ns_name.clone()).await {
            Ok(s) => break s,
            Err(e) if matches!(e.kind(), ErrorKind::ConnectionRefused | ErrorKind::NotFound) => {
                if retries == 0 {
                    let exe_path = std::env::current_exe()?;
                    let mut cmd = Command::new(exe_path);
                    cmd.arg("--hidden")
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null());

                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x01000000;
                        const DETACHED_PROCESS: u32 = 0x00000008;
                        cmd.creation_flags(CREATE_BREAKAWAY_FROM_JOB | DETACHED_PROCESS);
                    }

                    cmd.spawn()?;
                }
                if retries > 10 {
                    return Err(e.into());
                }
                retries += 1;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            Err(e) => return Err(e.into()),
        }
    };

    stream.write_all(format!("{msg}\n").as_bytes()).await?;
    write_native_message(&serde_json::json!({"status": "success"}));
    Ok(())
}
