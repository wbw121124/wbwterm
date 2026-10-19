// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! GPU 渲染后端 — wgpu 设备/队列/交换链
//!
//! ## 渲染管线
//! ```
//! TerminalState.dirty_rows
//!     └─► glyphon FontContext + Atlas
//!           └─► wgpu Device/Queue
//!                 └─► SwapChain → present()
//! ```
//!
//! ## 脏行追踪
//! - terminal::state 维护 BTreeSet<usize> 标记脏行
//! - renderer 每帧只重新合成脏行区域（减少 GPU 带宽）

use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

/// GPU 渲染器
pub struct GpuRenderer {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
}

impl GpuRenderer {
    /// 创建 GPU 渲染器（wgpu Device + Surface）
    ///
    /// TODO: P3 阶段完善，当前保留接口骨架
    pub async fn new(_surface: Surface<'static>, _width: u32, _height: u32) -> Self {
        // TODO: 初始化 wgpu Instance、Adapter、Device
        // self.config = SurfaceConfiguration { width, height, format, ... };
        Self {
            device: panic!("P3: wgpu Device 未初始化"),
            queue: panic!("P3: wgpu Queue 未初始化"),
            surface: _surface,
            config: SurfaceConfiguration {
                width: _width.max(1),
                height: _height.max(1),
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: vec![],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
            },
        }
    }

    /// 调整交换链尺寸（窗口 resize 时调用）
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        // TODO: self.surface.configure(&self.device, &self.config);
    }
}
