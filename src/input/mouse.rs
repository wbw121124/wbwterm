// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 鼠标事件编码 — 鼠标事件 → VT 序列
//!
//! ## 支持的协议
//! - XTerm 鼠标协议（mode 1000）：CSI M Cb Cx Cy
//! - XTerm 按钮跟踪（mode 1002）：同上，含按钮移动
//! - XTerm 扩展协议（mode 1006）：CSI < Cb;Cx;Cy;(M|m)
//! - SGR 增强协议（mode 1006）：CSI < Cb;Cx;Cy;M

use winit::event::MouseButton;
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

/// 将鼠标位置编码为 VT 序列
///
/// ## 参数
/// - `protocol`: 鼠标协议模式
/// - `button`: 鼠标按钮（0=移动, 1=左键, 2=中键, 3=右键）
/// - `event_type`: 'M' 按下, 'm' 释放, ' ' 移动
/// - `x`: 像素 X 坐标
/// - `y`: 像素 Y 坐标
pub fn mouse_event_to_vt(
    protocol: &MouseProtocol,
    button: u8,
    event_type: char,
    x: u32,
    y: u32,
) -> Option<Vec<u8>> {
    match protocol {
        MouseProtocol::Disabled => None,
        MouseProtocol::XTerm | MouseProtocol::ButtonTrack => {
            // XTerm 格式：CSI M <btn+32> <x+32> <y+32>
            let b = 32 + button;
            let cx = 32 + (x + 1);
            let cy = 32 + (y + 1);
            Some(vec![b'\x1b', b'M', b, cx as u8, cy as u8])
        }
        MouseProtocol::Sgr => {
            // SGR 格式：CSI < btn;cx;cy;M/m
            let b = button as u32;
            Some(format!("\x1b[<{b};{};{};{}]", x + 1, y + 1, event_type).into_bytes())
        }
    }
}

/// 根据鼠标按钮获取光标图标
pub fn cursor_for_mouse(button: &MouseButton) -> CursorIcon {
    match button {
        MouseButton::Left => CursorIcon::Default,
        MouseButton::Right => CursorIcon::ContextMenu,
        MouseButton::Middle => CursorIcon::Default,
        _ => CursorIcon::Default,
    }
}
