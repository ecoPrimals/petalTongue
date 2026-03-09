// SPDX-License-Identifier: AGPL-3.0-only
//! System Dashboard - Types and state
//!
//! Compact live system metrics with multimodal output (visual + audio + text).

use crate::audio::AudioSystemV2;
use crate::live_data::LiveMetric;
use crate::multimodal_stream::{
    CpuStream, MemoryStream, ModalityPreferences, MultiModalRenderer, SystemMetricRenderer,
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::System;

/// Compact system dashboard for sidebar with multimodal output
pub struct SystemDashboard {
    pub(crate) system: System,
    pub(crate) last_refresh: Instant,
    pub(crate) refresh_interval: Duration,
    pub(crate) cpu_metric: LiveMetric,
    pub(crate) memory_metric: LiveMetric,
    pub(crate) cpu_history: VecDeque<f32>,
    pub(crate) mem_history: VecDeque<f32>,
    pub(crate) max_history: usize,

    // Multimodal streams
    pub(crate) cpu_stream: CpuStream,
    pub(crate) memory_stream: MemoryStream,
    pub(crate) renderer: SystemMetricRenderer,
    pub(crate) modality_prefs: ModalityPreferences,

    // Audio output
    pub(crate) last_audio_update: Instant,
    pub(crate) audio_update_interval: Duration,
}

impl Default for SystemDashboard {
    fn default() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let total_memory = system.total_memory();

        Self {
            system,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(1),
            cpu_metric: LiveMetric::new("CPU".to_string(), "sysinfo".to_string(), 1.0),
            memory_metric: LiveMetric::new("Memory".to_string(), "sysinfo".to_string(), 1.0),
            cpu_history: VecDeque::new(),
            mem_history: VecDeque::new(),
            max_history: 30, // 30 seconds for mini view
            cpu_stream: CpuStream::new(),
            memory_stream: MemoryStream::new(total_memory),
            renderer: SystemMetricRenderer,
            modality_prefs: ModalityPreferences::default(),
            last_audio_update: Instant::now(),
            audio_update_interval: Duration::from_millis(200), // 5Hz audio updates
        }
    }
}

impl SystemDashboard {
    /// Enable/disable audio sonification
    pub fn set_audio_enabled(&mut self, enabled: bool) {
        self.modality_prefs.audio_enabled = enabled;
    }

    /// Get audio enabled state
    #[must_use]
    pub fn is_audio_enabled(&self) -> bool {
        self.modality_prefs.audio_enabled
    }

    /// Set audio volume
    pub fn set_audio_volume(&mut self, volume: f32) {
        self.modality_prefs.audio_volume = volume.clamp(0.0, 1.0);
    }

    /// Get modality preferences (for UI controls)
    pub fn modality_prefs_mut(&mut self) -> &mut ModalityPreferences {
        &mut self.modality_prefs
    }

    /// Refresh system data and update multimodal streams
    pub(crate) fn refresh(&mut self, audio_system: Option<&AudioSystemV2>) {
        let now = Instant::now();
        if now.duration_since(self.last_refresh) >= self.refresh_interval {
            self.system.refresh_all();
            self.last_refresh = now;

            // Calculate CPU usage
            let cpus = self.system.cpus();
            let cpu_usage = if cpus.is_empty() {
                0.0
            } else {
                cpus.iter().map(sysinfo::Cpu::cpu_usage).sum::<f32>() / cpus.len() as f32
            };

            self.cpu_history.push_back(cpu_usage);
            if self.cpu_history.len() > self.max_history {
                self.cpu_history.pop_front();
            }

            // Update CPU stream (normalized 0-1)
            self.cpu_stream.push_value(f64::from(cpu_usage / 100.0));

            // Calculate memory usage
            let used_mem = self.system.used_memory();
            let total_mem = self.system.total_memory();
            let mem_percent = if total_mem > 0 {
                ((used_mem as f64 / total_mem as f64) * 100.0) as f32
            } else {
                0.0
            };

            self.mem_history.push_back(mem_percent);
            if self.mem_history.len() > self.max_history {
                self.mem_history.pop_front();
            }

            // Update Memory stream
            self.memory_stream.push_value(used_mem);

            // Update live metrics
            self.cpu_metric
                .update(format!("{cpu_usage:.1}"), Some("%".to_string()));
            self.memory_metric
                .update(format!("{mem_percent:.1}"), Some("%".to_string()));
        }

        // Generate audio if enabled and interval passed
        if self.modality_prefs.audio_enabled
            && let Some(audio_system) = audio_system
            && now.duration_since(self.last_audio_update) >= self.audio_update_interval
        {
            self.generate_audio(audio_system);
            self.last_audio_update = now;
        }
    }

    /// Generate polyphonic audio for all streams
    fn generate_audio(&self, audio_system: &AudioSystemV2) {
        // Collect audio representations for all streams
        let mut tones = Vec::new();

        // CPU stream
        if let Some(cpu_audio) = self.renderer.render_audio(&self.cpu_stream) {
            tones.push((
                cpu_audio.frequency,
                cpu_audio.volume * self.modality_prefs.audio_volume,
                cpu_audio.waveform,
            ));
        }

        // Memory stream
        if let Some(mem_audio) = self.renderer.render_audio(&self.memory_stream) {
            tones.push((
                mem_audio.frequency,
                mem_audio.volume * self.modality_prefs.audio_volume,
                mem_audio.waveform,
            ));
        }

        // Play polyphonic audio (CPU + Memory simultaneously)
        if !tones.is_empty() {
            // Short duration (200ms) to update frequently
            let duration = self.audio_update_interval.as_secs_f64();
            if let Err(e) = audio_system.play_polyphonic(&tones, duration) {
                // Don't spam errors - only log occasionally
                static LAST_ERROR: std::sync::atomic::AtomicU64 =
                    std::sync::atomic::AtomicU64::new(0);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let last = LAST_ERROR.load(std::sync::atomic::Ordering::Relaxed);
                if now - last > 60 {
                    tracing::warn!("Failed to play audio: {}", e);
                    LAST_ERROR.store(now, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }
}
