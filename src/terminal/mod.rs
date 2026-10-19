// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! VT 终端引擎 — vte Parser + TerminalState
//!
//! ## 模块
//! - `engine`: vte Parser + Perform trait impl，将 VT 字节流转换为终端状态变更
//! - `state`: Grid、Cursor、Scrollback、Mode 等状态数据
//! - `events`: 终端侧边事件（bell / title / clipboard / size report 等）

pub mod engine;
pub mod events;
pub mod state;

use crate::terminal::engine::TerminalPerform;
use crate::terminal::events::TerminalEvents;
use crate::terminal::state::TerminalState;
use vte::Parser;

/// VT 终端引擎
pub struct TerminalEngine {
    parser: Parser,
    state: TerminalState,
    events: TerminalEvents,
}

impl TerminalEngine {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            parser: Parser::new(),
            state: TerminalState::new(cols, rows),
            events: TerminalEvents::new(),
        }
    }

    /// 向终端注入 VT 字节流
    pub fn feed(&mut self, input: &[u8]) {
        use crate::terminal::engine::TerminalPerform;
        self.parser.advance(
            &mut TerminalPerform {
                state: &mut self.state,
                events: &mut self.events,
                sgr: Default::default(),
            },
            input,
        );
    }

    /// 查询侧边事件队列（非阻塞）
    pub fn drain_events(&mut self) -> Vec<events::TerminalEvent> {
        self.events.drain()
    }

    /// 窗口尺寸变化时调用
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.state.resize(cols, rows);
        self.events
            .push(events::TerminalEvent::SizeChanged(cols, rows));
    }

    /// 获取终端状态快照（供渲染层使用）
    pub fn state(&self) -> &TerminalState {
        &self.state
    }

    /// 获取可变终端状态引用
    pub fn state_mut(&mut self) -> &mut TerminalState {
        &mut self.state
    }
}
