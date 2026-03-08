use crate::ipc::{IpcMessage, NS_NAME};
use interprocess::local_socket::{
    GenericNamespaced,
    tokio::{Stream, prelude::*},
};
use std::{
    io::{ErrorKind, Read, Write},
    process::{Command, Stdio, exit},
};
use tokio::io::AsyncWriteExt;

// pub const CHROME_EXT_ID: &str = "bcfnnnjblfknledeialnibeiflklcefk";
pub const CHROME_EXT_ID: &str = "mmblnfdpbgoeicbbnomdemlocdacecdf";
pub const FIREFOX_EXT_ID: &str = "fast-down@s121.top";

/// 全平台自动注册 Native Messaging
pub fn auto_register() -> color_eyre::Result<()> {
    let exe_path = std::env::current_exe()?;
    let manifest_json = serde_json::json!({
        "name": "top.s121.fd",
        "description": "fast-down native messaging host",
        "path": exe_path.to_string_lossy(),
        "type": "stdio",
        "allowed_origins": [
            format!("chrome-extension://{}/", CHROME_EXT_ID),
        ],
        "allowed_extensions": [
            FIREFOX_EXT_ID
        ]
    });
    let manifest_json = serde_json::to_string_pretty(&manifest_json)?;

    #[cfg(target_os = "windows")]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        let manifest_path = crate::persist::DB_DIR.join("fd_nm_manifest.json");
        std::fs::write(&manifest_path, &manifest_json)?;
        let manifest_path = manifest_path.to_string_lossy();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let paths = [
            "Software\\Google\\Chrome\\NativeMessagingHosts\\top.s121.fd",
            "Software\\Microsoft\\Edge\\NativeMessagingHosts\\top.s121.fd",
            "Software\\Mozilla\\NativeMessagingHosts\\top.s121.fd",
        ];
        for path in paths {
            if let Ok((key, _)) = hkcu.create_subkey(path) {
                let _ = key.set_value("", &manifest_path.as_ref());
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let base = home.join("Library/Application Support");
            let paths = [
                base.join("Google/Chrome/NativeMessagingHosts"),
                base.join("Microsoft Edge/NativeMessagingHosts"),
                base.join("Mozilla/NativeMessagingHosts"),
            ];
            write_manifests_to_dirs(&paths, &manifest_json);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            let config = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
            let paths = [
                config.join("google-chrome/NativeMessagingHosts"),
                config.join("chromium/NativeMessagingHosts"),
                config.join("microsoft-edge/NativeMessagingHosts"),
                home.join(".mozilla/native-messaging-hosts"),
            ];
            write_manifests_to_dirs(&paths, &manifest_json);
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
                    Command::new(exe_path)
                        .arg("--hidden")
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?;
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
    exit(0);
}
