// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! vte Parser 的 Perform trait 实现
//!
//! 将 vte 解析出的每个动作映射到 TerminalState 的状态变更。

use super::events::TerminalEvent;
use super::state::{Cell, TerminalState};
use unicode_width::UnicodeWidthChar;
use vte::{Params, Perform};

/// vte Perform 实现
pub struct TerminalPerform<'a> {
    pub state: &'a mut TerminalState,
    pub events: &'a mut super::events::TerminalEvents,
}

/// 从 Params 中提取第一个参数的第一个值（i64）
fn param_first_i64(params: &Params) -> Option<i64> {
    params
        .iter()
        .next()
        .and_then(|p| p.first())
        .copied()
        .map(|v| v as i64)
}

impl<'a> Perform for TerminalPerform<'a> {
    fn print(&mut self, ch: char) {
        let col = self.state.cursor.col;
        let row = self.state.cursor.row;

        if row < self.state.rows as usize && col < self.state.cols as usize {
            self.state.grid[row][col] = Cell {
                ch,
                fg: 0,
                bg: 1,
                bold: false,
                underline: false,
            };
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
                self.state.cursor.col = self.state.cursor.col.saturating_sub(1);
            }
            0x09 => {}
            0x0A => {
                self.state.cursor.row =
                    (self.state.cursor.row + 1).min(self.state.rows as usize - 1);
                self.state.mark_dirty(self.state.cursor.row);
            }
            0x0D => {
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
            self.state.cursor = super::state::Cursor::default();
            self.state.mark_dirty_range(0, self.state.rows as usize - 1);
        }
    }
}

impl<'a> TerminalPerform<'a> {
    fn handle_sgr(&mut self, params: &Params) {
        // Simplified: only handle reset (0)
        if let Some(0) = param_first_i64(params) {
            // would reset all styles
        }
    }

    fn handle_cuu(&mut self, params: &Params) {
        let n = param_first_i64(params).unwrap_or(1).max(1) as usize;
        self.state.cursor.row = self.state.cursor.row.saturating_sub(n);
        self.state.mark_dirty(self.state.cursor.row);
    }

    fn handle_cud(&mut self, params: &Params) {
        let n = param_first_i64(params).unwrap_or(1).max(1) as usize;
        self.state.cursor.row = (self.state.cursor.row + n).min(self.state.rows as usize - 1);
        self.state.mark_dirty(self.state.cursor.row);
    }

    fn handle_cuf(&mut self, params: &Params) {
        let n = param_first_i64(params).unwrap_or(1).max(1) as usize;
        self.state.cursor.col = (self.state.cursor.col + n).min(self.state.cols as usize - 1);
    }

    fn handle_cub(&mut self, params: &Params) {
        let n = param_first_i64(params).unwrap_or(1).max(1) as usize;
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
        let n = param_first_i64(params).unwrap_or(1).max(1) as usize;
        self.state.cursor.col = (n - 1).min(self.state.cols as usize - 1);
    }

    fn handle_ed(&mut self, params: &Params) {
        let mode = param_first_i64(params).unwrap_or(0) as u8;
        let row = self.state.cursor.row;
        match mode {
            0 => {
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
        let mode = param_first_i64(params).unwrap_or(0) as u8;
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
        let req = param_first_i64(params).unwrap_or(0);
        if req == 6 {
            let r = self.state.cursor.row + 1;
            let c = self.state.cursor.col + 1;
            self.events
                .push(TerminalEvent::UnknownSequence(format!("\x1b[{};{}R", r, c)));
        }
    }
}
