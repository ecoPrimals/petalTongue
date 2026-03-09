// SPDX-License-Identifier: AGPL-3.0-only
//! Status type definitions for machine-readable system reporting.

use serde::{Deserialize, Serialize};

/// Machine-readable status of the entire system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Timestamp of this status report
    pub timestamp: String,

    /// Overall health: "healthy", "degraded", "unhealthy"
    pub health: String,

    /// Modality capabilities (visual, audio, haptic, etc.)
    pub modalities: ModalityStatus,

    /// Audio system status
    pub audio: AudioStatus,

    /// Discovery status (mDNS, primals found, etc.)
    pub discovery: DiscoveryStatus,

    /// UI state
    pub ui: UIStatus,

    /// Recent events (last 10)
    pub recent_events: Vec<StatusEvent>,

    /// Warnings and errors
    pub issues: Vec<Issue>,
}

/// Modality status for all supported modalities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityStatus {
    /// 2D visual rendering status
    pub visual2d: ModalityState,
    /// Audio output status
    pub audio: ModalityState,
    /// Animation system status
    pub animation: ModalityState,
    /// Text description generation status
    pub text_description: ModalityState,
    /// Haptic feedback status
    pub haptic: ModalityState,
    /// 3D VR rendering status
    pub vr3d: ModalityState,
}

/// State of a single modality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityState {
    /// Whether modality is available
    pub available: bool,
    /// Whether modality has been tested
    pub tested: bool,
    /// Reason for current state (why available/unavailable)
    pub reason: String,
    /// Last time modality was used
    pub last_used: Option<String>,
}

/// Audio system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStatus {
    /// Is audio system initialized?
    pub initialized: bool,

    /// Current audio provider name
    pub current_provider: String,

    /// Available providers
    pub available_providers: Vec<AudioProviderInfo>,

    /// Last sound played (or attempted)
    pub last_sound: Option<AudioEvent>,

    /// Recent audio failures
    pub recent_failures: Vec<String>,

    /// Audio player availability
    pub system_players: Vec<String>,
}

/// Information about an audio provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioProviderInfo {
    /// Provider name
    pub name: String,
    /// Whether provider is currently available
    pub available: bool,
    /// Number of sounds available from this provider
    pub sounds_count: usize,
    /// Human-readable description of provider
    pub description: String,
}

/// Audio event (sound played or attempted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEvent {
    /// Name of sound played
    pub sound_name: String,
    /// Provider used to play sound
    pub provider: String,
    /// When event occurred
    pub timestamp: String,
    /// Whether playback succeeded
    pub success: bool,
    /// Error message if playback failed
    pub error_message: Option<String>,
}

/// Discovery status for primal and service discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStatus {
    /// Whether mDNS discovery is enabled
    pub mdns_enabled: bool,
    /// Number of visualization providers found
    pub providers_found: usize,
    /// Number of primals discovered
    pub primals_discovered: usize,
    /// Last successful discovery timestamp
    pub last_discovery: Option<String>,
    /// Recent discovery errors
    pub discovery_errors: Vec<String>,
}

/// UI status information for system monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIStatus {
    /// Whether the main window is currently open
    pub window_open: bool,
    /// Current frames per second
    pub fps: f32,
    /// Name of the currently active view
    pub active_view: String,
    /// Whether keyboard shortcuts are enabled
    pub keyboard_shortcuts_enabled: bool,
    /// List of active accessibility features
    pub accessibility_features: Vec<String>,
}

/// System status event for monitoring and diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEvent {
    /// RFC3339 timestamp when event occurred
    pub timestamp: String,
    /// Event category ("audio", "discovery", "ui", "modality")
    pub category: String,
    /// Event severity level ("info", "warning", "error")
    pub severity: String,
    /// Human-readable event message
    pub message: String,
}

/// System issue requiring attention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue severity level ("warning", "error", "critical")
    pub severity: String,
    /// Issue category for grouping
    pub category: String,
    /// Detailed issue description
    pub message: String,
    /// Suggested remediation action if available
    pub suggested_action: Option<String>,
    /// RFC3339 timestamp when issue was detected
    pub timestamp: String,
}
