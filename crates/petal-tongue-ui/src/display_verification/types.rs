// SPDX-License-Identifier: AGPL-3.0-or-later
//! Display verification types

use petal_tongue_core::rendering_awareness::{InteractivityState, VisibilityState};

/// Display topology - where is the output going?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayTopology {
    /// Direct local display (output and viewer on same physical display)
    DirectLocal,

    /// Forwarded/proxied (output goes through intermediary to viewer)
    Forwarded,

    /// Nested (rendering into another application's surface)
    Nested,

    /// Virtual (no physical display, rendering to memory/file)
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn probable(topology: DisplayTopology, evidence: Vec<String>) -> Self {
        Self {
            display_server_available: true,
            window_exists: true,
            window_visible: false,
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
    #[must_use]
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
            status_message: format!("Display verification failed: {reason}"),
            suggested_action: Some(
                "Check DISPLAY environment variable and display server".to_string(),
            ),
        }
    }
}
