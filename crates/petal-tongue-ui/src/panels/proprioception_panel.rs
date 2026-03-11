// SPDX-License-Identifier: AGPL-3.0-only
//! Proprioception Panel - Display SAME DAVE self-awareness
//!
//! Phase 1.4: Shows the system's self-knowledge using the SAME DAVE framework:
//! - Sensory: What the system perceives (active sockets)
//! - Awareness: What the system knows (primals, capabilities)
//! - Motor: What the system can do (deploy, execute, coordinate)
//! - Evaluative: How confident the system is (health, confidence)

use crate::panel_registry::{PanelFactory, PanelInstance};
use crate::proprioception_panel::{render_shared_health, render_shared_same_dave};
use crate::scenario::CustomPanelConfig;
use petal_tongue_core::proprioception::ProprioceptionData;
use petal_tongue_discovery::NeuralApiProvider;
use std::time::{Duration, Instant};

/// Panel that displays SAME DAVE proprioception
pub struct ProprioceptionPanel {
    provider: Option<NeuralApiProvider>,
    last_proprio: Option<ProprioceptionData>,
    last_update: Instant,
    update_interval: Duration,
    error_message: Option<String>,
}

impl ProprioceptionPanel {
    /// Create a new proprioception panel (provider connected later)
    #[must_use]
    pub fn new() -> Self {
        Self {
            provider: None,
            last_proprio: None,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(5), // Update every 5 seconds
            error_message: None,
        }
    }
}

impl PanelInstance for ProprioceptionPanel {
    fn title(&self) -> &'static str {
        "System Health"
    }

    fn on_open(&mut self) -> anyhow::Result<()> {
        tracing::info!("Proprioception Panel opened - discovering Neural API");

        // Discover Neural API in blocking context
        let provider = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { NeuralApiProvider::discover(None).await.ok() })
        });

        if provider.is_some() {
            tracing::info!("✅ Neural API discovered for proprioception panel");
        } else {
            tracing::warn!("⚠️  Neural API not available - proprioception will not update");
            self.error_message = Some("Neural API not available".to_string());
        }

        self.provider = provider;
        Ok(())
    }

    fn on_close(&mut self) -> anyhow::Result<()> {
        tracing::info!("Proprioception Panel closed");
        Ok(())
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        if self.last_update.elapsed() > self.update_interval && self.provider.is_some() {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if let Some(provider) = &self.provider {
                        match provider.get_proprioception().await {
                            Ok(proprio) => {
                                self.last_proprio = Some(proprio);
                                self.last_update = Instant::now();
                                self.error_message = None;
                            }
                            Err(e) => {
                                self.error_message = Some(format!("API error: {e}"));
                                tracing::warn!("Failed to get proprioception: {}", e);
                            }
                        }
                    }
                });
            });
        }

        ui.heading("🧠 NUCLEUS Proprioception");
        ui.separator();

        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, format!("⚠️  {error}"));
            ui.separator();
        }

        if let Some(proprio) = &self.last_proprio {
            render_shared_health(ui, &proprio.health);
            ui.add_space(4.0);
            render_shared_same_dave(ui, proprio);

            ui.add_space(2.0);
            let age = self.last_update.elapsed().as_secs();
            if age < 10 {
                ui.label(format!("Updated {age}s ago"));
            } else {
                ui.colored_label(egui::Color32::YELLOW, format!("Updated {age}s ago"));
            }
        } else if self.provider.is_none() {
            ui.label("Neural API not available");
            ui.label("Start NUCLEUS to see proprioception:");
            ui.code("nucleus serve --family nat0");
        } else {
            ui.label("Connecting to Neural API...");
        }
    }

    fn wants_keyboard_input(&self) -> bool {
        false
    }

    fn wants_mouse_input(&self) -> bool {
        false
    }
}

impl Default for ProprioceptionPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating Proprioception panels
pub struct ProprioceptionPanelFactory;

impl ProprioceptionPanelFactory {
    /// Create a new factory for proprioception (self-awareness) panels
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl PanelFactory for ProprioceptionPanelFactory {
    fn panel_type(&self) -> &'static str {
        "proprioception"
    }

    fn create(
        &self,
        _config: &CustomPanelConfig,
    ) -> crate::panel_registry::Result<Box<dyn PanelInstance>> {
        Ok(Box::new(ProprioceptionPanel::new()))
    }

    fn description(&self) -> &'static str {
        "Displays SAME DAVE proprioception - the system's self-awareness"
    }
}

impl Default for ProprioceptionPanelFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::proprioception::HealthStatus;

    #[test]
    fn test_proprioception_panel_creation() {
        let panel = ProprioceptionPanel::new();
        assert_eq!(panel.title(), "System Health");
        assert!(!panel.wants_keyboard_input());
        assert!(!panel.wants_mouse_input());
    }

    #[test]
    fn test_proprioception_panel_factory() {
        let factory = ProprioceptionPanelFactory::new();
        assert_eq!(factory.panel_type(), "proprioception");
        assert_eq!(
            factory.description(),
            "Displays SAME DAVE proprioception - the system's self-awareness"
        );

        let config = CustomPanelConfig {
            panel_type: "proprioception".to_string(),
            title: "Test Proprio".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::json!({}),
        };
        let panel = factory.create(&config);
        assert!(panel.is_ok());
    }

    #[test]
    fn test_health_status_emoji() {
        assert_eq!(HealthStatus::Healthy.emoji(), "💚");
        assert_eq!(HealthStatus::Degraded.emoji(), "💛");
        assert_eq!(HealthStatus::Critical.emoji(), "❤️");
    }

    #[test]
    fn test_health_status_color() {
        assert_eq!(HealthStatus::Healthy.color_rgb(), (34, 197, 94));
        assert_eq!(HealthStatus::Degraded.color_rgb(), (234, 179, 8));
        assert_eq!(HealthStatus::Critical.color_rgb(), (239, 68, 68));
    }
}
