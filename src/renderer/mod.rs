// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! GPU 渲染层 — wgpu + glyphon 集成
//!
//! ## 模块
//! - `gpu`: wgpu Device/Surface/Queue 管理
//! - `text`: glyphon FontContext + 字体枚举
//! - `screen`: 脏行追踪 + 帧合成

pub mod gpu;
pub mod screen;
pub mod text;

use crate::terminal::TerminalEngine;
use winit::window::Window;

/// 渲染器主结构体
pub struct Renderer {
    _gpu: gpu::GpuRenderer,
    _font: text::FontContext,
    _screen: screen::ScreenRenderer,
}

impl Renderer {
    /// 创建渲染器（初始化 wgpu 设备 + glyphon）
    pub async fn new(_window: Window) -> Self {
        // TODO: P3 阶段完善
        let _font = text::FontContext::new();
        #[allow(clippy::diverging_sub_expression)]
        let _gpu = gpu::GpuRenderer::new(panic!("P3: wgpu Surface 未初始化"), 800, 600).await;
        #[allow(clippy::diverging_sub_expression)]
        let _screen = screen::ScreenRenderer::new(panic!("P3"));

        Self {
            _gpu,
            _font,
            _screen,
        }
    }

    /// 窗口 resize 时调用
    pub fn resize(&mut self, _width: u32, _height: u32) {
        // TODO: P3
    }

    /// 根据终端状态渲染一帧
    pub fn render(&mut self, _engine: &TerminalEngine) {
        // TODO: P3
    }
}
