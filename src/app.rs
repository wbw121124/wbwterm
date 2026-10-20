// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! App 主结构体 — 整合窗口、终端引擎、管道、渲染

use std::sync::{Arc, Mutex};

use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

use crate::config::Config;
use crate::input::key::key_event_to_vt;
use crate::pipe::{PipeConfig, PipeManager};
use crate::renderer::Renderer;
use crate::terminal::TerminalEngine;

/// App 主结构体
pub struct App {
    config: Config,
    window: Option<Window>,
    /// 终端引擎（与 PipeManager 共享同一实例）
    engine: Option<Arc<Mutex<TerminalEngine>>>,
    pipe: Option<PipeManager>,
    renderer: Option<Renderer>,
    /// 键盘输入转发到子进程失败时的本地回显标志
    direct_feed: bool,
}

impl App {
    /// 创建 App
    pub fn new(config: Config) -> Self {
        Self {
            config,
            window: None,
            engine: None,
            pipe: None,
            renderer: None,
            direct_feed: false,
        }
    }

    /// 运行主循环
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;
        Ok(())
    }

    /// 初始化窗口
    ///
    /// 初始尺寸按默认 10×20px 单元估算；创建 renderer 后按真实字体度量重设
    fn init_window(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.cols as f64 * 10.0,
                self.config.rows as f64 * 20.0,
            ));
        let window = event_loop
            .create_window(attrs)
            .expect("Failed to create window");
        self.window = Some(window);
        println!("窗口创建成功: {}x{}", self.config.cols, self.config.rows);
    }

    /// 初始化管道管理器（含共享终端引擎与子进程）
    fn init_subprocess(&mut self) {
        let pipe_config = PipeConfig::new(
            self.config.command.clone(),
            self.config.cols,
            self.config.rows,
        );
        let mut pm = PipeManager::new(pipe_config);
        match pm.spawn() {
            Ok(_) => {
                self.engine = Some(pm.engine_ref());
                self.pipe = Some(pm);
                if self.direct_feed {
                    println!("直接回显模式（无子进程）");
                }
                println!("子进程已启动: {:?}", self.config.command);
            }
            Err(e) => {
                eprintln!("子进程启动失败: {e}");
                // 管道失败时保留本地引擎，走直接回显
                let engine = TerminalEngine::new(self.config.cols, self.config.rows);
                self.engine = Some(Arc::new(Mutex::new(engine)));
                self.direct_feed = true;
            }
        }
    }

    /// 初始化渲染器（wgpu 设备创建为异步，事件循环里阻塞等待）
    fn init_renderer(&mut self) {
        if let Some(window) = &self.window {
            let renderer = pollster::block_on(Renderer::new(
                window,
                &self.config.font,
                self.config.font_size,
                &self.config.font_dirs,
            ));
            // 按真实字体度量重置窗口与终端尺寸
            let (cw, ch) = renderer.cell_size();
            window.request_inner_size(winit::dpi::LogicalSize::new(
                self.config.cols as f64 * cw as f64,
                self.config.rows as f64 * ch as f64,
            ));
            renderer.set_term_size(self.config.cols, self.config.rows);
            self.renderer = Some(renderer);
            println!(
                "渲染器就绪: 单元格 {}x{}px, {}x{} 单元格",
                cw, ch, self.config.cols, self.config.rows
            );
        }
    }

    /// 键盘输入转发：优先写入子进程，无管道时本地回显
    fn handle_key(&mut self, event: &winit::event::KeyEvent) {
        let vt_seq = key_event_to_vt(event);
        let text_bytes: Option<Vec<u8>> = event.text.as_ref().map(|t| t.as_bytes().to_vec());

        if let Some(pm) = &self.pipe {
            if !self.direct_feed {
                if let Some(seq) = &vt_seq {
                    if let Err(e) = pm.write_to_child(seq) {
                        eprintln!("写入子进程失败: {e}");
                    }
                }
                if let Some(bytes) = &text_bytes {
                    if let Err(e) = pm.write_to_child(bytes) {
                        eprintln!("写入子进程失败: {e}");
                    }
                }
                return;
            }
        }
        if self.direct_feed {
            if let Some(engine) = &self.engine {
                let mut g = engine.lock().unwrap();
                if let Some(bytes) = &text_bytes {
                    g.feed(bytes);
                }
                if let Some(seq) = &vt_seq {
                    g.feed(seq);
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.init_window(event_loop);
        }
        if self.engine.is_none() {
            self.init_subprocess();
        }
        if self.renderer.is_none() {
            self.init_renderer();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // 渲染脏行（由 about_to_wait 在事件清空后触发）
        if let (Some(renderer), Some(engine)) = (&self.renderer, &self.engine) {
            let mut g = engine.lock().unwrap();
            if let Err(e) = renderer.render(&mut g) {
                eprintln!("渲染错误: {e}");
            }
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::ResumeTimeReached { .. } | StartCause::Poll => {}
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("收到关闭请求，退出");
                _event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state == winit::event::ElementState::Pressed {
                    self.handle_key(&key_event);
                }
            }
            WindowEvent::Resized(physical_size) => {
                println!("窗口大小: {}x{}", physical_size.width, physical_size.height);
                if let Some(engine) = &self.engine {
                    // 用真实字体度量估算行列数
                    let (cw, ch) = self
                        .renderer
                        .as_ref()
                        .map(|r| r.cell_size())
                        .unwrap_or((10, 20));
                    let cols = (physical_size.width as f64 / cw as f64) as u16;
                    let rows = (physical_size.height as f64 / ch as f64) as u16;
                    let mut g = engine.lock().unwrap();
                    g.resize(cols.max(1), rows.max(1));
                    drop(g);
                    if let Some(renderer) = &self.renderer {
                        renderer.set_term_size(cols.max(1), rows.max(1));
                        renderer.resize(self.window.as_ref().unwrap());
                    }
                }
            }
            _ => {}
        }
    }
}
