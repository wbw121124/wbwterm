// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 命令行参数解析

use std::env;

/// 应用配置
#[derive(Clone, Debug)]
pub struct Config {
    pub pipe_mode: PipeMode,
    pub command: Vec<String>,
    pub cols: u16,
    pub rows: u16,
    pub font: String,
    pub font_size: f32,
    pub title: String,
}

/// 管道模式
#[derive(Clone, Debug, Default)]
pub enum PipeMode {
    #[default]
    Inherited,
    NamedPipe,
}

impl Config {
    /// 从命令行参数解析配置
    pub fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut config = Self::default();

        let mut i = 1;
        let mut command_args: Vec<String> = Vec::new();
        let mut expect_command = false;

        while i < args.len() {
            match args[i].as_str() {
                "--pipe-mode=a" | "--pipe-mode=inherited" => {
                    config.pipe_mode = PipeMode::Inherited;
                }
                "--pipe-mode=b" | "--pipe-mode=named" => {
                    config.pipe_mode = PipeMode::NamedPipe;
                }
                "--command" | "-c" => {
                    expect_command = true;
                }
                "--width" | "-w" => {
                    if i + 1 < args.len() {
                        i += 1;
                        config.cols = args[i].parse().unwrap_or(80);
                    }
                }
                "--height" | "-h" => {
                    if i + 1 < args.len() {
                        i += 1;
                        config.rows = args[i].parse().unwrap_or(24);
                    }
                }
                "--font" => {
                    if i + 1 < args.len() {
                        i += 1;
                        config.font = args[i].clone();
                    }
                }
                "--size" => {
                    if i + 1 < args.len() {
                        i += 1;
                        config.font_size = args[i].parse().unwrap_or(14.0);
                    }
                }
                "--title" => {
                    if i + 1 < args.len() {
                        i += 1;
                        config.title = args[i].clone();
                    }
                }
                "--" => {
                    // 命令分隔符，之后全部作为子进程参数
                    i += 1;
                    command_args.extend(args[i..].to_vec());
                    break;
                }
                _ if !args[i].starts_with('-') => {
                    command_args.push(args[i].clone());
                }
                _ => {
                    eprintln!("未知参数: {}", args[i]);
                }
            }
            i += 1;
        }

        // 如果提供了 --command 但没有后续参数，从 args 中取
        if expect_command && !command_args.is_empty() {
            config.command = command_args;
        } else if command_args.is_empty() {
            // 默认启动 bash.exe
            config.command = vec!["bash.exe".to_string()];
        } else {
            config.command = command_args;
        }

        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pipe_mode: PipeMode::default(),
            command: vec!["bash.exe".to_string()],
            cols: 80,
            rows: 24,
            font: "Cascadia Code".to_string(),
            font_size: 14.0,
            title: "wbwterm".to_string(),
        }
    }
}
