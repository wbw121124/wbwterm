// Copyright (c) wbwterm contributors. Licensed under MPL-2.0.

//! 字体渲染 — glyphon FontSystem + TextAtlas + Cache

use glyphon::{Cache, FontSystem, SwashCache, TextAtlas};
use wgpu::{Device, Queue, TextureFormat};

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
}

impl FontContext {
    /// 创建字体上下文（扫描 Windows 系统字体）
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let mut font_system = FontSystem::new();
        font_system.db_mut().load_system_fonts();

        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let atlas = TextAtlas::new(device, queue, &cache, TextureFormat::R8Unorm);

        Self {
            font_system,
            atlas,
            cache,
            swash_cache,
        }
    }

    pub fn set_font(&mut self, _family_name: &str) {
        // TODO: P5
    }
}
