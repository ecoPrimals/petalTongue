// SPDX-License-Identifier: AGPL-3.0-or-later
//! Texture registry for raster content uploaded via IPC.
//!
//! Springs push raw pixel data through `visualization.texture.upload` and the
//! registry stores it keyed by `texture_id`. Renderers (egui, SVG export) look
//! up entries at paint time.

use bytes::Bytes;
use std::collections::HashMap;

/// Pixel format for uploaded textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    /// 8-bit RGBA (4 bytes per pixel).
    Rgba8,
}

/// A single uploaded texture entry.
///
/// Uses `bytes::Bytes` for refcounted zero-copy sharing of pixel data
/// across renderers without duplicating the buffer.
#[derive(Debug, Clone)]
pub struct TextureEntry {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    /// Raw pixel bytes (length = width * height * bytes_per_pixel).
    /// Refcounted via `Bytes` for zero-copy sharing across renderers.
    pub data: Bytes,
    /// Monotonically increasing version; bumped on re-upload so renderers
    /// know to refresh their GPU-side copy.
    pub version: u64,
}

/// Registry of uploaded textures, keyed by `texture_id`.
#[derive(Debug, Clone, Default)]
pub struct TextureRegistry {
    textures: HashMap<String, TextureEntry>,
    next_version: u64,
}

impl TextureRegistry {
    /// Create an empty texture registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            next_version: 1,
        }
    }

    /// Insert or replace a texture. Returns the assigned version.
    ///
    /// Accepts `impl Into<Bytes>` so callers can pass `Vec<u8>`, `Bytes`,
    /// or `&'static [u8]` without an explicit conversion.
    pub fn insert(
        &mut self,
        texture_id: String,
        width: u32,
        height: u32,
        format: TextureFormat,
        data: impl Into<Bytes>,
    ) -> u64 {
        let version = self.next_version;
        self.next_version += 1;
        self.textures.insert(
            texture_id,
            TextureEntry {
                width,
                height,
                format,
                data: data.into(),
                version,
            },
        );
        version
    }

    /// Look up a texture by ID.
    #[must_use]
    pub fn get(&self, texture_id: &str) -> Option<&TextureEntry> {
        self.textures.get(texture_id)
    }

    /// Remove a texture by ID.
    pub fn remove(&mut self, texture_id: &str) -> bool {
        self.textures.remove(texture_id).is_some()
    }

    /// Number of registered textures.
    #[must_use]
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    /// Iterate over all `(texture_id, entry)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &TextureEntry)> {
        self.textures.iter().map(|(k, v)| (k.as_str(), v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut reg = TextureRegistry::new();
        let v = reg.insert(
            "hero".into(),
            64,
            64,
            TextureFormat::Rgba8,
            vec![0; 64 * 64 * 4],
        );
        assert_eq!(v, 1);
        let entry = reg.get("hero").unwrap();
        assert_eq!(entry.width, 64);
        assert_eq!(entry.height, 64);
        assert_eq!(entry.version, 1);
    }

    #[test]
    fn re_upload_bumps_version() {
        let mut reg = TextureRegistry::new();
        reg.insert(
            "hero".into(),
            32,
            32,
            TextureFormat::Rgba8,
            vec![0; 32 * 32 * 4],
        );
        let v2 = reg.insert(
            "hero".into(),
            64,
            64,
            TextureFormat::Rgba8,
            vec![0; 64 * 64 * 4],
        );
        assert_eq!(v2, 2);
        assert_eq!(reg.get("hero").unwrap().width, 64);
    }

    #[test]
    fn remove_works() {
        let mut reg = TextureRegistry::new();
        reg.insert("bg".into(), 8, 8, TextureFormat::Rgba8, vec![0; 256]);
        assert!(reg.remove("bg"));
        assert!(reg.get("bg").is_none());
        assert!(!reg.remove("bg"));
    }

    #[test]
    fn len_and_empty() {
        let mut reg = TextureRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        reg.insert("a".into(), 1, 1, TextureFormat::Rgba8, vec![0; 4]);
        assert!(!reg.is_empty());
        assert_eq!(reg.len(), 1);
    }
}
