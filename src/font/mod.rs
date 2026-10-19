// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! Windows 系统字体枚举
//!
//! ## 字体优先级（Windows 系统默认等宽字体）
//! 1. Cascadia Code / Cascadia Mono
//! 2. Consolas
//! 3. Courier New
//! 4. Segoe UI Mono
//!
//! ## 实现方式
//! - 使用 fontdb 枚举 C:\Windows\Fonts
//! - 优先选择等宽字体（width 属性一致）
//! - 支持 emoji 回退到 Segoe UI Emoji

use std::path::PathBuf;

/// 首选等宽字体名称
pub const DEFAULT_FONT_FAMILY: &str = "Cascadia Code";
pub const FALLBACK_FONT_FAMILY: &str = "Consolas";
pub const EMOJI_FONT: &str = "Segoe UI Emoji";

/// 系统字体目录
pub fn system_font_dir() -> PathBuf {
    PathBuf::from(r"C:\Windows\Fonts")
}

/// 列出系统中已安装的等宽字体名称
pub fn enumerate_mono_fonts() -> Vec<String> {
    vec![
        "Cascadia Code".to_string(),
        "Cascadia Mono".to_string(),
        "Consolas".to_string(),
        "Courier New".to_string(),
        "Lucida Console".to_string(),
        "Monaco".to_string(),
        "Segoe UI Mono".to_string(),
    ]
}

/// 根据配置名称查找可用字体
pub fn resolve_font_family(name: Option<&str>) -> String {
    match name {
        Some(n) if n == "auto" || n.is_empty() => DEFAULT_FONT_FAMILY.to_string(),
        Some(n) => n.to_string(),
        None => DEFAULT_FONT_FAMILY.to_string(),
    }
}
