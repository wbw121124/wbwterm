// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 终端侧边事件 — bell / title / clipboard / size_report 等

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// 终端侧边事件
#[derive(Clone, Debug)]
pub enum TerminalEvent {
    /// 铃声音响（Bell）
    Bell,
    /// 标题变更（OSC 0）
    TitleChanged(String),
    /// 剪贴板写入请求（OSC 52）
    ClipboardWrite(u8, String),
    /// 终端尺寸报告请求（需要向子进程发送 CSI ? 8 t）
    SizeReportRequested,
    /// 实际尺寸已变化（从 winit 传递进来）
    SizeChanged(u16, u16),
    /// 未知 CSI 序列（调试用）
    UnknownSequence(String),
}

/// 事件分发器（线程安全）
#[derive(Clone, Default)]
pub struct TerminalEvents {
    inner: Arc<Mutex<VecDeque<TerminalEvent>>>,
}

impl TerminalEvents {
    pub fn new() -> Self {
        Self::default()
    }

    /// 推入一个事件
    pub fn push(&self, event: TerminalEvent) {
        self.inner.lock().unwrap().push_back(event);
    }

    /// 非阻塞取出所有待处理事件
    pub fn drain(&mut self) -> Vec<TerminalEvent> {
        let mut q = self.inner.lock().unwrap();
        let events: Vec<TerminalEvent> = q.drain(..).collect();
        events
    }

    /// 检查是否有未处理事件
    pub fn has_pending(&self) -> bool {
        !self.inner.lock().unwrap().is_empty()
    }
}
