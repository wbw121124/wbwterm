// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 模式 A：继承句柄管道
//!
//! ## 流程
//! 1. CreatePipe() 创建匿名管道（inheritable）
//! 2. 设置 STARTUPINFOEXW.hStdInput / hStdOutput / hStdError
//! 3. CreateProcessW() 启动子进程
//! 4. 子进程的 stdin/stdout/stderr 自动连接到管道

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, OwnedHandle, RawHandle};
use std::process::Command;
use std::ptr;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, GetCurrentProcess,
    InitializeProcThreadAttributeList, UpdateProcThreadAttribute, EXTENDED_STARTUPINFO_PRESENT,
    PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, STARTF_USESTDHANDLES,
    STARTUPINFOEXW,
};
use windows_sys::Win32::System::IO::OVERLAPPED;

/// 继承句柄模式的管道配置
#[derive(Clone, Debug)]
pub struct InheritedPipeConfig {
    pub exe: String,
    pub args: Vec<String>,
    pub cols: u16,
    pub rows: u16,
}

/// 继承句柄管道结果
#[derive(Debug)]
pub struct InheritedPipe {
    pub stdin: OwnedHandle,
    pub stdout: OwnedHandle,
    pub pid: u32,
    pub process_handle: OwnedHandle,
}

fn inheritable_security_attrs() -> SECURITY_ATTRIBUTES {
    SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    }
}

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

fn to_wide_string(s: &str) -> Vec<u16> {
    let mut wide: Vec<u16> = OsStr::new(s).encode_wide().collect();
    wide.push(0);
    wide
}

/// 通过继承句柄启动子进程
pub fn spawn_inherited(config: &InheritedPipeConfig) -> Result<InheritedPipe, String> {
    // Step 1: 创建匿名管道
    let sa = inheritable_security_attrs();
    let mut h_read: HANDLE = ptr::null_mut();
    let mut h_write: HANDLE = ptr::null_mut();

    unsafe {
        let ok = CreatePipe(&mut h_read, &mut h_write, &sa, 0);
        if ok == 0 {
            return Err(format!(
                "CreatePipe 失败: {}",
                std::io::Error::last_os_error()
            ));
        }
    }

    // Step 2: 查询 PROC_THREAD_ATTRIBUTE_LIST 缓冲区大小
    let mut attr_list_size: usize = 0;
    unsafe {
        InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut attr_list_size);
    }
    if attr_list_size == 0 {
        return Err("InitializeProcThreadAttributeList 查询大小失败".to_string());
    }

    let mut attr_buf: Vec<u8> = vec![0u8; attr_list_size];
    let attr_list_ptr = attr_buf.as_mut_ptr()
        as windows_sys::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST;

    // Step 3: 初始化属性列表
    unsafe {
        let ok = InitializeProcThreadAttributeList(attr_list_ptr, 1, 0, &mut attr_list_size);
        if ok == 0 {
            return Err("InitializeProcThreadAttributeList 失败".to_string());
        }
    }

    // Step 4: 设置父进程属性
    unsafe {
        let current_process = GetCurrentProcess();
        let ok = UpdateProcThreadAttribute(
            attr_list_ptr,
            0,
            PROC_THREAD_ATTRIBUTE_PARENT_PROCESS as usize,
            &current_process as *const _ as *const _,
            std::mem::size_of::<HANDLE>(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        if ok == 0 {
            DeleteProcThreadAttributeList(attr_list_ptr);
            return Err("UpdateProcThreadAttribute 失败".to_string());
        }
    }

    // Step 5: 构建 STARTUPINFOEXW
    let mut startup_info: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup_info.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup_info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup_info.StartupInfo.hStdInput = h_read;
    startup_info.StartupInfo.hStdOutput = h_write;
    startup_info.StartupInfo.hStdError = h_write;
    startup_info.lpAttributeList = attr_list_ptr;

    // Step 6: 构建命令行（UTF-16 LE）
    let cmd_wide = to_wide_string(&build_cmd_line(config));

    // Step 7: 调用 CreateProcessW
    unsafe {
        let mut proc_info: PROCESS_INFORMATION = std::mem::zeroed();
        let ok = CreateProcessW(
            ptr::null(),
            cmd_wide.as_ptr() as _,
            ptr::null(),
            ptr::null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT,
            ptr::null(),
            ptr::null(),
            &startup_info as *const _ as *const _,
            &mut proc_info as *mut _ as *mut _,
        );

        DeleteProcThreadAttributeList(attr_list_ptr);

        if ok == 0 {
            return Err(format!(
                "CreateProcessW 失败: {}",
                std::io::Error::last_os_error()
            ));
        }

        Ok(InheritedPipe {
            stdin: OwnedHandle::from_raw_handle(h_write as RawHandle),
            stdout: OwnedHandle::from_raw_handle(h_read as RawHandle),
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

/// 从已存在的句柄创建 InheritedPipe
pub fn from_handles(stdin: OwnedHandle, stdout: OwnedHandle, pid: u32) -> InheritedPipe {
    InheritedPipe {
        stdin,
        stdout,
        pid,
        process_handle: unsafe { OwnedHandle::from_raw_handle(0isize as RawHandle) },
    }
}
