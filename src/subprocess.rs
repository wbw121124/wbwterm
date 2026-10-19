// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 子进程生命周期管理

use std::os::windows::io::{OwnedHandle, RawHandle};
use std::thread;
use std::time::Duration;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, TerminateProcess, WaitForSingleObject, INFINITE, STILL_ACTIVE, WAIT_OBJECT_0,
};

/// 子进程状态
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    Running,
    Exited(u32),
    Timeout,
}

/// 子进程管理器
pub struct Subprocess {
    process_handle: OwnedHandle,
    pid: u32,
}

impl Subprocess {
    pub fn from_pipe_handle(handle: OwnedHandle, pid: u32) -> Self {
        Self {
            process_handle: handle,
            pid,
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn is_running(&self) -> bool {
        unsafe {
            let result = WaitForSingleObject(
                HANDLE(self.process_handle.as_raw_handle() as isize),
                0,
            );
            result != WAIT_OBJECT_0
        }
    }

    pub fn wait(&self) -> ProcessState {
        unsafe {
            let result = WaitForSingleObject(
                HANDLE(self.process_handle.as_raw_handle() as isize),
                INFINITE,
            );
            if result == WAIT_OBJECT_0 {
                let mut exit_code: u32 = STILL_ACTIVE.0;
                GetExitCodeProcess(
                    HANDLE(self.process_handle.as_raw_handle() as isize),
                    &mut exit_code,
                );
                ProcessState::Exited(exit_code)
            } else {
                ProcessState::Timeout
            }
        }
    }

    pub fn wait_with_timeout(&self, timeout_ms: u32) -> ProcessState {
        unsafe {
            let result = WaitForSingleObject(
                HANDLE(self.process_handle.as_raw_handle() as isize),
                timeout_ms,
            );
            if result == WAIT_OBJECT_0 {
                let mut exit_code: u32 = STILL_ACTIVE.0;
                GetExitCodeProcess(
                    HANDLE(self.process_handle.as_raw_handle() as isize),
                    &mut exit_code,
                );
                ProcessState::Exited(exit_code)
            } else {
                ProcessState::Timeout
            }
        }
    }

    pub fn spawn_wait(self) -> thread::JoinHandle<ProcessState> {
        let handle = self.process_handle;
        let pid = self.pid;
        thread::spawn(move || {
            unsafe {
                let result = WaitForSingleObject(
                    HANDLE(handle.as_raw_handle() as isize),
                    INFINITE,
                );
                if result == WAIT_OBJECT_0 {
                    let mut exit_code: u32 = STILL_ACTIVE.0;
                    GetExitCodeProcess(
                        HANDLE(handle.as_raw_handle() as isize),
                        &mut exit_code,
                    );
                    ProcessState::Exited(exit_code)
                } else {
                    ProcessState::Timeout
                }
            }
        })
    }

    pub fn terminate(&self, exit_code: u32) -> Result<(), String> {
        unsafe {
            let ok = TerminateProcess(
                HANDLE(self.process_handle.as_raw_handle() as isize),
                exit_code,
            );
            if ok == 0 {
                Err(format!(
                    "TerminateProcess 失败: {}",
                    std::io::Error::last_os_error()
                ))
            } else {
                Ok(())
            }
        }
    }

    pub fn shutdown(
        &self,
        grace_period: Duration,
        exit_code: u32,
    ) -> ProcessState {
        let ms = grace_period.as_millis() as u32;
        match self.wait_with_timeout(ms) {
            ProcessState::Exited(code) => ProcessState::Exited(code),
            _ => {
                if let Err(e) = self.terminate(exit_code) {
                    eprintln!("强制终止失败: {}", e);
                }
                self.wait()
            }
        }
    }
}
