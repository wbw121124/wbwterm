// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 子进程生命周期管理
//!
//! 封装 Windows 进程句柄，提供等待、退出码查询、优雅终止等功能。

use std::os::windows::io::{OwnedHandle, RawHandle};
use std::thread;
use std::time::Duration;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{
    WaitForSingleObject, INFINITE, STILL_ACTIVE,
};

/// 子进程状态
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    /// 正在运行
    Running,
    /// 已退出（带退出码）
    Exited(u32),
    /// 等待超时
    Timeout,
}

/// 子进程管理器
///
/// 封装进程句柄，提供等待、查询状态、优雅终止等功能。
pub struct Subprocess {
    /// 进程句柄
    process_handle: OwnedHandle,
    /// 子进程 PID
    pid: u32,
}

impl Subprocess {
    /// 从管道结果创建子进程管理器
    pub fn from_pipe_handle(handle: OwnedHandle, pid: u32) -> Self {
        Self {
            process_handle: handle,
            pid,
        }
    }

    /// 获取子进程 PID
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// 检查子进程是否仍在运行
    ///
    /// 使用 `WaitForSingleObject` 超时 0ms 查询，不阻塞。
    pub fn is_running(&self) -> bool {
        unsafe {
            let result = WaitForSingleObject(self.process_handle.as_raw_handle() as HANDLE, 0);
            result != WAIT_OBJECT_0
        }
    }

    /// 等待子进程退出（无限等待）
    ///
    /// ## 返回值
    /// - `ProcessState::Exited(code)`: 子进程已退出，返回退出码
    /// - `ProcessState::Timeout`: 不应该发生（无限等待）
    pub fn wait(&self) -> ProcessState {
        unsafe {
            let result = WaitForSingleObject(self.process_handle.as_raw_handle() as HANDLE, INFINITE);
            if result == WAIT_OBJECT_0 {
                // 查询退出码
                let mut exit_code: u32 = STILL_ACTIVE;
                GetExitCodeProcess(self.process_handle.as_raw_handle() as HANDLE, &mut exit_code);
                ProcessState::Exited(exit_code)
            } else {
                ProcessState::Timeout
            }
        }
    }

    /// 超时等待子进程退出
    ///
    /// ## 参数
    /// - `timeout_ms`: 最大等待时间（毫秒）
    ///
    /// ## 返回值
    /// - `ProcessState::Exited(code)`: 子进程在超时内退出
    /// - `ProcessState::Timeout`: 超时，子进程仍在运行
    pub fn wait_with_timeout(&self, timeout_ms: u32) -> ProcessState {
        unsafe {
            let result = WaitForSingleObject(self.process_handle.as_raw_handle() as HANDLE, timeout_ms);
            if result == WAIT_OBJECT_0 {
                let mut exit_code: u32 = STILL_ACTIVE;
                GetExitCodeProcess(self.process_handle.as_raw_handle() as HANDLE, &mut exit_code);
                ProcessState::Exited(exit_code)
            } else {
                ProcessState::Timeout
            }
        }
    }

    /// 异步等待子进程退出（后台线程）
    ///
    /// 返回 JoinHandle，调用 `.join()` 获取退出码。
    pub fn spawn_wait(self) -> thread::JoinHandle<ProcessState> {
        let handle = self.process_handle;
        let pid = self.pid;
        thread::spawn(move || {
            unsafe {
                let result = WaitForSingleObject(handle.as_raw_handle() as HANDLE, INFINITE);
                if result == WAIT_OBJECT_0 {
                    let mut exit_code: u32 = STILL_ACTIVE;
                    GetExitCodeProcess(handle.as_raw_handle() as HANDLE, &mut exit_code);
                    ProcessState::Exited(exit_code)
                } else {
                    ProcessState::Timeout
                }
            }
        })
    }

    /// 向子进程发送终止信号（Windows TerminateProcess）
    ///
    /// ## 注意
    /// - Windows 没有 POSIX 信号，`TerminateProcess` 是强制终止
    /// - 优先通过管道发送 Ctrl+C (VT 序列) 优雅终止
    pub fn terminate(&self, exit_code: u32) -> Result<(), String> {
        unsafe {
            use windows_sys::Win32::System::Threading::TerminateProcess;
            let ok = TerminateProcess(self.process_handle.as_raw_handle() as HANDLE, exit_code);
            if ok == 0 {
                Err(format!(
                    "TerminateProcess 失败, error={}",
                    std::io::Error::last_os_error()
                ))
            } else {
                Ok(())
            }
        }
    }

    /// 优雅关闭：先等待一段时间，超时后强制终止
    ///
    /// ## 参数
    /// - `grace_period`: 优雅等待时间（默认 2 秒）
    /// - `exit_code`: 强制终止时的退出码
    pub fn shutdown(
        &self,
        grace_period: Duration,
        exit_code: u32,
    ) -> ProcessState {
        // 先尝试优雅等待
        let ms = grace_period.as_millis() as u32;
        match self.wait_with_timeout(ms) {
            ProcessState::Exited(code) => {
                eprintln!("子进程 {} 已正常退出, code={}", self.pid, code);
                ProcessState::Exited(code)
            }
            _ => {
                // 超时，强制终止
                eprintln!(
                    "子进程 {} 未在 {:?} 内退出，强制终止",
                    self.pid, grace_period
                );
                if let Err(e) = self.terminate(exit_code) {
                    eprintln!("强制终止失败: {}", e);
                }
                // 等待终止完成
                self.wait()
            }
        }
    }
}

/// Windows API 常量
const WAIT_OBJECT_0: u32 = 0;

/// Windows API: 获取进程退出码
#[allow(non_snake_case)]
unsafe fn GetExitCodeProcess(
    hprocess: HANDLE,
    lpprocessexitcode: *mut u32,
) -> i32 {
    windows_sys::Win32::System::Threading::GetExitCodeProcess(hprocess, lpprocessexitcode)
}
