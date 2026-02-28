use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use color_eyre::eyre::{Context, ContextCompat};
use std::env;

pub fn get_auto_start() -> color_eyre::Result<AutoLaunch> {
    let app_name = "fast-down";
    let app_path = env::current_exe().context("无法获取当前程序路径")?;
    let app_path = app_path.to_str().context("无法获取当前程序路径")?;
    AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(app_path)
        .set_args(&["--hidden"])
        .build()
        .context("不支持在当前系统上实现开机自启动")
}
