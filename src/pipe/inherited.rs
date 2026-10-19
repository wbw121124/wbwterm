// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 模式 A：继承句柄管道
//!
//! ## 流程
//! 1. CreatePipe() 创建匿名管道（inheritable）
//! 2. 设置 STARTUPINFOEXW.hStdInput / hStdOutput / hStdError
//! 3. CreateProcessW() 启动子进程
//! 4. 子进程的 stdin/stdout/stderr 自动连接到管道
//!
//! ## Windows SDK 兼容说明
//! - CreateProcessW + STARTUPINFOEXW 在 Windows 7+ 可用
//! - ProcThreadAttribute 为 Windows 7 SP1+，Windows 8+
//!   注意：MSYS2 bash.exe 等子进程要求 Windows 7+ 环境

use std::os::windows::io::{FromRawHandle, OwnedHandle, RawHandle};
use std::process::Command;
use std::ptr;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT,
    GetCurrentProcess, InitializeProcThreadAttributeList, PROCESS_INFORMATION,
    PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, STARTF_USESTDHANDLES, STARTUPINFOEXW,
    UpdateProcThreadAttribute,
};

/// 继承句柄模式的管道配置
#[derive(Clone, Debug)]
pub struct InheritedPipeConfig {
    /// 子进程可执行文件路径（如 "bash.exe"）
    pub exe: String,
    /// 命令行参数
    pub args: Vec<String>,
    /// 初始列数（用于发送 resize 事件）
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
    /// 子进程句柄（用于 WaitForSingleObject 等）
    pub process_handle: OwnedHandle,
}

/// 创建安全属性（bInheritHandle = true）
fn inheritable_security_attrs() -> SECURITY_ATTRIBUTES {
    SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    }
}

/// 构建命令行字符串
fn build_cmd_line(config: &InheritedPipeConfig) -> String {
    let mut cmd = config.exe.clone();
    for arg in &config.args {
        cmd.push(' ');
        if arg.contains(' ') {
            cmd.push('"');
            cmd.push_str(arg);
            cmd.push('"');
        } else {
            cmd.push_str(arg);
        }
    }
    cmd
}

/// 将 Rust String 转换为 UTF-16 LE null-terminated 向量
fn to_wide_string(s: &str) -> Vec<u16> {
    let mut wide: Vec<u16> = s.encode_utf16().collect();
    wide.push(0);
    wide
}

/// 通过继承句柄启动子进程
///
/// ## 参数
/// - `config`: 管道配置（命令、参数、尺寸）
///
/// ## 返回值
/// - `Ok(InheritedPipe)`: 子进程已启动，返回读写句柄
/// - `Err(String)`: 启动失败，错误信息
///
/// ## 示例（伪代码）
/// ```ignore
/// let pipe_config = InheritedPipeConfig {
///     exe: "bash.exe".into(),
///     args: vec![],
///     cols: 80,
///     rows: 24,
/// };
/// match spawn_inherited(&pipe_config) {
///     Ok(pipe) => {
///         // pipe.stdin 可写入到子进程 stdin
///         // pipe.stdout 可从子进程 stdout 读取
///         // 子进程 PID: pipe.pid
///     }
///     Err(e) => eprintln!("启动失败: {}", e),
/// }
/// ```
pub fn spawn_inherited(config: &InheritedPipeConfig) -> Result<InheritedPipe, String> {
    // =========================================================
    // Step 1: 创建匿名管道（inheritable handles）
    // =========================================================
    let sa = inheritable_security_attrs();
    let mut h_read_pipe: windows_sys::Win32::Foundation::HANDLE = ptr::null_mut();
    let mut h_write_pipe: windows_sys::Win32::Foundation::HANDLE = ptr::null_mut();

    unsafe {
        let ok = CreatePipe(
            &mut h_read_pipe,
            &mut h_write_pipe,
            &sa,
            0, // default buffer size
        );
        if ok == 0 {
            return Err(format!(
                "CreatePipe 失败: {}",
                std::io::Error::last_os_error()
            ));
        }
    }

    // h_read_pipe: 读端（子进程写入 → Terminal 读取）
    // h_write_pipe: 写端（Terminal 写入 → 子进程读取）

    // =========================================================
    // Step 2: 查询 PROC_THREAD_ATTRIBUTE_LIST 缓冲区大小
    // =========================================================
    let mut attr_list_size: usize = 0;
    unsafe {
        InitializeProcThreadAttributeList(
            ptr::null_mut(),
            1,
            0,
            &mut attr_list_size,
        );
    }
    if attr_list_size == 0 {
        return Err("InitializeProcThreadAttributeList 查询大小失败".to_string());
    }

    let mut attr_buf: Vec<u8> = vec![0u8; attr_list_size];
    let attr_list_ptr = attr_buf.as_mut_ptr()
        as windows_sys::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST;

    // =========================================================
    // Step 3: 初始化属性列表
    // =========================================================
    unsafe {
        let ok = InitializeProcThreadAttributeList(
            attr_list_ptr,
            1,
            0,
            &mut attr_list_size,
        );
        if ok == 0 {
            return Err("InitializeProcThreadAttributeList 失败".to_string());
        }
    }

    // =========================================================
    // Step 4: 设置父进程属性（PROC_THREAD_ATTRIBUTE_PARENT_PROCESS）
    //         使子进程认为是当前进程的"直接子进程"
    // =========================================================
    unsafe {
        let current_process = GetCurrentProcess();
        let ok = UpdateProcThreadAttribute(
            attr_list_ptr,
            0,
            PROC_THREAD_ATTRIBUTE_PARENT_PROCESS as usize,
            &current_process as *const _ as *const _,
            std::mem::size_of::<windows_sys::Win32::Foundation::HANDLE>(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        if ok == 0 {
            DeleteProcThreadAttributeList(attr_list_ptr);
            return Err("UpdateProcThreadAttribute 失败".to_string());
        }
    }

    // =========================================================
    // Step 5: 构建 STARTUPINFOEXW
    // =========================================================
    let mut startup_info: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    unsafe {
        startup_info.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
        startup_info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup_info.StartupInfo.hStdInput = h_read_pipe;       // 子进程 stdin ← 管道读端
        startup_info.StartupInfo.hStdOutput = h_write_pipe;     // 子进程 stdout → 管道写端
        startup_info.StartupInfo.hStdError = h_write_pipe;      // 子进程 stderr → 管道写端（复用）
        startup_info.lpAttributeList = attr_list_ptr;
    }

    // =========================================================
    // Step 6: 构建命令行（UTF-16 LE）
    // =========================================================
    let cmd_wide = to_wide_string(&build_cmd_line(config));

    // =========================================================
    // Step 7: 调用 CreateProcessW
    // =========================================================
    unsafe {
        let mut proc_info: PROCESS_INFORMATION = std::mem::zeroed();
        let ok = CreateProcessW(
            ptr::null(),                          // lpApplicationName
            cmd_wide.as_ptr() as _,               // lpCommandLine（mutable，UTF-16）
            ptr::null(),                          // lpProcessAttributes
            ptr::null(),                          // lpThreadAttributes
            1,                                    // bInheritHandles = TRUE
            EXTENDED_STARTUPINFO_PRESENT,         // dwCreationFlags
            ptr::null(),                          // lpEnvironment
            ptr::null(),                          // lpCurrentDirectory
            &startup_info as *const _ as *const _, // lpStartupInfo
            &mut proc_info as *mut _ as *mut _,   // lpProcessInformation
        );

        DeleteProcThreadAttributeList(attr_list_ptr);

        if ok == 0 {
            return Err(format!(
                "CreateProcessW 失败: {}",
                std::io::Error::last_os_error()
            ));
        }

        Ok(InheritedPipe {
            stdin: OwnedHandle::from_raw_handle(h_write_pipe as RawHandle),
            stdout: OwnedHandle::from_raw_handle(h_read_pipe as RawHandle),
            pid: proc_info.dwProcessId,
            process_handle: OwnedHandle::from_raw_handle(proc_info.hProcess as RawHandle),
        })
    }
}

/// 构建子进程 Command（供测试或调试使用）
pub fn build_command(config: &InheritedPipeConfig) -> Command {
    let mut cmd = Command::new(&config.exe);
    cmd.args(&config.args);
    cmd
}

/// 从已存在的句柄创建 InheritedPipe（用于已启动的子进程）
pub fn from_handles(stdin: OwnedHandle, stdout: OwnedHandle, pid: u32) -> InheritedPipe {
    InheritedPipe {
        stdin,
        stdout,
        pid,
        process_handle: unsafe { OwnedHandle::from_raw_handle(0isize as RawHandle) },
    }
}
