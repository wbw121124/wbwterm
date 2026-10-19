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
    Inherited,
    NamedPipe,
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
    Ok {
        stdin: std::os::windows::io::OwnedHandle,
        stdout: std::os::windows::io::OwnedHandle,
        pid: u32,
    },
    NamedPipeConnected {
        stream: interprocess::local_socket::Stream,
        pid: u32,
    },
    Error(String),
}

/// 管道管理器
pub struct PipeManager {
    pub config: PipeConfig,
    pub engine: std::sync::Arc<std::sync::Mutex<TerminalEngine>>,
    pub events: std::sync::Arc<std::sync::Mutex<TerminalEvents>>,
    pub session: Option<session::Session>,
    pub pid: Option<u32>,
}

impl PipeManager {
    pub fn new(config: PipeConfig) -> Self {
        let engine = TerminalEngine::new(config.cols, config.rows);
        Self {
            config,
            engine: std::sync::Arc::new(std::sync::Mutex::new(engine)),
            events: std::sync::Arc::new(std::sync::Mutex::new(TerminalEvents::new())),
            session: None,
            pid: None,
        }
    }

    pub fn spawn(&mut self) -> Result<PipeResult, String> {
        match self.config.mode {
            PipeMode::Inherited => self.spawn_inherited(),
            PipeMode::NamedPipe => self.spawn_named_pipe(),
        }
    }

    fn spawn_inherited(&mut self) -> Result<PipeResult, String> {
        let cmd = &self.config.command;
        let exe = cmd
            .first()
            .cloned()
            .unwrap_or_else(|| "bash.exe".to_string());
        let args: Vec<String> = cmd[1..].to_vec();

        let pipe_config = inherited::InheritedPipeConfig {
            exe,
            args,
            cols: self.config.cols,
            rows: self.config.rows,
        };

        let pipe = inherited::spawn_inherited(&pipe_config)?;

        let engine_arc = self.engine.clone();
        let events_arc = self.events.clone();

        let sess = session::Session::new(
            engine_arc,
            events_arc,
            pipe.stdin.try_clone().unwrap(),
            pipe.stdout.try_clone().unwrap(),
        );

        self.session = Some(sess);
        self.pid = Some(pipe.pid);

        Ok(PipeResult::Ok {
            stdin: pipe.stdin,
            stdout: pipe.stdout,
            pid: pipe.pid,
        })
    }

    fn spawn_named_pipe(&mut self) -> Result<PipeResult, String> {
        Err("命名管道模式尚未实现".to_string())
    }

    pub fn write_to_child(&self, data: &[u8]) -> Result<(), String> {
        if let Some(ref session) = self.session {
            session.write(data.to_vec())
        } else {
            Err("管道未就绪".to_string())
        }
    }

    pub fn engine_ref(&self) -> std::sync::Arc<std::sync::Mutex<TerminalEngine>> {
        self.engine.clone()
    }

    pub fn events_ref(&self) -> std::sync::Arc<std::sync::Mutex<TerminalEvents>> {
        self.events.clone()
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
