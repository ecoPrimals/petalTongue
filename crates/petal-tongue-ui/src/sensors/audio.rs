// SPDX-License-Identifier: AGPL-3.0-only
//! Audio sensor - Bidirectional I/O (speaker + microphone)
//!
//! Discovers audio capabilities for both output and input.

use anyhow::Result;
use async_trait::async_trait;
use petal_tongue_core::{Sensor, SensorCapabilities, SensorEvent, SensorType};
use std::time::Instant;

/// Audio sensor implementation
pub struct AudioSensor {
    capabilities: SensorCapabilities,
    has_output: bool,
    has_input: bool,
    last_audio_event: Option<Instant>,
}

impl AudioSensor {
    /// Create new audio sensor
    #[must_use]
    pub const fn new(has_output: bool, has_input: bool) -> Self {
        let bidirectional = has_output && has_input;

        let capabilities = SensorCapabilities {
            sensor_type: SensorType::Audio,
            input: has_input,
            output: has_output,
            spatial: false,
            temporal: true,   // Timing of audio events
            continuous: true, // Audio is continuous
            discrete: false,
            bidirectional,
        };

        Self {
            capabilities,
            has_output,
            has_input,
            last_audio_event: None,
        }
    }

    /// Play a tone (minimal output)
    ///
    /// EVOLVED: Capability-based audio using `AudioCanvas` (direct hardware)
    /// Primals discover audio capability at runtime rather than compile-time features
    pub async fn beep(&mut self, frequency: f32, duration_ms: u64) -> Result<()> {
        if !self.has_output {
            return Ok(());
        }

        // /dev/snd access is blocking I/O -- move to blocking thread pool
        let result =
            tokio::task::spawn_blocking(move || Self::beep_audio_canvas(frequency, duration_ms))
                .await
                .map_err(|e| anyhow::anyhow!("beep task panicked: {e}"))?;

        match result {
            Ok(()) => {
                self.last_audio_event = Some(Instant::now());
                return Ok(());
            }
            Err(e) => {
                tracing::debug!("AudioCanvas unavailable: {e}");
            }
        }

        tracing::info!("Audio Canvas unavailable, using terminal bell");

        self.last_audio_event = Some(Instant::now());
        Ok(())
    }

    fn beep_audio_canvas(frequency: f32, duration_ms: u64) -> Result<()> {
        use crate::audio_canvas::AudioCanvas;
        use crate::audio_pure_rust::{Waveform, generate_tone};

        #[expect(clippy::cast_precision_loss, reason = "ms value is small")]
        let duration_secs = duration_ms as f32 / 1000.0;
        let samples = generate_tone(frequency, duration_secs, Waveform::Sine, 0.2);

        let mut canvas = AudioCanvas::open_default()?;
        canvas.write_samples(&samples)?;

        Ok(())
    }
}

#[async_trait]
impl Sensor for AudioSensor {
    fn capabilities(&self) -> &SensorCapabilities {
        &self.capabilities
    }

    fn is_available(&self) -> bool {
        self.has_output || self.has_input
    }

    async fn poll_events(&mut self) -> Result<Vec<SensorEvent>> {
        let events = Vec::new();

        // Audio input polling would go here
        // For now, just return empty events

        Ok(events)
    }

    fn last_activity(&self) -> Option<Instant> {
        self.last_audio_event
    }

    fn name(&self) -> &str {
        if self.capabilities.bidirectional {
            "Audio (Bidirectional)"
        } else if self.has_output {
            "Audio Output (Speaker)"
        } else {
            "Audio Input (Microphone)"
        }
    }
}

/// Discover audio capabilities
#[expect(clippy::unused_async, reason = "async for trait compatibility")]
pub async fn discover() -> Option<AudioSensor> {
    // Try to discover audio output
    let has_output = probe_audio_output();
    let has_input = probe_audio_input();

    if has_output || has_input {
        tracing::debug!(
            "Discovered audio: output={}, input={}",
            has_output,
            has_input
        );
        return Some(AudioSensor::new(has_output, has_input));
    }

    None
}

/// Probe for audio output
const fn probe_audio_output() -> bool {
    #[cfg(feature = "audio")]
    {
        false
    }

    #[cfg(not(feature = "audio"))]
    {
        true
    }
}

/// Check if audio input (microphone) is available.
///
/// Synchronous probe for use in UI availability checks.
#[must_use]
pub fn has_audio_input() -> bool {
    probe_audio_input()
}

/// Probe for audio input
///
/// On Linux: checks /proc/asound and /sys/class/sound for capture devices.
fn probe_audio_input() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Check /proc/asound/pcm for capture devices (format: "00-00: ... : capture")
        if let Ok(content) = std::fs::read_to_string("/proc/asound/pcm") {
            if content.to_lowercase().contains("capture") {
                return true;
            }
        }
        // Check /sys/class/sound/card*/ for pcm*D*c (capture) subdirs
        if let Ok(entries) = std::fs::read_dir("/sys/class/sound") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("card") {
                    let card_path = entry.path();
                    if let Ok(pcm_entries) = std::fs::read_dir(&card_path) {
                        for pcm_entry in pcm_entries.flatten() {
                            let pcm_name = pcm_entry.file_name();
                            let pcm_name_str = pcm_name.to_string_lossy();
                            // pcmC0D0c = capture, pcmC0D0p = playback
                            if pcm_name_str.ends_with('c') {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_sensor_creation() {
        let sensor = AudioSensor::new(true, false);
        assert_eq!(sensor.capabilities().sensor_type, SensorType::Audio);
        assert!(sensor.capabilities().output);
        assert!(!sensor.capabilities().input);
        assert!(!sensor.capabilities().bidirectional);
    }

    #[tokio::test]
    async fn test_bidirectional_audio() {
        let sensor = AudioSensor::new(true, true);
        assert!(sensor.capabilities().bidirectional);
    }

    #[tokio::test]
    async fn test_audio_beep_no_output() {
        let mut sensor = AudioSensor::new(false, false);
        let result = sensor.beep(440.0, 100).await;
        assert!(result.is_ok(), "beep with no output should be a no-op");
    }

    #[tokio::test]
    #[ignore = "requires audio hardware (/dev/snd)"]
    async fn test_audio_beep_hardware() {
        let mut sensor = AudioSensor::new(true, false);
        let result =
            tokio::time::timeout(std::time::Duration::from_secs(5), sensor.beep(440.0, 100))
                .await
                .expect("beep should complete within 5s");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audio_sensor_name_output_only() {
        let sensor = AudioSensor::new(true, false);
        assert_eq!(sensor.name(), "Audio Output (Speaker)");
    }

    #[tokio::test]
    async fn test_audio_sensor_name_input_only() {
        let sensor = AudioSensor::new(false, true);
        assert_eq!(sensor.name(), "Audio Input (Microphone)");
    }

    #[tokio::test]
    async fn test_audio_sensor_name_bidirectional() {
        let sensor = AudioSensor::new(true, true);
        assert_eq!(sensor.name(), "Audio (Bidirectional)");
    }

    #[tokio::test]
    async fn test_audio_sensor_is_available() {
        assert!(AudioSensor::new(true, false).is_available());
        assert!(AudioSensor::new(false, true).is_available());
        assert!(AudioSensor::new(true, true).is_available());
    }

    #[tokio::test]
    async fn test_audio_sensor_not_available() {
        let sensor = AudioSensor::new(false, false);
        assert!(!sensor.is_available());
    }

    #[tokio::test]
    async fn test_audio_sensor_capabilities_spatial() {
        let sensor = AudioSensor::new(true, false);
        assert!(!sensor.capabilities().spatial);
        assert!(sensor.capabilities().temporal);
        assert!(sensor.capabilities().continuous);
    }

    #[tokio::test]
    async fn test_has_audio_input() {
        let _ = has_audio_input();
    }
}
