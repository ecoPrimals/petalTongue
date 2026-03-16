// SPDX-License-Identifier: AGPL-3.0-or-later
//! Display verification tests

use petal_tongue_core::rendering_awareness::{InteractivityState, VisibilityState};

use super::types::{DisplayTopology, DisplayVerification, ViewerLocation};
use super::verifier::{
    continuous_verification, detect_display_topology, format_active_interaction_status,
    format_recent_interaction_status, interactivity_state_from_seconds,
    suggested_action_for_prolonged_idle, topology_description,
};

#[test]
fn test_display_verification_unknown() {
    let v = DisplayVerification::unknown();
    assert!(!v.display_server_available);
    assert!(!v.window_exists);
    assert!(!v.window_visible);
    assert!(!v.output_reaches_viewer);
    assert_eq!(v.visibility, VisibilityState::Unknown);
    assert_eq!(v.interactivity, InteractivityState::Unconfirmed);
    assert!(v.topology_evidence.is_empty());
}

#[test]
fn test_display_verification_confirmed_visible() {
    let v = DisplayVerification::confirmed_visible();
    assert!(v.display_server_available);
    assert!(v.window_exists);
    assert!(v.window_visible);
    assert!(v.output_reaches_viewer);
    assert_eq!(v.visibility, VisibilityState::Confirmed);
    assert_eq!(v.interactivity, InteractivityState::Active);
    assert_eq!(v.display_topology, DisplayTopology::DirectLocal);
    assert_eq!(v.viewer_location, ViewerLocation::SameMachine);
}

#[test]
fn test_display_verification_failed() {
    let v = DisplayVerification::failed("No DISPLAY");
    assert!(!v.display_server_available);
    assert!(!v.output_reaches_viewer);
    assert!(v.status_message.contains("No DISPLAY"));
    assert!(v.suggested_action.is_some());
    assert_eq!(v.topology_evidence.len(), 1);
}

#[test]
fn test_display_verification_probable() {
    let evidence = vec!["X11 forwarding detected".to_string()];
    let v = DisplayVerification::probable(DisplayTopology::Forwarded, evidence.clone());
    assert!(v.display_server_available);
    assert!(!v.window_visible);
    assert_eq!(v.visibility, VisibilityState::Probable);
    assert_eq!(v.display_topology, DisplayTopology::Forwarded);
    assert_eq!(v.topology_evidence, evidence);
    assert!(v.suggested_action.is_some());
}

#[test]
fn test_display_topology_variants() {
    assert_ne!(DisplayTopology::DirectLocal, DisplayTopology::Forwarded);
    assert_ne!(DisplayTopology::Virtual, DisplayTopology::Nested);
}

#[test]
fn test_viewer_location_variants() {
    assert_ne!(ViewerLocation::SameMachine, ViewerLocation::RemoteMachine);
}

#[test]
fn test_continuous_verification_interaction_states() {
    let v = continuous_verification("test-window", 2.0);
    assert_eq!(v.interactivity, InteractivityState::Active);

    let v = continuous_verification("test-window", 10.0);
    assert_eq!(v.interactivity, InteractivityState::Recent);

    let v = continuous_verification("test-window", 100.0);
    assert_eq!(v.interactivity, InteractivityState::Idle);

    let v = continuous_verification("test-window", 400.0);
    assert_eq!(v.interactivity, InteractivityState::Unconfirmed);
}

#[test]
fn test_display_verification_status_message_format() {
    let v = DisplayVerification::failed("custom reason");
    assert!(v.status_message.starts_with("Display verification failed:"));
    assert!(v.status_message.contains("custom reason"));
}

#[test]
fn test_display_verification_confirmed_evidence() {
    let v = DisplayVerification::confirmed_visible();
    assert!(!v.topology_evidence.is_empty());
    assert!(v.topology_evidence[0].contains("User interaction"));
}

#[test]
fn test_detect_display_topology_returns_valid() {
    let (topology, evidence) = detect_display_topology();
    match topology {
        DisplayTopology::DirectLocal
        | DisplayTopology::Forwarded
        | DisplayTopology::Nested
        | DisplayTopology::Virtual
        | DisplayTopology::Unknown => {}
    }
    let _ = evidence;
}

#[test]
fn test_continuous_verification_active_sets_visibility() {
    let v = continuous_verification("test-window", 1.0);
    assert_eq!(v.interactivity, InteractivityState::Active);
    assert_eq!(v.visibility, VisibilityState::Confirmed);
    assert!(v.output_reaches_viewer);
}

#[test]
fn test_continuous_verification_recent_sets_probable() {
    let v = continuous_verification("test-window", 15.0);
    assert_eq!(v.interactivity, InteractivityState::Recent);
    assert_eq!(v.visibility, VisibilityState::Probable);
    assert!(v.output_reaches_viewer);
}

#[test]
fn test_interactivity_state_from_seconds() {
    assert_eq!(
        interactivity_state_from_seconds(2.0),
        InteractivityState::Active
    );
    assert_eq!(
        interactivity_state_from_seconds(4.9),
        InteractivityState::Active
    );
    assert_eq!(
        interactivity_state_from_seconds(10.0),
        InteractivityState::Recent
    );
    assert_eq!(
        interactivity_state_from_seconds(29.9),
        InteractivityState::Recent
    );
    assert_eq!(
        interactivity_state_from_seconds(100.0),
        InteractivityState::Idle
    );
    assert_eq!(
        interactivity_state_from_seconds(299.9),
        InteractivityState::Idle
    );
    assert_eq!(
        interactivity_state_from_seconds(400.0),
        InteractivityState::Unconfirmed
    );
}

#[test]
fn test_topology_description() {
    assert_eq!(
        topology_description(&DisplayTopology::DirectLocal),
        "Direct local display"
    );
    assert_eq!(
        topology_description(&DisplayTopology::Forwarded),
        "Forwarded display"
    );
    assert_eq!(
        topology_description(&DisplayTopology::Virtual),
        "Virtual display"
    );
    assert_eq!(
        topology_description(&DisplayTopology::Unknown),
        "Unknown topology"
    );
}

#[test]
fn test_format_active_interaction_status() {
    let s = format_active_interaction_status(&DisplayTopology::DirectLocal);
    assert!(s.contains("actively interacting"));
    let s = format_active_interaction_status(&DisplayTopology::Forwarded);
    assert!(s.contains("user interaction proves visibility"));
}

#[test]
fn test_format_recent_interaction_status() {
    let s = format_recent_interaction_status(&DisplayTopology::DirectLocal, 15.0);
    assert!(s.contains("Direct local display"));
    assert!(s.contains("15"));
    assert!(s.contains("recent interaction"));
}

#[test]
fn test_suggested_action_for_prolonged_idle() {
    assert!(suggested_action_for_prolonged_idle(100.0).is_none());
    assert!(suggested_action_for_prolonged_idle(300.0).is_none());
    let action = suggested_action_for_prolonged_idle(301.0);
    assert!(action.is_some());
    let action = action.expect("some");
    assert!(action.contains("5+ minutes"));
}

#[test]
fn test_suggested_action_boundary() {
    assert!(suggested_action_for_prolonged_idle(299.9).is_none());
    assert!(suggested_action_for_prolonged_idle(300.1).is_some());
}

#[test]
fn test_topology_description_nested() {
    assert_eq!(
        topology_description(&DisplayTopology::Nested),
        "Nested display"
    );
}

#[test]
fn test_format_active_interaction_status_all_variants() {
    let s = format_active_interaction_status(&DisplayTopology::Nested);
    assert!(s.contains("Nested"));
    assert!(s.contains("user interaction proves visibility"));

    let s = format_active_interaction_status(&DisplayTopology::Virtual);
    assert!(s.contains("Virtual"));
    assert!(s.contains("unusual but confirmed"));

    let s = format_active_interaction_status(&DisplayTopology::Unknown);
    assert!(s.contains("unknown"));
    assert!(s.contains("user interaction confirms visibility"));
}

#[test]
fn test_format_recent_interaction_status_variants() {
    let s = format_recent_interaction_status(&DisplayTopology::Forwarded, 10.0);
    assert!(s.contains("Forwarded display"));
    assert!(s.contains("10"));
    assert!(s.contains("recent interaction"));

    let s = format_recent_interaction_status(&DisplayTopology::Virtual, 25.5);
    assert!(s.contains("Virtual display"));
    assert!(s.contains("26"));
}

#[test]
fn test_interactivity_state_boundaries() {
    assert_eq!(
        interactivity_state_from_seconds(5.0),
        InteractivityState::Recent
    );
    assert_eq!(
        interactivity_state_from_seconds(30.0),
        InteractivityState::Idle
    );
    assert_eq!(
        interactivity_state_from_seconds(300.0),
        InteractivityState::Unconfirmed
    );
    assert_eq!(
        interactivity_state_from_seconds(0.0),
        InteractivityState::Active
    );
}

#[test]
fn test_display_verification_probable_status() {
    let evidence = vec!["test".to_string()];
    let v = DisplayVerification::probable(DisplayTopology::Virtual, evidence);
    assert!(v.status_message.contains("uncertain"));
}

#[test]
fn test_display_verification_failed_suggested_action() {
    let v = DisplayVerification::failed("reason");
    assert!(v.suggested_action.is_some());
    assert!(v.suggested_action.unwrap().contains("DISPLAY"));
}
