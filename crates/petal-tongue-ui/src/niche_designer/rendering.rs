// SPDX-License-Identifier: AGPL-3.0-or-later
//! Niche Designer - Rendering and drawing logic
//!
//! Template selector, canvas, validation panel, and deploy button UI.

use crate::biomeos_integration::NicheTemplate;
use egui::{Color32, RichText, Ui};
use tracing::info;

use super::state::NicheDesigner;
use super::types::ValidationResult;

/// Get (icon, text, `color_rgb`) for a validation result.
#[must_use]
pub fn validation_display_info(result: &ValidationResult) -> (&'static str, String, [u8; 3]) {
    match result {
        ValidationResult::Valid => ("✓", "All requirements met".to_string(), [0, 255, 0]),
        ValidationResult::MissingRequirements(missing) => (
            "✖",
            format!("Missing required capabilities: {}", missing.join(", ")),
            [255, 0, 0],
        ),
        ValidationResult::InsufficientResources(msg) => ("⚠", msg.clone(), [255, 255, 0]),
        ValidationResult::Conflicts(_) => ("✖", "Conflicts detected:".to_string(), [255, 0, 0]),
    }
}

/// Whether deployment is allowed given the validation result.
#[must_use]
pub fn can_deploy(validation: &ValidationResult) -> bool {
    validation == &ValidationResult::Valid
}

#[must_use]
pub const fn slot_placeholder_text(required: bool) -> &'static str {
    if required {
        "Drop primal here (required)"
    } else {
        "Drop primal here (optional)"
    }
}

#[must_use]
pub fn slot_drop_hover_text(capability: &str) -> String {
    format!("Drop primal here to assign to {capability} capability")
}

#[must_use]
pub const fn deploy_hint_message() -> &'static str {
    "Complete all required assignments to deploy"
}

#[must_use]
pub fn format_conflict_items(conflicts: &[String]) -> Vec<String> {
    conflicts.iter().map(|c| format!("  • {c}")).collect()
}

impl NicheDesigner {
    /// Render the niche designer
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("🎨 Niche Designer");
        ui.separator();

        // Template selector
        self.render_template_selector(ui);
        ui.add_space(8.0);

        let selected = self.selected_template.clone();
        if let Some(template) = &selected {
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
                .map_or_else(|| "Select a template...".to_string(), |t| t.name.clone());

            egui::ComboBox::from_id_salt("template_selector")
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    let templates = self.templates.clone();
                    for template in &templates {
                        let is_selected = self
                            .selected_template
                            .as_ref()
                            .is_some_and(|t| t.id == template.id);

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
                    ui.label(format!("{capability}:"));

                    // Assignment status
                    if let Some(primal) = &assigned_primal {
                        ui.colored_label(Color32::GREEN, format!("✓ {}", primal.name));

                        // Unassign button
                        if ui.small_button("✖").clicked() {
                            self.unassign_primal(capability);
                        }
                    } else {
                        let text =
                            RichText::new(slot_placeholder_text(required)).color(if required {
                                Color32::RED
                            } else {
                                Color32::GRAY
                            });
                        ui.label(text);
                    }
                });
            })
            .response;

        // Check for dragged primal
        let dragged_primal_id =
            ui.memory(|mem| mem.data.get_temp::<String>(egui::Id::new("dragged_primal")));

        if let Some(_primal_id) = dragged_primal_id
            && slot_response.hovered()
        {
            // Highlight as drop zone
            let highlight_rect = slot_response.rect.expand(2.0);
            ui.painter()
                .rect_stroke(highlight_rect, 4.0, (2.0, Color32::LIGHT_BLUE));

            slot_response.on_hover_text(slot_drop_hover_text(capability));

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

    /// Render validation panel
    fn render_validation_panel(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Validation").strong());
                ui.separator();

                let (icon, text, [r, g, b]) = validation_display_info(&self.validation);
                let color = Color32::from_rgb(r, g, b);
                ui.colored_label(color, format!("{icon} {text}"));

                if let ValidationResult::Conflicts(conflicts) = &self.validation {
                    for line in format_conflict_items(conflicts) {
                        ui.label(line);
                    }
                }
            });
        });
    }

    /// Render deploy button
    fn render_deploy_button(&self, ui: &mut Ui) {
        let deploy_allowed = can_deploy(&self.validation);

        let button = egui::Button::new(RichText::new("🚀 Deploy Niche").strong().size(16.0).color(
            if deploy_allowed {
                Color32::WHITE
            } else {
                Color32::GRAY
            },
        ))
        .fill(if deploy_allowed {
            Color32::from_rgb(0, 120, 0)
        } else {
            Color32::from_gray(60)
        });

        if ui.add_enabled(deploy_allowed, button).clicked() {
            self.deploy_niche();
        }

        if !deploy_allowed {
            ui.colored_label(Color32::GRAY, deploy_hint_message());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_display_info_valid() {
        let (icon, text, rgb) = validation_display_info(&ValidationResult::Valid);
        assert_eq!(icon, "✓");
        assert_eq!(text, "All requirements met");
        assert_eq!(rgb, [0, 255, 0]);
    }

    #[test]
    fn validation_display_info_missing() {
        let (icon, text, rgb) =
            validation_display_info(&ValidationResult::MissingRequirements(vec![
                "cap1".to_string(),
                "cap2".to_string(),
            ]));
        assert_eq!(icon, "✖");
        assert!(text.contains("Missing required capabilities"));
        assert!(text.contains("cap1"));
        assert!(text.contains("cap2"));
        assert_eq!(rgb, [255, 0, 0]);
    }

    #[test]
    fn validation_display_info_insufficient() {
        let (icon, text, rgb) = validation_display_info(&ValidationResult::InsufficientResources(
            "low memory".to_string(),
        ));
        assert_eq!(icon, "⚠");
        assert_eq!(text, "low memory");
        assert_eq!(rgb, [255, 255, 0]);
    }

    #[test]
    fn validation_display_info_conflicts() {
        let (icon, text, rgb) =
            validation_display_info(&ValidationResult::Conflicts(vec!["conflict1".to_string()]));
        assert_eq!(icon, "✖");
        assert_eq!(text, "Conflicts detected:");
        assert_eq!(rgb, [255, 0, 0]);
    }

    #[test]
    fn can_deploy_valid() {
        assert!(can_deploy(&ValidationResult::Valid));
    }

    #[test]
    fn can_deploy_invalid() {
        assert!(!can_deploy(&ValidationResult::MissingRequirements(vec![
            "x".to_string()
        ])));
        assert!(!can_deploy(&ValidationResult::InsufficientResources(
            "msg".to_string()
        )));
        assert!(!can_deploy(&ValidationResult::Conflicts(vec![
            "c".to_string()
        ])));
    }

    #[test]
    fn validation_display_info_missing_empty() {
        let (icon, text, rgb) =
            validation_display_info(&ValidationResult::MissingRequirements(vec![]));
        assert_eq!(icon, "✖");
        assert!(text.contains("Missing required capabilities"));
        assert_eq!(rgb, [255, 0, 0]);
    }

    #[test]
    fn slot_placeholder_text_required() {
        assert_eq!(slot_placeholder_text(true), "Drop primal here (required)");
    }

    #[test]
    fn slot_placeholder_text_optional() {
        assert_eq!(slot_placeholder_text(false), "Drop primal here (optional)");
    }

    #[test]
    fn slot_drop_hover_text_format() {
        assert!(slot_drop_hover_text("auth").contains("auth"));
    }

    #[test]
    fn format_conflict_items_output() {
        let items = format_conflict_items(&["c1".to_string(), "c2".to_string()]);
        assert_eq!(items.len(), 2);
        assert!(items[0].contains("c1"));
        assert!(items[1].contains("c2"));
    }

    #[test]
    fn validation_display_info_conflicts_multiple() {
        let (icon, text, rgb) = validation_display_info(&ValidationResult::Conflicts(vec![
            "c1".to_string(),
            "c2".to_string(),
        ]));
        assert_eq!(icon, "✖");
        assert_eq!(text, "Conflicts detected:");
        assert_eq!(rgb, [255, 0, 0]);
    }
}
