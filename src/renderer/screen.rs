// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 屏幕合成 — 脏行追踪 + wgpu 帧绘制
//!
//! ## 渲染流程
//! ```
//! 1. terminal::state.dirty_rows 收集脏行
//! 2. 对每个脏行调用 glyphon 合成 → GlyphToTexture 批量上传
//! 3. wgpu::RenderPass 绘制到 SwapChain
//! 4. present() 提交帧
//! ```

use wgpu::Device;

/// 屏幕渲染器
///
/// TODO: P3 阶段完整实现
/// 核心职责：
/// 1. 读取 TerminalState 的 dirty_rows
/// 2. 调用 glyphon::Renderer 合成脏行区域
/// 3. 通过 wgpu 绘制到当前交换链帧
/// 4. 调用 Surface::present() 提交
pub struct ScreenRenderer {
    device: Device,
    // TODO: glyphon::Renderer, glyphon::Atlas, glyphon::BufferCache
}

impl ScreenRenderer {
    pub fn new(device: Device) -> Self {
        Self { device }
    }

    /// 标记某行为脏行（由 terminal::engine 调用）
    pub fn mark_dirty(&mut self, _row: usize) {
        // TODO: 实现
    }

    /// 合成并重绘所有脏行
    pub fn render_dirty(&mut self, _width: u32, _height: u32) {
        // TODO: 实现 glyphon rendering pipeline
    }
}
