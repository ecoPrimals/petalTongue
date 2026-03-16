// SPDX-License-Identifier: AGPL-3.0-or-later
//! Niche Designer - State and business logic
//!
//! Niche designer state, template selection, primal assignment, and validation.

use crate::biomeos_integration::{NicheTemplate, Primal};
use crate::ui_events::{UIEvent, UIEventHandler};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::ValidationResult;

/// Niche designer state
pub struct NicheDesigner {
    /// Available templates
    pub(crate) templates: Vec<NicheTemplate>,
    /// Selected template
    pub(crate) selected_template: Option<NicheTemplate>,
    /// Available primals (for assignment)
    pub(crate) available_primals: Vec<Primal>,
    /// Assigned primals (capability -> `primal_id`)
    pub(crate) assigned_primals: std::collections::HashMap<String, String>,
    /// Validation result
    pub(crate) validation: ValidationResult,
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
    #[expect(clippy::unused_async, reason = "async for trait compatibility")]
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

    /// Select a template
    pub(crate) fn select_template(&mut self, template: NicheTemplate) {
        info!("📋 Selected template: {}", template.name);
        self.selected_template = Some(template);
        self.assigned_primals.clear();
        self.validate();
    }

    /// Assign a primal to a capability
    pub(crate) fn assign_primal(&mut self, capability: String, primal_id: String) {
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
    pub(crate) fn unassign_primal(&mut self, capability: &str) {
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

            if missing.is_empty() {
                // All requirements met
                self.validation = ValidationResult::Valid;
            } else {
                self.validation = ValidationResult::MissingRequirements(missing);
            }
        } else {
            self.validation = ValidationResult::Valid;
        }
    }

    /// Deploy the niche (called from rendering)
    pub(crate) fn deploy_niche(&self) {
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
    #[must_use]
    pub const fn validation_result(&self) -> &ValidationResult {
        &self.validation
    }

    /// Get assigned primals
    #[must_use]
    pub const fn assigned_primals(&self) -> &std::collections::HashMap<String, String> {
        &self.assigned_primals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biomeos_integration::{Health, NicheTemplate, Primal};
    use crate::ui_events::UIEventHandler;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn make_event_handler() -> Arc<RwLock<UIEventHandler>> {
        Arc::new(RwLock::new(UIEventHandler::new()))
    }

    fn make_template(required: Vec<&str>) -> NicheTemplate {
        NicheTemplate {
            id: "t1".to_string(),
            name: "Test".to_string(),
            description: "Desc".to_string(),
            required_primals: required.into_iter().map(String::from).collect(),
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        }
    }

    fn make_primal(id: &str, caps: Vec<&str>) -> Primal {
        Primal {
            id: id.to_string(),
            name: id.to_string(),
            health: Health::Healthy,
            capabilities: caps.into_iter().map(String::from).collect(),
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        }
    }

    #[tokio::test]
    async fn validation_result_getter() {
        let handler = make_event_handler();
        let mut designer = NicheDesigner::new(handler);
        assert!(matches!(
            designer.validation_result(),
            ValidationResult::Valid
        ));

        designer.select_template(make_template(vec!["compute"]));
        match designer.validation_result() {
            ValidationResult::MissingRequirements(m) => assert_eq!(m, &["compute".to_string()]),
            _ => panic!("expected MissingRequirements"),
        }
    }

    #[tokio::test]
    async fn assigned_primals_getter() {
        let handler = make_event_handler();
        let mut designer = NicheDesigner::new(handler);
        designer
            .refresh(
                vec![make_template(vec!["compute"])],
                vec![make_primal("p1", vec!["compute"])],
            )
            .await;
        designer.select_template(make_template(vec!["compute"]));
        designer.assign_primal("compute".to_string(), "p1".to_string());

        let assigned = designer.assigned_primals();
        assert_eq!(assigned.get("compute"), Some(&"p1".to_string()));
    }

    #[tokio::test]
    async fn no_template_valid() {
        let handler = make_event_handler();
        let designer = NicheDesigner::new(handler);
        assert!(matches!(
            designer.validation_result(),
            ValidationResult::Valid
        ));
    }

    #[tokio::test]
    async fn select_template_clears_assigned() {
        let handler = make_event_handler();
        let mut designer = NicheDesigner::new(handler);
        designer
            .refresh(
                vec![
                    make_template(vec!["compute"]),
                    make_template(vec!["storage"]),
                ],
                vec![
                    make_primal("p1", vec!["compute"]),
                    make_primal("p2", vec!["storage"]),
                ],
            )
            .await;
        designer.select_template(make_template(vec!["compute"]));
        designer.assign_primal("compute".to_string(), "p1".to_string());
        assert_eq!(designer.assigned_primals().len(), 1);

        designer.select_template(make_template(vec!["storage"]));
        assert!(designer.assigned_primals().is_empty());
    }
}
