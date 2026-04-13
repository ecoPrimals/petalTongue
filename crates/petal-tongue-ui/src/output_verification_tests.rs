// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use std::time::Duration;

#[test]
fn test_output_modality_variants() {
    assert_eq!(OutputModality::Visual, OutputModality::Visual);
    assert_eq!(OutputModality::Audio, OutputModality::Audio);
    assert_eq!(OutputModality::Haptic, OutputModality::Haptic);
    assert_eq!(OutputModality::Olfactory, OutputModality::Olfactory);
    assert_eq!(OutputModality::Thermal, OutputModality::Thermal);
    assert_eq!(
        OutputModality::Generic("custom".to_string()),
        OutputModality::Generic("custom".to_string())
    );
    assert_ne!(OutputModality::Visual, OutputModality::Audio);
}

#[test]
fn test_output_topology_variants() {
    assert_eq!(OutputTopology::Direct, OutputTopology::Direct);
    assert_eq!(OutputTopology::Forwarded, OutputTopology::Forwarded);
    assert_eq!(OutputTopology::Nested, OutputTopology::Nested);
    assert_eq!(OutputTopology::Virtual, OutputTopology::Virtual);
    assert_eq!(OutputTopology::Unknown, OutputTopology::Unknown);
}

#[test]
fn test_output_confirmation_variants() {
    assert_eq!(
        OutputConfirmation::UserInteraction,
        OutputConfirmation::UserInteraction
    );
    assert_eq!(OutputConfirmation::DeviceAck, OutputConfirmation::DeviceAck);
    assert_eq!(OutputConfirmation::Echo, OutputConfirmation::Echo);
    assert_eq!(
        OutputConfirmation::IndirectEvidence,
        OutputConfirmation::IndirectEvidence
    );
    assert_eq!(
        OutputConfirmation::Unconfirmable,
        OutputConfirmation::Unconfirmable
    );
}

#[test]
fn test_output_verification_unverified() {
    let v = OutputVerification::unverified(OutputModality::Visual);
    assert!(!v.device_available);
    assert!(!v.output_active);
    assert_eq!(v.topology, OutputTopology::Unknown);
    assert!(!v.reaches_user);
    assert_eq!(v.confirmation_method, OutputConfirmation::Unconfirmable);
    assert!(v.evidence.is_empty());
    assert!(v.last_confirmed.is_none());
    assert!(v.status_message.contains("not yet verified"));
    assert!(v.suggested_action.is_none());
}

#[test]
fn test_confirm_via_interaction() {
    let mut v = OutputVerification::unverified(OutputModality::Audio);
    v.confirm_via_interaction();
    assert!(v.reaches_user);
    assert_eq!(v.confirmation_method, OutputConfirmation::UserInteraction);
    assert!(v.last_confirmed.is_some());
    assert!(v.status_message.contains("user interaction"));
    assert!(v.suggested_action.is_none());
    assert!(!v.evidence.is_empty());
}

#[test]
fn test_confirm_via_device_ack() {
    let mut v = OutputVerification::unverified(OutputModality::Visual);
    v.confirm_via_device_ack("HDMI-1".to_string());
    assert!(v.reaches_user);
    assert_eq!(v.confirmation_method, OutputConfirmation::DeviceAck);
    assert!(v.last_confirmed.is_some());
    assert!(v.status_message.contains("HDMI-1"));
    assert!(!v.evidence.is_empty());
}

#[test]
fn test_confirm_via_echo() {
    let mut v = OutputVerification::unverified(OutputModality::Audio);
    v.confirm_via_echo("mic feedback".to_string());
    assert!(v.reaches_user);
    assert_eq!(v.confirmation_method, OutputConfirmation::Echo);
    assert!(v.last_confirmed.is_some());
    assert!(v.status_message.contains("echo"));
    assert!(v.evidence.iter().any(|e| e.contains("mic feedback")));
}

#[test]
fn test_is_stale_none_confirmed() {
    let v = OutputVerification::unverified(OutputModality::Visual);
    assert!(v.is_stale(Duration::from_secs(30)));
}

#[test]
fn test_is_stale_recently_confirmed() {
    let mut v = OutputVerification::unverified(OutputModality::Visual);
    v.confirm_via_interaction();
    assert!(!v.is_stale(Duration::from_secs(300)));
}

#[test]
fn test_output_verification_system_new() {
    let sys = OutputVerificationSystem::new();
    assert!(sys.get_all_verifications().is_empty());
    assert!(!sys.has_unconfirmed_outputs());
}

#[test]
fn test_output_verification_system_default() {
    let sys = OutputVerificationSystem::default();
    assert!(sys.get_all_verifications().is_empty());
}

#[test]
fn test_register_output() {
    let mut sys = OutputVerificationSystem::new();
    sys.register_output(OutputModality::Visual);
    sys.register_output(OutputModality::Audio);
    assert_eq!(sys.get_all_verifications().len(), 2);
    assert!(sys.has_unconfirmed_outputs());
    assert!(sys.get_verification(&OutputModality::Visual).is_some());
    assert!(sys.get_verification(&OutputModality::Audio).is_some());
    assert!(sys.get_verification(&OutputModality::Haptic).is_none());
}

#[test]
fn test_register_output_idempotent() {
    let mut sys = OutputVerificationSystem::new();
    sys.register_output(OutputModality::Visual);
    sys.register_output(OutputModality::Visual);
    assert_eq!(sys.get_all_verifications().len(), 1);
}

#[test]
fn test_confirm_via_interaction_system() {
    let mut sys = OutputVerificationSystem::new();
    sys.register_output(OutputModality::Visual);
    sys.confirm_via_interaction(&OutputModality::Visual);
    assert!(!sys.has_unconfirmed_outputs());
    let v = sys.get_verification(&OutputModality::Visual).unwrap();
    assert!(v.reaches_user);
}

#[test]
fn test_confirm_via_device_system() {
    let mut sys = OutputVerificationSystem::new();
    sys.register_output(OutputModality::Audio);
    sys.confirm_via_device(&OutputModality::Audio, "speaker-0".to_string());
    let v = sys.get_verification(&OutputModality::Audio).unwrap();
    assert!(v.reaches_user);
    assert!(v.status_message.contains("speaker-0"));
}

#[test]
fn test_confirm_via_echo_system() {
    let mut sys = OutputVerificationSystem::new();
    sys.register_output(OutputModality::Haptic);
    sys.confirm_via_echo(&OutputModality::Haptic, "feedback".to_string());
    let v = sys.get_verification(&OutputModality::Haptic).unwrap();
    assert!(v.reaches_user);
}

#[test]
fn test_get_status_summary() {
    let mut sys = OutputVerificationSystem::new();
    sys.register_output(OutputModality::Visual);
    sys.register_output(OutputModality::Audio);
    sys.confirm_via_interaction(&OutputModality::Visual);
    let summary = sys.get_status_summary();
    assert!(summary.contains("Outputs:"));
    assert!(summary.contains('1'));
    assert!(summary.contains('2'));
}

#[test]
fn test_detect_visual_topology() {
    let (topo, evidence) = detect_visual_topology();
    assert!(
        matches!(
            topo,
            OutputTopology::Direct
                | OutputTopology::Forwarded
                | OutputTopology::Nested
                | OutputTopology::Virtual
                | OutputTopology::Unknown
        ),
        "topo = {topo:?}"
    );
    assert!(!evidence.is_empty() || topo == OutputTopology::Unknown);
}

#[test]
fn test_detect_audio_topology() {
    let (topo, evidence) = detect_audio_topology();
    assert!(
        matches!(
            topo,
            OutputTopology::Direct | OutputTopology::Forwarded | OutputTopology::Unknown
        ),
        "topo = {topo:?}"
    );
    assert!(!evidence.is_empty());
}

#[test]
fn test_detect_haptic_topology() {
    let (topo, evidence) = detect_haptic_topology();
    assert!(
        matches!(topo, OutputTopology::Direct | OutputTopology::Unknown),
        "topo = {topo:?}"
    );
    assert!(!evidence.is_empty());
}

#[test]
fn test_update_early_return_when_elapsed_under_5_secs() {
    let mut sys = OutputVerificationSystem::new();
    sys.register_output(OutputModality::Visual);
    sys.update();
    let v = sys.get_verification(&OutputModality::Visual).unwrap();
    assert_eq!(
        v.state,
        petal_tongue_core::rendering_awareness::VisibilityState::Unknown
    );
}
