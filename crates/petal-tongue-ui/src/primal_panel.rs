// SPDX-License-Identifier: AGPL-3.0-only
//! Primal Panel - Primal Status Management UI
//!
//! Displays all discovered primals with their health, capabilities, load, and device assignments.
//! Provides drag-and-drop drop zones for device assignment.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ PrimalPanel                                                 │
//! │  ├─ Filter Bar (All/Healthy/Degraded/Error)                 │
//! │  ├─ Search Box                                              │
//! │  └─ Primal List                                             │
//! │      ├─ PrimalCard (security-primal) [drop zone]            │
//! │      │   ├─ Health: Healthy                                 │
//! │      │   ├─ Load: 45%                                       │
//! │      │   ├─ Capabilities: 5                                 │
//! │      │   └─ Devices: GPU-0, CPU-1                           │
//! │      ├─ PrimalCard (discovery-primal) [drop zone]           │
//! │      └─ PrimalCard (compute-primal) [drop zone]             │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::biomeos_integration::{Health, Primal};
use crate::ui_events::{UIEvent, UIEventHandler};
use egui::{Color32, RichText, Ui};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Primal filter options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimalFilter {
    /// Show all primals
    All,
    /// Show only healthy primals
    Healthy,
    /// Show only degraded primals
    Degraded,
}

/// Primal panel - main UI component for primal status management
pub struct PrimalPanel {
    /// All primals (updated from provider)
    primals: Vec<Primal>,
    /// Selected primal ID
    selected: Option<String>,
    /// Current filter
    filter: PrimalFilter,
    /// Search query
    search_query: String,
    /// Event handler for real-time updates
    event_handler: Arc<RwLock<UIEventHandler>>,
    /// Last refresh time
    last_refresh: std::time::Instant,
}

impl PrimalPanel {
    /// Create a new primal panel
    #[must_use]
    pub fn new(event_handler: Arc<RwLock<UIEventHandler>>) -> Self {
        info!("🌸 Creating primal panel");

        Self {
            primals: Vec::new(),
            selected: None,
            filter: PrimalFilter::All,
            search_query: String::new(),
            event_handler,
            last_refresh: std::time::Instant::now(),
        }
    }

    /// Update primals from provider
    #[expect(clippy::unused_async, reason = "async for trait compatibility")]
    pub async fn refresh(&mut self, primals: Vec<Primal>) {
        debug!("🔄 Refreshing primal panel with {} primals", primals.len());
        self.primals = primals;
        self.last_refresh = std::time::Instant::now();
    }

    /// Process pending events
    pub async fn process_events(&mut self) {
        let events = self
            .event_handler
            .write()
            .await
            .consume_primal_panel_events()
            .await;

        for event in events {
            match event {
                UIEvent::PrimalDiscovered(primal) => {
                    info!("📥 Primal discovered: {}", primal.name);
                    self.primals.push(primal);
                }
                UIEvent::PrimalRemoved(primal_id) => {
                    info!("📤 Primal removed: {}", primal_id);
                    self.primals.retain(|p| p.id != primal_id);
                    if self.selected.as_ref() == Some(&primal_id) {
                        self.selected = None;
                    }
                }
                UIEvent::PrimalHealthChanged(primal_id, new_health) => {
                    if let Some(primal) = self.primals.iter_mut().find(|p| p.id == primal_id) {
                        debug!("🔄 Primal {} health changed to {:?}", primal_id, new_health);
                        primal.health = new_health;
                    }
                }
                UIEvent::PrimalLoadChanged(primal_id, new_load) => {
                    if let Some(primal) = self.primals.iter_mut().find(|p| p.id == primal_id) {
                        primal.load = new_load;
                    }
                }
                _ => {} // Other events not relevant to primal panel
            }
        }
    }

    /// Render the primal panel
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("🌸 Primals");
        ui.separator();

        // Filter bar
        self.render_filter_bar(ui);
        ui.add_space(8.0);

        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_query);
        });
        ui.add_space(8.0);

        // Stats
        self.render_stats(ui);
        ui.add_space(8.0);

        // Primal list
        egui::ScrollArea::vertical()
            .id_salt("primal_list")
            .show(ui, |ui| {
                // Clone primals to avoid borrow checker issues with mutable UI rendering
                let filtered_primals: Vec<Primal> =
                    self.filtered_primals().into_iter().cloned().collect();

                if filtered_primals.is_empty() {
                    ui.colored_label(Color32::GRAY, "No primals found");
                } else {
                    for primal in &filtered_primals {
                        self.render_primal_card(ui, primal);
                    }
                }
            });
    }

    /// Render filter bar
    fn render_filter_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");

            if ui
                .selectable_label(self.filter == PrimalFilter::All, "All")
                .clicked()
            {
                self.filter = PrimalFilter::All;
            }

            if ui
                .selectable_label(self.filter == PrimalFilter::Healthy, "Healthy")
                .clicked()
            {
                self.filter = PrimalFilter::Healthy;
            }

            if ui
                .selectable_label(self.filter == PrimalFilter::Degraded, "Degraded")
                .clicked()
            {
                self.filter = PrimalFilter::Degraded;
            }

            if ui
                .selectable_label(self.filter == PrimalFilter::Degraded, "Error")
                .clicked()
            {
                self.filter = PrimalFilter::Degraded;
            }
        });
    }

    /// Render stats
    fn render_stats(&self, ui: &mut Ui) {
        let total = self.primals.len();
        let healthy = self
            .primals
            .iter()
            .filter(|p| p.health == Health::Healthy)
            .count();
        let degraded = self
            .primals
            .iter()
            .filter(|p| p.health == Health::Degraded)
            .count();
        let error = self
            .primals
            .iter()
            .filter(|p| p.health == Health::Offline)
            .count();

        ui.horizontal(|ui| {
            ui.label(format!("Total: {total}"));
            ui.separator();
            ui.colored_label(Color32::GREEN, format!("Healthy: {healthy}"));
            ui.separator();
            ui.colored_label(Color32::YELLOW, format!("Degraded: {degraded}"));
            ui.separator();
            ui.colored_label(Color32::RED, format!("Error: {error}"));
        });
    }

    /// Render individual primal card
    fn render_primal_card(&mut self, ui: &mut Ui, primal: &Primal) {
        let is_selected = self.selected.as_ref() == Some(&primal.id);

        let response = egui::Frame::none()
            .fill(if is_selected {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().faint_bg_color
            })
            .inner_margin(egui::Margin::same(8.0))
            .rounding(4.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Primal header
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&primal.name).strong().size(16.0));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Health indicator
                            let (color, text) = match primal.health {
                                Health::Healthy => (Color32::GREEN, "● Healthy"),
                                Health::Degraded => (Color32::YELLOW, "● Degraded"),
                                Health::Offline => (Color32::RED, "● Offline"),
                            };
                            ui.colored_label(color, text);
                        });
                    });

                    ui.add_space(4.0);

                    // Primal stats
                    ui.horizontal(|ui| {
                        // Load bar
                        ui.label("Load:");
                        let load = primal.load;
                        let bar_color = if load > 0.9 {
                            Color32::RED
                        } else if load > 0.7 {
                            Color32::YELLOW
                        } else {
                            Color32::GREEN
                        };

                        ui.add(
                            egui::ProgressBar::new(load as f32)
                                .fill(bar_color)
                                .show_percentage(),
                        );
                    });

                    ui.add_space(4.0);

                    // Capabilities
                    ui.horizontal(|ui| {
                        ui.label("Capabilities:");
                        ui.label(format!("{}", primal.capabilities.len()));
                    });

                    ui.add_space(4.0);

                    // Assigned devices
                    ui.horizontal(|ui| {
                        ui.label("Devices:");
                        if primal.assigned_devices.is_empty() {
                            ui.colored_label(Color32::GRAY, "None");
                        } else {
                            ui.label(format!("{}", primal.assigned_devices.len()));
                        }
                    });
                });
            })
            .response;

        // Selection
        if response.clicked() {
            self.selected = if is_selected {
                None
            } else {
                Some(primal.id.clone())
            };
        }

        // Drop zone for device assignment
        let is_dragging_device = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new("dragged_device"))
                .is_some()
        });

        if is_dragging_device {
            // Check if hovering over this primal
            if response.hovered() {
                // Highlight as drop zone
                let highlight_rect = response.rect.expand(2.0);
                ui.painter()
                    .rect_stroke(highlight_rect, 4.0, (2.0, Color32::LIGHT_BLUE));

                // Show drop hint
                response.on_hover_text(format!("Drop device here to assign to {}", primal.name));

                // Handle drop
                if !ui.input(|i| i.pointer.is_decidedly_dragging()) {
                    // Drag ended, check if we can get the device ID
                    if let Some(device_id) = ui.memory_mut(|mem| {
                        mem.data
                            .remove_temp::<String>(egui::Id::new("dragged_device"))
                    }) {
                        info!("🎯 Device {} dropped on primal {}", device_id, primal.id);

                        // Send assignment event
                        let primal_id = primal.id.clone();
                        let event_handler = self.event_handler.clone();
                        tokio::spawn(async move {
                            event_handler
                                .write()
                                .await
                                .handle_event(UIEvent::DeviceAssigned(device_id, primal_id))
                                .await;
                        });
                    }
                }
            }
        }

        ui.add_space(4.0);
    }

    /// Get filtered primals based on current filter and search
    fn filtered_primals(&self) -> Vec<&Primal> {
        self.primals
            .iter()
            .filter(|primal| {
                // Apply filter
                let filter_match = match self.filter {
                    PrimalFilter::All => true,
                    PrimalFilter::Healthy => primal.health == Health::Healthy,
                    PrimalFilter::Degraded => primal.health == Health::Degraded,
                };

                // Apply search
                let search_match = if self.search_query.is_empty() {
                    true
                } else {
                    let query = self.search_query.to_lowercase();
                    primal.name.to_lowercase().contains(&query)
                        || primal.id.to_lowercase().contains(&query)
                        || primal
                            .capabilities
                            .iter()
                            .any(|c| c.to_lowercase().contains(&query))
                };

                filter_match && search_match
            })
            .collect()
    }

    /// Get selected primal
    #[must_use]
    pub fn selected_primal(&self) -> Option<&Primal> {
        self.selected
            .as_ref()
            .and_then(|id| self.primals.iter().find(|p| &p.id == id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biomeos_integration::Health;

    #[tokio::test]
    async fn test_primal_panel_creation() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let panel = PrimalPanel::new(event_handler);

        assert_eq!(panel.primals.len(), 0);
        assert_eq!(panel.filter, PrimalFilter::All);
        assert!(panel.search_query.is_empty());
    }

    #[tokio::test]
    async fn test_primal_panel_refresh() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler);

        let primals = vec![
            Primal {
                id: "primal-1".to_string(),
                name: "Test Primal 1".to_string(),
                health: Health::Healthy,
                capabilities: vec!["compute".to_string()],
                load: 0.5,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
            Primal {
                id: "primal-2".to_string(),
                name: "Test Primal 2".to_string(),
                health: Health::Healthy,
                capabilities: vec!["storage".to_string()],
                load: 0.3,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
        ];

        panel.refresh(primals).await;

        assert_eq!(panel.primals.len(), 2);
    }

    #[tokio::test]
    async fn test_primal_panel_event_processing() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler.clone());

        // Add initial primal
        panel
            .refresh(vec![Primal {
                id: "primal-1".to_string(),
                name: "Test Primal".to_string(),
                health: Health::Healthy,
                capabilities: vec!["compute".to_string()],
                load: 0.5,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            }])
            .await;

        // Send health change event
        event_handler
            .write()
            .await
            .handle_event(UIEvent::PrimalHealthChanged(
                "primal-1".to_string(),
                Health::Degraded,
            ))
            .await;

        // Process events
        panel.process_events().await;

        // Check health was updated
        assert_eq!(panel.primals[0].health, Health::Degraded);
    }

    #[tokio::test]
    async fn test_primal_panel_filtering() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler);

        let primals = vec![
            Primal {
                id: "primal-1".to_string(),
                name: "Healthy Primal".to_string(),
                health: Health::Healthy,
                capabilities: vec!["compute".to_string()],
                load: 0.5,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
            Primal {
                id: "primal-2".to_string(),
                name: "Degraded Primal".to_string(),
                health: Health::Degraded,
                capabilities: vec!["storage".to_string()],
                load: 0.8,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
            Primal {
                id: "primal-3".to_string(),
                name: "Offline Primal".to_string(),
                health: Health::Offline,
                capabilities: vec!["network".to_string()],
                load: 0.0,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
        ];

        panel.refresh(primals).await;

        // Test All filter
        panel.filter = PrimalFilter::All;
        assert_eq!(panel.filtered_primals().len(), 3);

        // Test Healthy filter
        panel.filter = PrimalFilter::Healthy;
        assert_eq!(panel.filtered_primals().len(), 1);
        assert_eq!(panel.filtered_primals()[0].name, "Healthy Primal");

        // Test Degraded filter (matches only Degraded health)
        panel.filter = PrimalFilter::Degraded;
        assert_eq!(panel.filtered_primals().len(), 1);
        assert_eq!(panel.filtered_primals()[0].name, "Degraded Primal");
    }

    #[tokio::test]
    async fn test_primal_panel_search() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler);

        let primals = vec![
            Primal {
                id: "primal-1".to_string(),
                name: "Beardog".to_string(),
                health: Health::Healthy,
                capabilities: vec!["compute".to_string()],
                load: 0.5,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
            Primal {
                id: "primal-2".to_string(),
                name: "Songbird".to_string(),
                health: Health::Healthy,
                capabilities: vec!["orchestration".to_string()],
                load: 0.3,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
        ];

        panel.refresh(primals).await;

        // Search for "bear"
        panel.search_query = "bear".to_string();
        assert_eq!(panel.filtered_primals().len(), 1);
        assert_eq!(panel.filtered_primals()[0].name, "Beardog");

        // Search for "orchestration" (capability)
        panel.search_query = "orchestration".to_string();
        assert_eq!(panel.filtered_primals().len(), 1);
        assert_eq!(panel.filtered_primals()[0].name, "Songbird");

        // Empty search
        panel.search_query = String::new();
        assert_eq!(panel.filtered_primals().len(), 2);
    }

    #[tokio::test]
    async fn test_primal_panel_selection() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler);

        let primals = vec![Primal {
            id: "primal-1".to_string(),
            name: "Test Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        }];

        panel.refresh(primals).await;

        // No selection initially
        assert!(panel.selected_primal().is_none());

        // Select primal
        panel.selected = Some("primal-1".to_string());
        assert!(panel.selected_primal().is_some());
        assert_eq!(
            panel.selected_primal().expect("selected").name,
            "Test Primal"
        );
    }

    #[tokio::test]
    async fn test_primal_panel_search_by_id() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler);

        let primals = vec![
            Primal {
                id: "primal-abc-123".to_string(),
                name: "Alpha".to_string(),
                health: Health::Healthy,
                capabilities: vec![],
                load: 0.0,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
            Primal {
                id: "primal-xyz-456".to_string(),
                name: "Beta".to_string(),
                health: Health::Healthy,
                capabilities: vec![],
                load: 0.0,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
        ];
        panel.refresh(primals).await;

        panel.search_query = "xyz".to_string();
        let filtered = panel.filtered_primals();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "primal-xyz-456");
    }

    #[tokio::test]
    async fn test_primal_panel_primal_removed_clears_selection() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler.clone());

        panel
            .refresh(vec![Primal {
                id: "p1".to_string(),
                name: "P1".to_string(),
                health: Health::Healthy,
                capabilities: vec![],
                load: 0.0,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            }])
            .await;
        panel.selected = Some("p1".to_string());

        event_handler
            .write()
            .await
            .handle_event(UIEvent::PrimalRemoved("p1".to_string()))
            .await;
        panel.process_events().await;

        assert!(panel.selected.is_none());
        assert!(panel.primals.is_empty());
    }

    #[tokio::test]
    async fn test_primal_panel_load_changed() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler.clone());

        panel
            .refresh(vec![Primal {
                id: "p1".to_string(),
                name: "P1".to_string(),
                health: Health::Healthy,
                capabilities: vec![],
                load: 0.3,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            }])
            .await;

        event_handler
            .write()
            .await
            .handle_event(UIEvent::PrimalLoadChanged("p1".to_string(), 0.9))
            .await;
        panel.process_events().await;

        assert!((panel.primals[0].load - 0.9).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_primal_panel_primal_discovered() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler.clone());

        event_handler
            .write()
            .await
            .handle_event(UIEvent::PrimalDiscovered(Primal {
                id: "new".to_string(),
                name: "New Primal".to_string(),
                health: Health::Healthy,
                capabilities: vec!["compute".to_string()],
                load: 0.0,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            }))
            .await;
        panel.process_events().await;

        assert_eq!(panel.primals.len(), 1);
        assert_eq!(panel.primals[0].name, "New Primal");
    }

    #[tokio::test]
    async fn test_primal_filter_all_includes_offline() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = PrimalPanel::new(event_handler);

        let primals = vec![
            Primal {
                id: "h".to_string(),
                name: "Healthy".to_string(),
                health: Health::Healthy,
                capabilities: vec![],
                load: 0.0,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
            Primal {
                id: "o".to_string(),
                name: "Offline".to_string(),
                health: Health::Offline,
                capabilities: vec![],
                load: 0.0,
                assigned_devices: vec![],
                metadata: serde_json::json!({}),
            },
        ];
        panel.refresh(primals).await;
        panel.filter = PrimalFilter::All;

        assert_eq!(panel.filtered_primals().len(), 2);
    }
}
