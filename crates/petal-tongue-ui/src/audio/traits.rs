// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio Backend Traits - Universal Audio Interface
//!
//! Defines WHAT we need from audio, not HOW to do it!

#[cfg(feature = "audio-direct")]
use crate::audio::backends::DirectBackend;
#[cfg(feature = "audio-socket")]
use crate::audio::backends::SocketBackend;
use crate::audio::backends::{SilentBackend, SoftwareBackend};
use crate::error::Result;
use std::future::Future;

/// Universal audio backend trait
///
/// Any audio provider implements this - we don't care HOW!
/// - Network/compute provider (network synthesis)
/// - Software (pure Rust generation)
/// - Socket servers (`PipeWire`, `PulseAudio`, etc. discovered at runtime)
/// - Direct devices (/dev/snd, /dev/audio, etc. discovered at runtime)
/// - Silent (graceful degradation)
pub trait AudioBackend: Send + Sync {
    /// Get backend metadata (for display only!)
    ///
    /// **NEVER use for routing logic** - use `capabilities()` instead
    fn metadata(&self) -> BackendMetadata;

    /// Get priority (lower = preferred)
    ///
    /// Priority order:
    /// - 10: Network (compute provider)
    /// - 30: Socket (PipeWire/PulseAudio/etc.)
    /// - 40: Direct (/dev/snd, /dev/audio, etc.)
    /// - 50: Software (pure Rust synthesis)
    /// - 255: Silent (fallback)
    fn priority(&self) -> u8;

    /// Check if backend is currently available
    ///
    /// This is called at runtime to verify availability.
    /// May return false if hardware is disconnected, service stopped, etc.
    fn is_available(&self) -> impl Future<Output = bool> + Send;

    /// Initialize backend (prepare for playback)
    ///
    /// Called once before first use.
    /// May allocate buffers, connect to services, open devices, etc.
    fn initialize(&mut self) -> impl Future<Output = Result<()>> + Send;

    /// Play audio samples (async, non-blocking)
    ///
    /// Samples are f32 in range [-1.0, 1.0]
    /// Backend handles conversion to hardware format (i16, u8, etc.)
    fn play_samples(
        &mut self,
        samples: &[f32],
        sample_rate: u32,
    ) -> impl Future<Output = Result<()>> + Send;

    /// Get capabilities (what can this backend do?)
    ///
    /// **Use this for routing logic**, not metadata.name!
    fn capabilities(&self) -> AudioCapabilities;
}

/// Enum dispatch for concrete audio backends (replaces `Box<dyn AudioBackend>`).
pub enum AudioBackendImpl {
    /// Direct hardware device backend (`audio-direct` feature).
    #[cfg(feature = "audio-direct")]
    Direct(DirectBackend),
    /// Socket-based audio server (`audio-socket` feature).
    #[cfg(feature = "audio-socket")]
    Socket(SocketBackend),
    /// Pure Rust software synthesis.
    Software(SoftwareBackend),
    /// Silent fallback.
    Silent(SilentBackend),
}

async fn audio_backend_impl_is_available(backend: &AudioBackendImpl) -> bool {
    match backend {
        #[cfg(feature = "audio-direct")]
        AudioBackendImpl::Direct(b) => b.is_available().await,
        #[cfg(feature = "audio-socket")]
        AudioBackendImpl::Socket(b) => b.is_available().await,
        AudioBackendImpl::Software(b) => b.is_available().await,
        AudioBackendImpl::Silent(b) => b.is_available().await,
    }
}

async fn audio_backend_impl_initialize(backend: &mut AudioBackendImpl) -> Result<()> {
    match backend {
        #[cfg(feature = "audio-direct")]
        AudioBackendImpl::Direct(b) => b.initialize().await,
        #[cfg(feature = "audio-socket")]
        AudioBackendImpl::Socket(b) => b.initialize().await,
        AudioBackendImpl::Software(b) => b.initialize().await,
        AudioBackendImpl::Silent(b) => b.initialize().await,
    }
}

async fn audio_backend_impl_play_samples(
    backend: &mut AudioBackendImpl,
    samples: &[f32],
    sample_rate: u32,
) -> Result<()> {
    match backend {
        #[cfg(feature = "audio-direct")]
        AudioBackendImpl::Direct(b) => b.play_samples(samples, sample_rate).await,
        #[cfg(feature = "audio-socket")]
        AudioBackendImpl::Socket(b) => b.play_samples(samples, sample_rate).await,
        AudioBackendImpl::Software(b) => b.play_samples(samples, sample_rate).await,
        AudioBackendImpl::Silent(b) => b.play_samples(samples, sample_rate).await,
    }
}

impl AudioBackend for AudioBackendImpl {
    fn metadata(&self) -> BackendMetadata {
        match self {
            #[cfg(feature = "audio-direct")]
            Self::Direct(b) => b.metadata(),
            #[cfg(feature = "audio-socket")]
            Self::Socket(b) => b.metadata(),
            Self::Software(b) => b.metadata(),
            Self::Silent(b) => b.metadata(),
        }
    }

    fn priority(&self) -> u8 {
        match self {
            #[cfg(feature = "audio-direct")]
            Self::Direct(b) => b.priority(),
            #[cfg(feature = "audio-socket")]
            Self::Socket(b) => b.priority(),
            Self::Software(b) => b.priority(),
            Self::Silent(b) => b.priority(),
        }
    }

    fn is_available(&self) -> impl Future<Output = bool> + Send {
        audio_backend_impl_is_available(self)
    }

    fn initialize(&mut self) -> impl Future<Output = Result<()>> + Send {
        audio_backend_impl_initialize(self)
    }

    fn play_samples(
        &mut self,
        samples: &[f32],
        sample_rate: u32,
    ) -> impl Future<Output = Result<()>> + Send {
        audio_backend_impl_play_samples(self, samples, sample_rate)
    }

    fn capabilities(&self) -> AudioCapabilities {
        match self {
            #[cfg(feature = "audio-direct")]
            Self::Direct(b) => b.capabilities(),
            #[cfg(feature = "audio-socket")]
            Self::Socket(b) => b.capabilities(),
            Self::Software(b) => b.capabilities(),
            Self::Silent(b) => b.capabilities(),
        }
    }
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
    /// Network audio (compute provider)
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
