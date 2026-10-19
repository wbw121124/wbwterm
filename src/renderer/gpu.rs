// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! GPU 渲染后端 — wgpu 设备/队列/交换链（DX12）

use wgpu::{Instance, Surface, SurfaceConfiguration};
use winit::window::Window;

/// GPU 渲染器 — wgpu 设备 / 队列 / 交换链
pub struct GpuRenderer {
    pub _instance: Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
}

impl GpuRenderer {
    pub async fn new(window: &Window) -> Self {
        let instance = Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("wgpu: 无法获取兼容的 GPU adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("wgpu: 无法创建设备");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            alpha_mode: caps.alpha_modes[0],
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        surface.configure(&device, &config);

        Self {
            _instance: instance,
            device,
            queue,
            surface: unsafe { std::mem::transmute::<Surface<'_>, Surface<'static>>(surface) },
            config,
        }
    }

    pub fn resize(&mut self, window: &Window) {
        let size = window.inner_size();
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn get_current_texture(&self) -> wgpu::CurrentSurfaceTexture {
        self.surface.get_current_texture()
    }
}
