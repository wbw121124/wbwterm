// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 模式 B：命名管道
//!
//! ## 流程
//! 1. Terminal 创建 NamedPipe Server（\\.\pipe\wbwterm-xxx）
//! 2. 等待子进程作为 NamedPipe Client 主动连接
//! 3. 连接成功后，双向字节流通信
//!
//! ## Windows API 依赖
//! - windows-sys::Win32::System::Pipes (CreateNamedPipeW, ConnectNamedPipe)
//!
//! ## 用途
//! - 子进程为自定义桥接程序（如 wbwterm-bridge.exe）
//! - 需要跨用户边界或远程主机连接的场景
//! - 标准 shell（bash/cmd）使用继承句柄模式（A）更简单

use interprocess::local_socket::{Listener, Stream};
use interprocess::local_socket::prelude::*;

/// 命名管道服务端配置
#[derive(Clone, Debug)]
pub struct NamedPipeConfig {
    /// 管道名称（不含 \\.\pipe\ 前缀）
    pub name: String,
}

impl NamedPipeConfig {
    pub fn new(suffix: &str) -> Self {
        Self {
            name: format!("wbwterm-{}", suffix),
        }
    }
}

/// 命名管道连接结果
#[derive(Debug)]
pub struct NamedPipeConnection {
    pub stream: Stream,
    pub pid: u32,
}

/// 创建命名管道服务端并等待客户端连接
///
/// ## 安全说明
/// - 管道名含随机后缀防止碰撞
/// - Windows 上命名管道路径为 \\.\pipe\wbwterm-xxx
/// - 管道权限默认允许当前用户读写
pub fn listen_named_pipe(_config: &NamedPipeConfig) -> Result<NamedPipeConnection, String> {
    // TODO: P1 实施阶段填充具体实现
    // 核心步骤：
    // 1. 使用 interprocess::local_socket::Listener::new() 创建服务端
    // 2. .accept() 等待客户端连接
    // 3. 返回 Stream（双向字节流）
    Err("命名管道模式 — P1 阶段待实现".to_string())
}

/// 命名管道客户端连接（供自定义子进程桥接使用）
///
/// 子进程主动连接 Terminal 创建的管道：
/// ```c
/// HANDLE hPipe = CreateFileW(L"\\\\.\\pipe\\wbwterm-xxx", ...);
/// ```
pub fn connect_named_pipe(_name: &str) -> Result<Stream, String> {
    // TODO: P1 实施阶段填充
    Err("命名管道客户端 — P1 阶段待实现".to_string())
}
