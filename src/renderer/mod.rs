// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! GPU 渲染层 — wgpu + glyphon 集成
//!
//! ## 模块
//! - `gpu`: wgpu Device/Surface/Queue 管理
//! - `text`: glyphon FontContext + 字体枚举
//! - `screen`: 脏行追踪 + glyphon 合成 + wgpu 绘制

pub mod gpu;
pub mod screen;
pub mod text;

use crate::terminal::TerminalEngine;
use std::sync::{Arc, Mutex};
use winit::window::Window;

/// 渲染器主结构体（Arc<Mutex<>> 包装以支持跨线程访问）
pub struct Renderer {
    inner: Arc<Mutex<RendererInner>>,
}

struct RendererInner {
    gpu: gpu::GpuRenderer,
    screen: screen::ScreenRenderer,
}

impl Renderer {
    /// 创建渲染器（初始化 wgpu 设备 + glyphon 字体上下文）
    ///
    /// ## 参数
    /// - `font_family`: 用户指定字体族（空/"auto" 走候选链）
    /// - `font_size`: 字号（pt）
    /// - `font_dirs`: 自定义字体目录/文件
    pub async fn new(
        window: &Window,
        font_family: &str,
        font_size: f32,
        font_dirs: &[String],
    ) -> Self {
        let gpu = gpu::GpuRenderer::new(window).await;
        let font_context = text::FontContext::new(&gpu.device, &gpu.queue, font_family, font_dirs);
        let resolved_family = font_context.resolved_family.clone();
        let mono_count = font_context.enumerate_mono().len();
        let screen = screen::ScreenRenderer::new(
            gpu.device.clone(),
            gpu.queue.clone(),
            font_context,
            font_size,
        );
        eprintln!(
            "字体解析: {}（等宽候选 {} 个）",
            resolved_family, mono_count
        );
        Self {
            inner: Arc::new(Mutex::new(RendererInner { gpu, screen })),
        }
    }

    /// 窗口 resize 时调用
    pub fn resize(&self, window: &Window) {
        let mut inner = self.inner.lock().unwrap();
        inner.gpu.resize(window);
    }

    /// 当前单元格尺寸（宽, 高）px
    pub fn cell_size(&self) -> (u32, u32) {
        self.inner.lock().unwrap().screen.cell_size()
    }

    /// 终端尺寸变化时更新渲染器
    pub fn set_term_size(&self, cols: u16, rows: u16) {
        self.inner.lock().unwrap().screen.set_term_size(cols, rows);
    }

    /// 渲染一帧（根据 dirty_rows 增量合成）
    pub fn render(&self, engine: &mut TerminalEngine) -> Result<(), String> {
        let state = engine.state();
        let dirty = state.dirty_rows.clone();
        if dirty.is_empty() {
            return Ok(());
        }

        {
            let mut inner = self.inner.lock().unwrap();
            inner.screen.clear_dirty();
            for &row in &dirty {
                inner.screen.mark_dirty(row);
            }
        }

        let current = self.inner.lock().unwrap().gpu.get_current_texture();
        match current {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                let _view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                // TODO: P3 完整实现 — glyphon 渲染管线
                drop(_view);
                // wgpu 30: 提交空命令以标记帧已呈现
                self.inner
                    .lock()
                    .unwrap()
                    .gpu
                    .queue
                    .submit(std::iter::empty());
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation => {
                return Err("surface texture unavailable".to_string());
            }
        }

        engine.state_mut().clear_dirty();
        Ok(())
    }
}
