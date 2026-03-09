// SPDX-License-Identifier: AGPL-3.0-only
//! Display verification tests

use petal_tongue_core::rendering_awareness::{InteractivityState, VisibilityState};

use super::types::{DisplayTopology, DisplayVerification, ViewerLocation};
use super::verifier::continuous_verification;

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
}
