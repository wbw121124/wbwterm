// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! App 主结构体 — 整合窗口、终端引擎、管道、渲染

use winit::application::ApplicationHandler;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

use crate::config::Config;
use crate::input::key::{key_event_to_vt, unicode_input_to_vt};
use crate::terminal::TerminalEngine;

/// App 主结构体
pub struct App {
    config: Config,
    window: Option<Window>,
    engine: Option<TerminalEngine>,
}

impl App {
    /// 创建 App
    pub fn new(config: Config) -> Self {
        Self {
            config,
            window: None,
            engine: None,
        }
    }

    /// 运行主循环
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;
        Ok(())
    }

    /// 初始化窗口
    fn init_window(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.cols as f64 * 10.0,
                self.config.rows as f64 * 18.0,
            ));
        let window = event_loop
            .create_window(attrs)
            .expect("Failed to create window");
        self.window = Some(window);
        println!("窗口创建成功: {}x{}", self.config.cols, self.config.rows);
    }

    /// 初始化终端引擎
    fn init_engine(&mut self) {
        let cols = self.config.cols;
        let rows = self.config.rows;
        self.engine = Some(TerminalEngine::new(cols, rows));
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.init_window(event_loop);
        }
        if self.engine.is_none() {
            self.init_engine();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // 渲染逻辑在 P3 阶段实现
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
                if let Some(engine) = &mut self.engine {
                    if let Some(text) = &key_event.text {
                        engine.feed(text.as_bytes());
                    }
                    if let Some(vt_seq) = key_event_to_vt(&key_event) {
                        engine.feed(&vt_seq);
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                println!("窗口大小: {}x{}", physical_size.width, physical_size.height);
                if let Some(engine) = &mut self.engine {
                    // 用物理尺寸估算行列数（假设 10px/col, 18px/row）
                    let cols = (physical_size.width as f64 / 10.0) as u16;
                    let rows = (physical_size.height as f64 / 18.0) as u16;
                    engine.resize(cols.max(1), rows.max(1));
                }
            }
            _ => {}
        }
    }
}
