// SPDX-License-Identifier: AGPL-3.0-only
//! Thread-safe status reporter implementation.

use super::types::{
    AudioEvent, AudioProviderInfo, AudioStatus, DiscoveryStatus, Issue, ModalityState,
    ModalityStatus, StatusEvent, SystemStatus, UIStatus,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

/// Lock mutex, recovering from poison by extracting the guard
fn lock_ignore_poison<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
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
        let mut status = lock_ignore_poison(&self.status).clone();
        status.timestamp = chrono::Utc::now().to_rfc3339();

        // Include recent events
        let events = lock_ignore_poison(&self.events);
        status.recent_events = events.iter().rev().take(10).cloned().collect();

        status
    }

    /// Update modality status
    pub fn update_modality(&self, modality: &str, available: bool, tested: bool, reason: String) {
        {
            // Scope the lock to avoid deadlock
            let mut status = lock_ignore_poison(&self.status);
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
        let mut status = lock_ignore_poison(&self.status);

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
        let mut status = lock_ignore_poison(&self.status);
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
        let mut status = lock_ignore_poison(&self.status);
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

        let mut events = lock_ignore_poison(&self.events);
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
        let status = lock_ignore_poison(&self.status);
        status.health == "healthy"
    }

    /// Get critical issues (errors and critical warnings)
    #[must_use]
    pub fn get_critical_issues(&self) -> Vec<Issue> {
        let status = lock_ignore_poison(&self.status);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_reporter_creation() {
        let reporter = StatusReporter::new();
        let status = reporter.get_status();
        assert_eq!(status.health, "initializing");
        assert!(!status.modalities.visual2d.available);
        assert!(!status.audio.initialized);
        assert_eq!(status.ui.active_view, "initializing");
    }

    #[test]
    fn test_update_modality() {
        let reporter = StatusReporter::new();
        reporter.update_modality("visual2d", true, true, "Test passed".to_string());
        let status = reporter.get_status();
        assert!(status.modalities.visual2d.available);
        assert!(status.modalities.visual2d.tested);
        assert_eq!(status.modalities.visual2d.reason, "Test passed");
        assert!(status.modalities.visual2d.last_used.is_some());
    }

    #[test]
    fn test_update_modality_unknown() {
        let reporter = StatusReporter::new();
        reporter.update_modality("unknown_mod", true, false, "test".to_string());
        // Should not panic - unknown modality is logged
        let _ = reporter.get_status();
    }

    #[test]
    fn test_report_audio_event_success() {
        let reporter = StatusReporter::new();
        reporter.report_audio_event("startup", "rodio", true, None);
        let status = reporter.get_status();
        assert!(status.audio.last_sound.is_some());
        let event = status
            .audio
            .last_sound
            .as_ref()
            .expect("last_sound present");
        assert_eq!(event.sound_name, "startup");
        assert_eq!(event.provider, "rodio");
        assert!(event.success);
        assert!(status.audio.recent_failures.is_empty());
    }

    #[test]
    fn test_report_audio_event_failure() {
        let reporter = StatusReporter::new();
        reporter.report_audio_event(
            "startup",
            "rodio",
            false,
            Some("Device not found".to_string()),
        );
        let status = reporter.get_status();
        assert!(status.audio.last_sound.is_some());
        let event = status
            .audio
            .last_sound
            .as_ref()
            .expect("last_sound present");
        assert!(!event.success);
        assert_eq!(event.error_message.as_deref(), Some("Device not found"));
        assert!(!status.audio.recent_failures.is_empty());
        assert!(!status.issues.is_empty());
    }

    #[test]
    fn test_update_health() {
        let reporter = StatusReporter::new();
        reporter.update_health("healthy");
        assert!(reporter.is_healthy());
        reporter.update_health("degraded");
        assert!(!reporter.is_healthy());
    }

    #[test]
    fn test_get_critical_issues() {
        let reporter = StatusReporter::new();
        reporter.report_audio_event("x", "y", false, Some("err".to_string()));
        let issues = reporter.get_critical_issues();
        assert!(!issues.is_empty());
        assert_eq!(issues[0].severity, "error");
        assert_eq!(issues[0].category, "audio");
    }

    #[test]
    fn test_get_status_json() {
        let reporter = StatusReporter::new();
        let json = reporter.get_status_json().expect("status JSON");
        assert!(json.contains("\"health\""));
        assert!(json.contains("initializing"));
    }

    #[test]
    fn test_update_audio_system() {
        let reporter = StatusReporter::new();
        reporter.update_audio_system(
            "rodio".to_string(),
            vec![AudioProviderInfo {
                name: "rodio".to_string(),
                available: true,
                sounds_count: 5,
                description: "Pure Rust".to_string(),
            }],
            vec!["aplay".to_string()],
        );
        let status = reporter.get_status();
        assert!(status.audio.initialized);
        assert_eq!(status.audio.current_provider, "rodio");
        assert_eq!(status.audio.available_providers.len(), 1);
        assert_eq!(status.audio.system_players.len(), 1);
    }

    #[test]
    fn test_default_impl() {
        let reporter = StatusReporter::default();
        let _ = reporter.get_status();
    }

    #[test]
    fn test_all_modality_updates() {
        let reporter = StatusReporter::new();
        for modality in [
            "visual2d",
            "audio",
            "animation",
            "text_description",
            "haptic",
            "vr3d",
        ] {
            reporter.update_modality(modality, true, true, "Test".to_string());
        }
        let status = reporter.get_status();
        assert!(status.modalities.visual2d.available);
        assert!(status.modalities.audio.available);
        assert!(status.modalities.animation.available);
        assert!(status.modalities.text_description.available);
        assert!(status.modalities.haptic.available);
        assert!(status.modalities.vr3d.available);
    }

    #[test]
    fn test_update_modality_unavailable() {
        let reporter = StatusReporter::new();
        reporter.update_modality("visual2d", false, true, "Test failed".to_string());
        let status = reporter.get_status();
        assert!(!status.modalities.visual2d.available);
        assert!(status.modalities.visual2d.last_used.is_none());
    }

    #[test]
    fn test_report_audio_event_failure_no_message() {
        let reporter = StatusReporter::new();
        reporter.report_audio_event("x", "y", false, None);
        let status = reporter.get_status();
        assert!(
            !status
                .audio
                .last_sound
                .as_ref()
                .expect("last_sound")
                .success
        );
        assert_eq!(
            status
                .audio
                .last_sound
                .as_ref()
                .expect("last_sound")
                .error_message,
            None
        );
        assert!(!status.audio.recent_failures.is_empty());
    }

    #[test]
    fn test_serde_roundtrip_system_status() {
        let reporter = StatusReporter::new();
        reporter.update_health("healthy");
        let status = reporter.get_status();
        let json = serde_json::to_string(&status).expect("serialize status");
        let restored: SystemStatus = serde_json::from_str(&json).expect("deserialize status");
        assert_eq!(restored.health, status.health);
        assert_eq!(
            restored.modalities.visual2d.available,
            status.modalities.visual2d.available
        );
    }

    #[test]
    fn test_serde_roundtrip_issue() {
        let issue = Issue {
            severity: "error".to_string(),
            category: "audio".to_string(),
            message: "Test".to_string(),
            suggested_action: Some("Fix it".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        let json = serde_json::to_string(&issue).expect("serialize issue");
        let restored: Issue = serde_json::from_str(&json).expect("deserialize issue");
        assert_eq!(restored.severity, issue.severity);
    }

    #[test]
    fn test_serde_roundtrip_status_event() {
        let event = StatusEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            category: "modality".to_string(),
            severity: "info".to_string(),
            message: "Test event".to_string(),
        };
        let json = serde_json::to_string(&event).expect("serialize event");
        let restored: StatusEvent = serde_json::from_str(&json).expect("deserialize event");
        assert_eq!(restored.message, event.message);
    }

    #[test]
    fn test_get_critical_issues_filters_warning() {
        let reporter = StatusReporter::new();
        // Add a warning - should NOT be in critical issues
        let mut status = lock_ignore_poison(&*reporter.status);
        status.issues.push(Issue {
            severity: "warning".to_string(),
            category: "test".to_string(),
            message: "Warning".to_string(),
            suggested_action: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
        drop(status);
        let issues = reporter.get_critical_issues();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_enable_status_file() {
        let mut reporter = StatusReporter::new();
        let temp = std::env::temp_dir().join("petaltongue_status_test.json");
        reporter.enable_status_file(temp.clone());
        reporter.update_health("healthy");
        reporter.force_write();
        assert!(temp.exists());
        let _ = std::fs::remove_file(temp);
    }
}
