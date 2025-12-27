//! `BingoCube` tool integration module
//!
//! Demonstrates how petalTongue uses external tools (`BingoCube`) as a primal.
//! This module encapsulates all BingoCube-related state and UI rendering.
//!
//! `BingoCube` implements the `ToolPanel` trait, making it a capability-based tool
//! that petalTongue can use without hardcoded knowledge.

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::tool_integration::{ToolCapability, ToolMetadata, ToolPanel};
use bingocube_adapters::audio::BingoCubeAudioRenderer;
use bingocube_adapters::visual::BingoCubeVisualRenderer;
use bingocube_core::{BingoCube, Config as BingoCubeConfig};

/// `BingoCube` integration state and UI
pub struct BingoCubeIntegration {
    /// Show `BingoCube` panel
    pub show_panel: bool,
    /// `BingoCube` instance (tool being used)
    pub cube: Option<BingoCube>,
    /// `BingoCube` visual renderer (adapter)
    pub renderer: Option<BingoCubeVisualRenderer>,
    /// `BingoCube` audio renderer (adapter)
    pub audio_renderer: Option<BingoCubeAudioRenderer>,
    /// `BingoCube` seed input
    pub seed: String,
    /// `BingoCube` reveal parameter (0.0-1.0)
    pub reveal_x: f64,
    /// `BingoCube` configuration
    pub config: BingoCubeConfig,
    /// `BingoCube` error message
    pub error: Option<String>,
    /// Show `BingoCube` configuration panel
    pub show_config: bool,
    /// Show `BingoCube` audio panel
    pub show_audio: bool,
}

impl Default for BingoCubeIntegration {
    fn default() -> Self {
        Self {
            show_panel: false,
            cube: None,
            renderer: None,
            audio_renderer: None,
            seed: "example-seed".to_string(),
            reveal_x: 1.0,
            config: BingoCubeConfig::default(),
            error: None,
            show_config: false,
            show_audio: false,
        }
    }
}

impl BingoCubeIntegration {
    /// Create a new `BingoCube` integration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a new `BingoCube` from the current seed and configuration
    pub fn generate(&mut self) {
        // Validate configuration first
        if let Err(e) = self.config.validate() {
            self.error = Some(format!("Invalid configuration: {e}"));
            tracing::error!("Invalid BingoCube configuration: {}", e);
            return;
        }

        match BingoCube::from_seed(self.seed.as_bytes(), self.config.clone()) {
            Ok(cube) => {
                let renderer = BingoCubeVisualRenderer::new().with_reveal(self.reveal_x);
                let audio_renderer = BingoCubeAudioRenderer::new(cube.clone());
                self.cube = Some(cube);
                self.renderer = Some(renderer);
                self.audio_renderer = Some(audio_renderer);
                self.error = None;
                tracing::info!(
                    "Generated BingoCube from seed '{}' with config: {}×{}, {} colors",
                    self.seed,
                    self.config.grid_size,
                    self.config.grid_size,
                    self.config.palette_size
                );
            }
            Err(e) => {
                self.error = Some(format!("Failed to generate BingoCube: {e}"));
                tracing::error!("Failed to generate BingoCube: {}", e);
                self.cube = None;
                self.renderer = None;
                self.audio_renderer = None;
            }
        }
    }

    /// Render the `BingoCube` panel
    pub fn render_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(egui::RichText::new("🎲 BingoCube Tool Integration").size(24.0));
            ui.label(
                egui::RichText::new("Demonstrating Primal Tool Use")
                    .size(14.0)
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(10.0);
        });

        ui.separator();
        ui.add_space(10.0);

        // Error display
        if let Some(error) = self.error.clone() {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(80, 30, 30))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⚠")
                                .size(20.0)
                                .color(egui::Color32::from_rgb(255, 100, 100)),
                        );
                        ui.label(
                            egui::RichText::new(&error)
                                .color(egui::Color32::from_rgb(255, 200, 200)),
                        );
                        if ui.button("✕").clicked() {
                            self.error = None;
                        }
                    });
                });
            ui.add_space(10.0);
        }

        // Controls
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(30, 30, 35))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Seed:");
                    let response = ui.text_edit_singleline(&mut self.seed);

                    if response.changed() {
                        // Generate new BingoCube when seed changes
                        self.generate();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Reveal (x):");
                    if ui
                        .add(egui::Slider::new(&mut self.reveal_x, 0.0..=1.0).text(""))
                        .changed()
                    {
                        // Update renderer reveal
                        if let Some(renderer) = &mut self.renderer {
                            renderer.set_reveal(self.reveal_x);
                        }
                    }
                    ui.label(format!("{:.0}%", self.reveal_x * 100.0));
                });

                ui.horizontal(|ui| {
                    if ui.button("🎲 Generate New").clicked() {
                        self.generate();
                    }

                    if ui.button("▶ Animate Reveal").clicked()
                        && let Some(renderer) = &mut self.renderer {
                            renderer.set_reveal(0.0).animate_to(1.0);
                            self.reveal_x = 0.0;
                        }

                    if ui.button("⚙ Config").clicked() {
                        self.show_config = !self.show_config;
                    }

                    if ui.button("🎵 Audio").clicked() {
                        self.show_audio = !self.show_audio;
                    }
                });
            });

        ui.add_space(10.0);

        // Configuration panel (collapsible)
        if self.show_config {
            self.render_config_panel(ui);
            ui.add_space(10.0);
        }

        // Render BingoCube if available
        if let (Some(cube), Some(renderer)) = (&self.cube, &mut self.renderer) {
            // Sync reveal parameter
            self.reveal_x = renderer.get_reveal();

            egui::Frame::none()
                .fill(egui::Color32::from_rgb(25, 25, 30))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    renderer.render(ui, cube);
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No BingoCube generated yet").color(egui::Color32::GRAY),
                );
            });
        }

        ui.add_space(20.0);

        // Audio panel (collapsible)
        if self.show_audio {
            self.render_audio_panel(ui);
        }
    }

    /// Render the configuration panel
    fn render_config_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(35, 35, 40))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.heading(egui::RichText::new("Configuration").size(16.0));
                ui.add_space(10.0);

                let mut config_changed = false;

                ui.horizontal(|ui| {
                    ui.label("Grid Size:");
                    if ui
                        .add(egui::Slider::new(&mut self.config.grid_size, 3..=12).text(""))
                        .changed()
                    {
                        // Adjust universe size to maintain divisibility
                        self.config.universe_size = self.config.grid_size
                            * (self.config.universe_size / self.config.grid_size);
                        config_changed = true;
                    }
                    ui.label(format!(
                        "{}×{}",
                        self.config.grid_size, self.config.grid_size
                    ));
                });

                ui.horizontal(|ui| {
                    ui.label("Palette Size:");
                    let mut palette_log2 = (self.config.palette_size as f32).log2() as usize;
                    if ui
                        .add(egui::Slider::new(&mut palette_log2, 2..=8).text(""))
                        .changed()
                    {
                        self.config.palette_size = 1 << palette_log2;
                        config_changed = true;
                    }
                    ui.label(format!("{} colors", self.config.palette_size));
                });

                ui.horizontal(|ui| {
                    if ui.button("Small (5×5)").clicked() {
                        self.config = BingoCubeConfig::small();
                        config_changed = true;
                    }
                    if ui.button("Medium (8×8)").clicked() {
                        self.config = BingoCubeConfig::medium();
                        config_changed = true;
                    }
                    if ui.button("Large (12×12)").clicked() {
                        self.config = BingoCubeConfig::large();
                        config_changed = true;
                    }
                });

                if config_changed {
                    self.generate();
                }
            });
    }

    /// Render the audio panel
    fn render_audio_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(35, 35, 40))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.heading(egui::RichText::new("🎵 Audio Sonification").size(16.0));
                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new(
                        "BingoCube audio demonstrates how tools can provide multi-modal output.",
                    )
                    .color(egui::Color32::GRAY),
                );
                ui.add_space(10.0);

                if self.cube.is_some() {
                    ui.horizontal(|ui| {
                        if ui.button("▶ Play Reveal Sequence").clicked() {
                            // TODO: Integrate with audio system
                            tracing::info!("Play BingoCube reveal sequence");
                        }

                        if ui.button("⏹ Stop").clicked() {
                            // TODO: Integrate with audio system
                            tracing::info!("Stop BingoCube audio");
                        }
                    });

                    ui.add_space(10.0);

                    if let Some(audio_renderer) = &self.audio_renderer {
                        let description = audio_renderer.describe_soundscape(self.reveal_x);
                        ui.label(
                            egui::RichText::new(description)
                                .size(12.0)
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                    }
                } else {
                    ui.label(
                        egui::RichText::new(
                            "Generate a BingoCube to hear its audio representation",
                        )
                        .color(egui::Color32::GRAY),
                    );
                }
            });
    }
}

// Implement ToolPanel trait for capability-based tool integration
impl ToolPanel for BingoCubeIntegration {
    fn metadata(&self) -> &ToolMetadata {
        // Static metadata - could be lazy_static in production
        static METADATA: std::sync::OnceLock<ToolMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| ToolMetadata {
            name: "BingoCube".to_string(),
            description: "Human-verifiable cryptographic commitment system".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec![
                ToolCapability::Visual,
                ToolCapability::Audio,
                ToolCapability::TextInput,
                ToolCapability::Progressive,
                ToolCapability::Export,
            ],
            icon: "🎲".to_string(),
            source: Some("https://github.com/ecoPrimals/bingoCube".to_string()),
        })
    }

    fn is_visible(&self) -> bool {
        self.show_panel
    }

    fn toggle_visibility(&mut self) {
        self.show_panel = !self.show_panel;
    }

    fn render_panel(&mut self, ui: &mut egui::Ui) {
        // Inline the rendering logic here since we're implementing the trait
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(egui::RichText::new("🎲 BingoCube Tool Integration").size(24.0));
            ui.label(
                egui::RichText::new("Demonstrating Primal Tool Use")
                    .size(14.0)
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(10.0);
        });

        ui.separator();
        ui.add_space(10.0);

        // Error display
        if let Some(error) = self.error.clone() {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(80, 30, 30))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⚠")
                                .size(20.0)
                                .color(egui::Color32::from_rgb(255, 100, 100)),
                        );
                        ui.label(
                            egui::RichText::new(&error)
                                .color(egui::Color32::from_rgb(255, 200, 200)),
                        );
                        if ui.button("✕").clicked() {
                            self.error = None;
                        }
                    });
                });
            ui.add_space(10.0);
        }

        // Controls (inline)
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(30, 30, 35))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Seed:");
                    let response = ui.text_edit_singleline(&mut self.seed);

                    if response.changed() {
                        self.generate();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Reveal (x):");
                    if ui
                        .add(egui::Slider::new(&mut self.reveal_x, 0.0..=1.0).text(""))
                        .changed()
                        && let Some(renderer) = &mut self.renderer {
                            renderer.set_reveal(self.reveal_x);
                        }
                    ui.label(format!("{:.0}%", self.reveal_x * 100.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Animation:");

                    if ui.button("▶ Animate Reveal").clicked()
                        && let Some(renderer) = &mut self.renderer {
                            renderer.set_reveal(0.0).animate_to(1.0);
                            self.reveal_x = 0.0;
                        }

                    if ui.button("⚙ Config").clicked() {
                        self.show_config = !self.show_config;
                    }

                    if ui.button("🎵 Audio").clicked() {
                        self.show_audio = !self.show_audio;
                    }
                });
            });

        ui.add_space(10.0);

        // Configuration panel (if visible)
        if self.show_config {
            self.render_config_panel(ui);
            ui.add_space(10.0);
        }

        // Render BingoCube if available
        if let (Some(cube), Some(renderer)) = (&self.cube, &mut self.renderer) {
            self.reveal_x = renderer.get_reveal();

            egui::Frame::none()
                .fill(egui::Color32::from_rgb(25, 25, 30))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    renderer.render(ui, cube);
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No BingoCube generated yet").color(egui::Color32::GRAY),
                );
            });
        }

        ui.add_space(20.0);

        // Audio panel (if visible)
        if self.show_audio {
            self.render_audio_panel(ui);
        }
    }

    fn status_message(&self) -> Option<String> {
        if let Some(error) = &self.error {
            Some(format!("Error: {error}"))
        } else if self.cube.is_some() {
            Some(format!("Generated from seed '{}'", self.seed))
        } else {
            Some("Ready to generate".to_string())
        }
    }

    fn handle_action(&mut self, action: &str) -> Result<(), String> {
        match action {
            "generate" => {
                self.generate();
                Ok(())
            }
            "toggle_config" => {
                self.show_config = !self.show_config;
                Ok(())
            }
            "toggle_audio" => {
                self.show_audio = !self.show_audio;
                Ok(())
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}
