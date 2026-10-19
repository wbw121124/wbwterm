// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 单会话管理 — 读写线程 + channel 分派
//!
//! ## 数据流
//! ```
//! 管道读端 ──► read_thread() ──► channel ──► TerminalEngine.feed()
//! TerminalEngine.events() ──► channel ──► App 分发
//! App 输入 ──► write_thread() ──► 管道写端
//! ```

use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::mpsc;
use std::thread;

/// 读写线程句柄
#[derive(Clone)]
pub struct SessionHandles {
    pub tx: Sender<Vec<u8>>,
    pub rx: Receiver<Vec<u8>>,
}

/// 会话管理器
///
/// 管理单个子进程的管道读写线程和 channel 通信
pub struct Session {
    pub handles: SessionHandles,
    /// 管道读取线程（后台运行）
    pub read_handle: Option<thread::JoinHandle<()>>,
    /// 管道写入线程（后台运行）
    pub write_handle: Option<thread::JoinHandle<()>>,
}

impl Session {
    /// 创建新会话（管道读写线程已启动）
    pub fn new(stdin_tx: Sender<Vec<u8>>, stdout_rx: Receiver<Vec<u8>>) -> Self {
        Self {
            handles: SessionHandles {
                tx: stdin_tx,
                rx: stdout_rx,
            },
            read_handle: None,
            write_handle: None,
        }
    }

    /// 向子进程写入数据
    pub fn write(&self, data: Vec<u8>) -> Result<(), String> {
        self.handles.tx.send(data).map_err(|e| e.to_string())
    }
}
