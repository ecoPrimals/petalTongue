// SPDX-License-Identifier: AGPL-3.0-only
//! Proprioception Panel - SAME DAVE Self-Awareness Visualization
//!
//! SAME DAVE is neuroanatomy, not AI — the channel model maps specialized
//! unidirectional pathways analogous to the spinal cord's dorsal/ventral roots.
//! Sensory Afferent pathways carry input TO the proprioception core. Motor
//! Efferent pathways carry commands FROM the core to effectors. Classification
//! nodes along each channel act like nodes of Ranvier, enabling saltatory
//! signal routing.
//!
//! Displays Neural API proprioception data (system self-awareness) in an egui panel.
//! Updates automatically every 5 seconds with fresh data from Neural API.

use egui::{Color32, ProgressBar, RichText, Ui};
use petal_tongue_core::{
    ChannelSnapshot, MotorData, ProprioceptionData, SelfAwarenessData, SensoryData,
    channel::ChannelDirection,
};
use petal_tongue_discovery::NeuralApiProvider;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct MotorHistoryEntry {
    pub command: String,
    pub timestamp: Instant,
}

/// Auto-refresh interval for proprioception data (5 seconds)
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

/// Proprioception visualization panel
pub struct ProprioceptionPanel {
    /// Current proprioception data (None if not yet fetched)
    data: Option<ProprioceptionData>,

    /// Last update timestamp
    last_update: Instant,

    /// Whether data is currently being fetched
    fetching: bool,

    pub(crate) motor_history: Vec<MotorHistoryEntry>,
    pub(crate) current_mode: String,
    pub(crate) session_domain: Option<String>,
}

impl ProprioceptionPanel {
    /// Create a new proprioception panel
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: None,
            last_update: Instant::now()
                .checked_sub(REFRESH_INTERVAL)
                .unwrap_or_else(Instant::now),
            fetching: false,
            motor_history: Vec::new(),
            current_mode: "default".to_string(),
            session_domain: None,
        }
    }

    /// Update proprioception data from Neural API (async)
    ///
    /// This should be called from an async context. The UI will show stale data
    /// while fetching new data.
    pub async fn update(&mut self, provider: &NeuralApiProvider) {
        if self.last_update.elapsed() < REFRESH_INTERVAL {
            return; // Too soon to refresh
        }

        if self.fetching {
            return; // Already fetching
        }

        self.fetching = true;
        debug!("Fetching proprioception data from Neural API...");

        match provider.get_proprioception().await {
            Ok(data) => {
                debug!("Proprioception data received: {}", data.summary());
                self.data = Some(data);
                self.last_update = Instant::now();
            }
            Err(e) => {
                warn!("Failed to fetch proprioception data: {}", e);
                // Keep old data if fetch fails (graceful degradation)
            }
        }

        self.fetching = false;
    }

    /// Merge local channel snapshots into proprioception data.
    pub fn merge_local_channels(
        &mut self,
        afferent: impl IntoIterator<Item = ChannelSnapshot>,
        efferent: impl IntoIterator<Item = ChannelSnapshot>,
    ) {
        let afferent: Vec<_> = afferent.into_iter().collect();
        let efferent: Vec<_> = efferent.into_iter().collect();
        if afferent.is_empty() && efferent.is_empty() {
            return;
        }
        if let Some(data) = &mut self.data {
            for snap in afferent {
                if let Some(existing) = data.afferent_channels.iter_mut().find(|c| c.id == snap.id)
                {
                    *existing = snap;
                } else {
                    data.afferent_channels.push(snap);
                }
            }
            for snap in efferent {
                if let Some(existing) = data.efferent_channels.iter_mut().find(|c| c.id == snap.id)
                {
                    *existing = snap;
                } else {
                    data.efferent_channels.push(snap);
                }
            }
        } else {
            let mut data = ProprioceptionData::empty("local");
            data.afferent_channels = afferent;
            data.efferent_channels = efferent;
            self.data = Some(data);
        }
    }

    pub fn record_motor_command(&mut self, description: &str) {
        const MAX_HISTORY: usize = 20;
        self.motor_history.push(MotorHistoryEntry {
            command: description.to_string(),
            timestamp: Instant::now(),
        });
        if self.motor_history.len() > MAX_HISTORY {
            self.motor_history.remove(0);
        }
    }

    pub fn set_current_mode(&mut self, mode: &str) {
        self.current_mode = mode.to_string();
    }

    pub fn set_session_domain(&mut self, domain: Option<String>) {
        self.session_domain = domain;
    }

    /// Render the proprioception panel
    pub fn render(&self, ui: &mut Ui) {
        ui.heading("🧠 SAME DAVE Proprioception");
        ui.label(
            RichText::new("Sensory Afferent · Motor Efferent")
                .color(Color32::from_rgb(156, 163, 175))
                .italics(),
        );

        ui.separator();

        if let Some(data) = &self.data {
            self.render_health_indicator(ui, data);
            ui.add_space(8.0);
            self.render_confidence_meter(ui, data);
            ui.add_space(8.0);
            self.render_channel_overview(ui, data);
            ui.add_space(8.0);
            self.render_same_dave_panel(ui, data);
            ui.add_space(8.0);
            self.render_timestamp(ui, data);
        } else {
            ui.label(
                RichText::new("No proprioception data available")
                    .color(Color32::from_rgb(156, 163, 175)),
            );
            ui.label("Waiting for Neural API...");
        }

        ui.add_space(8.0);
        self.render_motor_status(ui);
    }

    /// Render health indicator with color coding
    fn render_health_indicator(&self, ui: &mut Ui, data: &ProprioceptionData) {
        ui.horizontal(|ui| {
            // Emoji indicator
            ui.label(RichText::new(data.health.status.emoji()).size(24.0));

            ui.vertical(|ui| {
                // Status text with color
                let (r, g, b) = data.health.status.color_rgb();
                let color = Color32::from_rgb(r, g, b);

                ui.label(
                    RichText::new(format!("Health: {:.1}%", data.health.percentage))
                        .size(18.0)
                        .color(color)
                        .strong(),
                );

                ui.label(RichText::new(format!("Status: {}", data.health.status)).color(color));
            });
        });
    }

    /// Render confidence meter as a progress bar
    fn render_confidence_meter(&self, ui: &mut Ui, data: &ProprioceptionData) {
        ui.label(RichText::new("Confidence").strong());

        // Progress bar with color coding
        let progress = data.confidence / 100.0;
        let color = if data.is_confident() {
            Color32::from_rgb(34, 197, 94) // green-500
        } else if data.confidence >= 50.0 {
            Color32::from_rgb(234, 179, 8) // yellow-500
        } else {
            Color32::from_rgb(239, 68, 68) // red-500
        };

        ui.add(
            ProgressBar::new(progress)
                .fill(color)
                .text(format!("{:.1}%", data.confidence)),
        );
    }

    /// Render SAME DAVE system detail (sensory/motor/awareness).
    fn render_same_dave_panel(&self, ui: &mut Ui, data: &ProprioceptionData) {
        ui.group(|ui| {
            ui.label(RichText::new("System Snapshot").strong().size(14.0));

            ui.separator();

            self.render_sensory_section(ui, &data.sensory);
            ui.add_space(4.0);

            self.render_awareness_section(ui, &data.self_awareness);
            ui.add_space(4.0);

            self.render_motor_section(ui, &data.motor);
            ui.add_space(4.0);

            self.render_evaluative_section(ui, data);
        });
    }

    /// Render afferent/efferent channel overview.
    fn render_channel_overview(&self, ui: &mut Ui, data: &ProprioceptionData) {
        ui.group(|ui| {
            ui.label(RichText::new("Channel Health").strong().size(14.0));
            ui.separator();

            let afferent: Vec<_> = data
                .afferent_channels
                .iter()
                .filter(|c| c.direction == ChannelDirection::Afferent)
                .collect();
            let efferent: Vec<_> = data
                .efferent_channels
                .iter()
                .filter(|c| c.direction == ChannelDirection::Efferent)
                .collect();

            ui.label(
                RichText::new(format!("Afferent (sensory): {} channels", afferent.len()))
                    .color(Color32::from_rgb(96, 165, 250)), // blue-400
            );
            for ch in &afferent {
                self.render_channel_row(ui, ch);
            }

            ui.add_space(4.0);

            ui.label(
                RichText::new(format!("Efferent (motor): {} channels", efferent.len()))
                    .color(Color32::from_rgb(52, 211, 153)), // emerald-400
            );
            for ch in &efferent {
                self.render_channel_row(ui, ch);
            }

            if afferent.is_empty() && efferent.is_empty() {
                ui.label(
                    RichText::new("No channel data (local mode)")
                        .color(Color32::from_rgb(156, 163, 175)),
                );
            }
        });
    }

    /// Render a single channel status row.
    fn render_channel_row(&self, ui: &mut Ui, ch: &ChannelSnapshot) {
        ui.horizontal(|ui| {
            let active_color = if ch.active {
                Color32::from_rgb(34, 197, 94)
            } else {
                Color32::from_rgb(107, 114, 128)
            };
            let dot = if ch.active { "●" } else { "○" };
            ui.label(RichText::new(dot).color(active_color));
            ui.label(&ch.id);

            if ch.signals_in > 0 {
                ui.label(
                    RichText::new(format!("{}→{}", ch.signals_in, ch.signals_out))
                        .color(Color32::from_rgb(156, 163, 175)),
                );
                ui.add(
                    ProgressBar::new(ch.throughput)
                        .desired_width(50.0)
                        .fill(active_color),
                );
            }

            if ch.node_count > 0 {
                ui.label(
                    RichText::new(format!("{}n", ch.node_count))
                        .color(Color32::from_rgb(156, 163, 175)),
                );
            }
        });
    }

    /// Render sensory section
    fn render_sensory_section(&self, ui: &mut Ui, sensory: &SensoryData) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("👁️ Sensory:").strong());
            ui.label(format!(
                "{} active sockets detected",
                sensory.active_sockets
            ));
        });

        // Show scan recency
        let age = (chrono::Utc::now() - sensory.last_scan).num_seconds();
        ui.label(
            RichText::new(format!("  Last scan: {age}s ago"))
                .color(Color32::from_rgb(156, 163, 175)),
        ); // gray-400
    }

    /// Render awareness section
    fn render_awareness_section(&self, ui: &mut Ui, awareness: &SelfAwarenessData) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🧠 Awareness:").strong());
            ui.label(format!("Knows about {} primals", awareness.knows_about));
        });

        ui.horizontal(|ui| {
            ui.label("  Core Systems:");
            if awareness.has_security {
                ui.label(RichText::new("✅ Security").color(Color32::from_rgb(34, 197, 94)));
            }
            if awareness.has_discovery {
                ui.label(RichText::new("✅ Discovery").color(Color32::from_rgb(34, 197, 94)));
            }
            if awareness.has_compute {
                ui.label(RichText::new("✅ Compute").color(Color32::from_rgb(34, 197, 94)));
            }
        });

        if awareness.can_coordinate {
            ui.label(
                RichText::new("  ✅ Can coordinate multiple primals")
                    .color(Color32::from_rgb(34, 197, 94)),
            );
        }
    }

    /// Render motor section
    fn render_motor_section(&self, ui: &mut Ui, motor: &MotorData) {
        ui.label(RichText::new("💪 Motor:").strong());

        ui.horizontal(|ui| {
            ui.label("  Capabilities:");

            if motor.can_deploy {
                ui.label(RichText::new("✅ Deploy").color(Color32::from_rgb(34, 197, 94)));
            }
            if motor.can_execute_graphs {
                ui.label(RichText::new("✅ Execute").color(Color32::from_rgb(34, 197, 94)));
            }
            if motor.can_coordinate_primals {
                ui.label(RichText::new("✅ Coordinate").color(Color32::from_rgb(34, 197, 94)));
            }
        });
    }

    /// Render evaluative section
    fn render_evaluative_section(&self, ui: &mut Ui, data: &ProprioceptionData) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚖️ Evaluative:").strong());

            let status_text = if data.is_healthy() && data.is_confident() {
                "System is healthy and confident"
            } else if data.is_healthy() {
                "System is healthy but low confidence"
            } else if data.is_confident() {
                "System is confident but degraded"
            } else {
                "System requires attention"
            };

            let color = if data.is_healthy() && data.is_confident() {
                Color32::from_rgb(34, 197, 94) // green-500
            } else {
                Color32::from_rgb(234, 179, 8) // yellow-500
            };

            ui.label(RichText::new(status_text).color(color));
        });
    }

    /// Render timestamp and freshness indicator
    fn render_timestamp(&self, ui: &mut Ui, data: &ProprioceptionData) {
        let age_secs = data.age().num_seconds();
        let age_text = if age_secs < 60 {
            format!("{age_secs}s ago")
        } else {
            format!("{}m ago", age_secs / 60)
        };

        let color = if data.is_stale() {
            Color32::from_rgb(239, 68, 68) // red-500 (stale)
        } else {
            Color32::from_rgb(156, 163, 175) // gray-400 (fresh)
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new("Last updated:").color(color));
            ui.label(RichText::new(age_text).color(color));

            if data.is_stale() {
                ui.label(RichText::new("⚠️ Stale data").color(Color32::from_rgb(239, 68, 68)));
            }
        });

        // Show next refresh countdown
        let next_refresh = REFRESH_INTERVAL
            .as_secs()
            .saturating_sub(self.last_update.elapsed().as_secs());
        if next_refresh > 0 {
            ui.label(
                RichText::new(format!("Next refresh in {next_refresh}s"))
                    .color(Color32::from_rgb(156, 163, 175)),
            ); // gray-400
        } else if self.fetching {
            ui.label(RichText::new("Fetching...").color(Color32::from_rgb(59, 130, 246))); // blue-500
        }
    }

    fn render_motor_status(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Motor Status").strong().size(14.0));
            ui.separator();

            ui.horizontal(|ui| {
                ui.label(RichText::new("Mode:").strong());
                ui.label(&self.current_mode);
            });

            if let Some(ref domain) = self.session_domain {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Domain:").strong());
                    ui.label(domain);
                });
            }

            if self.motor_history.is_empty() {
                ui.label(
                    RichText::new("No motor commands received")
                        .color(Color32::from_rgb(156, 163, 175)),
                );
            } else {
                ui.label(RichText::new("Recent Commands").strong());
                let display_count = self.motor_history.len().min(8);
                for entry in self.motor_history.iter().rev().take(display_count) {
                    let age = entry.timestamp.elapsed().as_secs();
                    let age_text = if age < 60 {
                        format!("{age}s ago")
                    } else {
                        format!("{}m ago", age / 60)
                    };
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&entry.command).color(Color32::from_rgb(156, 163, 175)),
                        );
                        ui.label(
                            RichText::new(age_text)
                                .color(Color32::from_rgb(107, 114, 128))
                                .small(),
                        );
                    });
                }
            }
        });
    }
}

impl Default for ProprioceptionPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::ProprioceptionData;

    #[test]
    fn test_new_panel() {
        let panel = ProprioceptionPanel::new();
        assert!(panel.data.is_none());
        assert!(!panel.fetching);
    }

    #[test]
    fn test_panel_with_healthy_data() {
        let mut data = ProprioceptionData::empty("test");
        data.health.percentage = 95.0;
        data.confidence = 90.0;

        let mut panel = ProprioceptionPanel::new();
        panel.data = Some(data);

        assert!(panel.data.is_some());
        assert!(panel.data.as_ref().unwrap().is_healthy());
    }

    #[test]
    fn test_motor_history_recording() {
        let mut panel = ProprioceptionPanel::new();
        panel.record_motor_command("SetMode(clinical)");
        panel.record_motor_command("FitToView");
        assert_eq!(panel.motor_history.len(), 2);
    }

    #[test]
    fn test_motor_history_max_entries() {
        let mut panel = ProprioceptionPanel::new();
        for i in 0..25 {
            panel.record_motor_command(&format!("Command {i}"));
        }
        assert_eq!(panel.motor_history.len(), 20);
    }

    #[test]
    fn test_current_mode() {
        let mut panel = ProprioceptionPanel::new();
        assert_eq!(panel.current_mode, "default");
        panel.set_current_mode("clinical");
        assert_eq!(panel.current_mode, "clinical");
    }

    #[test]
    fn test_session_domain() {
        let mut panel = ProprioceptionPanel::new();
        assert!(panel.session_domain.is_none());
        panel.set_session_domain(Some("health".to_string()));
        assert_eq!(panel.session_domain.as_deref(), Some("health"));
    }
}
