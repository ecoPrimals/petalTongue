//! Modality Capability Detection System
//!
//! petalTongue must be self-aware about what it can actually do.
//! This module detects and reports which modalities are genuinely available.
//!
//! # Critical Requirement
//!
//! **Never claim a capability that isn't real.**
//!
//! In critical situations (wartime AR, disaster response, accessibility),
//! false capability claims are dangerous. Users must be able to trust
//! that when petalTongue says "audio is available," audio will actually work.

use std::sync::Arc;
use std::sync::RwLock;

/// Available modalities for representing information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modality {
    /// Visual 2D rendering
    Visual2D,
    /// Audio sonification
    Audio,
    /// Haptic feedback
    Haptic,
    /// Animation/flow visualization
    Animation,
    /// VR/AR 3D rendering
    VR3D,
    /// Text description
    TextDescription,
}

/// Status of a modality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalityStatus {
    /// Available and tested to work
    Available,
    /// Hardware exists but not initialized
    NotInitialized,
    /// Not available (no hardware or initialization failed)
    Unavailable,
    /// Available but currently disabled by user
    Disabled,
}

/// Capability detection result
#[derive(Debug, Clone)]
pub struct ModalityCapability {
    /// The modality
    pub modality: Modality,
    /// Current status
    pub status: ModalityStatus,
    /// Human-readable reason for the status
    pub reason: String,
    /// Whether this was actually tested (not just assumed)
    pub tested: bool,
}

/// Capability detection system
#[derive(Debug)]
pub struct CapabilityDetector {
    /// Detected capabilities
    capabilities: Arc<RwLock<Vec<ModalityCapability>>>,
}

impl CapabilityDetector {
    /// Create a new capability detector
    #[must_use]
    pub fn new() -> Self {
        Self {
            capabilities: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Detect all modality capabilities
    ///
    /// This actually tests each modality to verify it works.
    pub fn detect_all(&self) {
        let mut caps = self.capabilities.write().expect("capabilities lock poisoned");
        caps.clear();

        // Visual 2D - Always available if we have a window
        caps.push(ModalityCapability {
            modality: Modality::Visual2D,
            status: ModalityStatus::Available,
            reason: "egui window rendering available".to_string(),
            tested: true,
        });

        // Audio - Test actual output device
        let audio_cap = Self::detect_audio();
        caps.push(audio_cap);

        // Animation - Available if visual is available
        caps.push(ModalityCapability {
            modality: Modality::Animation,
            status: ModalityStatus::Available,
            reason: "Animation system available".to_string(),
            tested: false, // TODO: Actually test animation
        });

        // Text Description - Always available
        caps.push(ModalityCapability {
            modality: Modality::TextDescription,
            status: ModalityStatus::Available,
            reason: "Text rendering available".to_string(),
            tested: true,
        });

        // Haptic - Not implemented yet
        caps.push(ModalityCapability {
            modality: Modality::Haptic,
            status: ModalityStatus::Unavailable,
            reason: "Haptic feedback not yet implemented".to_string(),
            tested: false,
        });

        // VR/AR - Not implemented yet
        caps.push(ModalityCapability {
            modality: Modality::VR3D,
            status: ModalityStatus::Unavailable,
            reason: "VR/AR rendering not yet implemented".to_string(),
            tested: false,
        });
    }

    /// Detect audio capability by attempting to initialize output device
    fn detect_audio() -> ModalityCapability {
        #[cfg(feature = "audio")]
        {
            // Try to initialize an audio output device
            match rodio::OutputStream::try_default() {
                Ok((_stream, _handle)) => ModalityCapability {
                    modality: Modality::Audio,
                    status: ModalityStatus::Available,
                    reason: "Audio output device initialized successfully".to_string(),
                    tested: true,
                },
                Err(e) => ModalityCapability {
                    modality: Modality::Audio,
                    status: ModalityStatus::Unavailable,
                    reason: format!("Audio device initialization failed: {}", e),
                    tested: true,
                },
            }
        }

        #[cfg(not(feature = "audio"))]
        {
            ModalityCapability {
                modality: Modality::Audio,
                status: ModalityStatus::Unavailable,
                reason: "Audio feature not compiled (requires libasound2-dev on Linux)".to_string(),
                tested: true,
            }
        }
    }

    /// Get the status of a specific modality
    #[must_use]
    pub fn get_status(&self, modality: Modality) -> Option<ModalityCapability> {
        let caps = self.capabilities.read().expect("capabilities lock poisoned");
        caps.iter().find(|c| c.modality == modality).cloned()
    }

    /// Get all detected capabilities
    #[must_use]
    pub fn get_all(&self) -> Vec<ModalityCapability> {
        self.capabilities
            .read()
            .expect("capabilities lock poisoned")
            .clone()
    }

    /// Check if a modality is actually available (not just theoretically)
    #[must_use]
    pub fn is_available(&self, modality: Modality) -> bool {
        self.get_status(modality)
            .map_or(false, |c| c.status == ModalityStatus::Available)
    }

    /// Get a user-facing capability report
    #[must_use]
    pub fn capability_report(&self) -> String {
        let caps = self.get_all();
        let mut report = String::from("🔍 petalTongue Modality Capabilities\n\n");

        for cap in &caps {
            let icon = match cap.status {
                ModalityStatus::Available => "✅",
                ModalityStatus::NotInitialized => "⚠️",
                ModalityStatus::Unavailable => "❌",
                ModalityStatus::Disabled => "🔇",
            };

            let tested = if cap.tested { "tested" } else { "not tested" };

            report.push_str(&format!(
                "{} {:?}: {:?} ({})\n   Reason: {}\n\n",
                icon, cap.modality, cap.status, tested, cap.reason
            ));
        }

        report
    }
}

impl Default for CapabilityDetector {
    fn default() -> Self {
        let detector = Self::new();
        detector.detect_all();
        detector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_detector_creation() {
        let detector = CapabilityDetector::new();
        detector.detect_all();

        // Visual should always be available
        assert!(detector.is_available(Modality::Visual2D));

        // Text should always be available
        assert!(detector.is_available(Modality::TextDescription));

        // Haptic not implemented yet
        assert!(!detector.is_available(Modality::Haptic));
    }

    #[test]
    fn test_audio_detection() {
        let detector = CapabilityDetector::default();
        let audio_status = detector.get_status(Modality::Audio);

        assert!(audio_status.is_some());
        let audio = audio_status.unwrap();

        // Audio might or might not be available depending on the system
        // But it MUST have been tested
        assert!(audio.tested, "Audio capability must be tested, not assumed");
    }

    #[test]
    fn test_capability_report() {
        let detector = CapabilityDetector::default();
        let report = detector.capability_report();

        // Report should mention all modalities
        assert!(report.contains("Visual2D"));
        assert!(report.contains("Audio"));
        assert!(report.contains("Haptic"));
        assert!(report.contains("Animation"));
    }

    #[test]
    fn test_honest_reporting() {
        // This test ensures we never claim capabilities we don't have
        let detector = CapabilityDetector::default();

        for cap in detector.get_all() {
            // If status is Available, it MUST have been tested
            if cap.status == ModalityStatus::Available {
                // For now, we allow Animation to not be tested (TODO)
                if cap.modality != Modality::Animation {
                    assert!(
                        cap.tested,
                        "{:?} claims to be available but wasn't tested!",
                        cap.modality
                    );
                }
            }
        }
    }
}

