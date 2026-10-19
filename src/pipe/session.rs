// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 单会话管理 — 读写线程 + channel 分派

use crate::terminal::{events::TerminalEvents, TerminalEngine};
use crossbeam_channel::{Receiver, Sender};
use std::os::windows::io::{AsRawHandle, OwnedHandle, RawHandle};
use std::thread;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows_sys::Win32::System::IO::OVERLAPPED;

/// 会话内发送通道
pub struct SessionHandles {
    pub tx: Sender<Vec<u8>>,
}

/// 会话管理器
pub struct Session {
    pub handles: SessionHandles,
    pub read_handle: Option<std::thread::JoinHandle<()>>,
    pub write_handle: Option<std::thread::JoinHandle<()>>,
}

impl Session {
    pub fn new(
        engine: std::sync::Arc<std::sync::Mutex<TerminalEngine>>,
        _events: std::sync::Arc<std::sync::Mutex<TerminalEvents>>,
        stdin: OwnedHandle,
        stdout: OwnedHandle,
    ) -> Self {
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = crossbeam_channel::unbounded();

        let engine_read = engine.clone();
        let stdout_raw = stdout.as_raw_handle() as isize;
        let read_handle = thread::spawn(move || {
            Self::read_loop(&engine_read, stdout_raw);
        });

        let stdin_raw = stdin.as_raw_handle() as isize;
        let write_handle = thread::spawn(move || {
            Self::write_loop(stdin_raw, rx);
        });

        Self {
            handles: SessionHandles { tx },
            read_handle: Some(read_handle),
            write_handle: Some(write_handle),
        }
    }

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
                if ok == 0 || bytes_read == 0 {
                    break;
                }
            }

            let chunk: Vec<u8> = buf[..bytes_read as usize].to_vec();
            if let Ok(mut eng) = engine.lock() {
                eng.feed(&chunk);
            }
        }
    }

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
                    break;
                }
            }
        }
    }

    pub fn write(&self, data: Vec<u8>) -> Result<(), String> {
        self.handles.tx.send(data).map_err(|e| e.to_string())
    }

    pub fn wait_read(&mut self) {
        if let Some(h) = self.read_handle.take() {
            let _ = h.join();
        }
    }
}
