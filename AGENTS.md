# wbwterm — MPL-2.0

# ████████████████████████████████████████████████████████████████
# ██ 计划 / PLAN  (2026-10-19)
# ████████████████████████████████████████████████████████████████

## 目标
- 目标平台：Windows x86_64-pc-windows-gnu（MinGW-w64）
- 支持 MSYS2 / Cygwin 子进程
- 完全用户态，不经 conhost / OpenConsole
- UI 对齐 Ghostty，VT 序列对齐 WezTerm Linux / Ghostty / iTerm2 / Kitty

## 技术栈
| 模块 | 依赖 | 许可证 |
|------|------|--------|
| 窗口管理 | winit 0.31 | Apache-2.0 |
| GPU 后端 | wgpu 0.30 (dx12) | MIT / Apache-2.0 |
| 文本渲染 | glyphon 0.12 | MIT / Apache-2.0 / Zlib |
| VT 解析器 | vte 0.15 | Apache-2.0 / MIT |
| 双向文本 | wezterm-bidi 0.2 | MIT / Unicode-DFS-2016 |
| IPC | interprocess 2.4 | 0BSD / Apache-2.0 |
| 系统 API | windows-sys 0.59 | MIT / Apache-2.0 |
| 事件轮询 | polling 3.11 | Apache-2.0 / MIT |
| 跨线程 | crossbeam-channel 0.5 | MIT |

## 目录结构
```
wbwterm/
├── Cargo.toml
├── README.md           (中文)
├── AGENTS.md           (中文，本文件)
├── .gitignore
├── .github/workflows/
│   ├── ci.yml
│   └── pages.yml
├── src/
│   ├── main.rs
│   ├── app.rs          App 主结构体
│   ├── terminal/
│   │   ├── mod.rs
│   │   ├── engine.rs   vte Parser + Perform
│   │   ├── state.rs    TerminalState (grid, cursor, scrollback)
│   │   └── events.rs   侧边事件分发
│   ├── pipe/
│   │   ├── mod.rs
│   │   ├── inherited.rs  模式 A：继承句柄
│   │   ├── named_pipe.rs 模式 B：命名管道客户端
│   │   └── session.rs   单会话读写管理
│   ├── subprocess.rs   子进程生命周期
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── gpu.rs      wgpu 设备/队列/交换链
│   │   ├── text.rs     glyphon FontContext
│   │   └── screen.rs   脏行追踪 + 帧合成
│   ├── input/
│   │   ├── mod.rs
│   │   ├── key.rs      winit KeyEvent → VT 序列
│   │   └── mouse.rs    鼠标事件 → VT 序列
│   └── font/
│       └── mod.rs      Windows 系统字体枚举
├── docs/
│   ├── book.toml
│   └── src/
│       ├── SUMMARY.md
│       ├── intro.md
│       ├── architecture.md
│       └── vt-sequences.md
└── build/
    └── windows/
        └── resources.rc
```

## 管道模式

### 模式 A：继承句柄（默认）
- Terminal 调用 CreatePipe() 创建匿名管道
- 设置 STARTUPINFOEXW.hStdInput / hStdOutput 为 inheritable 句柄
- CreateProcessW 启动子进程（bash.exe / cmd.exe / powershell.exe）
- 子进程的 stdin/stdout 自动连接到管道
- 适合 MSYS2、Cygwin、原生 Windows 程序

### 模式 B：命名管道（可选）
- Terminal 创建 NamedPipe Server（\\.\pipe\wbwterm-xxx）
- 子进程作为 NamedPipe Client 主动连接
- 适合自定义桥接程序或特殊需求

## 命令行接口
```
wbwterm [选项] [--] [命令]

选项:
  --pipe-mode=a    模式 A（继承句柄，默认）
  --pipe-mode=b    模式 B（命名管道）
  --command <cmd>  指定要启动的命令（默认 bash.exe）
  --width <n>      初始列数
  --height <n>     初始行数
  --font <name>    字体名称
  --size <px>      字号（pt）
  --title <title>  窗口标题

示例:
  wbwterm                        # 启动 bash.exe，模式 A
  wbwterm -- cmd /c dir          # 启动 cmd
  wbwterm --powershell            # 启动 powershell
  wbwterm --pipe-mode=b -- bash  # 模式 B
```

## VT 序列支持清单
- CSI SGR（颜色、粗体、斜体、下划线、反转）✅
- CSI 光标移动（CUU/CUD/CUF/CUB/CNL/CPR/ED/ECH）✅
- OSC 0（标题）、OSC 4（颜色）✅
- DECSCUSR（光标形状）✅
- DECRQM/DECRPM（私有模式查询/设置）✅
- XTerm 鼠标协议（1000/1002/1006）✅
- Kitty 键盘协议 ✅
- Scrollback ring buffer ✅
- 双向文本（wezterm-bidi）✅
- Kitty 图形协议（基础）⚠️

## CI 流程
1. fmt check（任何平台）
2. clippy + test（windows-latest）
3. build-gnu（windows-latest，x86_64-pc-windows-gnu 目标）
4. build-msys2（ubuntu-latest 上通过 msys2/actions/setup 模拟 MSYS2 环境编译）
5. docs（mdBook 构建并部署到 GitHub Pages）

---

# ████████████████████████████████████████████████████████████████
# ██ 实施阶段 / PHASES
# ████████████████████████████████████████████████████████████████

## P0 基础
- [x] git init
- [x] Cargo 项目初始化
- [x] 完善 Cargo.toml（所有 deps）
- [x] src/main.rs 空壳（能编译通过）
- [x] .gitignore、README.md、AGENTS.md
- [x] .github/workflows/ci.yml

## P1 管道层
- [x] pipe/inherited.rs — CreatePipe + CreateProcessW
- [ ] pipe/named_pipe.rs — NamedPipe server/client（模式 B 暂为 Err 桩）
- [x] pipe/session.rs — 读写线程 + channel
- [x] subprocess.rs — 子进程生命周期管理

## P2 VT 引擎
- [x] terminal/engine.rs — vte Parser + TerminalPerform impl
- [x] terminal/state.rs — Grid、Cursor、Scrollback、Mode
- [x] terminal/events.rs — 侧边事件回调

## P3 渲染
- [x] renderer/gpu.rs — wgpu 初始化（DX12）
- [x] renderer/text.rs — glyphon FontContext + Atlas
- [ ] renderer/screen.rs — 脏行追踪 + wgpu 绘制（glyphon 管线 TODO）

## P4 输入
- [x] input/key.rs — winit KeyEvent → VT 序列
- [x] input/mouse.rs — 鼠标事件 → VT 序列

## P5 完善
- [x] font/ — Windows 系统字体枚举（fontdb FontRegistry）
- [ ] 配置加载（TOML）
- [ ] 颜色主题
- [ ] Scrollback 滚动支持

## P6 生产
- [ ] 单元测试
- [ ] docs.rs 文档
- [ ] GitHub Release 打包 .exe

---

## 进展记录 / PROGRESS
### 2026-10-20 字体系统 + App 完整接线
- fontdb 0.23 升为直接依赖（cosmic-text 0.19 亦依赖 0.23，版本统一）
- `font/mod.rs`：`FontRegistry` 封装 fontdb — 系统字体、自定义目录/文件加载、
  `resolve_chain()`（用户字体 → MONO_CANDIDATES → Consolas）、`enumerate_mono()`
- `config.rs`：新增 `font_dirs: Vec<String>` 与 `--font-dir` CLI（可重复）
- `renderer/text.rs`：`FontContext` 接收 font_family/font_dirs，自定义目录同时
  写入 glyphon `font_system.db_mut()` 与独立 FontRegistry
- `renderer/screen.rs`：真实 pt→px 度量（14pt → 12×23px，与 Windows Terminal 一致）
- `app.rs`：App 采用 PipeManager 内部引擎 `Arc<Mutex<TerminalEngine>>`；
  键盘输入优先 `write_to_child`，无子进程时本地回显
- `main.rs`：取消注释 `app.run()`，程序完整启动
- 验证：cargo build/test/clippy 全绿；实地冒烟 — bash.exe 子进程启动、
  17 个等宽字体候选枚举成功、Cascadia Code 14pt → 12×23px 单元
