// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! VT 终端状态 — Grid、Cursor、Scrollback、Mode
//!
//! Grid 采用单行 Vec<Cell>，行号从 0 开始（scrollback 虚拟行号独立计算）。
//! Cursor 记录当前格位与可见光标线。
//! Mode 记录 DEC 私有模式与终端模式。

use unicode_width::UnicodeWidthChar;
use wezterm_bidi::BidiContext;

/// 终端格子单元格
#[derive(Clone, Debug, Default)]
pub struct Cell {
    /// 显示字符（0, 1 或 2 列宽）
    pub ch: char,
    /// SGR 属性（前景色、背景色、粗体等）
    pub fg: u32,
    pub bg: u32,
    pub bold: bool,
    pub underline: bool,
}

/// 光标位置（0-indexed）
#[derive(Clone, Debug, Default)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
    pub visible: bool,
    pub style: CursorStyle,
}

/// 光标形状
#[derive(Clone, Debug, Default)]
pub enum CursorStyle {
    #[default]
    Block,
    Underline,
    Beam,
}

/// DEC 私有模式
#[derive(Clone, Debug, Default)]
pub struct ModeFlags {
    /// mode 1000: XTerm 鼠标跟踪
    pub mouse_tracking: bool,
    /// mode 1002: 按钮式鼠标跟踪
    pub button_tracking: bool,
    /// mode 1006: 扩展鼠标协议
    pub extended_protocol: bool,
    /// mode 2004: 粘贴帧协议
    pub paste_protocol: bool,
    /// mode 5522: Kitty 剪贴板协议
    pub kitty_clipboard: bool,
    /// mode 1049: 备用屏幕缓冲区
    pub alt_screen: bool,
}

/// 终端状态
pub struct TerminalState {
    /// 列数（可见区域）
    pub cols: u16,
    /// 行数（可见区域）
    pub rows: u16,
    /// 主缓冲区（当前可见行，不含 scrollback）
    pub grid: Vec<Vec<Cell>>,
    /// 前导空白行（scrollback top）
    pub scrollback: Vec<Vec<Vec<Cell>>>,
    /// 当前光标位置
    pub cursor: Cursor,
    /// 保存的光标（DECSC）
    pub saved_cursor: Option<Cursor>,
    /// 当前模式标志
    pub modes: ModeFlags,
    /// 标题字符串
    pub title: String,
    /// 颜色调色板（OSC 4）
    pub palette: [u32; 256],
    /// 脏行集合（供渲染层按需重绘）
    pub dirty_rows: std::collections::BTreeSet<usize>,
    /// 双向文本上下文
    pub bidi: BidiContext,
}

impl TerminalState {
    /// 创建新终端状态
    pub fn new(cols: u16, rows: u16) -> Self {
        let mut grid = Vec::with_capacity(rows as usize);
        for _ in 0..rows {
            grid.push(vec![Cell::default(); cols as usize]);
        }
        Self {
            cols,
            rows,
            grid,
            scrollback: Vec::new(),
            cursor: Cursor::default(),
            saved_cursor: None,
            modes: ModeFlags::default(),
            title: String::new(),
            palette: [0; 256],
            dirty_rows: std::collections::BTreeSet::new(),
            bidi: BidiContext::new(),
        }
    }

    /// 标记指定行为脏行
    pub fn mark_dirty(&mut self, row: usize) {
        self.dirty_rows.insert(row);
    }

    /// 标记多行脏
    pub fn mark_dirty_range(&mut self, start: usize, end: usize) {
        for r in start..=end {
            self.dirty_rows.insert(r);
        }
    }

    /// 清除所有脏行标记
    pub fn clear_dirty(&mut self) {
        self.dirty_rows.clear();
    }

    /// 窗口尺寸变化
    pub fn resize(&mut self, cols: u16, rows: u16) {
        let old_cols = self.cols;
        let old_rows = self.rows;
        self.cols = cols;
        self.rows = rows;

        // 将旧网格内容迁移到新网格
        let mut new_grid = Vec::with_capacity(rows as usize);
        for r in 0..rows {
            let mut row = Vec::with_capacity(cols as usize);
            for c in 0..cols {
                if c < old_cols {
                    if let Some(old_row) = self.grid.get(old_rows as usize) {
                        if let Some(cell) = old_row.get(c as usize) {
                            row.push(cell.clone());
                        } else {
                            row.push(Cell::default());
                        }
                    } else {
                        row.push(Cell::default());
                    }
                } else {
                    row.push(Cell::default());
                }
            }
            new_grid.push(row);
        }
        self.grid = new_grid;
    }

    /// 获取指定格子的可见宽度
    pub fn cell_width(&self, col: usize, row: usize) -> usize {
        if row >= self.rows as usize { return 0; }
        let ch = self.grid[row].get(col).map(|c| c.ch).unwrap_or(' ');
        ch.width().unwrap_or(0)
    }
}
