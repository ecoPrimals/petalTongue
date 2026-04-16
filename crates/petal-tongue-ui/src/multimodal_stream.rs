// SPDX-License-Identifier: AGPL-3.0-or-later
// Multimodal data stream rendering
// Converts any data stream into multiple output modalities
// Vision: ONE UI with pluggable I/O for true universal accessibility

use std::time::{Duration, Instant};

/// A data stream that can be rendered in multiple modalities
pub trait DataStream: Send + Sync {
    /// Current value (normalized 0.0-1.0)
    fn value(&self) -> f64;

    /// Value range for mapping
    fn range(&self) -> (f64, f64);

    /// Human-readable label
    fn label(&self) -> &str;

    /// Optional: Historical values for visual rendering
    fn history(&self) -> Option<&[f64]> {
        None
    }

    /// Update the stream (called each frame)
    fn update(&mut self) {}
}

/// Renders a data stream in multiple modalities simultaneously
pub trait MultiModalRenderer {
    /// Render visually (line chart, bar, etc.)
    fn render_visual(&self, ui: &mut egui::Ui, stream: &impl DataStream);

    /// Generate audio representation (frequency, volume, waveform)
    fn render_audio(&self, stream: &impl DataStream) -> Option<AudioRepresentation>;

    /// Generate text description (for assistive technologies)
    fn render_text(&self, stream: &impl DataStream) -> String;

    /// Generate haptic feedback (future: vibration patterns)
    fn render_haptic(&self, stream: &impl DataStream) -> Option<HapticPattern>;
}

/// Audio representation of a data stream
#[derive(Debug, Clone)]
pub struct AudioRepresentation {
    /// Frequency in Hz (mapped from data value)
    pub frequency: f64,

    /// Volume 0.0-1.0 (mapped from data value)
    pub volume: f32,

    /// Waveform type
    pub waveform: crate::audio_pure_rust::Waveform,

    /// Duration (for continuous streams, very short with overlap)
    pub duration: Duration,

    /// Stereo panning -1.0 (left) to 1.0 (right)
    pub pan: f32,
}

/// Haptic pattern representation (future)
#[derive(Debug, Clone)]
pub struct HapticPattern {
    /// Vibration intensity 0.0-1.0
    pub intensity: f32,

    /// Duration
    pub duration: Duration,

    /// Pattern (pulse, continuous, etc.)
    pub pattern: HapticPatternType,
}

/// Haptic pattern type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticPatternType {
    /// Continuous vibration
    Continuous,
    /// Single pulse vibration
    Pulse,
    /// Rhythmic pattern vibration
    Rhythm,
}

/// Configuration for modality preferences
#[derive(Debug, Clone)]
pub struct ModalityPreferences {
    /// Enable visual output (default: true)
    pub visual_enabled: bool,

    /// Visual opacity 0.0-1.0 (for low vision users)
    pub visual_opacity: f32,

    /// Enable audio sonification (default: false, user must opt-in)
    pub audio_enabled: bool,

    /// Audio volume 0.0-1.0
    pub audio_volume: f32,

    /// Enable text descriptions (default: true)
    pub text_enabled: bool,

    /// Enable haptic feedback (default: false, when available)
    pub haptic_enabled: bool,
}

impl Default for ModalityPreferences {
    fn default() -> Self {
        Self {
            visual_enabled: true,
            visual_opacity: 1.0,
            audio_enabled: false, // Opt-in
            audio_volume: 0.5,
            text_enabled: true,
            haptic_enabled: false,
        }
    }
}

/// System CPU usage stream
pub struct CpuStream {
    label: String,
    history: Vec<f64>,
    max_history: usize,
    last_update: Option<Instant>,
}

impl Default for CpuStream {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuStream {
    /// Create new CPU usage stream
    #[must_use]
    pub fn new() -> Self {
        Self {
            label: "CPU Usage".to_string(),
            history: Vec::new(),
            max_history: 120, // 2 minutes at 1Hz
            last_update: None,
        }
    }

    /// Add new CPU usage value to stream
    pub fn push_value(&mut self, value: f64) {
        self.history.push(value);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        self.last_update = Some(Instant::now());
    }
}

impl DataStream for CpuStream {
    fn value(&self) -> f64 {
        self.history.last().copied().unwrap_or(0.0)
    }

    fn range(&self) -> (f64, f64) {
        (0.0, 1.0) // 0-100%
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn history(&self) -> Option<&[f64]> {
        Some(&self.history)
    }
}

/// Memory usage stream
pub struct MemoryStream {
    label: String,
    history: Vec<f64>,
    max_history: usize,
    total_bytes: u64,
}

impl MemoryStream {
    /// Create new memory usage stream with total memory capacity
    #[must_use]
    pub fn new(total_bytes: u64) -> Self {
        Self {
            label: "Memory Usage".to_string(),
            history: Vec::new(),
            max_history: 120,
            total_bytes,
        }
    }

    /// Add new memory usage value to stream
    pub fn push_value(&mut self, used_bytes: u64) {
        let value = used_bytes as f64 / self.total_bytes as f64;
        self.history.push(value);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }
}

impl DataStream for MemoryStream {
    fn value(&self) -> f64 {
        self.history.last().copied().unwrap_or(0.0)
    }

    fn range(&self) -> (f64, f64) {
        (0.0, 1.0) // 0-100%
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn history(&self) -> Option<&[f64]> {
        Some(&self.history)
    }
}

/// Network traffic stream
pub struct NetworkStream {
    label: String,
    history: Vec<f64>,
    max_history: usize,
    max_bps: f64, // Maximum observed bits/sec (for normalization)
}

impl Default for NetworkStream {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkStream {
    /// Create new network traffic stream
    #[must_use]
    pub fn new() -> Self {
        Self {
            label: "Network Traffic".to_string(),
            history: Vec::new(),
            max_history: 120,
            max_bps: 1_000_000.0, // Start with 1 Mbps, will adjust
        }
    }

    /// Add new network traffic value to stream
    pub fn push_value(&mut self, bits_per_second: f64) {
        // Auto-adjust max for normalization
        if bits_per_second > self.max_bps {
            self.max_bps = bits_per_second;
        }

        let value = (bits_per_second / self.max_bps).min(1.0);
        self.history.push(value);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }
}

impl DataStream for NetworkStream {
    fn value(&self) -> f64 {
        self.history.last().copied().unwrap_or(0.0)
    }

    fn range(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn history(&self) -> Option<&[f64]> {
        Some(&self.history)
    }
}

/// Default multimodal renderer for system metrics
pub struct SystemMetricRenderer;

impl MultiModalRenderer for SystemMetricRenderer {
    fn render_visual(&self, ui: &mut egui::Ui, stream: &impl DataStream) {
        use egui::{Color32, Rect, Sense, vec2};

        let value = stream.value();
        let label = stream.label();

        // Visual: Bar with color coding
        ui.horizontal(|ui| {
            ui.label(label);

            let bar_width = ui.available_width() - 60.0;
            let bar_height = 20.0;

            let (rect, _response) =
                ui.allocate_exact_size(vec2(bar_width, bar_height), Sense::hover());

            // Color code by value
            let color = if value < 0.5 {
                Color32::from_rgb(0, 200, 0) // Green
            } else if value < 0.8 {
                Color32::from_rgb(255, 200, 0) // Yellow
            } else {
                Color32::from_rgb(255, 100, 0) // Red
            };

            // Background
            ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));

            // Fill
            let fill_width = rect.width() * value as f32;
            let fill_rect = Rect::from_min_size(rect.min, vec2(fill_width, rect.height()));
            ui.painter().rect_filled(fill_rect, 2.0, color);

            // Percentage text
            ui.label(format!("{:.1}%", value * 100.0));
        });
    }

    fn render_audio(&self, stream: &impl DataStream) -> Option<AudioRepresentation> {
        let value = stream.value();
        let label = stream.label();

        // Map different metrics to different audio characteristics
        if label.contains("CPU") {
            // CPU: Frequency mapping (200Hz at 0% to 2000Hz at 100%)
            Some(AudioRepresentation {
                frequency: value.mul_add(1800.0, 200.0),
                volume: 0.3,
                waveform: crate::audio_pure_rust::Waveform::Sine,
                duration: Duration::from_millis(200),
                pan: -0.5, // Left channel
            })
        } else if label.contains("Memory") {
            // Memory: Volume mapping (quiet to loud)
            Some(AudioRepresentation {
                frequency: 400.0,
                volume: (value * 0.5) as f32,
                waveform: crate::audio_pure_rust::Waveform::Triangle,
                duration: Duration::from_millis(200),
                pan: 0.0, // Center
            })
        } else if label.contains("Network") {
            // Network: Rhythmic pulses, speed varies with traffic
            Some(AudioRepresentation {
                frequency: 800.0,
                volume: 0.3,
                waveform: crate::audio_pure_rust::Waveform::Square,
                duration: Duration::from_millis(100),
                pan: 0.5, // Right channel
            })
        } else {
            None
        }
    }

    fn render_text(&self, stream: &impl DataStream) -> String {
        let value = stream.value();
        let label = stream.label();
        let percent = value * 100.0;

        let status = if value < 0.5 {
            "idle"
        } else if value < 0.8 {
            "active"
        } else {
            "busy"
        };

        format!("{label}: {percent:.1}% - {status}")
    }

    fn render_haptic(&self, stream: &impl DataStream) -> Option<HapticPattern> {
        let value = stream.value();

        Some(HapticPattern {
            intensity: value as f32,
            duration: Duration::from_millis(100),
            pattern: if value > 0.8 {
                HapticPatternType::Pulse
            } else {
                HapticPatternType::Continuous
            },
        })
    }
}

#[cfg(test)]
#[path = "multimodal_stream_tests.rs"]
mod tests;
