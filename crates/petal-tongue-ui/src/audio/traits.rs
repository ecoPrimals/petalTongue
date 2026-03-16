// SPDX-License-Identifier: AGPL-3.0-only
//! Audio Backend Traits - Universal Audio Interface
//!
//! Defines WHAT we need from audio, not HOW to do it!

use crate::error::Result;
use async_trait::async_trait;

/// Universal audio backend trait
///
/// Any audio provider implements this - we don't care HOW!
/// - `ToadStool` (network synthesis)
/// - Software (pure Rust generation)
/// - Socket servers (`PipeWire`, `PulseAudio`, etc. discovered at runtime)
/// - Direct devices (/dev/snd, /dev/audio, etc. discovered at runtime)
/// - Silent (graceful degradation)
#[async_trait]
pub trait AudioBackend: Send + Sync {
    /// Get backend metadata (for display only!)
    ///
    /// **NEVER use for routing logic** - use `capabilities()` instead
    fn metadata(&self) -> BackendMetadata;

    /// Get priority (lower = preferred)
    ///
    /// Priority order:
    /// - 10: Network (`ToadStool`)
    /// - 30: Socket (PipeWire/PulseAudio/etc.)
    /// - 40: Direct (/dev/snd, /dev/audio, etc.)
    /// - 50: Software (pure Rust synthesis)
    /// - 255: Silent (fallback)
    fn priority(&self) -> u8;

    /// Check if backend is currently available
    ///
    /// This is called at runtime to verify availability.
    /// May return false if hardware is disconnected, service stopped, etc.
    async fn is_available(&self) -> bool;

    /// Initialize backend (prepare for playback)
    ///
    /// Called once before first use.
    /// May allocate buffers, connect to services, open devices, etc.
    async fn initialize(&mut self) -> Result<()>;

    /// Play audio samples (async, non-blocking)
    ///
    /// Samples are f32 in range [-1.0, 1.0]
    /// Backend handles conversion to hardware format (i16, u8, etc.)
    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()>;

    /// Get capabilities (what can this backend do?)
    ///
    /// **Use this for routing logic**, not metadata.name!
    fn capabilities(&self) -> AudioCapabilities;
}

/// Backend metadata (display only, never for logic!)
#[derive(Debug, Clone)]
pub struct BackendMetadata {
    /// Backend name (for display only!)
    pub name: String,
    /// Backend type (for display only!)
    pub backend_type: BackendType,
    /// Human-readable description
    pub description: String,
}

/// Backend type (categorization only)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendType {
    /// Network audio (`ToadStool` primal)
    Network,
    /// Pure Rust software synthesis
    Software,
    /// Socket-based audio server
    Socket,
    /// Direct hardware device
    Direct,
    /// Silent mode (no audio)
    Silent,
}

/// Audio capabilities (what can this backend do?)
#[derive(Debug, Clone, Default)]
pub struct AudioCapabilities {
    /// Can play audio
    pub can_play: bool,
    /// Can record audio
    pub can_record: bool,
    /// Maximum sample rate (Hz)
    pub max_sample_rate: u32,
    /// Maximum channels
    pub max_channels: u8,
    /// Estimated latency (milliseconds)
    pub latency_estimate_ms: u32,
}
