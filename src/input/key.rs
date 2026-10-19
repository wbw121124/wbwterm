// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 键盘事件编码 — winit KeyEvent → VT 序列
//!
//! ## 支持的协议
//! - Kitty 键盘协议（mode 1）：CSI modifiers;keycode u
//! - 标准 ANSI 兼容

use winit::event::KeyEvent;
use winit::keyboard::{Key, KeyCode, PhysicalKey};

/// 将 winit KeyEvent 编码为 Kitty VT 序列
pub fn key_event_to_vt(key_event: &KeyEvent) -> Option<Vec<u8>> {
    let code = match &key_event.physical_key {
        PhysicalKey::Code(c) => *c,
        PhysicalKey::Unidentified(_) => return None,
    };

    if !key_event.state.is_pressed() {
        return None;
    }

    let keycode = key_code_to_kitty(code);
    if keycode == 0 {
        // 尝试从 logical_key 获取字符
        if let Key::Character(c) = &key_event.logical_key {
            if let Some(ch) = c.chars().next() {
                if ch.is_ascii() {
                    return Some(vec![ch as u8]);
                }
            }
        }
        return None;
    }

    // Kitty 协议：无修饰键时直接发 CSI keycode u
    // 修饰键需通过 event loop 的 modifiers_state() 获取
    Some(format!("\x1b[{};{}u", 0, keycode).into_bytes())
}

/// 将 winit KeyCode 映射到 Kitty 键码
fn key_code_to_kitty(code: KeyCode) -> u16 {
    match code {
        KeyCode::Backspace => 283,
        KeyCode::Tab => 292,
        KeyCode::Enter => 291,
        KeyCode::Escape => 278,
        KeyCode::Insert => 268,
        KeyCode::Delete => 269,
        KeyCode::Home => 264,
        KeyCode::End => 266,
        KeyCode::PageUp => 265,
        KeyCode::PageDown => 267,
        KeyCode::ArrowUp => 259,
        KeyCode::ArrowDown => 261,
        KeyCode::ArrowLeft => 260,
        KeyCode::ArrowRight => 262,
        KeyCode::F1 => 330,
        KeyCode::F2 => 331,
        KeyCode::F3 => 332,
        KeyCode::F4 => 333,
        KeyCode::F5 => 334,
        KeyCode::F6 => 335,
        KeyCode::F7 => 336,
        KeyCode::F8 => 337,
        KeyCode::F9 => 338,
        KeyCode::F10 => 339,
        KeyCode::F11 => 340,
        KeyCode::F12 => 341,
        KeyCode::Numpad0 => 320,
        KeyCode::Numpad1 => 321,
        KeyCode::Numpad2 => 322,
        KeyCode::Numpad3 => 323,
        KeyCode::Numpad4 => 324,
        KeyCode::Numpad5 => 325,
        KeyCode::Numpad6 => 326,
        KeyCode::Numpad7 => 327,
        KeyCode::Numpad8 => 328,
        KeyCode::Numpad9 => 329,
        KeyCode::Backquote => 96,
        KeyCode::Digit0 => 48,
        KeyCode::Digit1 => 49,
        KeyCode::Digit2 => 50,
        KeyCode::Digit3 => 51,
        KeyCode::Digit4 => 52,
        KeyCode::Digit5 => 53,
        KeyCode::Digit6 => 54,
        KeyCode::Digit7 => 55,
        KeyCode::Digit8 => 56,
        KeyCode::Digit9 => 57,
        KeyCode::KeyA => 97,
        KeyCode::KeyB => 98,
        KeyCode::KeyC => 99,
        KeyCode::KeyD => 100,
        KeyCode::KeyE => 101,
        KeyCode::KeyF => 102,
        KeyCode::KeyG => 103,
        KeyCode::KeyH => 104,
        KeyCode::KeyI => 105,
        KeyCode::KeyJ => 106,
        KeyCode::KeyK => 107,
        KeyCode::KeyL => 108,
        KeyCode::KeyM => 109,
        KeyCode::KeyN => 110,
        KeyCode::KeyO => 111,
        KeyCode::KeyP => 112,
        KeyCode::KeyQ => 113,
        KeyCode::KeyR => 114,
        KeyCode::KeyS => 115,
        KeyCode::KeyT => 116,
        KeyCode::KeyU => 117,
        KeyCode::KeyV => 118,
        KeyCode::KeyW => 119,
        KeyCode::KeyX => 120,
        KeyCode::KeyY => 121,
        KeyCode::KeyZ => 122,
        KeyCode::Period => 46,
        KeyCode::Comma => 44,
        KeyCode::Slash => 47,
        KeyCode::Backslash => 92,
        KeyCode::Semicolon => 59,
        KeyCode::BracketLeft => 91,
        KeyCode::BracketRight => 93,
        KeyCode::Equal => 61,
        KeyCode::Minus => 45,
        _ => 0,
    }
}

/// 处理文本输入事件（Unicode 字符）
pub fn unicode_input_to_vt(input: &str) -> Vec<u8> {
    input.as_bytes().to_vec()
}
