// SPDX-License-Identifier: AGPL-3.0-only
//! Silent Backend - Graceful Degradation
//!
//! Always available, never fails.
//! Used when no other audio backend is available.

use crate::audio::traits::{AudioBackend, AudioCapabilities, BackendMetadata, BackendType};
use crate::error::Result;
use async_trait::async_trait;
use tracing::debug;

/// Silent backend - graceful degradation
pub struct SilentBackend;

impl SilentBackend {
    /// Create a new silent backend
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for SilentBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AudioBackend for SilentBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            name: "Silent Mode".to_string(),
            backend_type: BackendType::Silent,
            description: "No audio output (visual-only mode)".to_string(),
        }
    }

    fn priority(&self) -> u8 {
        255 // Lowest priority (last resort)
    }

    async fn is_available(&self) -> bool {
        true // ALWAYS available!
    }

    async fn initialize(&mut self) -> Result<()> {
        debug!("🔇 Silent backend initialized (no-op)");
        Ok(())
    }

    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        debug!(
            "🔇 Silent mode - skipping playback of {} samples at {} Hz",
            samples.len(),
            sample_rate
        );
        Ok(())
    }

    fn capabilities(&self) -> AudioCapabilities {
        AudioCapabilities {
            can_play: false,
            can_record: false,
            max_sample_rate: 0,
            max_channels: 0,
            latency_estimate_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::traits::BackendType;

    #[tokio::test]
    async fn test_silent_backend() {
        let mut backend = SilentBackend::new();

        assert!(backend.is_available().await);
        assert!(backend.initialize().await.is_ok());

        let samples = vec![0.0; 1000];
        assert!(backend.play_samples(&samples, 44100).await.is_ok());
    }

    #[test]
    fn test_silent_backend_metadata() {
        let backend = SilentBackend::new();
        let meta = backend.metadata();
        assert_eq!(meta.name, "Silent Mode");
        assert_eq!(meta.backend_type, BackendType::Silent);
        assert!(meta.description.contains("visual-only"));
    }

    #[test]
    fn test_silent_backend_priority() {
        let backend = SilentBackend::new();
        assert_eq!(backend.priority(), 255);
    }

    #[test]
    fn test_silent_backend_capabilities() {
        let backend = SilentBackend::new();
        let caps = backend.capabilities();
        assert!(!caps.can_play);
        assert!(!caps.can_record);
        assert_eq!(caps.max_sample_rate, 0);
        assert_eq!(caps.max_channels, 0);
        assert_eq!(caps.latency_estimate_ms, 0);
    }

    #[test]
    fn test_silent_backend_default() {
        let backend = SilentBackend;
        assert_eq!(backend.metadata().name, "Silent Mode");
    }

    #[tokio::test]
    async fn test_silent_backend_empty_samples() {
        let mut backend = SilentBackend::new();
        assert!(backend.play_samples(&[], 48000).await.is_ok());
    }
}
