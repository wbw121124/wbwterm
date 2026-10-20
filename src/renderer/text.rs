// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 字体渲染 — glyphon FontSystem + TextAtlas + Cache
//!
//! 自定义字体加载链：
//! 1. `FontSystem::new()` 加载系统字体（内部 fontdb）
//! 2. 追加加载 `--font-dir` 指定的目录/文件到同一 Database
//! 3. `FontRegistry` 解析字体族链（用户字体 → 候选链 → Consolas）

use std::path::Path;

use glyphon::{Cache, FontSystem, SwashCache, TextAtlas};
use wgpu::{Device, Queue, TextureFormat};

use crate::font::{FontRegistry, FALLBACK_FONT_FAMILY, MONO_CANDIDATES};

/// 字体上下文管理器
pub struct FontContext {
    /// glyphon 字体系统（系统字体枚举 + 缓存）
    pub font_system: FontSystem,
    /// glyphon 字形位图纹理缓存（Atlas）
    pub atlas: TextAtlas,
    /// glyphon 共享资源缓存（pipeline、bind group 等）
    pub cache: Cache,
    /// glyphon 字形光栅化缓存
    pub swash_cache: SwashCache,
    /// 字体注册表（fontdb 封装，支持枚举/解析）
    pub registry: FontRegistry,
    /// 最终解析出的等宽字体族名
    pub resolved_family: String,
}

impl FontContext {
    /// 创建字体上下文
    ///
    /// ## 参数
    /// - `font_family`: 用户指定的字体族（空/"auto" 时走候选链）
    /// - `font_dirs`: 自定义字体目录或文件路径
    pub fn new(device: &Device, queue: &Queue, font_family: &str, font_dirs: &[String]) -> Self {
        let mut font_system = FontSystem::new();
        let mut registry = FontRegistry::new();

        // 自定义字体目录/文件 — 同时写入 glyphon 的 db 与独立注册表
        for dir in font_dirs {
            let path = Path::new(dir);
            if path.is_dir() {
                font_system.db_mut().load_fonts_dir(path);
                registry.load_custom_dir(path);
            } else if path.is_file() {
                if let Err(e) = registry.load_font_file(path) {
                    eprintln!("{e}");
                }
            } else {
                eprintln!("字体路径无效: {}", path.display());
            }
        }

        // 解析字体族链
        let resolved_family = registry
            .resolve_chain(Some(font_family), MONO_CANDIDATES, FALLBACK_FONT_FAMILY)
            .unwrap_or_else(|| FALLBACK_FONT_FAMILY.to_string());

        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let atlas = TextAtlas::new(device, queue, &cache, TextureFormat::R8Unorm);

        Self {
            font_system,
            atlas,
            cache,
            swash_cache,
            registry,
            resolved_family,
        }
    }

    /// 切换字体族（按名称重新解析）
    pub fn set_font(&mut self, family_name: &str) -> bool {
        match self.registry.resolve(family_name) {
            Some(_) => {
                self.resolved_family = family_name.to_string();
                true
            }
            None => false,
        }
    }

    /// 枚举系统中的等宽字体
    pub fn enumerate_mono(&self) -> Vec<String> {
        self.registry.enumerate_mono()
    }
}
