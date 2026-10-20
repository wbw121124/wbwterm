// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 屏幕合成 — glyphon 脏行追踪 + wgpu 帧绘制

use std::collections::BTreeSet;
use wgpu::{Device, Queue, RenderPass, TextureView};

use super::text::FontContext;

/// 屏幕渲染器
///
/// 核心职责：
/// 1. 读取 TerminalState 的 dirty_rows
/// 2. 调用 glyphon 合成脏行区域 → TextAtlas
/// 3. 通过 wgpu RenderPass 绘制到交换链帧
/// 4. 调用 Surface::present() 提交
pub struct ScreenRenderer {
    device: Device,
    queue: Queue,
    font_context: FontContext,
    /// 脏行集合（由 terminal engine 更新）
    dirty_rows: BTreeSet<usize>,
    /// 终端列数
    cols: u16,
    /// 终端行数
    rows: u16,
    /// 字体大小（pt）
    font_size: f32,
    /// 单元格宽高（px，由 font_size 推导）
    cell_width: u32,
    cell_height: u32,
}

impl ScreenRenderer {
    /// 创建屏幕渲染器
    pub fn new(device: Device, queue: Queue, font_context: FontContext, font_size: f32) -> Self {
        // pt → px（96 DPI）：px = pt × 96 / 72 = pt × 4/3
        let size_px = font_size * 96.0 / 72.0;
        // 等宽字体典型度量：字符 advance ≈ 0.6em，行高 ≈ 1.22em
        // 14pt Cascadia Code → ≈12×23px（与 Windows Terminal 一致）
        let cell_w = (size_px * 0.6).ceil() as u32;
        let cell_h = (size_px * 1.22).ceil() as u32;

        Self {
            device,
            queue,
            font_context,
            dirty_rows: BTreeSet::new(),
            cols: 80,
            rows: 24,
            font_size,
            cell_width: cell_w.clamp(4, 64),
            cell_height: cell_h.clamp(8, 128),
        }
    }

    /// 更新终端尺寸（由 App resize 事件触发）
    pub fn set_term_size(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
    }

    /// 标记某行为脏行（由 terminal engine 在状态变更后调用）
    pub fn mark_dirty(&mut self, row: usize) {
        self.dirty_rows.insert(row);
    }

    /// 标记行范围脏
    pub fn mark_dirty_range(&mut self, start: usize, end: usize) {
        for r in start..=end {
            self.dirty_rows.insert(r);
        }
    }

    /// 清除所有脏标记
    pub fn clear_dirty(&mut self) {
        self.dirty_rows.clear();
    }

    /// 获取脏行集合（供外部迭代）
    pub fn dirty_rows(&self) -> &BTreeSet<usize> {
        &self.dirty_rows
    }

    /// 当前单元格尺寸（宽, 高）px
    pub fn cell_size(&self) -> (u32, u32) {
        (self.cell_width, self.cell_height)
    }

    /// 合成并重绘所有脏行
    ///
    /// ## 参数
    /// - `width`: 窗口像素宽度
    /// - `height`: 窗口像素高度
    /// - `output_view`: 交换链当前帧的 TextureView
    pub fn render_dirty(
        &mut self,
        _width: u32,
        _height: u32,
        output_view: &TextureView,
        _device: &Device,
        _queue: &Queue,
    ) {
        // TODO: P3 完整实现
        // 1. 遍历 dirty_rows
        // 2. 对每个脏行构建 cosmic_text::Buffer
        // 3. 调用 glyphon::TextRenderer::prepare() 光栅化
        // 4. 在 RenderPass 中调用 glyphon::TextRenderer::render()
        // 5. 绘制背景/光标
        // 6. 清除 dirty_rows
    }
}
