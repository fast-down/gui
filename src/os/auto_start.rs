use auto_launch::AutoLaunchBuilder;
use color_eyre::eyre::{Context, ContextCompat};
use std::env;

pub fn update_auto_start() -> color_eyre::Result<()> {
    let app_name = "fast-down";
    let app_path = env::current_exe().context("无法获取当前程序路径")?;
    let app_path = app_path.to_str().context("无法获取当前程序路径")?;
    let auto = AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(app_path)
        .set_args(&["--hidden"])
        .build()
        .context("不支持在当前系统上实现开机自启动")?;
    auto.enable().context("更新开机自启路径失败")
}
