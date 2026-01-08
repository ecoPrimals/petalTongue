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
    pub async fn beep(&mut self, frequency: f32, duration_ms: u64) -> Result<()> {
        if !self.has_output {
            return Ok(());
        }

        // Simple beep using rodio
        #[cfg(feature = "audio")]
        {
            use rodio::{
                OutputStream, Sink,
                source::{SineWave, Source},
            };
            use std::time::Duration;

            if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    let source = SineWave::new(frequency)
                        .take_duration(Duration::from_millis(duration_ms))
                        .amplify(0.20);

                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        }

        #[cfg(not(feature = "audio"))]
        {
            // Fallback: print to terminal
            println!("\x07"); // Bell character
        }

        self.last_audio_event = Some(Instant::now());
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
        use rodio::OutputStream;
        OutputStream::try_default().is_ok()
    }

    #[cfg(not(feature = "audio"))]
    {
        // Fallback: assume terminal bell works
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
    async fn test_audio_beep() {
        let mut sensor = AudioSensor::new(true, false);
        let result = sensor.beep(440.0, 100).await;
        assert!(result.is_ok());
    }
}
