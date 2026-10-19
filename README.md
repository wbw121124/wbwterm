# wbwterm

**wbwterm** 是一款面向 Windows 的高性能终端模拟器，不经过 conhost / OpenConsole，完全在用户态运行。

- VT 序列覆盖对齐 WezTerm Linux / Ghostty / iTerm2 / Kitty
- UI 风格对齐 Ghostty（现代、简洁、高性能）
- 支持 MSYS2 和 Cygwin 子进程

## 技术选型

| 层 | 库 | 说明 |
|----|----|------|
| 窗口 | winit 0.31 | 事件循环、窗口管理 |
| GPU | wgpu 0.30 (DX12) | 硬件加速渲染 |
| 文本 | glyphon 0.12 | wgpu 原生字体渲染 |
| VT 解析 | vte 0.15 | 纯 Rust，Alacritty 同款 |
| 双向文本 | wezterm-bidi 0.2 | WezTerm UBA 实现 |
| IPC | interprocess 2.4 | 命名管道 + 继承句柄 |
| 系统 API | windows-sys 0.59 | 低层级 Win32 访问 |

## 管道模式

**模式 A（继承句柄）** — 默认
- 子进程 stdin/stdout/stderr 直接继承管道句柄
- 支持 bash.exe、cmd.exe、powershell.exe 等原生程序

**模式 B（命名管道）** — 可选
- Terminal 创建命名管道服务端
- 子进程主动连接，双向字节流

## 快速开始

```bash
# 构建
cargo build --release

# 运行（默认启动 bash.exe）
cargo run --release

# 指定命令
cargo run --release -- cmd /c "dir"
```

## 构建要求

- Rust 工具链（稳定版），MSRV 1.87
- 目标：`x86_64-pc-windows-gnu`（MinGW-w64）
- MSYS2 环境（可选，用于测试 MSYS2 bash）

```bash
# 安装目标
rustup target add x86_64-pc-windows-gnu
```

## 许可证

本项目采用 **MPL-2.0** 许可证。
