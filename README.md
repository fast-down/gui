<div align="center">
<h1>fast-down: 高性能多线程下载器</h1>

<h3>极速下载 · 超强重试 · 断点续传 · 增量续传</h3>

<p>
   <img src="https://img.shields.io/badge/Build with-Rust-DEA584?style=flat&logo=rust&logoColor=white" alt="Rust">
   <img src="https://img.shields.io/badge/Arch-x86__64%2C%20x86%2C%20ARM64-blue" alt="Hardware">
   <img src="https://img.shields.io/badge/OS-Windows%2C%20macOS%2C%20Linux-orange" alt="Hardware">
   <br>
   <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
   <img src="https://img.shields.io/github/last-commit/fast-down/gui/main" alt="Last commit">
   <img src="https://github.com/fast-down/gui/workflows/Test/badge.svg" alt="Test">
   <img src="https://img.shields.io/crates/v/fast-down-gui.svg" alt="Latest version">
   <br>
   <a href="https://fd.s121.top/"><img src="https://img.shields.io/badge/Website-fd.s121.top-blue?style=flat&logo=google-chrome&logoColor=white" alt="Website"></a>
<a href="https://dc.vacu.top/"><img src="https://img.shields.io/badge/Discord-Online-5865F2.svg?logo=discord&logoColor=white" alt="Discord"></a>
</p>

</div>

![gui 界面](https://fd.s121.top/gui.png)

**[访问官网](https://fd.s121.top/)**

## 特性

- **⚡️ 极速下载**  
  自研 [fast-steal](https://github.com/fast-down/core/tree/main/crates/fast-pull) 任务窃取算法，实测下载速度是 NDM 的 **2.43 倍**
- **🔄 超强重试**  
  下载时，切换 WiFi、关闭 WiFi、切换代理，都能保证**文件内容正确**
- **⛓️‍💥 断点续传**  
  下到一半**随时暂停**，之后还能**继续传输**
- **⛓️‍💥 增量续传**  
  服务器日志今天下载完成，明天又多了 1000 行，增量续传功能实现**只传输新增的 1000 行**
- **💰 开源免费**  
  所有代码全部公开，由 [share121](https://github.com/share121)、[Cyan](https://github.com/CyanChanges) 与其他贡献者一起维护
- **💻 跨平台**

## 下载

| 架构  | Windows   | Linux     | Mac OS    |
| ----- | --------- | --------- | --------- |
| 64 位 | [下载][1] | [下载][2] | [下载][3] |
| 32 位 | [下载][4] | [下载][8] | ❌ 不支持 |
| Arm64 | [下载][5] | [下载][6] | [下载][7] |

[1]: https://fast-down-update.s121.top/gui/download/latest/windows/64bit
[2]: https://fast-down-update.s121.top/gui/download/latest/linux/64bit
[3]: https://fast-down-update.s121.top/gui/download/latest/macos/64bit
[4]: https://fast-down-update.s121.top/gui/download/latest/windows/32bit
[5]: https://fast-down-update.s121.top/gui/download/latest/windows/arm64
[6]: https://fast-down-update.s121.top/gui/download/latest/linux/arm64
[7]: https://fast-down-update.s121.top/gui/download/latest/macos/arm64
[8]: https://fast-down-update.s121.top/gui/download/latest/linux/32bit
