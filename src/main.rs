// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 主入口 — 解析命令行参数并启动 App

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

    let app = app::App::new(config);
    // TODO: P3 完成后调用 app.run()
    // app.run().expect("App 运行失败");

    println!("（空壳运行，P3 渲染完成后启动完整循环）");
}
