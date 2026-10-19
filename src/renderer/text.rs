// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 字体渲染 — glyphon FontContext + GlyphAtlas
//!
//! ## 字体加载
//! - Windows 系统字体目录：C:\Windows\Fonts
//! - 优先选择等宽字体：Consolas, Cascadia Code, Segoe UI Mono
//! - 支持 emoji：Noto Color Emoji（Windows 10+）
//!
//! ## glyphon 使用要点
//! - FontSystem: 字体缓存（系统字体枚举）
//! - Family: 字体族名（Mono 等宽优先）
//! - Buffer: 待渲染文本缓冲区
//! - Atlas: glyph 位图纹理缓存
//! - Renderer: wgpu 绘制命令生成
//!
//! ## 字形 Shaping
//! - 基础字形定位：glyphon 内置（FreeType）
//! - 复杂脚本（阿拉伯语、印度语系）：harfbuzz 后端
//! - 双向文本：wezterm-bidi（在 terminal 层处理，传入 glyphon 前已排列）

use glyphon::FontSystem;

/// 字体上下文管理器
pub struct FontContext {
    /// glyphon 字体系统
    pub font_system: FontSystem,
}

impl FontContext {
    /// 创建字体上下文（扫描 Windows 系统字体）
    pub fn new() -> Self {
        // TODO: P3 阶段完善
        // 使用 fontdue 或 skrifa 枚举系统字体
        // 优先选择等宽字体：Consolas, Cascadia Code, Segoe UI Mono
        Self {
            font_system: FontSystem::new(),
        }
    }

    /// 设置首选等宽字体
    pub fn set_font(&mut self, _family_name: &str) {
        // TODO: 在 font_system 中注册指定字体
    }
}
