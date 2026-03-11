// SPDX-License-Identifier: AGPL-3.0-only
//! Display verification logic

use std::process::Command;

use petal_tongue_core::rendering_awareness::{InteractivityState, VisibilityState};
use tracing::{debug, info, warn};

use super::types::{DisplayTopology, DisplayVerification, ViewerLocation};

// ============================================================================
// Pure verification logic (testable, no I/O)
// ============================================================================

/// Map seconds since last interaction to interactivity state
#[must_use]
pub fn interactivity_state_from_seconds(secs: f32) -> InteractivityState {
    if secs < 5.0 {
        InteractivityState::Active
    } else if secs < 30.0 {
        InteractivityState::Recent
    } else if secs < 300.0 {
        InteractivityState::Idle
    } else {
        InteractivityState::Unconfirmed
    }
}

/// Human-readable topology description for status messages
#[must_use]
pub const fn topology_description(topology: &DisplayTopology) -> &'static str {
    match topology {
        DisplayTopology::DirectLocal => "Direct local display",
        DisplayTopology::Forwarded => "Forwarded display",
        DisplayTopology::Nested => "Nested display",
        DisplayTopology::Virtual => "Virtual display",
        DisplayTopology::Unknown => "Unknown topology",
    }
}

/// Status message when user is actively interacting
#[must_use]
pub fn format_active_interaction_status(topology: &DisplayTopology) -> String {
    match topology {
        DisplayTopology::DirectLocal => {
            "Direct local display - user actively interacting".to_string()
        }
        DisplayTopology::Forwarded => {
            "Forwarded display confirmed - user interaction proves visibility".to_string()
        }
        DisplayTopology::Nested => {
            "Nested display confirmed - user interaction proves visibility".to_string()
        }
        DisplayTopology::Virtual => {
            "Virtual display with active interaction (unusual but confirmed)".to_string()
        }
        DisplayTopology::Unknown => {
            "Display topology unknown, but user interaction confirms visibility".to_string()
        }
    }
}

/// Status message for recent interaction (5-30s ago)
#[must_use]
pub fn format_recent_interaction_status(topology: &DisplayTopology, secs: f32) -> String {
    format!(
        "{} - recent interaction ({secs:.0}s ago) suggests viewer can still see output",
        topology_description(topology)
    )
}

/// Suggested action when no interaction for 5+ minutes
#[must_use]
pub fn suggested_action_for_prolonged_idle(secs: f32) -> Option<String> {
    if secs > 300.0 {
        Some(
            "No interaction for 5+ minutes. If viewing remotely, verify connection and window visibility.".to_string()
        )
    } else {
        None
    }
}

/// Detect display topology (agnostic - no vendor names)
#[must_use]
pub fn detect_display_topology() -> (DisplayTopology, Vec<String>) {
    let mut evidence = Vec::new();

    let ssh_connection = std::env::var("SSH_CONNECTION").is_ok()
        || std::env::var("SSH_CLIENT").is_ok()
        || std::env::var("SSH_TTY").is_ok();

    if ssh_connection {
        evidence.push("SSH environment detected - display likely forwarded".to_string());
    }

    if let Ok(display) = std::env::var("DISPLAY") {
        evidence.push(format!("DISPLAY={display}"));

        if display.starts_with("localhost:") {
            evidence.push("Display on localhost - possible forwarding/nesting".to_string());
        } else if display.starts_with(":0") {
            evidence.push("Display :0 - typically primary physical display".to_string());
        } else if display.starts_with(':') && display.len() > 2 {
            evidence.push("Display :1+ - may be virtual/nested/headless".to_string());
        }
    }

    use crate::display_pure_rust;

    if display_pure_rust::is_virtual_display() {
        evidence.push("Virtual display detected (pure Rust detection)".to_string());
        return (DisplayTopology::Virtual, evidence);
    }

    if std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("DISPLAY").is_ok() {
        evidence.push("Both Wayland and X11 detected - possible XWayland nesting".to_string());
        return (DisplayTopology::Nested, evidence);
    }

    let topology = if ssh_connection {
        DisplayTopology::Forwarded
    } else if evidence
        .iter()
        .any(|e| e.contains("forwarding") || e.contains("localhost"))
    {
        DisplayTopology::Forwarded
    } else if evidence
        .iter()
        .any(|e| e.contains("virtual") || e.contains("Xvfb"))
    {
        DisplayTopology::Virtual
    } else if evidence.iter().any(|e| e.contains(":0")) {
        DisplayTopology::DirectLocal
    } else {
        DisplayTopology::Unknown
    };

    (topology, evidence)
}

/// Verify display substrate is actually reaching the user
pub fn verify_display_substrate(window_title: &str) -> DisplayVerification {
    info!("🔍 Verifying display substrate...");

    let display_available = check_display_server();
    if !display_available {
        warn!("❌ No display server detected");
        return DisplayVerification::failed("No display server (DISPLAY or WAYLAND_DISPLAY)");
    }

    info!("✅ Display server available");

    let (topology, mut evidence) = detect_display_topology();
    info!("📊 Display topology: {:?}", topology);
    for ev in &evidence {
        debug!("   Evidence: {}", ev);
    }

    let wm_responsive = check_window_manager();
    if wm_responsive {
        evidence.push("Window manager responsive".to_string());
    } else {
        warn!("⚠️  Window manager not responsive (may be OK for forwarded displays)");
        evidence.push("Window manager tools unavailable".to_string());
    }

    let window_found = find_window_by_title(window_title);
    if window_found {
        evidence.push("Window found in window list".to_string());
    }

    match (window_found, wm_responsive, &topology) {
        (true, true, DisplayTopology::DirectLocal) => {
            info!("✅ Direct local display, window found");
            let mut verif = DisplayVerification::confirmed_visible();
            verif.display_topology = DisplayTopology::DirectLocal;
            verif.topology_evidence = evidence;
            verif.status_message = "Direct local display - window created and findable".to_string();
            verif
        }
        (_, _, DisplayTopology::Forwarded | DisplayTopology::Nested) => {
            warn!("⚠️  Forwarded/nested display - cannot confirm viewer can see output");
            let mut verif = DisplayVerification::probable(topology.clone(), evidence);
            verif.status_message = format!(
                "Display topology: {:?} - output path uncertain, needs user interaction to confirm",
                verif.display_topology
            );
            verif.suggested_action = Some(
                "If you can see this window, interact with it to confirm visibility".to_string(),
            );
            verif
        }
        (_, _, DisplayTopology::Virtual) => {
            info!("ℹ️  Virtual display detected");
            let mut verif = DisplayVerification::probable(topology.clone(), evidence);
            verif.status_message =
                "Virtual display detected - no physical viewer expected".to_string();
            verif
        }
        (true, _, DisplayTopology::Unknown) => {
            warn!("❓ Unknown display topology, but window found");
            let mut verif = DisplayVerification::probable(topology.clone(), evidence);
            verif.status_message =
                "Unknown display topology, but window found - probable visibility".to_string();
            verif
        }
        _ => {
            warn!("❓ Display topology unclear and window not found");
            let mut verif = DisplayVerification::probable(topology.clone(), evidence);
            verif.status_message =
                "Display topology unclear and window not found - uncertain".to_string();
            verif
        }
    }
}

/// Check if a display server is available
fn check_display_server() -> bool {
    std::env::var("DISPLAY").is_ok()
        || std::env::var("WAYLAND_DISPLAY").is_ok()
        || cfg!(target_os = "windows")
        || cfg!(target_os = "macos")
}

/// Check if window manager is responsive
fn check_window_manager() -> bool {
    use crate::display_pure_rust;

    let monitors = display_pure_rust::get_all_monitors();
    if !monitors.is_empty() {
        debug!(
            "✅ Display system available ({} monitor(s))",
            monitors.len()
        );
        return true;
    }

    if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
        debug!("✅ Display environment detected (DISPLAY/WAYLAND_DISPLAY)");
        return true;
    }

    if let Ok(output) = Command::new("wmctrl").arg("-m").output()
        && output.status.success()
    {
        debug!("✅ wmctrl available");
        return true;
    }

    if let Ok(output) = Command::new("xwininfo").arg("-root").arg("-tree").output()
        && output.status.success()
    {
        debug!("✅ xwininfo available");
        return true;
    }

    if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
        return true;
    }

    debug!("⚠️  No window manager tools available (wmctrl, xwininfo)");
    false
}

/// Try to find a window by title
fn find_window_by_title(title: &str) -> bool {
    use crate::display_pure_rust;

    let monitors = display_pure_rust::get_all_monitors();
    if !monitors.is_empty() {
        debug!(
            "✅ Display system available - assuming window '{}' exists",
            title
        );
        return true;
    }

    if let Ok(output) = Command::new("wmctrl").arg("-l").output()
        && output.status.success()
        && let Ok(stdout) = String::from_utf8(output.stdout)
        && stdout.contains(title)
    {
        debug!("✅ Found window via wmctrl");
        return true;
    }

    if let Ok(output) = Command::new("xwininfo").arg("-root").arg("-tree").output()
        && output.status.success()
        && let Ok(stdout) = String::from_utf8(output.stdout)
        && stdout.contains(title)
    {
        debug!("✅ Found window via xwininfo");
        return true;
    }

    debug!("❓ Could not find window with title: {}", title);
    false
}

/// Continuously verify display visibility (for use in app loop)
#[must_use]
pub fn continuous_verification(
    window_title: &str,
    last_interaction_secs: f32,
) -> DisplayVerification {
    let mut verification = verify_display_substrate(window_title);

    verification.interactivity = interactivity_state_from_seconds(last_interaction_secs);

    if verification.interactivity == InteractivityState::Active {
        verification.visibility = VisibilityState::Confirmed;
        verification.output_reaches_viewer = true;
        verification.viewer_location = ViewerLocation::Unknown;
        verification.status_message =
            format_active_interaction_status(&verification.display_topology);
        verification.suggested_action = None;
        verification.topology_evidence.push(format!(
            "User interaction within last {last_interaction_secs:.1}s confirms output reaches viewer"
        ));
    } else if verification.interactivity == InteractivityState::Recent {
        verification.visibility = VisibilityState::Probable;
        verification.output_reaches_viewer = true;
        verification.status_message =
            format_recent_interaction_status(&verification.display_topology, last_interaction_secs);
    } else {
        match verification.display_topology {
            DisplayTopology::Forwarded | DisplayTopology::Nested | DisplayTopology::Unknown => {
                verification.visibility = VisibilityState::Uncertain;
                verification.output_reaches_viewer = false;
                verification.status_message = format!(
                    "Display topology: {:?} - No recent interaction ({:.0}s). Cannot confirm viewer sees output.",
                    verification.display_topology, last_interaction_secs
                );
                verification.suggested_action =
                    suggested_action_for_prolonged_idle(last_interaction_secs);
            }
            DisplayTopology::DirectLocal => {
                if verification.window_exists {
                    verification.visibility = VisibilityState::Probable;
                    verification.status_message =
                        "Direct local display - window exists but no recent interaction"
                            .to_string();
                }
            }
            DisplayTopology::Virtual => {
                verification.visibility = VisibilityState::Probable;
                verification.status_message =
                    "Virtual display - no human interaction expected".to_string();
            }
        }
    }

    verification
}
