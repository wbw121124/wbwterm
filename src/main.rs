// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 主入口 — 解析命令行参数并启动 App

#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unreachable_code,
    unreachable_patterns,
    unused_must_use,
    clippy::new_without_default,
    clippy::upper_case_acronyms
)]

mod app;
mod config;
mod font;
mod input;
mod pipe;
mod renderer;
mod terminal;

fn main() {
    let config = config::Config::from_args();
    println!("wbwterm 启动");
    println!("管道模式: {:?}", config.pipe_mode);
    println!("命令: {:?}", config.command);
    println!("尺寸: {}x{}", config.cols, config.rows);
    println!("字体: {} @ {}pt", config.font, config.font_size);
    if !config.font_dirs.is_empty() {
        println!("自定义字体目录: {:?}", config.font_dirs);
    }

    let app = app::App::new(config);
    app.run().expect("App 运行失败");
}
