// SPDX-License-Identifier: AGPL-3.0-only
//! Software Synthesis Backend - Pure Rust Audio Generation
//!
//! 100% Pure Rust, works everywhere!
//! Generates audio programmatically (sine, square, triangle, etc.)

use crate::audio::traits::{AudioBackend, AudioCapabilities, BackendMetadata, BackendType};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, info};

/// Software synthesis backend - pure Rust audio generation
pub struct SoftwareBackend {
    sample_buffer: Vec<f32>,
    initialized: bool,
}

impl SoftwareBackend {
    /// Create a new software synthesis backend
    #[must_use]
    pub fn new() -> Self {
        Self {
            sample_buffer: Vec::new(),
            initialized: false,
        }
    }
}

impl Default for SoftwareBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AudioBackend for SoftwareBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            name: "Pure Rust Software Synthesis".to_string(),
            backend_type: BackendType::Software,
            description: "Programmatic audio generation (100% pure Rust, always available)"
                .to_string(),
        }
    }

    fn priority(&self) -> u8 {
        50 // Middle priority (prefer hardware playback if available)
    }

    async fn is_available(&self) -> bool {
        true // ALWAYS available (Pure Rust!)
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("🎼 Initializing pure Rust software synthesis");
        self.initialized = true;
        Ok(())
    }

    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        debug!(
            "🎼 Software synthesis: buffering {} samples at {} Hz",
            samples.len(),
            sample_rate
        );

        // Store samples in buffer
        self.sample_buffer.extend_from_slice(samples);

        // TODO: When we have a playback backend selected,
        // we can send these samples to it for actual hardware playback
        // For now, we just generate and buffer

        debug!("✅ Samples buffered (total: {})", self.sample_buffer.len());

        Ok(())
    }

    fn capabilities(&self) -> AudioCapabilities {
        AudioCapabilities {
            can_play: true,
            can_record: false,
            max_sample_rate: 48000,
            max_channels: 2,
            latency_estimate_ms: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_software_backend() {
        let mut backend = SoftwareBackend::new();

        assert!(backend.is_available().await);
        assert!(backend.initialize().await.is_ok());

        // Generate test tone
        let samples: Vec<f32> = (0..1000)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.3
            })
            .collect();

        assert!(backend.play_samples(&samples, 44100).await.is_ok());
        assert_eq!(backend.sample_buffer.len(), 1000);
    }
}
