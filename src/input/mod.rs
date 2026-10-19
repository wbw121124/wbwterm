// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 键盘和鼠标事件编码 — winit 事件 → VT 序列
//!
//! ## 模块
//! - `key`: 键盘事件编码（含 Kitty 协议支持）
//! - `mouse`: 鼠标事件编码（XTerm/SGR/Kitty 协议）

pub mod key;
pub mod mouse;

use crate::input::key::{key_event_to_vt, unicode_input_to_vt};
use crate::input::mouse::MouseProtocol;
use winit::event::{Event, WindowEvent};

/// 输入处理器
pub struct InputHandler {
    pub protocol: MouseProtocol,
    pub vt_output: Vec<u8>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            protocol: MouseProtocol::Sgr,
            vt_output: Vec::new(),
        }
    }

    /// 处理窗口事件，返回 VT 序列（可能为空）
    #[allow(clippy::single_match, clippy::collapsible_match)]
    pub fn handle_event(&mut self, event: &Event<()>) {
        self.vt_output.clear();
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::KeyboardInput {
                event: key_event, ..
            } = event
            {
                if let Some(t) = &key_event.text {
                    self.vt_output.extend(unicode_input_to_vt(t));
                }
                if let Some(vt_seq) = key_event_to_vt(key_event) {
                    self.vt_output.extend(vt_seq);
                }
            }
        }
    }

    /// 获取累积的 VT 输出
    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.vt_output)
    }
}
