//! Niche Designer - Visual Niche Editor & Deployment UI
//!
//! Provides a visual interface for designing, validating, and deploying niches.
//! Supports templates, drag-and-drop primal assignment, requirement validation,
//! and AI suggestions.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ NicheDesigner                                               │
//! │  ├─ Template Selector (dropdown)                            │
//! │  ├─ Canvas (visual niche representation)                    │
//! │  │   ├─ Primal Slots (required/optional)                    │
//! │  │   └─ Drop zones for primals                              │
//! │  ├─ Validation Panel                                        │
//! │  │   ├─ Requirements check                                  │
//! │  │   ├─ Resource estimation                                 │
//! │  │   └─ Warnings/Errors                                     │
//! │  └─ Deploy Button                                           │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::biomeos_integration::{NicheTemplate, Primal};
use crate::ui_events::{UIEvent, UIEventHandler};
use egui::{Color32, RichText, Ui};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Validation result for niche design
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// All requirements met
    Valid,
    /// Missing required primals
    MissingRequirements(Vec<String>),
    /// Resource constraints not met
    InsufficientResources(String),
    /// Configuration conflicts
    Conflicts(Vec<String>),
}

/// Niche designer state
pub struct NicheDesigner {
    /// Available templates
    templates: Vec<NicheTemplate>,
    /// Selected template
    selected_template: Option<NicheTemplate>,
    /// Available primals (for assignment)
    available_primals: Vec<Primal>,
    /// Assigned primals (capability -> primal_id)
    assigned_primals: std::collections::HashMap<String, String>,
    /// Validation result
    validation: ValidationResult,
    /// Event handler for real-time updates
    event_handler: Arc<RwLock<UIEventHandler>>,
    /// Last refresh time
    last_refresh: std::time::Instant,
}

impl NicheDesigner {
    /// Create a new niche designer
    #[must_use]
    pub fn new(event_handler: Arc<RwLock<UIEventHandler>>) -> Self {
        info!("🎨 Creating niche designer");

        Self {
            templates: Vec::new(),
            selected_template: None,
            available_primals: Vec::new(),
            assigned_primals: std::collections::HashMap::new(),
            validation: ValidationResult::Valid,
            event_handler,
            last_refresh: std::time::Instant::now(),
        }
    }

    /// Update templates and primals
    pub async fn refresh(&mut self, templates: Vec<NicheTemplate>, primals: Vec<Primal>) {
        debug!(
            "🔄 Refreshing niche designer with {} templates, {} primals",
            templates.len(),
            primals.len()
        );
        self.templates = templates;
        self.available_primals = primals;
        self.last_refresh = std::time::Instant::now();

        // Re-validate if template is selected
        if self.selected_template.is_some() {
            self.validate();
        }
    }

    /// Process pending events
    pub async fn process_events(&mut self) {
        let events = self
            .event_handler
            .write()
            .await
            .consume_niche_designer_events()
            .await;

        for event in events {
            match event {
                UIEvent::NicheDeployed(niche_id, template) => {
                    info!("🚀 Niche deployed: {} ({})", niche_id, template.name);
                }
                UIEvent::NicheRemoved(niche_id) => {
                    info!("🗑️ Niche removed: {}", niche_id);
                }
                UIEvent::AISuggestion(suggestion) => {
                    info!(
                        "💡 AI suggestion received: {} (confidence: {})",
                        suggestion.id, suggestion.confidence
                    );
                }
                _ => {} // Other events not relevant to niche designer
            }
        }
    }

    /// Render the niche designer
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("🎨 Niche Designer");
        ui.separator();

        // Template selector
        self.render_template_selector(ui);
        ui.add_space(8.0);

        if let Some(template) = &self.selected_template.clone() {
            // Canvas (visual representation)
            self.render_canvas(ui, template);
            ui.add_space(8.0);

            // Validation panel
            self.render_validation_panel(ui);
            ui.add_space(8.0);

            // Deploy button
            self.render_deploy_button(ui);
        } else {
            ui.colored_label(Color32::GRAY, "Select a template to start designing");
        }
    }

    /// Render template selector
    fn render_template_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Template:");

            let selected_name = self
                .selected_template
                .as_ref()
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "Select a template...".to_string());

            egui::ComboBox::from_id_source("template_selector")
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    for template in &self.templates.clone() {
                        let is_selected = self
                            .selected_template
                            .as_ref()
                            .map(|t| t.id == template.id)
                            .unwrap_or(false);

                        if ui.selectable_label(is_selected, &template.name).clicked() {
                            self.select_template(template.clone());
                        }
                    }
                });
        });
    }

    /// Render canvas (visual niche representation)
    fn render_canvas(&mut self, ui: &mut Ui, template: &NicheTemplate) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(&template.name).strong().size(16.0));
                ui.label(&template.description);
                ui.add_space(8.0);

                // Required primals
                ui.label(RichText::new("Required Primals:").strong());
                for capability in &template.required_primals {
                    self.render_primal_slot(ui, capability, true);
                }

                ui.add_space(8.0);

                // Optional primals
                if !template.optional_primals.is_empty() {
                    ui.label(RichText::new("Optional Primals:").strong());
                    for capability in &template.optional_primals {
                        self.render_primal_slot(ui, capability, false);
                    }
                }
            });
        });
    }

    /// Render a primal slot (drop zone)
    fn render_primal_slot(&mut self, ui: &mut Ui, capability: &str, required: bool) {
        let assigned_primal_id = self.assigned_primals.get(capability).cloned();
        let assigned_primal = assigned_primal_id
            .and_then(|id| self.available_primals.iter().find(|p| p.id == id))
            .cloned();

        let slot_response = ui
            .group(|ui| {
                ui.horizontal(|ui| {
                    // Capability label
                    ui.label(format!("{}:", capability));

                    // Assignment status
                    if let Some(primal) = &assigned_primal {
                        ui.colored_label(Color32::GREEN, format!("✓ {}", primal.name));

                        // Unassign button
                        if ui.small_button("✖").clicked() {
                            self.unassign_primal(capability);
                        }
                    } else {
                        let text = if required {
                            RichText::new("Drop primal here (required)").color(Color32::RED)
                        } else {
                            RichText::new("Drop primal here (optional)").color(Color32::GRAY)
                        };
                        ui.label(text);
                    }
                });
            })
            .response;

        // Check for dragged primal
        let dragged_primal_id =
            ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_primal")));

        if let Some(primal_id) = dragged_primal_id {
            if slot_response.hovered() {
                // Highlight as drop zone
                let highlight_rect = slot_response.rect.expand(2.0);
                ui.painter()
                    .rect_stroke(highlight_rect, 4.0, (2.0, Color32::LIGHT_BLUE));

                slot_response.clone().on_hover_text(format!(
                    "Drop primal here to assign to {} capability",
                    capability
                ));

                // Handle drop
                if !ui.input(|i| i.pointer.is_decidedly_dragging()) {
                    // Drag ended
                    if let Some(primal_id_final) = ui.memory_mut(|mem| {
                        mem.data
                            .remove_temp::<String>(egui::Id::new("dragged_primal"))
                    }) {
                        info!(
                            "🎯 Primal {} dropped on capability {}",
                            primal_id_final, capability
                        );
                        self.assign_primal(capability.to_string(), primal_id_final);
                    }
                }
            }
        }
    }

    /// Render validation panel
    fn render_validation_panel(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Validation").strong());
                ui.separator();

                match &self.validation {
                    ValidationResult::Valid => {
                        ui.colored_label(Color32::GREEN, "✓ All requirements met");
                    }
                    ValidationResult::MissingRequirements(missing) => {
                        ui.colored_label(
                            Color32::RED,
                            format!("✖ Missing required capabilities: {}", missing.join(", ")),
                        );
                    }
                    ValidationResult::InsufficientResources(msg) => {
                        ui.colored_label(Color32::YELLOW, format!("⚠ {}", msg));
                    }
                    ValidationResult::Conflicts(conflicts) => {
                        ui.colored_label(Color32::RED, "✖ Conflicts detected:");
                        for conflict in conflicts {
                            ui.label(format!("  • {}", conflict));
                        }
                    }
                }
            });
        });
    }

    /// Render deploy button
    fn render_deploy_button(&self, ui: &mut Ui) {
        let can_deploy = self.validation == ValidationResult::Valid;

        let button = egui::Button::new(RichText::new("🚀 Deploy Niche").strong().size(16.0).color(
            if can_deploy {
                Color32::WHITE
            } else {
                Color32::GRAY
            },
        ))
        .fill(if can_deploy {
            Color32::from_rgb(0, 120, 0)
        } else {
            Color32::from_gray(60)
        });

        if ui.add_enabled(can_deploy, button).clicked() {
            self.deploy_niche();
        }

        if !can_deploy {
            ui.colored_label(Color32::GRAY, "Complete all required assignments to deploy");
        }
    }

    /// Select a template
    fn select_template(&mut self, template: NicheTemplate) {
        info!("📋 Selected template: {}", template.name);
        self.selected_template = Some(template);
        self.assigned_primals.clear();
        self.validate();
    }

    /// Assign a primal to a capability
    fn assign_primal(&mut self, capability: String, primal_id: String) {
        // Check if primal has the required capability
        if let Some(primal) = self.available_primals.iter().find(|p| p.id == primal_id) {
            if primal.capabilities.contains(&capability) {
                info!("✓ Assigned {} to {}", primal.name, capability);
                self.assigned_primals.insert(capability, primal_id);
                self.validate();
            } else {
                warn!(
                    "⚠ Primal {} does not have capability {}",
                    primal.name, capability
                );
            }
        }
    }

    /// Unassign a primal from a capability
    fn unassign_primal(&mut self, capability: &str) {
        if let Some(primal_id) = self.assigned_primals.remove(capability) {
            info!("✖ Unassigned {} from {}", primal_id, capability);
            self.validate();
        }
    }

    /// Validate current niche design
    fn validate(&mut self) {
        if let Some(template) = &self.selected_template {
            // Check required capabilities
            let missing: Vec<String> = template
                .required_primals
                .iter()
                .filter(|cap| !self.assigned_primals.contains_key(*cap))
                .cloned()
                .collect();

            if !missing.is_empty() {
                self.validation = ValidationResult::MissingRequirements(missing);
            } else {
                // All requirements met
                self.validation = ValidationResult::Valid;
            }
        } else {
            self.validation = ValidationResult::Valid;
        }
    }

    /// Deploy the niche
    fn deploy_niche(&self) {
        if let Some(template) = &self.selected_template {
            info!("🚀 Deploying niche: {}", template.name);

            // Generate niche ID
            let niche_id = format!("niche-{}", uuid::Uuid::new_v4());

            // Send deployment event
            let event_handler = self.event_handler.clone();
            let template_clone = template.clone();
            tokio::spawn(async move {
                event_handler
                    .write()
                    .await
                    .handle_event(UIEvent::NicheDeployed(niche_id, template_clone))
                    .await;
            });
        }
    }

    /// Get validation result
    pub fn validation_result(&self) -> &ValidationResult {
        &self.validation
    }

    /// Get assigned primals
    pub fn assigned_primals(&self) -> &std::collections::HashMap<String, String> {
        &self.assigned_primals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biomeos_integration::Health;

    #[tokio::test]
    async fn test_niche_designer_creation() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let designer = NicheDesigner::new(event_handler);

        assert_eq!(designer.templates.len(), 0);
        assert!(designer.selected_template.is_none());
        assert_eq!(designer.validation, ValidationResult::Valid);
    }

    #[tokio::test]
    async fn test_niche_designer_refresh() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut designer = NicheDesigner::new(event_handler);

        let templates = vec![NicheTemplate {
            id: "template-1".to_string(),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            required_primals: vec!["compute".to_string()],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        }];

        let primals = vec![Primal {
            id: "primal-1".to_string(),
            name: "Test Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        }];

        designer.refresh(templates, primals).await;

        assert_eq!(designer.templates.len(), 1);
        assert_eq!(designer.available_primals.len(), 1);
    }

    #[tokio::test]
    async fn test_template_selection() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut designer = NicheDesigner::new(event_handler);

        let template = NicheTemplate {
            id: "template-1".to_string(),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            required_primals: vec!["compute".to_string()],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        };

        designer.select_template(template.clone());

        assert!(designer.selected_template.is_some());
        assert_eq!(designer.selected_template.unwrap().id, "template-1");
        assert_eq!(designer.assigned_primals.len(), 0);
    }

    #[tokio::test]
    async fn test_primal_assignment() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut designer = NicheDesigner::new(event_handler);

        let template = NicheTemplate {
            id: "template-1".to_string(),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            required_primals: vec!["compute".to_string()],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        };

        let primal = Primal {
            id: "primal-1".to_string(),
            name: "Test Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        };

        designer.refresh(vec![template.clone()], vec![primal]).await;
        designer.select_template(template);
        designer.assign_primal("compute".to_string(), "primal-1".to_string());

        assert_eq!(designer.assigned_primals.len(), 1);
        assert_eq!(
            designer.assigned_primals.get("compute"),
            Some(&"primal-1".to_string())
        );
    }

    #[tokio::test]
    async fn test_validation_missing_requirements() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut designer = NicheDesigner::new(event_handler);

        let template = NicheTemplate {
            id: "template-1".to_string(),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            required_primals: vec!["compute".to_string(), "storage".to_string()],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        };

        designer.select_template(template);

        // No primals assigned - should fail validation
        match designer.validation {
            ValidationResult::MissingRequirements(ref missing) => {
                assert_eq!(missing.len(), 2);
                assert!(missing.contains(&"compute".to_string()));
                assert!(missing.contains(&"storage".to_string()));
            }
            _ => panic!("Expected MissingRequirements validation result"),
        }
    }

    #[tokio::test]
    async fn test_validation_success() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut designer = NicheDesigner::new(event_handler);

        let template = NicheTemplate {
            id: "template-1".to_string(),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            required_primals: vec!["compute".to_string()],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        };

        let primal = Primal {
            id: "primal-1".to_string(),
            name: "Test Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        };

        designer.refresh(vec![template.clone()], vec![primal]).await;
        designer.select_template(template);
        designer.assign_primal("compute".to_string(), "primal-1".to_string());

        assert_eq!(designer.validation, ValidationResult::Valid);
    }

    #[tokio::test]
    async fn test_primal_unassignment() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut designer = NicheDesigner::new(event_handler);

        let template = NicheTemplate {
            id: "template-1".to_string(),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            required_primals: vec!["compute".to_string()],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        };

        let primal = Primal {
            id: "primal-1".to_string(),
            name: "Test Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        };

        designer.refresh(vec![template.clone()], vec![primal]).await;
        designer.select_template(template);
        designer.assign_primal("compute".to_string(), "primal-1".to_string());

        assert_eq!(designer.assigned_primals.len(), 1);

        designer.unassign_primal("compute");

        assert_eq!(designer.assigned_primals.len(), 0);
        assert_ne!(designer.validation, ValidationResult::Valid);
    }

    #[tokio::test]
    async fn test_capability_mismatch() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut designer = NicheDesigner::new(event_handler);

        let primal = Primal {
            id: "primal-1".to_string(),
            name: "Compute Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()], // Only has compute
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        };

        designer.refresh(vec![], vec![primal]).await;

        // Try to assign to storage capability (which the primal doesn't have)
        designer.assign_primal("storage".to_string(), "primal-1".to_string());

        // Should not be assigned
        assert_eq!(designer.assigned_primals.len(), 0);
    }

    #[tokio::test]
    async fn test_optional_primals() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut designer = NicheDesigner::new(event_handler);

        let template = NicheTemplate {
            id: "template-1".to_string(),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            required_primals: vec!["compute".to_string()],
            optional_primals: vec!["storage".to_string()],
            metadata: serde_json::json!({}),
        };

        let compute_primal = Primal {
            id: "primal-1".to_string(),
            name: "Compute Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        };

        designer
            .refresh(vec![template.clone()], vec![compute_primal])
            .await;
        designer.select_template(template);
        designer.assign_primal("compute".to_string(), "primal-1".to_string());

        // Should be valid even without optional primal
        assert_eq!(designer.validation, ValidationResult::Valid);
    }

    #[tokio::test]
    async fn test_multiple_assignments() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut designer = NicheDesigner::new(event_handler);

        let template = NicheTemplate {
            id: "template-1".to_string(),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            required_primals: vec!["compute".to_string(), "storage".to_string()],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        };

        let compute_primal = Primal {
            id: "primal-1".to_string(),
            name: "Compute Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        };

        let storage_primal = Primal {
            id: "primal-2".to_string(),
            name: "Storage Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["storage".to_string()],
            load: 0.3,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        };

        designer
            .refresh(vec![template.clone()], vec![compute_primal, storage_primal])
            .await;
        designer.select_template(template);
        designer.assign_primal("compute".to_string(), "primal-1".to_string());
        designer.assign_primal("storage".to_string(), "primal-2".to_string());

        assert_eq!(designer.assigned_primals.len(), 2);
        assert_eq!(designer.validation, ValidationResult::Valid);
    }
}
