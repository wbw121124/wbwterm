// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 模式 A：继承句柄管道
//!
//! ## 流程
//! 1. CreatePipe() 创建匿名管道（inheritable）
//! 2. 设置 STARTUPINFOEXW.hStdInput / hStdOutput
//! 3. CreateProcessW() 启动子进程
//! 4. 子进程的 stdin/stdout 自动连接到管道
//!
//! ## Windows API 依赖
//! - windows-sys::Win32::System::Pipes (CreateNamedPipeW, ConnectNamedPipe)
//! - windows-sys::Win32::System::Threading (CreateProcessW, STARTUPINFOEXW)
//! - windows-sys::Win32::Security (SecurityAttributes, SetHandleInformation)

use std::os::windows::io::OwnedHandle;
use std::process::Command;

/// 继承句柄模式的管道配置
#[derive(Clone, Debug)]
pub struct InheritedPipeConfig {
    /// 子进程可执行文件路径
    pub exe: String,
    /// 命令行参数
    pub args: Vec<String>,
    /// 初始列数
    pub cols: u16,
    /// 初始行数
    pub rows: u16,
}

/// 继承句柄管道结果
#[derive(Debug)]
pub struct InheritedPipe {
    /// stdin 写入端（Terminal 写 → 子进程读）
    pub stdin: OwnedHandle,
    /// stdout/stderr 读取端（子进程写 → Terminal 读）
    pub stdout: OwnedHandle,
    /// 子进程 PID
    pub pid: u32,
}

/// 通过继承句柄启动子进程
///
/// ## Windows SDK 兼容说明
/// - CreateProcessW 在 Windows 2000 及以上可用
/// - STARTUPINFOEXW 在 Windows Vista 及以上可用
/// - 本项目目标：Windows 7+ (通过 windows-sys 0.59 最低支持版本)
pub fn spawn_inherited(config: &InheritedPipeConfig) -> Result<InheritedPipe, String> {
    // TODO: P1 实施阶段填充具体 Windows API 调用
    // 核心步骤：
    // 1. 调用 unsafe { winapi::um::winbase::CreatePipe(...) }
    // 2. 设置 SECURITY_ATTRIBUTES.bInheritHandle = TRUE
    // 3. 调用 winapi::um::winbase::SetHandleInformation(h, HANDLE_FLAGS_INHERIT, ...)
    // 4. 构建 STARTUPINFOEXW
    // 5. 调用 winapi::um::processthreadsapi::CreateProcessW(...)
    Err("继承句柄模式 — P1 阶段待实现".to_string())
}

/// 构建子进程 Command（供测试使用）
pub fn build_command(config: &InheritedPipeConfig) -> Command {
    let mut cmd = Command::new(&config.exe);
    cmd.args(&config.args);
    cmd
}
