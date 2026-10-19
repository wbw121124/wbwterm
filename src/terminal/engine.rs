// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! vte Parser 的 Perform trait 实现
//!
//! 将 vte 解析出的每个动作映射到 TerminalState 的状态变更。

use super::events::TerminalEvent;
use super::state::{Cell, CursorStyle, TerminalState};
use unicode_width::UnicodeWidthChar;
use vte::{Params, Perform};

/// SGR 属性（颜色、粗体、斜体、下划线、反转）
#[derive(Clone, Debug, Default)]
pub struct SGRAttrs {
    pub fg: Option<u32>,
    pub bg: Option<u32>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub concealed: bool,
    pub crossed_out: bool,
}

impl SGRAttrs {
    fn apply_to_cell(&self, cell: &mut Cell) {
        if let Some(fg) = self.fg {
            cell.fg = fg;
        }
        if let Some(bg) = self.bg {
            cell.bg = bg;
        }
        if self.bold {
            cell.bold = true;
        }
        if self.underline {
            cell.underline = true;
        }
    }
}

/// vte Perform 实现
pub struct TerminalPerform<'a> {
    pub state: &'a mut TerminalState,
    pub events: &'a mut super::events::TerminalEvents,
    /// 当前 SGR 属性累积
    pub sgr: SGRAttrs,
}

/// 从 Params 中提取第 i 个参数的第一个值（i64）
fn param_first(params: &Params) -> i64 {
    params
        .iter()
        .next()
        .and_then(|p| p.first())
        .copied()
        .map(|v| v as i64)
        .unwrap_or(0)
}

/// 将 SGR 索引（0-255）解析为 u32（RRGGBB）
fn sgr_index_to_color(idx: i64) -> u32 {
    match idx {
        0 => 0x000000,
        1 => 0x800000,
        2 => 0x008000,
        3 => 0x808000,
        4 => 0x000080,
        5 => 0x800080,
        6 => 0x008080,
        7 => 0xC0C0C0,
        8 => 0x808080,
        9 => 0xFF0000,
        10 => 0x00FF00,
        11 => 0xFFFF00,
        12 => 0x0000FF,
        13 => 0xFF00FF,
        14 => 0x00FFFF,
        15 => 0xFFFFFF,
        16..=231 => {
            let n = (idx - 16) as u32;
            let ri = n / 36;
            let gi = (n % 36) / 6;
            let bi = n % 6;
            let r_val = if ri == 0 { 0 } else { 0x5F + 0x20 * (ri - 1) };
            let g_val = if gi == 0 { 0 } else { 0x5F + 0x20 * (gi - 1) };
            let b_val = if bi == 0 { 0 } else { 0x5F + 0x20 * (bi - 1) };
            r_val << 16 | g_val << 8 | b_val
        }
        232..=255 => {
            let n = (idx - 232) as u32;
            let v = 0x08 + 0x0A * n;
            v << 16 | v << 8 | v
        }
        _ => 0x000000,
    }
}

/// 从 Params 解析 SGR 参数列表，返回新的 SGRAttrs
fn parse_sgr(params: &Params) -> SGRAttrs {
    let mut attrs = SGRAttrs::default();
    let mut raw: Vec<i64> = params
        .iter()
        .flat_map(|p| p.iter().copied())
        .map(|v| v as i64)
        .collect();

    if raw.is_empty() {
        raw.push(0);
    }

    let mut i = 0;
    while i < raw.len() {
        let code = raw[i];
        match code {
            0 => {
                attrs = SGRAttrs::default();
            }
            1 => {
                attrs.bold = true;
            }
            2 => {
                attrs.dim = true;
            }
            3 => {
                attrs.italic = true;
            }
            4 => {
                attrs.underline = true;
            }
            5 => {
                attrs.blink = true;
            }
            7 => {
                attrs.reverse = true;
            }
            8 => {
                attrs.concealed = true;
            }
            9 => {
                attrs.crossed_out = true;
            }
            21 => {
                attrs.bold = false;
            }
            22 => {
                attrs.bold = false;
                attrs.dim = false;
            }
            23 => {
                attrs.italic = false;
            }
            24 => {
                attrs.underline = false;
            }
            25 => {
                attrs.blink = false;
            }
            27 => {
                attrs.reverse = false;
            }
            28 => {
                attrs.concealed = false;
            }
            29 => {
                attrs.crossed_out = false;
            }
            30..=37 => {
                attrs.fg = Some(sgr_index_to_color(code - 30));
            }
            38 => {
                // 38;5;n 或 38;2;r;g;b
                if i + 1 < raw.len() {
                    if raw[i + 1] == 5 && i + 2 < raw.len() {
                        attrs.fg = Some(sgr_index_to_color(raw[i + 2]));
                        i += 3;
                        continue;
                    } else if raw[i + 1] == 2 && i + 4 < raw.len() {
                        let r = raw[i + 2] as u32;
                        let g = raw[i + 3] as u32;
                        let b = raw[i + 4] as u32;
                        attrs.fg = Some((r << 16) | (g << 8) | b);
                        i += 5;
                        continue;
                    }
                }
            }
            39 => {
                attrs.fg = None;
            }
            40..=47 => {
                attrs.bg = Some(sgr_index_to_color(code - 40));
            }
            48 => {
                if i + 1 < raw.len() {
                    if raw[i + 1] == 5 && i + 2 < raw.len() {
                        attrs.bg = Some(sgr_index_to_color(raw[i + 2]));
                        i += 3;
                        continue;
                    } else if raw[i + 1] == 2 && i + 4 < raw.len() {
                        let r = raw[i + 2] as u32;
                        let g = raw[i + 3] as u32;
                        let b = raw[i + 4] as u32;
                        attrs.bg = Some((r << 16) | (g << 8) | b);
                        i += 5;
                        continue;
                    }
                }
            }
            49 => {
                attrs.bg = None;
            }
            _ => {}
        }
        i += 1;
    }
    attrs
}

impl<'a> Perform for TerminalPerform<'a> {
    fn print(&mut self, ch: char) {
        let col = self.state.cursor.col;
        let row = self.state.cursor.row;

        if row < self.state.rows as usize && col < self.state.cols as usize {
            let mut cell = Cell {
                ch,
                ..Default::default()
            };
            self.sgr.apply_to_cell(&mut cell);
            self.state.grid[row][col] = cell;
            self.state.mark_dirty(row);
        }

        let cell_width = ch.width().unwrap_or(1);
        if col + cell_width >= self.state.cols as usize {
            self.state.cursor.col = 0;
            self.state.cursor.row = (self.state.cursor.row + 1).min(self.state.rows as usize - 1);
            self.state.mark_dirty(self.state.cursor.row);
        } else {
            self.state.cursor.col += cell_width;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x07 => self.events.push(TerminalEvent::Bell),
            0x08 => {
                // BS
                self.state.cursor.col = self.state.cursor.col.saturating_sub(1);
            }
            0x09 => {
                // HT — tab to next 8-column boundary
                self.state.cursor.col =
                    (self.state.cursor.col + 8).min(self.state.cols as usize - 1);
            }
            0x0A..=0x0C => {
                // LF / VT / FF — move down
                self.state.cursor.row =
                    (self.state.cursor.row + 1).min(self.state.rows as usize - 1);
                self.state.mark_dirty(self.state.cursor.row);
            }
            0x0D => {
                // CR — move to column 0
                self.state.cursor.col = 0;
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
        match _action as u8 {
            b'm' => self.handle_sgr(params),
            b'A' => self.handle_cuu(params),
            b'B' => self.handle_cud(params),
            b'C' => self.handle_cuf(params),
            b'D' => self.handle_cub(params),
            b'E' => self.handle_cnl(params),
            b'F' => self.handle_cpl(params),
            b'G' => self.handle_cha(params),
            b'J' => self.handle_ed(params),
            b'K' => self.handle_eol(params),
            b'n' => self.handle_dsr(params),
            b'q' => self.handle_decscusr(params),
            b'?' => {
                // DEC private mode set/query
                // CSI ? Pm h  (Set) / CSI ? Pm l  (Reset) / CSI ? Pm n  (Report)
            }
            _ => {}
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.is_empty() {
            return;
        }
        if params[0] == b"0" && params.len() >= 2 {
            let title = String::from_utf8_lossy(params[1]).into_owned();
            self.state.title = title.clone();
            self.events.push(TerminalEvent::TitleChanged(title));
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        if byte == b'c' {
            // RIS — Full Reset
            self.state.reset();
            self.sgr = SGRAttrs::default();
        }
    }
}

impl<'a> TerminalPerform<'a> {
    fn handle_sgr(&mut self, params: &Params) {
        let new_attrs = parse_sgr(params);
        // 只更新非 None 的字段，保留现有属性
        if new_attrs.fg.is_some() {
            self.sgr.fg = new_attrs.fg;
        }
        if new_attrs.bg.is_some() {
            self.sgr.bg = new_attrs.bg;
        }
        self.sgr.bold |= new_attrs.bold;
        self.sgr.italic |= new_attrs.italic;
        self.sgr.underline |= new_attrs.underline;
        // 特殊处理：重置全部（code 0）时完全覆盖
        if params.iter().flat_map(|p| p.iter()).any(|&v| v == 0) {
            self.sgr = new_attrs;
        }
    }

    fn handle_cuu(&mut self, params: &Params) {
        let n = param_first(params).max(1) as usize;
        self.state.cursor.row = self.state.cursor.row.saturating_sub(n);
        self.state.mark_dirty(self.state.cursor.row);
    }

    fn handle_cud(&mut self, params: &Params) {
        let n = param_first(params).max(1) as usize;
        self.state.cursor.row = (self.state.cursor.row + n).min(self.state.rows as usize - 1);
        self.state.mark_dirty(self.state.cursor.row);
    }

    fn handle_cuf(&mut self, params: &Params) {
        let n = param_first(params).max(1) as usize;
        self.state.cursor.col = (self.state.cursor.col + n).min(self.state.cols as usize - 1);
    }

    fn handle_cub(&mut self, params: &Params) {
        let n = param_first(params).max(1) as usize;
        self.state.cursor.col = self.state.cursor.col.saturating_sub(n);
    }

    fn handle_cnl(&mut self, _params: &Params) {
        self.state.cursor.row = (self.state.cursor.row + 1).min(self.state.rows as usize - 1);
        self.state.cursor.col = 0;
        self.state.mark_dirty(self.state.cursor.row);
    }

    fn handle_cpl(&mut self, _params: &Params) {
        self.state.cursor.row = self.state.cursor.row.saturating_sub(1);
        self.state.cursor.col = 0;
        self.state.mark_dirty(self.state.cursor.row);
    }

    fn handle_cha(&mut self, params: &Params) {
        let n = param_first(params).max(1) as usize;
        self.state.cursor.col = (n - 1).min(self.state.cols as usize - 1);
    }

    fn handle_ed(&mut self, params: &Params) {
        let mode = param_first(params) as u8;
        let row = self.state.cursor.row;
        match mode {
            0 => {
                // ED: erase from cursor to end of screen
                for r in row..self.state.rows as usize {
                    let start = if r == row {
                        self.state.cursor.col + 1
                    } else {
                        0
                    };
                    for c in start..self.state.cols as usize {
                        self.state.grid[r][c] = Cell::default();
                    }
                    self.state.mark_dirty(r);
                }
            }
            1 => {
                // ED: erase from start to cursor
                for r in 0..row {
                    for c in 0..self.state.cols as usize {
                        self.state.grid[r][c] = Cell::default();
                    }
                    self.state.mark_dirty(r);
                }
                for c in 0..=self.state.cursor.col.min(self.state.cols as usize - 1) {
                    self.state.grid[row][c] = Cell::default();
                }
                self.state.mark_dirty(row);
            }
            2 => {
                // ED: erase entire screen
                for r in 0..self.state.rows as usize {
                    for c in 0..self.state.cols as usize {
                        self.state.grid[r][c] = Cell::default();
                    }
                    self.state.mark_dirty(r);
                }
                self.state.cursor.row = 0;
                self.state.cursor.col = 0;
            }
            _ => {}
        }
    }

    fn handle_eol(&mut self, params: &Params) {
        let mode = param_first(params) as u8;
        let row = self.state.cursor.row;
        let start = match mode {
            0 => self.state.cursor.col + 1,
            1 | 2 => 0,
            _ => self.state.cursor.col + 1,
        };
        for c in start..self.state.cols as usize {
            self.state.grid[row][c] = Cell::default();
        }
        self.state.mark_dirty(row);
    }

    fn handle_dsr(&mut self, params: &Params) {
        let req = param_first(params);
        if req == 6 {
            // CPR — Cursor Position Report
            let r = self.state.cursor.row + 1;
            let c = self.state.cursor.col + 1;
            self.events
                .push(TerminalEvent::UnknownSequence(format!("\x1b[{};{}R", r, c)));
        }
    }

    /// DECSCUSR — 设置光标形状
    /// CSI Sp q  其中 p 为:
    ///   0, 1: block cursor (blinking)
    ///   2: block cursor (steady)
    ///   3: underscore cursor (blinking)
    ///   4: underscore cursor (steady)
    ///   5: bar cursor (blinking)
    ///   6: bar cursor (steady)
    fn handle_decscusr(&mut self, params: &Params) {
        let p = param_first(params);
        match p {
            0 | 1 => self.state.cursor.style = CursorStyle::Block,
            2 => self.state.cursor.style = CursorStyle::Block,
            3 | 4 => self.state.cursor.style = CursorStyle::Underline,
            5 | 6 => self.state.cursor.style = CursorStyle::Beam,
            _ => {}
        }
    }
}
