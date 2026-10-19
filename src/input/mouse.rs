// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 鼠标事件编码 — 鼠标事件 → VT 序列
//!
//! ## 支持的协议
//! - XTerm 鼠标协议（mode 1000/1002/1006）
//! - SGR 增强协议（mode 1006）：CSI < button; x; y; M/m
//! - Kitty 扩展：携带修饰键信息

use winit::window::CursorIcon;

/// 鼠标协议模式
#[derive(Clone, Debug, Default)]
pub enum MouseProtocol {
    #[default]
    Disabled,
    XTerm,
    ButtonTrack,
    Sgr,
}

/// 将鼠标位置编码为 VT 序列（简化）
pub fn mouse_event_to_vt(_protocol: &MouseProtocol, _pos: (u32, u32)) -> Option<Vec<u8>> {
    None
}

/// 获取鼠标光标图标
pub fn cursor_for_mouse(_button: &winit::event::MouseButton) -> CursorIcon {
    CursorIcon::Default
}
