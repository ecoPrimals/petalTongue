//! Machine-readable status reporter for AI and external system inspection
//!
//! This module makes petalTongue's internal state observable to:
//! - AI systems that need to understand what's happening
//! - External monitoring tools
//! - Automated testing systems
//! - Other primals that need to adjust their behavior

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

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

/// Thread-safe status reporter
pub struct StatusReporter {
    status: Arc<Mutex<SystemStatus>>,
    events: Arc<Mutex<Vec<StatusEvent>>>,
    status_file_path: Option<PathBuf>,
}

impl StatusReporter {
    /// Create a new status reporter with default state
    #[must_use]
    pub fn new() -> Self {
        let status = SystemStatus {
            timestamp: chrono::Utc::now().to_rfc3339(),
            health: "initializing".to_string(),
            modalities: ModalityStatus {
                visual2d: ModalityState {
                    available: false,
                    tested: false,
                    reason: "Not yet tested".to_string(),
                    last_used: None,
                },
                audio: ModalityState {
                    available: false,
                    tested: false,
                    reason: "Not yet tested".to_string(),
                    last_used: None,
                },
                animation: ModalityState {
                    available: false,
                    tested: false,
                    reason: "Not yet tested".to_string(),
                    last_used: None,
                },
                text_description: ModalityState {
                    available: false,
                    tested: false,
                    reason: "Not yet tested".to_string(),
                    last_used: None,
                },
                haptic: ModalityState {
                    available: false,
                    tested: false,
                    reason: "Not yet tested".to_string(),
                    last_used: None,
                },
                vr3d: ModalityState {
                    available: false,
                    tested: false,
                    reason: "Not yet tested".to_string(),
                    last_used: None,
                },
            },
            audio: AudioStatus {
                initialized: false,
                current_provider: "none".to_string(),
                available_providers: Vec::new(),
                last_sound: None,
                recent_failures: Vec::new(),
                system_players: Vec::new(),
            },
            discovery: DiscoveryStatus {
                mdns_enabled: false,
                providers_found: 0,
                primals_discovered: 0,
                last_discovery: None,
                discovery_errors: Vec::new(),
            },
            ui: UIStatus {
                window_open: false,
                fps: 0.0,
                active_view: "initializing".to_string(),
                keyboard_shortcuts_enabled: false,
                accessibility_features: Vec::new(),
            },
            recent_events: Vec::new(),
            issues: Vec::new(),
        };

        Self {
            status: Arc::new(Mutex::new(status)),
            events: Arc::new(Mutex::new(Vec::new())),
            status_file_path: None,
        }
    }

    /// Enable writing status to a file for external inspection
    pub fn enable_status_file(&mut self, path: PathBuf) {
        info!("📊 Status reporter will write to: {:?}", path);
        self.status_file_path = Some(path);
    }

    /// Get current status (thread-safe)
    #[must_use]
    pub fn get_status(&self) -> SystemStatus {
        let mut status = self.status.lock().unwrap().clone();
        status.timestamp = chrono::Utc::now().to_rfc3339();

        // Include recent events
        let events = self.events.lock().unwrap();
        status.recent_events = events.iter().rev().take(10).cloned().collect();

        status
    }

    /// Update modality status
    pub fn update_modality(&self, modality: &str, available: bool, tested: bool, reason: String) {
        {
            // Scope the lock to avoid deadlock
            let mut status = self.status.lock().unwrap();
            let state = ModalityState {
                available,
                tested,
                reason,
                last_used: if available {
                    Some(chrono::Utc::now().to_rfc3339())
                } else {
                    None
                },
            };

            match modality {
                "visual2d" => status.modalities.visual2d = state,
                "audio" => status.modalities.audio = state,
                "animation" => status.modalities.animation = state,
                "text_description" => status.modalities.text_description = state,
                "haptic" => status.modalities.haptic = state,
                "vr3d" => status.modalities.vr3d = state,
                _ => warn!("Unknown modality: {}", modality),
            }
        } // Lock dropped here

        self.add_event(
            "modality",
            "info",
            &format!(
                "{} modality: {}",
                modality,
                if available {
                    "available"
                } else {
                    "unavailable"
                }
            ),
        );
        self.write_status_file();
    }

    /// Report audio event (play attempt)
    pub fn report_audio_event(
        &self,
        sound_name: &str,
        provider: &str,
        success: bool,
        error_message: Option<String>,
    ) {
        let mut status = self.status.lock().unwrap();

        let event = AudioEvent {
            sound_name: sound_name.to_string(),
            provider: provider.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            success,
            error_message: error_message.clone(),
        };

        if success {
            self.add_event(
                "audio",
                "info",
                &format!("Played '{sound_name}' with {provider}"),
            );
        } else {
            let failure_msg = format!(
                "Failed to play '{}': {}",
                sound_name,
                error_message.as_deref().unwrap_or("unknown")
            );
            status.audio.recent_failures.push(failure_msg.clone());

            // Add as an issue
            status.issues.push(Issue {
                severity: "error".to_string(),
                category: "audio".to_string(),
                message: failure_msg.clone(),
                suggested_action: Some(
                    "Check audio system availability. Install mpv or aplay.".to_string(),
                ),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });

            self.add_event("audio", "error", &failure_msg);
        }

        status.audio.last_sound = Some(event);
        drop(status);

        self.write_status_file();
    }

    /// Update audio system status
    pub fn update_audio_system(
        &self,
        current_provider: String,
        providers: Vec<AudioProviderInfo>,
        system_players: Vec<String>,
    ) {
        let mut status = self.status.lock().unwrap();
        status.audio.initialized = true;
        status.audio.current_provider = current_provider;
        status.audio.available_providers = providers;
        status.audio.system_players = system_players;
        drop(status);

        self.add_event("audio", "info", "Audio system initialized");
        self.write_status_file();
    }

    /// Update overall health
    pub fn update_health(&self, health: &str) {
        let mut status = self.status.lock().unwrap();
        status.health = health.to_string();
        drop(status);

        self.write_status_file();
    }

    /// Add an event to the event log
    fn add_event(&self, category: &str, severity: &str, message: &str) {
        let event = StatusEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            category: category.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        // Keep only last 100 events
        if events.len() > 100 {
            let excess = events.len() - 100;
            events.drain(0..excess);
        }
    }

    /// Write status to file (for AI/external inspection)
    fn write_status_file(&self) {
        if let Some(path) = &self.status_file_path {
            let status = self.get_status();
            match serde_json::to_string_pretty(&status) {
                Ok(json) => match std::fs::write(path, json) {
                    Ok(()) => {
                        info!("✅ Status file written to: {:?}", path);
                    }
                    Err(e) => {
                        error!("❌ Failed to write status file to {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    error!("❌ Failed to serialize status: {}", e);
                }
            }
        } else {
            warn!("⚠️  Status file path not set - cannot write status file");
        }
    }

    /// Get status as JSON string
    pub fn get_status_json(&self) -> Result<String, String> {
        let status = self.get_status();
        serde_json::to_string_pretty(&status).map_err(|e| e.to_string())
    }

    /// Health check - returns true if system is healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        let status = self.status.lock().unwrap();
        status.health == "healthy"
    }

    /// Get critical issues (errors and critical warnings)
    #[must_use]
    pub fn get_critical_issues(&self) -> Vec<Issue> {
        let status = self.status.lock().unwrap();
        status
            .issues
            .iter()
            .filter(|issue| issue.severity == "error" || issue.severity == "critical")
            .cloned()
            .collect()
    }

    /// Force write status file immediately (for initialization)
    pub fn force_write(&self) {
        info!("🔄 Forcing status file write...");
        self.write_status_file();
    }
}

impl Default for StatusReporter {
    fn default() -> Self {
        Self::new()
    }
}
