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
    pub fn new(has_output: bool, has_input: bool) -> Self {
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
        println!("\x07");

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
fn probe_audio_output() -> bool {
    #[cfg(feature = "audio")]
    {
        false
    }

    #[cfg(not(feature = "audio"))]
    {
        true
    }
}

/// Probe for audio input
fn probe_audio_input() -> bool {
    // Audio input requires more complex setup
    // For Phase 1, we'll just assume it's not available
    false
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
}
