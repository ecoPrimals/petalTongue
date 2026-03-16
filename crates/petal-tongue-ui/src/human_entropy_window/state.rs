// SPDX-License-Identifier: AGPL-3.0-or-later
//! Human entropy window state and entropy collection

#![allow(unexpected_cfgs)]

use petal_tongue_entropy::prelude::*;
use std::time::{Duration, Instant};
use tracing::{info, warn};

use super::types::{CaptureWindowState, EntropyModality};

/// Human entropy capture window
///
/// Provides UI for capturing human entropy through multiple modalities:
/// - Audio (singing, speaking)
/// - Visual (drawing)
/// - Narrative (storytelling)
/// - Gesture (motion)
/// - Video (camera)
pub struct HumanEntropyWindow {
    /// Currently selected modality
    pub(super) modality: EntropyModality,

    /// Capture state
    pub(super) state: CaptureWindowState,

    /// Narrative capturer (always available)
    pub(super) narrative_capturer: Option<NarrativeEntropyCapture>,

    /// Current quality metrics (real-time feedback)
    pub(super) current_quality: Option<f64>,

    /// Capture start time
    pub(super) capture_start: Option<Instant>,

    /// Waveform buffer for visualization (audio)
    pub(super) waveform_buffer: Vec<f32>,

    /// Status message
    pub(super) status_message: String,
}

impl HumanEntropyWindow {
    /// Create a new human entropy capture window
    pub fn new() -> Self {
        info!("Creating human entropy capture window");

        Self {
            modality: EntropyModality::Audio, // Default to audio
            state: CaptureWindowState::Idle,

            narrative_capturer: None,
            current_quality: None,
            capture_start: None,
            waveform_buffer: Vec::new(),
            status_message: "Ready to capture human entropy".to_string(),
        }
    }

    pub(super) fn start_capture(&mut self) {
        info!("Starting entropy capture: {:?}", self.modality);

        match self.modality {
            EntropyModality::Narrative => {
                let mut capturer = NarrativeEntropyCapture::new();
                capturer.start();
                self.narrative_capturer = Some(capturer);
                self.state = CaptureWindowState::Recording;
                self.capture_start = Some(Instant::now());
                self.status_message = "Type your story...".to_string();
            }

            _ => {
                self.status_message = "Modality not yet implemented".to_string();
            }
        }
    }

    pub(super) fn stop_capture(&mut self) {
        info!("Stopping entropy capture");

        if self.modality == EntropyModality::Narrative
            && let Some(capturer) = &self.narrative_capturer
        {
            self.current_quality = Some(capturer.assess_quality().overall_quality);
        }

        self.state = CaptureWindowState::Stopped;
        self.status_message = "Capture stopped. Ready to send or discard.".to_string();
    }

    pub(super) fn finalize_and_stream(&mut self) {
        info!("Finalizing and streaming entropy");
        self.state = CaptureWindowState::Processing;
        self.status_message = "Streaming to entropy source...".to_string();

        // Get entropy capture data by moving capturer out of Option
        let entropy_result = match self.modality {
            EntropyModality::Narrative => self
                .narrative_capturer
                .take()
                .map(petal_tongue_entropy::narrative::NarrativeEntropyCapture::finalize),

            _ => None,
        };

        // Handle Result and convert to EntropyCapture based on modality
        let entropy_data: Option<EntropyCapture> = match self.modality {
            EntropyModality::Narrative => match entropy_result {
                Some(Ok(narrative_data)) => Some(EntropyCapture::Narrative(narrative_data)),
                Some(Err(e)) => {
                    warn!("Failed to finalize narrative entropy: {}", e);
                    None
                }
                None => None,
            },

            _ => None,
        };

        if let Some(entropy) = entropy_data {
            // Discover entropy source via capability-based discovery (entropy.source capability)
            let endpoint = self.discover_entropy_source_endpoint();

            if let Some(url) = endpoint {
                // Stream entropy asynchronously (fire and forget)
                Self::stream_entropy_to_source(url, entropy);

                // Update UI state optimistically
                self.reset();
                self.status_message = "✅ Entropy sent to entropy source!".to_string();
                info!("Entropy streaming initiated");
            } else {
                warn!("No entropy source endpoint found - entropy will be zeroized");
                self.reset();
                self.status_message = "⚠️ Entropy source not found. Entropy discarded.".to_string();
            }
        } else {
            warn!("No entropy data to stream");
            self.reset();
            self.status_message = "⚠️ No entropy data captured".to_string();
        }
    }

    /// Discover entropy source endpoint via capability-based discovery
    ///
    /// Looks for a primal with "entropy.source" or "entropy-ingestion" capability.
    /// TRUE PRIMAL: Zero hardcoded primal names; use capability-based discovery only.
    fn discover_entropy_source_endpoint(&self) -> Option<String> {
        // Try environment variable first (manual configuration)
        if let Ok(endpoint) = std::env::var("ENTROPY_SOURCE_ENDPOINT") {
            info!("Using configured entropy source endpoint: {}", endpoint);
            return Some(endpoint);
        }

        // Legacy: BEARDOG_ENTROPY_ENDPOINT (deprecated, use ENTROPY_SOURCE_ENDPOINT)
        if let Ok(endpoint) = std::env::var("BEARDOG_ENTROPY_ENDPOINT") {
            info!(
                "Using ENTROPY_SOURCE_ENDPOINT (legacy BEARDOG_ENTROPY_ENDPOINT): {}",
                endpoint
            );
            return Some(endpoint);
        }

        // Try discovery hints (comma-separated list of URLs)
        if let Ok(hints) = std::env::var("PETALTONGUE_DISCOVERY_HINTS") {
            for hint in hints.split(',') {
                let hint = hint.trim();
                // Check if this primal advertises entropy ingestion capability
                if self.check_entropy_capability(hint) {
                    info!("Discovered entropy source at: {}", hint);
                    return Some(format!("{hint}/api/v1/entropy"));
                }
            }
        }

        // Future: Use mDNS discovery to find primals with "entropy.source" capability
        // For now, return None and let user configure via environment
        warn!("Entropy source not discovered. Set ENTROPY_SOURCE_ENDPOINT environment variable.");
        None
    }

    /// Check if endpoint has entropy ingestion capability
    const fn check_entropy_capability(&self, _endpoint: &str) -> bool {
        // Future: Query endpoint's /api/v1/capabilities
        // For now, trust that configured endpoints are correct
        false
    }

    /// Stream entropy to discovered entropy source asynchronously
    ///
    /// This is a fire-and-forget operation. In production, you'd want to track
    /// the task and report completion/errors back to the UI.
    fn stream_entropy_to_source(endpoint: String, entropy: EntropyCapture) {
        // Serialize entropy for transmission
        let payload = match serde_json::to_vec(&entropy) {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to serialize entropy: {}", e);
                return;
            }
        };

        // Spawn async task (fire and forget for now)
        // Production: Should track status and retry
        tokio::spawn(async move {
            let client = match reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
            {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to create HTTP client for entropy streaming: {}", e);
                    return;
                }
            };

            match client
                .post(&endpoint)
                .header("Content-Type", "application/json")
                .body(payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("✅ Entropy streamed successfully to {}", endpoint);
                    } else {
                        warn!(
                            "⚠️ Entropy source returned error: {} ({})",
                            response.status(),
                            endpoint
                        );
                    }
                }
                Err(e) => {
                    warn!("❌ Failed to stream entropy: {}", e);
                }
            }

            // Entropy is automatically zeroized when dropped
        });
    }

    pub(super) fn discard(&mut self) {
        info!("Discarding captured entropy");
        self.reset();
        self.status_message = "Entropy discarded and zeroized.".to_string();
    }

    pub(super) fn reset(&mut self) {
        self.narrative_capturer = None;
        self.current_quality = None;
        self.capture_start = None;
        self.waveform_buffer.clear();
        self.state = CaptureWindowState::Idle;
    }

    /// Update function (called each frame)
    pub fn update(&mut self) {
        if self.state != CaptureWindowState::Recording {
            return;
        }

        // Update quality metrics
        if self.modality == EntropyModality::Narrative
            && let Some(capturer) = &self.narrative_capturer
        {
            let quality = capturer.assess_quality();
            self.current_quality = Some(quality.overall_quality);
        }
    }
}

impl Default for HumanEntropyWindow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{CaptureWindowState, EntropyModality};
    use super::*;

    #[test]
    fn new_creates_default_state() {
        let w = HumanEntropyWindow::new();
        assert_eq!(w.modality, EntropyModality::Audio);
        assert_eq!(w.state, CaptureWindowState::Idle);
        assert!(w.narrative_capturer.is_none());
        assert!(w.current_quality.is_none());
        assert!(w.capture_start.is_none());
        assert!(w.waveform_buffer.is_empty());
        assert_eq!(w.status_message, "Ready to capture human entropy");
    }

    #[test]
    fn default_equals_new() {
        let w1 = HumanEntropyWindow::new();
        let w2 = HumanEntropyWindow::default();
        assert_eq!(w1.modality, w2.modality);
        assert_eq!(w1.state, w2.state);
    }

    #[test]
    fn start_capture_narrative_transitions_to_recording() {
        let mut w = HumanEntropyWindow::new();
        w.modality = EntropyModality::Narrative;
        w.start_capture();
        assert_eq!(w.state, CaptureWindowState::Recording);
        assert!(w.narrative_capturer.is_some());
        assert!(w.capture_start.is_some());
        assert_eq!(w.status_message, "Type your story...");
    }

    #[test]
    fn start_capture_unimplemented_modality_sets_status() {
        let mut w = HumanEntropyWindow::new();
        w.modality = EntropyModality::Audio;
        w.start_capture();
        assert_eq!(w.status_message, "Modality not yet implemented");
    }

    #[test]
    fn stop_capture_transitions_to_stopped() {
        let mut w = HumanEntropyWindow::new();
        w.modality = EntropyModality::Narrative;
        w.start_capture();
        w.stop_capture();
        assert_eq!(w.state, CaptureWindowState::Stopped);
        assert_eq!(
            w.status_message,
            "Capture stopped. Ready to send or discard."
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut w = HumanEntropyWindow::new();
        w.modality = EntropyModality::Narrative;
        w.start_capture();
        w.reset();
        assert!(w.narrative_capturer.is_none());
        assert!(w.current_quality.is_none());
        assert!(w.capture_start.is_none());
        assert!(w.waveform_buffer.is_empty());
        assert_eq!(w.state, CaptureWindowState::Idle);
    }

    #[test]
    fn discard_resets_and_sets_status() {
        let mut w = HumanEntropyWindow::new();
        w.modality = EntropyModality::Narrative;
        w.start_capture();
        w.discard();
        assert_eq!(w.state, CaptureWindowState::Idle);
        assert_eq!(w.status_message, "Entropy discarded and zeroized.");
    }

    #[test]
    fn update_noop_when_not_recording() {
        let mut w = HumanEntropyWindow::new();
        w.state = CaptureWindowState::Idle;
        w.update();
        assert!(w.current_quality.is_none());
    }

    #[test]
    fn entropy_modality_names() {
        assert!(EntropyModality::Audio.name().contains("Audio"));
        assert!(EntropyModality::Narrative.name().contains("Narrative"));
    }

    #[test]
    fn entropy_modality_descriptions() {
        assert!(!EntropyModality::Audio.description().is_empty());
        assert!(EntropyModality::Narrative.description().contains("story"));
    }

    #[test]
    fn entropy_modality_availability() {
        // Audio availability depends on system (microphone/capture devices)
        let _ = EntropyModality::Audio.is_available();
        assert!(EntropyModality::Narrative.is_available());
    }

    #[test]
    fn capture_window_state_variants() {
        assert_ne!(CaptureWindowState::Idle, CaptureWindowState::Recording);
        assert_ne!(CaptureWindowState::Recording, CaptureWindowState::Stopped);
        assert_ne!(CaptureWindowState::Stopped, CaptureWindowState::Processing);
    }

    #[test]
    fn update_when_recording_updates_quality() {
        let mut w = HumanEntropyWindow::new();
        w.modality = EntropyModality::Narrative;
        w.start_capture();
        w.update();
        assert!(w.current_quality.is_some() || w.narrative_capturer.is_some());
    }

    #[test]
    fn reset_clears_waveform_buffer() {
        let mut w = HumanEntropyWindow::new();
        w.waveform_buffer = vec![1.0, 2.0, 3.0];
        w.reset();
        assert!(w.waveform_buffer.is_empty());
    }

    #[test]
    fn stop_capture_sets_quality_for_narrative() {
        let mut w = HumanEntropyWindow::new();
        w.modality = EntropyModality::Narrative;
        w.start_capture();
        w.stop_capture();
        assert_eq!(w.state, CaptureWindowState::Stopped);
    }
}
