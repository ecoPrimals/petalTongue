// SPDX-License-Identifier: AGPL-3.0-only
//! Niche Designer - Rendering and drawing logic
//!
//! Template selector, canvas, validation panel, and deploy button UI.

use crate::biomeos_integration::NicheTemplate;
use egui::{Color32, RichText, Ui};
use tracing::info;

use super::state::NicheDesigner;
use super::types::ValidationResult;

impl NicheDesigner {
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
                .map_or_else(|| "Select a template...".to_string(), |t| t.name.clone());

            egui::ComboBox::from_id_salt("template_selector")
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    for template in &self.templates.clone() {
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

        if let Some(_primal_id) = dragged_primal_id
            && slot_response.hovered()
        {
            // Highlight as drop zone
            let highlight_rect = slot_response.rect.expand(2.0);
            ui.painter()
                .rect_stroke(highlight_rect, 4.0, (2.0, Color32::LIGHT_BLUE));

            slot_response.on_hover_text(format!(
                "Drop primal here to assign to {capability} capability"
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
                        ui.colored_label(Color32::YELLOW, format!("⚠ {msg}"));
                    }
                    ValidationResult::Conflicts(conflicts) => {
                        ui.colored_label(Color32::RED, "✖ Conflicts detected:");
                        for conflict in conflicts {
                            ui.label(format!("  • {conflict}"));
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
}
