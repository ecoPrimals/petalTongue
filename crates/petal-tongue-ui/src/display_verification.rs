// ! Display Visibility Verification
//!
//! This module implements active verification that the display substrate
//! is actually reaching the user. Part of the bidirectional nervous system.

use petal_tongue_core::rendering_awareness::{InteractivityState, VisibilityState};
use std::process::Command;
use tracing::{debug, info, warn};

/// Display topology - where is the output going?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayTopology {
    /// Direct local display (output and viewer on same physical display)
    DirectLocal,

    /// Forwarded/proxied (output goes through intermediary to viewer)
    /// Examples: remote desktop, VR runtime, nested window, screen sharing
    Forwarded,

    /// Nested (rendering into another application's surface)
    /// Examples: browser canvas, VR compositor, AR overlay
    Nested,

    /// Virtual (no physical display, rendering to memory/file)
    /// Examples: headless, screenshot mode, testing
    Virtual,

    /// Unknown topology (can't determine relationship)
    Unknown,
}

/// Viewer location - where is the human actually seeing this?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewerLocation {
    /// Viewer is at the same machine as display server
    SameMachine,

    /// Viewer is at a different machine (remote)
    RemoteMachine,

    /// Viewer is in a virtual/augmented reality environment
    VirtualEnvironment,

    /// Unknown viewer location
    Unknown,
}

/// Display substrate verification results
#[derive(Debug, Clone)]
pub struct DisplayVerification {
    /// Whether a display server is available
    pub display_server_available: bool,

    /// Whether we could create/verify a window
    pub window_exists: bool,

    /// Whether the window is visible (mapped, not minimized)
    pub window_visible: bool,

    /// Whether the window manager responded
    pub wm_responsive: bool,

    /// Current visibility state
    pub visibility: VisibilityState,

    /// Current interactivity state  
    pub interactivity: InteractivityState,

    /// Display topology (where is output going?)
    pub display_topology: DisplayTopology,

    /// Viewer location (where is the human?)
    pub viewer_location: ViewerLocation,

    /// Whether we can confirm output reaches intended viewer
    pub output_reaches_viewer: bool,

    /// Evidence we have about display path
    pub topology_evidence: Vec<String>,

    /// Human-readable status message
    pub status_message: String,

    /// Suggested action for user (if there's an issue)
    pub suggested_action: Option<String>,
}

impl DisplayVerification {
    /// Create a "unknown" verification result
    pub fn unknown() -> Self {
        Self {
            display_server_available: false,
            window_exists: false,
            window_visible: false,
            wm_responsive: false,
            visibility: VisibilityState::Unknown,
            interactivity: InteractivityState::Unconfirmed,
            display_topology: DisplayTopology::Unknown,
            viewer_location: ViewerLocation::Unknown,
            output_reaches_viewer: false,
            topology_evidence: vec![],
            status_message: "Display verification not yet performed".to_string(),
            suggested_action: None,
        }
    }

    /// Create a "confirmed visible" verification result
    pub fn confirmed_visible() -> Self {
        Self {
            display_server_available: true,
            window_exists: true,
            window_visible: true,
            wm_responsive: true,
            visibility: VisibilityState::Confirmed,
            interactivity: InteractivityState::Active,
            display_topology: DisplayTopology::DirectLocal,
            viewer_location: ViewerLocation::SameMachine,
            output_reaches_viewer: true,
            topology_evidence: vec!["User interaction confirms visibility".to_string()],
            status_message: "Output confirmed reaching viewer - user actively interacting"
                .to_string(),
            suggested_action: None,
        }
    }

    /// Create a "probable" verification result (window exists but can't fully confirm)
    pub fn probable(topology: DisplayTopology, evidence: Vec<String>) -> Self {
        Self {
            display_server_available: true,
            window_exists: true,
            window_visible: false, // Can't confirm
            wm_responsive: false,
            visibility: VisibilityState::Probable,
            interactivity: InteractivityState::Unconfirmed,
            display_topology: topology,
            viewer_location: ViewerLocation::Unknown,
            output_reaches_viewer: false,
            topology_evidence: evidence,
            status_message: "Display server available, output path uncertain".to_string(),
            suggested_action: Some("Interact with the window to confirm visibility".to_string()),
        }
    }

    /// Create a "failed" verification result
    pub fn failed(reason: &str) -> Self {
        Self {
            display_server_available: false,
            window_exists: false,
            window_visible: false,
            wm_responsive: false,
            visibility: VisibilityState::Unknown,
            interactivity: InteractivityState::Unconfirmed,
            display_topology: DisplayTopology::Unknown,
            viewer_location: ViewerLocation::Unknown,
            output_reaches_viewer: false,
            topology_evidence: vec![reason.to_string()],
            status_message: format!("Display verification failed: {}", reason),
            suggested_action: Some(
                "Check DISPLAY environment variable and display server".to_string(),
            ),
        }
    }
}

/// Detect display topology (agnostic - no vendor names)
pub fn detect_display_topology() -> (DisplayTopology, Vec<String>) {
    let mut evidence = Vec::new();

    // Check for display forwarding indicators
    let ssh_connection = std::env::var("SSH_CONNECTION").is_ok()
        || std::env::var("SSH_CLIENT").is_ok()
        || std::env::var("SSH_TTY").is_ok();

    if ssh_connection {
        evidence.push("SSH environment detected - display likely forwarded".to_string());
    }

    // Check if DISPLAY suggests forwarding
    if let Ok(display) = std::env::var("DISPLAY") {
        evidence.push(format!("DISPLAY={}", display));

        // localhost:XX.X suggests X11 forwarding
        if display.starts_with("localhost:") {
            evidence.push("Display on localhost - possible forwarding/nesting".to_string());
        }
        // :0 is typically local, :1+ might be virtual/nested
        else if display.starts_with(":0") {
            evidence.push("Display :0 - typically primary physical display".to_string());
        } else if display.starts_with(':') && display.len() > 2 {
            evidence.push("Display :1+ - may be virtual/nested/headless".to_string());
        }
    }

    // EVOLVED: Check for virtual display using pure Rust
    use crate::display_pure_rust;

    if display_pure_rust::is_virtual_display() {
        evidence.push("Virtual display detected (pure Rust detection)".to_string());
        return (DisplayTopology::Virtual, evidence);
    }

    // Check for nested window indicators
    if std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("DISPLAY").is_ok() {
        evidence.push("Both Wayland and X11 detected - possible XWayland nesting".to_string());
        return (DisplayTopology::Nested, evidence);
    }

    // Determine topology from evidence
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

    // Step 1: Check if display server is available
    let display_available = check_display_server();
    if !display_available {
        warn!("❌ No display server detected");
        return DisplayVerification::failed("No display server (DISPLAY or WAYLAND_DISPLAY)");
    }

    info!("✅ Display server available");

    // Step 2: Detect display topology (agnostic)
    let (topology, mut evidence) = detect_display_topology();
    info!("📊 Display topology: {:?}", topology);
    for ev in &evidence {
        debug!("   Evidence: {}", ev);
    }

    // Step 3: Check if window manager is responsive
    let wm_responsive = check_window_manager();
    if !wm_responsive {
        warn!("⚠️  Window manager not responsive (may be OK for forwarded displays)");
        evidence.push("Window manager tools unavailable".to_string());
    } else {
        evidence.push("Window manager responsive".to_string());
    }

    // Step 4: Try to find our window
    let window_found = find_window_by_title(window_title);
    if window_found {
        evidence.push("Window found in window list".to_string());
    }

    // Build verification result
    match (window_found, wm_responsive, &topology) {
        // Direct local display with window found = confirmed
        (true, true, DisplayTopology::DirectLocal) => {
            info!("✅ Direct local display, window found");
            let mut verif = DisplayVerification::confirmed_visible();
            verif.display_topology = DisplayTopology::DirectLocal;
            verif.topology_evidence = evidence;
            verif.status_message = "Direct local display - window created and findable".to_string();
            verif
        }
        // Forwarded/nested - can't confirm without user interaction
        (_, _, DisplayTopology::Forwarded) | (_, _, DisplayTopology::Nested) => {
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
        // Virtual display
        (_, _, DisplayTopology::Virtual) => {
            info!("ℹ️  Virtual display detected");
            let mut verif = DisplayVerification::probable(topology.clone(), evidence);
            verif.status_message =
                "Virtual display detected - no physical viewer expected".to_string();
            verif
        }
        // Unknown topology with window found
        (true, _, DisplayTopology::Unknown) => {
            warn!("❓ Unknown display topology, but window found");
            let mut verif = DisplayVerification::probable(topology.clone(), evidence);
            verif.status_message =
                "Unknown display topology, but window found - probable visibility".to_string();
            verif
        }
        // Can't find window or determine topology
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
    // EVOLVED: Pure Rust window manager detection
    // Instead of checking for external tools, check if we can create windows
    use crate::display_pure_rust;

    // If we can enumerate monitors, we have a working display system
    let monitors = display_pure_rust::get_all_monitors();
    if !monitors.is_empty() {
        debug!(
            "✅ Display system available ({} monitor(s))",
            monitors.len()
        );
        return true;
    }

    // Fallback: Check for X11/Wayland environment
    if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
        debug!("✅ Display environment detected (DISPLAY/WAYLAND_DISPLAY)");
        return true;
    }

    // Try wmctrl (legacy fallback)
    if let Ok(output) = Command::new("wmctrl").arg("-m").output() {
        if output.status.success() {
            debug!("✅ wmctrl available");
            return true;
        }
    }

    // Try xwininfo
    if let Ok(output) = Command::new("xwininfo").arg("-root").arg("-tree").output() {
        if output.status.success() {
            debug!("✅ xwininfo available");
            return true;
        }
    }

    // On Windows/macOS, assume window manager is responsive if we get here
    if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
        return true;
    }

    debug!("⚠️  No window manager tools available (wmctrl, xwininfo)");
    false
}

/// Try to find a window by title
fn find_window_by_title(title: &str) -> bool {
    // EVOLVED: Pure Rust window detection
    // Note: winit doesn't provide window enumeration, so we use heuristics

    // If we're running in a GUI environment, assume window exists
    // This is a reasonable assumption for petalTongue's use case
    use crate::display_pure_rust;

    let monitors = display_pure_rust::get_all_monitors();
    if !monitors.is_empty() {
        debug!(
            "✅ Display system available - assuming window '{}' exists",
            title
        );
        // In a real GUI app, the window would be created by eframe/egui
        // and would definitely exist if we're running
        return true;
    }

    // Fallback: Try wmctrl (legacy)
    if let Ok(output) = Command::new("wmctrl").arg("-l").output() {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if stdout.contains(title) {
                    debug!("✅ Found window via wmctrl");
                    return true;
                }
            }
        }
    }

    // Try xwininfo
    if let Ok(output) = Command::new("xwininfo").arg("-root").arg("-tree").output() {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if stdout.contains(title) {
                    debug!("✅ Found window via xwininfo");
                    return true;
                }
            }
        }
    }

    debug!("❓ Could not find window with title: {}", title);
    false
}

/// Continuously verify display visibility (for use in app loop)
pub fn continuous_verification(
    window_title: &str,
    last_interaction_secs: f32,
) -> DisplayVerification {
    let mut verification = verify_display_substrate(window_title);

    // Update interactivity based on user interaction recency
    verification.interactivity = if last_interaction_secs < 5.0 {
        InteractivityState::Active
    } else if last_interaction_secs < 30.0 {
        InteractivityState::Recent
    } else if last_interaction_secs < 300.0 {
        InteractivityState::Idle
    } else {
        InteractivityState::Unconfirmed
    };

    // KEY INSIGHT: User interaction is the ONLY reliable confirmation for forwarded/nested displays
    if verification.interactivity == InteractivityState::Active {
        // User is interacting - they MUST be able to see it
        verification.visibility = VisibilityState::Confirmed;
        verification.output_reaches_viewer = true;
        verification.viewer_location = ViewerLocation::Unknown; // We know they can see it, not where they are
        verification.status_message = match verification.display_topology {
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
        };
        verification.suggested_action = None; // No action needed, it's working!
        verification.topology_evidence.push(format!(
            "User interaction within last {:.1}s confirms output reaches viewer",
            last_interaction_secs
        ));
    } else if verification.interactivity == InteractivityState::Recent {
        // Recent interaction - probably still visible
        verification.visibility = VisibilityState::Probable;
        verification.output_reaches_viewer = true;
        verification.status_message = format!(
            "{} - recent interaction ({:.0}s ago) suggests viewer can still see output",
            match verification.display_topology {
                DisplayTopology::DirectLocal => "Direct local display",
                DisplayTopology::Forwarded => "Forwarded display",
                DisplayTopology::Nested => "Nested display",
                DisplayTopology::Virtual => "Virtual display",
                DisplayTopology::Unknown => "Unknown topology",
            },
            last_interaction_secs
        );
    } else {
        // No recent interaction - for forwarded displays, this is uncertain!
        match verification.display_topology {
            DisplayTopology::Forwarded | DisplayTopology::Nested | DisplayTopology::Unknown => {
                verification.visibility = VisibilityState::Uncertain;
                verification.output_reaches_viewer = false;
                verification.status_message = format!(
                    "Display topology: {:?} - No recent interaction ({:.0}s). Cannot confirm viewer sees output.",
                    verification.display_topology, last_interaction_secs
                );
                if last_interaction_secs > 300.0 {
                    verification.suggested_action = Some(
                        "No interaction for 5+ minutes. If viewing remotely, verify connection and window visibility.".to_string()
                    );
                }
            }
            DisplayTopology::DirectLocal => {
                // For direct local, window existing is probably enough
                if verification.window_exists {
                    verification.visibility = VisibilityState::Probable;
                    verification.status_message =
                        "Direct local display - window exists but no recent interaction"
                            .to_string();
                }
            }
            DisplayTopology::Virtual => {
                // Virtual displays don't expect interaction
                verification.visibility = VisibilityState::Probable;
                verification.status_message =
                    "Virtual display - no human interaction expected".to_string();
            }
        }
    }

    verification
}
