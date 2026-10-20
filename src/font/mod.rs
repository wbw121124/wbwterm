// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! Windows 系统字体枚举（fontdb）
//!
//! ## 字体优先级（Windows 系统默认等宽字体）
//! 1. Cascadia Code / Cascadia Mono
//! 2. Consolas
//! 3. Courier New
//!
//! ## 实现方式
//! - `FontSystem::new()`（cosmic-text）内部已加载系统字体
//! - `FontRegistry` 包装 fontdb 的 Database，负责：
//!   - 加载 `C:\Windows\Fonts` 系统字体
//!   - 加载用户自定义字体目录（--font-dir）
//!   - 按字体族名解析出 fontdb::ID
//!   - 枚举所有等宽字体族
//! - 支持 emoji 回退到 Segoe UI Emoji

use std::path::{Path, PathBuf};

/// 首选等宽字体名称
pub const DEFAULT_FONT_FAMILY: &str = "Cascadia Code";
pub const FALLBACK_FONT_FAMILY: &str = "Consolas";
pub const EMOJI_FONT: &str = "Segoe UI Emoji";

/// 等宽字体候选链（按优先级）
pub const MONO_CANDIDATES: &[&str] = &[
    "Cascadia Code",
    "Cascadia Mono",
    "Consolas",
    "Courier New",
    "Lucida Console",
    "Segoe UI Mono",
];

/// 系统字体目录
pub fn system_font_dir() -> PathBuf {
    PathBuf::from(r"C:\Windows\Fonts")
}

/// 字体注册表 — 封装 fontdb Database
pub struct FontRegistry {
    db: fontdb::Database,
    /// 已加载的自定义目录
    custom_dirs: Vec<String>,
}

impl FontRegistry {
    /// 创建注册表：加载 Windows 系统字体
    pub fn new() -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        Self {
            db,
            custom_dirs: Vec::new(),
        }
    }

    /// 加载自定义字体目录（.ttf/.otf/.ttc/.otc）
    pub fn load_custom_dir(&mut self, dir: &Path) {
        if !dir.is_dir() {
            eprintln!("字体目录不存在: {}", dir.display());
            return;
        }
        self.db.load_fonts_dir(dir);
        self.custom_dirs.push(dir.display().to_string());
    }

    /// 加载单个字体文件
    pub fn load_font_file(&mut self, path: &Path) -> Result<(), String> {
        self.db
            .load_font_file(path)
            .map_err(|e| format!("加载字体文件失败 {}: {}", path.display(), e))
    }

    /// 按字体族名解析 fontdb ID（仅精确匹配族名）
    pub fn resolve(&self, family: &str) -> Option<fontdb::ID> {
        self.db.query(&fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        })
    }

    /// 解析字体族：用户指定 → 候选链 → 回退 Consolas；全部失败返回 None
    pub fn resolve_chain(
        &self,
        preferred: Option<&str>,
        candidates: &[&str],
        fallback: &str,
    ) -> Option<String> {
        let mut tries: Vec<&str> = Vec::new();
        if let Some(p) = preferred {
            if !p.is_empty() && p != "auto" {
                tries.push(p);
            }
        }
        tries.extend(candidates.iter().copied());
        tries.push(fallback);
        for name in tries {
            if self.resolve(name).is_some() {
                return Some(name.to_string());
            }
        }
        None
    }

    /// 枚举系统中所有等宽字体族名（去重）
    pub fn enumerate_mono(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut names = Vec::new();
        for face in self.db.faces() {
            if face.monospaced {
                if let Some(family) = face.families.first() {
                    if seen.insert(family.0.clone()) {
                        names.push(family.0.clone());
                    }
                }
            }
        }
        names.sort();
        names
    }

    /// 已加载的自定义目录
    pub fn custom_dirs(&self) -> &[String] {
        &self.custom_dirs
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 根据配置名称查找可用字体（保留旧接口，供 FontContext 调用）
pub fn resolve_font_family(name: Option<&str>) -> String {
    match name {
        Some(n) if n == "auto" || n.is_empty() => DEFAULT_FONT_FAMILY.to_string(),
        Some(n) => n.to_string(),
        None => DEFAULT_FONT_FAMILY.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_resolves_system_mono() {
        let reg = FontRegistry::new();
        // Windows 上至少应解析出 Consolas 或 Courier New
        let found = reg.resolve_chain(None, MONO_CANDIDATES, FALLBACK_FONT_FAMILY);
        assert!(found.is_some(), "resolves some monospace font");
        let mono = reg.enumerate_mono();
        assert!(
            mono.iter().any(|n| n == "Consolas" || n == "Courier New"),
            "expected a classic mono font, got: {mono:?}"
        );
    }
}
