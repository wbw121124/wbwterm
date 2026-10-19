// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 单会话管理 — 读写线程 + channel 分派
//!
//! ## 数据流
//! ```
//! 管道读端 ──► read_thread() ──► TerminalEngine.feed()
//! TerminalEngine.events() ──► App 分发（标题/剪贴板/尺寸）
//! App 输入   ──► tx 发送 ──► write_thread() ──► 管道写端 → 子进程 stdin
//! ```

use crate::terminal::{events::TerminalEvents, TerminalEngine};
use crossbeam_channel::{Receiver, Sender};
use std::os::windows::io::{AsRawHandle, OwnedHandle, RawHandle};
use std::thread;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows_sys::Win32::System::IO::OVERLAPPED;

/// 会话内发送/接收通道
pub struct SessionHandles {
    /// App → 子进程 stdin
    pub tx: Sender<Vec<u8>>,
}

/// 会话生命周期句柄
pub struct SessionHandlesWrapper {
    pub inner: SessionHandles,
}

/// 会话管理器
///
/// 管理单个子进程的管道读写线程和 channel 通信
pub struct Session {
    pub handles: SessionHandlesWrapper,
    /// 管道读取线程
    pub read_handle: Option<std::thread::JoinHandle<()>>,
    /// 管道写入线程
    pub write_handle: Option<std::thread::JoinHandle<()>>,
}

impl Session {
    /// 创建新会话，启动读写线程
    ///
    /// ## 参数
    /// - `engine`: 终端引擎（读写线程通过 Arc<Mutex<>> 访问）
    /// - `events`: 事件分发器
    /// - `stdin`: 子进程 stdin 写入端
    /// - `stdout`: 子进程 stdout 读取端
    pub fn new(
        engine: std::sync::Arc<std::sync::Mutex<TerminalEngine>>,
        events: std::sync::Arc<std::sync::Mutex<TerminalEvents>>,
        stdin: OwnedHandle,
        stdout: OwnedHandle,
    ) -> Self {
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = crossbeam_channel::unbounded();

        // 克隆 Arc 给读线程
        let engine_read = engine.clone();
        let stdout_raw = stdout.as_raw_handle() as isize;
        let read_handle = thread::spawn(move || {
            Self::read_loop(&engine_read, stdout_raw);
        });

        // 克隆 Arc 给写线程
        let _events = events.clone();
        let stdin_raw = stdin.as_raw_handle() as isize;
        let write_handle = thread::spawn(move || {
            Self::write_loop(stdin_raw, rx);
        });

        Self {
            handles: SessionHandlesWrapper {
                inner: SessionHandles { tx },
            },
            read_handle: Some(read_handle),
            write_handle: Some(write_handle),
        }
    }

    /// 读取循环：从管道读取子进程输出，送入终端引擎
    fn read_loop(engine: &std::sync::Arc<std::sync::Mutex<TerminalEngine>>, stdout: isize) {
        const BUF_SIZE: usize = 4096;
        let mut buf = [0u8; BUF_SIZE];

        loop {
            let mut bytes_read: u32 = 0;
            let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };

            unsafe {
                let ok = ReadFile(
                    stdout as HANDLE,
                    buf.as_mut_ptr(),
                    BUF_SIZE as u32,
                    &mut bytes_read,
                    &mut overlapped as *mut _ as *mut _,
                );
                if ok == 0 {
                    // ReadFile 失败（EOF 或其他错误）
                    break;
                }
            }

            if bytes_read == 0 {
                // EOF — 子进程退出
                break;
            }

            let chunk: Vec<u8> = buf[..bytes_read as usize].to_vec();
            if let Ok(mut eng) = engine.lock() {
                eng.feed(&chunk);
            }
        }
    }

    /// 写入循环：从 channel 接收数据，写入子进程 stdin
    fn write_loop(stdin: isize, rx: Receiver<Vec<u8>>) {
        while let Ok(data) = rx.recv() {
            let mut bytes_written: u32 = 0;
            let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
            unsafe {
                let ok = WriteFile(
                    stdin as HANDLE,
                    data.as_ptr(),
                    data.len() as u32,
                    &mut bytes_written,
                    &mut overlapped as *mut _ as *mut _,
                );
                if ok == 0 {
                    // WriteFile 失败
                    break;
                }
            }
        }
    }

    /// 向子进程写入数据
    pub fn write(&self, data: Vec<u8>) -> Result<(), String> {
        self.handles.inner.tx.send(data).map_err(|e| e.to_string())
    }

    /// 等待读取线程结束（子进程退出后调用）
    pub fn wait_read(&mut self) {
        if let Some(h) = self.read_handle.take() {
            let _ = h.join();
        }
    }

    /// 取消写入线程（channel 关闭时自动退出）
    pub fn cancel_write(&mut self) {
        // write 线程在 rx 关闭时自动退出
    }
}
