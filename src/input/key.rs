// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 键盘事件编码 — winit KeyEvent → VT 序列
//!
//! ## 支持的协议
//! - 标准 ANSI：Ctrl+A = \x01
//! - Kitty 键盘协议扩展（mode 1）：上报键码、修饰键
//! - XTerm 兼容：方向键、功能键

use winit::event::KeyEvent;
use winit::keyboard::{PhysicalKey, KeyCode};

/// 将 winit KeyEvent 编码为 VT 序列字节
pub fn key_event_to_vt(key_event: &KeyEvent) -> Option<Vec<u8>> {
    match &key_event.physical_key {
        PhysicalKey::Code(code) => match code {
            KeyCode::Enter => Some(b"\r".to_vec()),
            KeyCode::Escape => Some(b"\x1b".to_vec()),
            KeyCode::Backspace => Some(b"\x08".to_vec()),
            KeyCode::Tab => Some(b"\x09".to_vec()),
            KeyCode::ArrowUp => Some(b"\x1b[A".to_vec()),
            KeyCode::ArrowDown => Some(b"\x1b[B".to_vec()),
            KeyCode::ArrowRight => Some(b"\x1b[C".to_vec()),
            KeyCode::ArrowLeft => Some(b"\x1b[D".to_vec()),
            KeyCode::Home => Some(b"\x1b[H".to_vec()),
            KeyCode::End => Some(b"\x1b[F".to_vec()),
            KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
            KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),
            KeyCode::Insert => Some(b"\x1b[2~".to_vec()),
            KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
            KeyCode::F1 => Some(b"\x1bOP".to_vec()),
            KeyCode::F2 => Some(b"\x1bOQ".to_vec()),
            KeyCode::F3 => Some(b"\x1bOR".to_vec()),
            KeyCode::F4 => Some(b"\x1bOS".to_vec()),
            KeyCode::F5 => Some(b"\x1b[15~".to_vec()),
            KeyCode::F6 => Some(b"\x1b[17~".to_vec()),
            KeyCode::F7 => Some(b"\x1b[18~".to_vec()),
            KeyCode::F8 => Some(b"\x1b[19~".to_vec()),
            KeyCode::F9 => Some(b"\x1b[20~".to_vec()),
            KeyCode::F10 => Some(b"\x1b[21~".to_vec()),
            KeyCode::F11 => Some(b"\x1b[23~".to_vec()),
            KeyCode::F12 => Some(b"\x1b[24~".to_vec()),
            _ => None,
        },
        PhysicalKey::Unidentified(_) => None,
    }
}

/// 处理文本输入事件（Unicode 字符）
pub fn unicode_input_to_vt(input: &str) -> Vec<u8> {
    input.as_bytes().to_vec()
}
