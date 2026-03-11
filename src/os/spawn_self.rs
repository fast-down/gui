use std::process::{Command, Stdio};

pub async fn spawn_self() -> color_eyre::Result<()> {
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
    Ok(())
}
