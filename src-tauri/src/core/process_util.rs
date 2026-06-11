//! 跨平台子进程工具,Windows 下隐藏黑色命令行窗口

use std::process::{Command, Output, Stdio};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// 创建一个 Command(Windows 下默认带 CREATE_NO_WINDOW)
pub fn new_command(program: &str) -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new(program);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }
    #[cfg(not(windows))]
    {
        Command::new(program)
    }
}

/// 执行命令并获取输出(失败返回 error)
pub fn run_capture(program: &str, args: &[&str]) -> std::io::Result<Output> {
    let mut cmd = new_command(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.output()
}

/// 异步 spawn(不等待退出),用于 explorer 之类立即返回的进程
pub fn spawn_detached(program: &str, args: &[&str]) -> std::io::Result<std::process::Child> {
    let mut cmd = new_command(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd.spawn()
}
