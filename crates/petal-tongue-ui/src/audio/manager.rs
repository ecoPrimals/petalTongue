// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio Manager - Substrate-Agnostic Audio Coordination
//!
//! Discovers ALL available audio backends at runtime and manages playback.
//! Mirrors the `DisplayManager` pattern (proven success!).

#[cfg(feature = "audio-direct")]
use super::backends::DirectBackend;
#[cfg(feature = "audio-socket")]
use super::backends::SocketBackend;
use super::backends::{SilentBackend, SoftwareBackend};
use super::traits::AudioBackend;
use crate::error::{AudioError, Result};
use tracing::{info, warn};

/// Audio manager - coordinates multiple backends
pub struct AudioManager {
    backends: Vec<Box<dyn AudioBackend>>,
    active_backend_idx: Option<usize>,
}

impl AudioManager {
    /// Initialize audio manager and discover available backends
    ///
    /// This follows TRUE PRIMAL principles:
    /// - Runtime discovery (no hardcoding)
    /// - Graceful degradation (always works)
    /// - Capability-based selection
    ///
    /// # Errors
    ///
    /// Returns an error if no audio backends are available (including silent fallback).
    pub async fn init() -> Result<Self> {
        info!("🎵 Discovering audio backends (TRUE PRIMAL)...");

        let mut backends: Vec<Box<dyn AudioBackend>> = Vec::new();

        // Tier 1: Network Audio (compute primal with audio.synthesis capability)
        // Discovered at runtime via capability probing; not hardcoded to any primal.

        // Tier 2: Socket-Based Audio Servers (optional stub — see `audio-socket` feature)
        #[cfg(feature = "audio-socket")]
        {
            info!("🔌 Checking for socket-based audio servers...");
            for socket_backend in SocketBackend::discover_all().await {
                info!("✅ Socket audio server: {}", socket_backend.metadata().name);
                backends.push(Box::new(socket_backend));
            }
        }

        // Tier 3: Direct Device Access (optional stub — see `audio-direct` feature)
        #[cfg(feature = "audio-direct")]
        {
            info!("🎨 Checking for direct audio devices...");
            for direct_backend in DirectBackend::discover_all() {
                info!("✅ Direct audio device: {}", direct_backend.metadata().name);
                backends.push(Box::new(direct_backend));
            }
        }

        // Tier 4: Pure Rust Software Synthesis (always available)
        info!("🎼 Pure Rust software synthesis available");
        backends.push(Box::new(SoftwareBackend::new()));

        // Tier 5: Silent Mode (always available, last resort)
        info!("🔇 Silent mode available (graceful degradation)");
        backends.push(Box::new(SilentBackend::new()));

        if backends.is_empty() {
            return Err(AudioError::NoBackendsAvailable.into());
        }

        // Sort by priority (lower number = higher priority)
        backends.sort_by_key(|backend| backend.priority());

        info!("🎵 Found {} audio backend(s)", backends.len());
        for backend in &backends {
            let meta = backend.metadata();
            info!(
                "  - {} (priority: {}, type: {:?})",
                meta.name,
                backend.priority(),
                meta.backend_type
            );
        }

        Ok(Self {
            backends,
            active_backend_idx: None,
        })
    }

    /// Select and initialize the best available backend
    async fn select_backend(&mut self) -> Result<()> {
        if self.active_backend_idx.is_some() {
            return Ok(()); // Already selected
        }

        info!("🎯 Selecting best audio backend...");

        for (idx, backend) in self.backends.iter_mut().enumerate() {
            let meta = backend.metadata();

            if backend.is_available().await {
                info!("✅ Trying backend: {}", meta.name);

                match backend.initialize().await {
                    Ok(()) => {
                        info!("🎵 Selected audio backend: {}", meta.name);
                        self.active_backend_idx = Some(idx);
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to initialize {}: {}", meta.name, e);
                        continue;
                    }
                }
            }
            info!("⏭️  Backend {} not available", meta.name);
        }

        Err(AudioError::NoBackendInitialized.into())
    }

    /// Play audio samples using the best available backend
    ///
    /// Automatically selects backend on first call.
    /// Falls back to next backend if current one fails.
    ///
    /// # Errors
    ///
    /// Returns an error if no backend can be selected or initialized, or if playback fails on all backends.
    pub async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        // Select backend if not already selected
        if self.active_backend_idx.is_none() {
            self.select_backend().await?;
        }

        let idx = self.active_backend_idx.ok_or(AudioError::NoActiveBackend)?;

        // Try active backend
        match self.backends[idx].play_samples(samples, sample_rate).await {
            Ok(()) => Ok(()),
            Err(e) => {
                let meta = self.backends[idx].metadata();
                warn!("⚠️  Playback failed on {}: {}", meta.name, e);

                // Try to fall back to next backend
                self.active_backend_idx = None;
                self.select_backend().await?;

                // Retry with new backend
                let new_idx = self
                    .active_backend_idx
                    .ok_or(AudioError::NoFallbackBackend)?;
                self.backends[new_idx]
                    .play_samples(samples, sample_rate)
                    .await
            }
        }
    }

    /// Get active backend metadata (for display only!)
    #[must_use]
    pub fn active_backend_metadata(&self) -> Option<super::traits::BackendMetadata> {
        self.active_backend_idx
            .and_then(|idx| self.backends.get(idx))
            .map(|backend| backend.metadata())
    }

    /// Get all available backends (for display/debugging)
    #[must_use]
    pub fn available_backends(&self) -> Vec<super::traits::BackendMetadata> {
        self.backends.iter().map(|b| b.metadata()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_manager_init() {
        // Should not panic and should find at least silent backend
        let manager = AudioManager::init().await.unwrap_or_else(|e| {
            panic!("Failed to initialize AudioManager in test: {}", e);
        });

        assert!(
            !manager.backends.is_empty(),
            "Should have at least silent backend"
        );

        // Should have silent backend as last resort
        let backends = manager.available_backends();
        let has_silent = backends
            .iter()
            .any(|b| b.backend_type == super::super::traits::BackendType::Silent);
        assert!(has_silent, "Should always have silent backend");
    }

    #[tokio::test]
    async fn test_audio_manager_play() {
        let mut manager = AudioManager::init().await.unwrap_or_else(|e| {
            panic!("Failed to initialize AudioManager in test: {}", e);
        });

        // Generate test tone (440 Hz A4, 0.1 seconds)
        let sample_rate = 44100;
        let duration = 0.1;
        let frequency = 440.0;

        let samples: Vec<f32> = (0..((sample_rate as f32 * duration) as usize))
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.3
            })
            .collect();

        // Should not panic (may be silent backend)
        manager
            .play_samples(&samples, sample_rate)
            .await
            .unwrap_or_else(|e| {
                panic!("Failed to play samples in test: {}", e);
            });

        // Should have selected a backend
        assert!(
            manager.active_backend_idx.is_some(),
            "Should have selected a backend"
        );
    }
}
