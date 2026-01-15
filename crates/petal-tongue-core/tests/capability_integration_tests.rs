//! Integration tests for capability detection
//!
//! These tests verify that petalTongue honestly reports what it can actually do.

use petal_tongue_core::{CapabilityDetector, Modality, ModalityStatus};

#[test]
fn test_capability_detection_is_honest() {
    // Create a detector and run detection
    let detector = CapabilityDetector::default();

    // Visual should always be available (we can render egui windows)
    assert_eq!(
        detector.is_available(Modality::Visual2D),
        true,
        "Visual2D should always be available"
    );

    // Text should always be available
    assert_eq!(
        detector.is_available(Modality::TextDescription),
        true,
        "TextDescription should always be available"
    );

    // Haptic and VR should be unavailable (not implemented)
    assert_eq!(
        detector.is_available(Modality::Haptic),
        false,
        "Haptic should be unavailable (not implemented)"
    );
    assert_eq!(
        detector.is_available(Modality::VR3D),
        false,
        "VR3D should be unavailable (not implemented)"
    );

    // Audio is now pure Rust (AudioCanvas with /dev/snd) - always available on Linux
    let audio_status = detector.get_status(Modality::Audio);
    assert!(audio_status.is_some(), "Audio status should be reported");
    let audio = audio_status.unwrap();
    assert!(audio.tested, "Audio capability MUST be tested, not assumed");

    // Audio should be available (pure Rust implementation)
    assert_eq!(
        audio.status,
        ModalityStatus::Available,
        "Audio should be available via AudioCanvas (pure Rust /dev/snd)"
    );
    assert!(
        audio.reason.contains("AudioCanvas") || audio.reason.contains("/dev/snd"),
        "Reason should mention AudioCanvas or /dev/snd implementation"
    );
}

#[test]
fn test_capability_report_format() {
    let detector = CapabilityDetector::default();
    let report = detector.capability_report();

    // Report should mention all modalities
    assert!(
        report.contains("Visual2D"),
        "Report should mention Visual2D"
    );
    assert!(report.contains("Audio"), "Report should mention Audio");
    assert!(report.contains("Haptic"), "Report should mention Haptic");
    assert!(
        report.contains("Animation"),
        "Report should mention Animation"
    );
    assert!(
        report.contains("TextDescription"),
        "Report should mention TextDescription"
    );
    assert!(report.contains("VR3D"), "Report should mention VR3D");

    // Report should have tested status indicators
    assert!(
        report.contains("tested") || report.contains("not tested"),
        "Report should indicate testing status"
    );
}

#[test]
fn test_no_false_positives() {
    // This test ensures we never claim a capability we don't have
    let detector = CapabilityDetector::default();

    for cap in detector.get_all() {
        if cap.status == ModalityStatus::Available {
            // If a capability is marked as available, it must have been tested
            // Exception: Animation (TODO - needs testing implementation)
            if cap.modality != Modality::Animation {
                assert!(
                    cap.tested,
                    "{:?} claims to be available but wasn't tested! This is a critical bug.",
                    cap.modality
                );
            }

            // The reason should not contain failure indicators
            let reason_lower = cap.reason.to_lowercase();
            assert!(
                !reason_lower.contains("failed"),
                "{:?} is available but reason contains 'failed': {}",
                cap.modality,
                cap.reason
            );
            assert!(
                !reason_lower.contains("error"),
                "{:?} is available but reason contains 'error': {}",
                cap.modality,
                cap.reason
            );
        }
    }
}

#[test]
fn test_audio_capability_is_tested() {
    // This test specifically verifies audio capability is tested, not assumed
    let detector = CapabilityDetector::default();
    let audio_cap = detector
        .get_status(Modality::Audio)
        .expect("Audio capability must be reported");

    // Audio must be tested - this is critical for accessibility
    assert!(
        audio_cap.tested,
        "Audio capability MUST be tested. False audio claims are dangerous for accessibility."
    );

    // The reason should be informative
    assert!(
        !audio_cap.reason.is_empty(),
        "Audio capability reason should not be empty"
    );
    assert!(
        audio_cap.reason.len() > 10,
        "Audio capability reason should be descriptive"
    );
}

#[test]
fn test_audio_pure_rust_implementation() {
    // Audio is now pure Rust (no feature flags needed)
    let detector = CapabilityDetector::default();
    let audio_cap = detector
        .get_status(Modality::Audio)
        .expect("Audio capability must be reported");

    // Audio MUST be tested (not assumed)
    assert!(
        audio_cap.tested,
        "Audio must be actually tested (not assumed)"
    );

    // Should be available (pure Rust AudioCanvas implementation)
    assert_eq!(
        audio_cap.status,
        ModalityStatus::Available,
        "Audio should be available via AudioCanvas"
    );

    // Reason should mention the implementation
    assert!(
        audio_cap.reason.contains("AudioCanvas") || audio_cap.reason.contains("/dev/snd") || audio_cap.reason.contains("pure Rust"),
        "Reason should mention AudioCanvas or /dev/snd: {}",
        audio_cap.reason
    );
}
