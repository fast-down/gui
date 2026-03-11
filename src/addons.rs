use crate::{
    ipc::{IpcMessage, NS_NAME},
    os::spawn_self,
};
use interprocess::local_socket::{
    GenericNamespaced,
    tokio::{Stream, prelude::*},
};
use std::io::{ErrorKind, Read, Write};
use tokio::io::AsyncWriteExt;

pub const APP_NAME: &str = "top.s121.fd";
pub const CHROME_EXT_IDS: &[&str] = &[
    "bcfnnnjblfknledeialnibeiflklcefk", // Edge 商店 ID
    "egbcpdbchfloplcckfdknckhfikicidm", // 本地开发的固定 ID
];
pub const FIREFOX_EXT_ID: &str = "fast-down@s121.top";

/// 全平台自动注册 Native Messaging
pub fn auto_register() -> color_eyre::Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    // 基础通用的配置
    let manifest_base = serde_json::json!({
        "name": APP_NAME,
        "description": "fast-down native messaging host",
        "path": exe_path_str,
        "type": "stdio",
    });

    // 1. 生成 Chromium 系专属版本 (使用 allowed_origins)
    let mut manifest_chrome = manifest_base.clone();
    let allowed_origins: Vec<String> = CHROME_EXT_IDS
        .iter()
        .map(|id| format!("chrome-extension://{}/", id))
        .collect();
    manifest_chrome["allowed_origins"] = serde_json::to_value(allowed_origins)?;
    let chrome_json = serde_json::to_string_pretty(&manifest_chrome)?;

    // 2. 生成 Firefox 系专属版本 (使用 allowed_extensions)
    let mut manifest_firefox = manifest_base;
    manifest_firefox["allowed_extensions"] = serde_json::json!([FIREFOX_EXT_ID]);
    let firefox_json = serde_json::to_string_pretty(&manifest_firefox)?;

    register(&chrome_json, &firefox_json)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn register(chrome_json: &str, firefox_json: &str) -> color_eyre::Result<()> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    // 1. 将 manifest 写入到磁盘
    let chrome_manifest_path = crate::persist::DB_DIR.join("fd_nm_manifest_chrome.json");
    let firefox_manifest_path = crate::persist::DB_DIR.join("fd_nm_manifest_firefox.json");
    std::fs::write(&chrome_manifest_path, chrome_json)?;
    std::fs::write(&firefox_manifest_path, firefox_json)?;

    // 2. 写入注册表
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // 所有 Chromium 内核浏览器
    let chrome_paths = [
        "Software\\Google\\Chrome\\NativeMessagingHosts",
        "Software\\Microsoft\\Edge\\NativeMessagingHosts",
        "Software\\Chromium\\NativeMessagingHosts",
        "Software\\BraveSoftware\\Brave-Browser\\NativeMessagingHosts",
        "Software\\Vivaldi\\NativeMessagingHosts",
    ];

    for path in chrome_paths {
        let full_path = format!("{}\\{}", path, APP_NAME);
        if let Ok((key, _)) = hkcu.create_subkey(&full_path) {
            let _ = key.set_value("", &chrome_manifest_path.to_string_lossy().as_ref());
        }
    }

    // 所有 Firefox 内核浏览器
    let firefox_paths = [
        "Software\\Mozilla\\NativeMessagingHosts",
        "Software\\Waterfox\\NativeMessagingHosts",
        "Software\\LibreWolf\\NativeMessagingHosts",
    ];

    for path in firefox_paths {
        let full_path = format!("{}\\{}", path, APP_NAME);
        if let Ok((key, _)) = hkcu.create_subkey(&full_path) {
            let _ = key.set_value("", &firefox_manifest_path.to_string_lossy().as_ref());
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn register(chrome_json: &str, firefox_json: &str) -> color_eyre::Result<()> {
    use color_eyre::eyre::ContextCompat;

    let home = dirs::home_dir().context("无法获取 home 目录")?;
    let base = home.join("Library/Application Support");

    let chrome_paths = [
        base.join("Google/Chrome/NativeMessagingHosts"),
        base.join("Microsoft Edge/NativeMessagingHosts"),
        base.join("Chromium/NativeMessagingHosts"),
        base.join("BraveSoftware/Brave-Browser/NativeMessagingHosts"),
        base.join("Vivaldi/NativeMessagingHosts"),
    ];
    write_manifests_to_dirs(&chrome_paths, chrome_json);

    let firefox_paths = [
        base.join("Mozilla/NativeMessagingHosts"),
        base.join("Waterfox/NativeMessagingHosts"),
    ];
    write_manifests_to_dirs(&firefox_paths, firefox_json);

    Ok(())
}

#[cfg(target_os = "linux")]
fn register(chrome_json: &str, firefox_json: &str) -> color_eyre::Result<()> {
    use color_eyre::eyre::ContextCompat;

    let home = dirs::home_dir().context("无法获取 home 目录")?;
    let config = dirs::config_dir().unwrap_or_else(|| home.join(".config"));

    let chrome_paths = [
        config.join("google-chrome/NativeMessagingHosts"),
        config.join("chromium/NativeMessagingHosts"),
        config.join("microsoft-edge/NativeMessagingHosts"),
        config.join("BraveSoftware/Brave-Browser/NativeMessagingHosts"),
        config.join("vivaldi/NativeMessagingHosts"),
    ];
    write_manifests_to_dirs(&chrome_paths, chrome_json);

    let firefox_paths = [
        home.join(".mozilla/native-messaging-hosts"),
        home.join(".waterfox/native-messaging-hosts"),
        home.join(".librewolf/native-messaging-hosts"),
    ];
    write_manifests_to_dirs(&firefox_paths, firefox_json);

    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn write_manifests_to_dirs(dirs: &[std::path::PathBuf], json: &str) {
    let filename = format!("{}.json", APP_NAME);
    for dir in dirs {
        if std::fs::create_dir_all(dir).is_ok() {
            let file_path = dir.join(&filename);
            let _ = std::fs::write(file_path, json);
        }
    }
}

/// 读取浏览器从 stdin 传来的数据
fn read_native_message() -> Option<IpcMessage> {
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
                    spawn_self().await?;
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
