// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 管道通信层 — 命名管道 + 继承句柄两种模式

pub mod inherited;
pub mod named_pipe;
pub mod session;

use super::terminal::events::TerminalEvents;
use super::terminal::TerminalEngine;
use std::process::Command;

/// 管道通信模式
#[derive(Clone, Debug, Default)]
pub enum PipeMode {
    #[default]
    Inherited, // 模式 A：继承句柄
    NamedPipe, // 模式 B：命名管道
}

/// 管道会话配置
#[derive(Clone, Debug)]
pub struct PipeConfig {
    pub mode: PipeMode,
    pub command: Vec<String>,
    pub cols: u16,
    pub rows: u16,
}

impl PipeConfig {
    pub fn new(command: Vec<String>, cols: u16, rows: u16) -> Self {
        Self {
            mode: PipeMode::default(),
            command,
            cols,
            rows,
        }
    }
}

/// 管道启动结果
#[derive(Debug)]
pub enum PipeResult {
    /// 子进程成功启动，返回读写句柄
    Ok {
        /// 子进程的 stdin 写入端（Terminal 写→子进程读）
        stdin: std::os::windows::io::OwnedHandle,
        /// 子进程的 stdout/stderr 读取端（子进程写→Terminal 读）
        stdout: std::os::windows::io::OwnedHandle,
        /// 子进程 PID（用于生命周期管理）
        pid: u32,
    },
    /// 命名管道客户端连接
    NamedPipeConnected {
        stream: interprocess::local_socket::Stream,
        pid: u32,
    },
    /// 启动失败
    Error(String),
}

/// 管道管理器（P1 阶段骨架）
///
/// TODO: 实现 inherited.rs 和 named_pipe.rs
/// 当前仅保留接口定义，待 P1 实施阶段填充
pub struct PipeManager {
    pub config: PipeConfig,
    pub engine: Option<TerminalEngine>,
    pub events: TerminalEvents,
}

impl PipeManager {
    pub fn new(config: PipeConfig) -> Self {
        let engine = TerminalEngine::new(config.cols, config.rows);
        Self {
            config,
            engine: Some(engine),
            events: TerminalEvents::new(),
        }
    }

    /// 根据配置启动子进程，返回管道句柄
    pub fn spawn(&mut self) -> PipeResult {
        match self.config.mode {
            PipeMode::Inherited => {
                // TODO: 调用 inherited::spawn_inherited()
                PipeResult::Error("继承句柄模式尚未实现".to_string())
            }
            PipeMode::NamedPipe => {
                // TODO: 调用 named_pipe::spawn_named_pipe()
                PipeResult::Error("命名管道模式尚未实现".to_string())
            }
        }
    }

    /// 将用户输入字节流写入子进程 stdin
    pub fn write_to_child(&self, _data: &[u8]) -> Result<(), String> {
        // TODO: 根据当前模式写入对应管道
        Err("管道未就绪".to_string())
    }

    /// 读取子进程输出并送入 VT 引擎
    pub fn read_from_child(&mut self, data: &[u8]) {
        if let Some(engine) = &mut self.engine {
            engine.feed(data);
        }
    }

    /// 获取 VT 引擎引用
    pub fn engine_mut(&mut self) -> Option<&mut TerminalEngine> {
        self.engine.as_mut()
    }
}

/// 构建子进程 Command（供 inherited 模式使用）
pub fn build_subprocess_command(args: &[String]) -> Command {
    let (exe, exe_args) = if args.is_empty() {
        ("bash.exe".to_string(), vec![] as Vec<String>)
    } else {
        (args[0].clone(), args[1..].to_vec())
    };
    let mut cmd = Command::new(exe);
    cmd.args(exe_args);
    cmd
}
