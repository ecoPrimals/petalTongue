//! Main application logic for petalTongue UI

use petal_tongue_core::{GraphEngine, PrimalInfo, PrimalHealthStatus, TopologyEdge, LayoutAlgorithm};
use petal_tongue_graph::{Visual2DRenderer, AudioSonificationRenderer};
use std::sync::{Arc, RwLock};

/// The main petalTongue UI application
pub struct PetalTongueApp {
    /// The graph engine (shared between renderers)
    graph: Arc<RwLock<GraphEngine>>,
    /// Visual renderer
    visual_renderer: Visual2DRenderer,
    /// Audio renderer
    audio_renderer: AudioSonificationRenderer,
    /// Current layout algorithm
    current_layout: LayoutAlgorithm,
    /// Show audio description panel
    show_audio_panel: bool,
    /// Show controls panel
    show_controls: bool,
}

impl PetalTongueApp {
    /// Create a new application
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create graph engine
        let mut graph = GraphEngine::new();
        
        // Add sample primals
        Self::populate_sample_graph(&mut graph);
        
        // Apply initial layout
        graph.set_layout(LayoutAlgorithm::ForceDirected);
        graph.layout(100);
        
        let graph = Arc::new(RwLock::new(graph));
        
        // Create renderers
        let visual_renderer = Visual2DRenderer::new(Arc::clone(&graph));
        let audio_renderer = AudioSonificationRenderer::new(Arc::clone(&graph));
        
        Self {
            graph,
            visual_renderer,
            audio_renderer,
            current_layout: LayoutAlgorithm::ForceDirected,
            show_audio_panel: true,
            show_controls: true,
        }
    }
    
    /// Populate graph with sample primals for demonstration
    fn populate_sample_graph(graph: &mut GraphEngine) {
        // Add BearDog (Security)
        graph.add_node(PrimalInfo {
            id: "beardog-1".to_string(),
            name: "BearDog Security".to_string(),
            primal_type: "Security".to_string(),
            endpoint: "http://localhost:8001".to_string(),
            capabilities: vec![
                "authentication".to_string(),
                "authorization".to_string(),
                "encryption".to_string(),
            ],
            health: PrimalHealthStatus::Healthy,
            last_seen: 1703376000,
        });
        
        // Add ToadStool (Compute)
        graph.add_node(PrimalInfo {
            id: "toadstool-1".to_string(),
            name: "ToadStool Compute".to_string(),
            primal_type: "Compute".to_string(),
            endpoint: "http://localhost:8002".to_string(),
            capabilities: vec![
                "container_runtime".to_string(),
                "workload_execution".to_string(),
            ],
            health: PrimalHealthStatus::Warning,
            last_seen: 1703376060,
        });
        
        // Add Songbird (Discovery)
        graph.add_node(PrimalInfo {
            id: "songbird-1".to_string(),
            name: "Songbird Discovery".to_string(),
            primal_type: "Discovery".to_string(),
            endpoint: "http://localhost:8003".to_string(),
            capabilities: vec![
                "service_discovery".to_string(),
                "capability_matching".to_string(),
            ],
            health: PrimalHealthStatus::Healthy,
            last_seen: 1703376120,
        });
        
        // Add NestGate (Storage)
        graph.add_node(PrimalInfo {
            id: "nestgate-1".to_string(),
            name: "NestGate Storage".to_string(),
            primal_type: "Storage".to_string(),
            endpoint: "http://localhost:8004".to_string(),
            capabilities: vec![
                "permanent_storage".to_string(),
                "content_addressing".to_string(),
                "attribution".to_string(),
            ],
            health: PrimalHealthStatus::Healthy,
            last_seen: 1703376180,
        });
        
        // Add Squirrel (AI)
        graph.add_node(PrimalInfo {
            id: "squirrel-1".to_string(),
            name: "Squirrel AI".to_string(),
            primal_type: "AI".to_string(),
            endpoint: "http://localhost:8005".to_string(),
            capabilities: vec![
                "intent_parsing".to_string(),
                "task_planning".to_string(),
            ],
            health: PrimalHealthStatus::Critical,
            last_seen: 1703376240,
        });
        
        // Add connections
        graph.add_edge(TopologyEdge {
            from: "beardog-1".to_string(),
            to: "toadstool-1".to_string(),
            edge_type: "authenticates".to_string(),
            label: Some("Auth Flow".to_string()),
        });
        
        graph.add_edge(TopologyEdge {
            from: "songbird-1".to_string(),
            to: "beardog-1".to_string(),
            edge_type: "discovers".to_string(),
            label: None,
        });
        
        graph.add_edge(TopologyEdge {
            from: "toadstool-1".to_string(),
            to: "nestgate-1".to_string(),
            edge_type: "stores_to".to_string(),
            label: Some("Data Flow".to_string()),
        });
        
        graph.add_edge(TopologyEdge {
            from: "squirrel-1".to_string(),
            to: "songbird-1".to_string(),
            edge_type: "queries".to_string(),
            label: None,
        });
        
        graph.add_edge(TopologyEdge {
            from: "squirrel-1".to_string(),
            to: "toadstool-1".to_string(),
            edge_type: "orchestrates".to_string(),
            label: Some("Task Execution".to_string()),
        });
    }
}

impl eframe::App for PetalTongueApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set dark theme with custom colors
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 220));
        style.visuals.window_fill = egui::Color32::from_rgb(25, 25, 30);
        style.visuals.panel_fill = egui::Color32::from_rgb(30, 30, 35);
        ctx.set_style(style);
        
        // Top menu bar
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 20, 25)).inner_margin(8.0))
            .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.heading(egui::RichText::new("🌸 petalTongue").size(20.0).color(egui::Color32::from_rgb(255, 182, 193)));
                ui.label(egui::RichText::new("Universal Representation System").size(14.0).color(egui::Color32::GRAY));
                
                ui.separator();
                
                if ui.button("Reset Camera").clicked() {
                    self.visual_renderer.reset_camera();
                }
                
                ui.separator();
                
                ui.label("Layout:");
                egui::ComboBox::from_id_salt("layout_selector")
                    .selected_text(format!("{:?}", self.current_layout))
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut self.current_layout, LayoutAlgorithm::ForceDirected, "Force-Directed").clicked() {
                            let mut graph = self.graph.write().unwrap();
                            graph.set_layout(LayoutAlgorithm::ForceDirected);
                            graph.layout(100);
                        }
                        if ui.selectable_value(&mut self.current_layout, LayoutAlgorithm::Hierarchical, "Hierarchical").clicked() {
                            let mut graph = self.graph.write().unwrap();
                            graph.set_layout(LayoutAlgorithm::Hierarchical);
                            graph.layout(1);
                        }
                        if ui.selectable_value(&mut self.current_layout, LayoutAlgorithm::Circular, "Circular").clicked() {
                            let mut graph = self.graph.write().unwrap();
                            graph.set_layout(LayoutAlgorithm::Circular);
                            graph.layout(1);
                        }
                    });
                
                ui.separator();
                
                ui.checkbox(&mut self.show_controls, "Controls");
                ui.checkbox(&mut self.show_audio_panel, "Audio Info");
            });
        });
        
        // Left panel - Controls
        if self.show_controls {
            egui::SidePanel::left("controls_panel")
                .default_width(280.0)
                .frame(egui::Frame::none().fill(egui::Color32::from_rgb(30, 30, 35)).inner_margin(12.0))
                .show(ctx, |ui| {
                    ui.heading(egui::RichText::new("⚙️ Controls").size(18.0));
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("🖱️ Mouse Controls").strong());
                    ui.add_space(4.0);
                    ui.label("  • Drag: Pan camera");
                    ui.label("  • Scroll: Zoom in/out");
                    ui.label("  • Click: Select node");
                    
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                    ui.heading(egui::RichText::new("🎨 Health Legend").size(16.0));
                    
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(40, 180, 40), "⬤");
                        ui.label("Healthy");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(200, 180, 40), "⬤");
                        ui.label("Warning");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(200, 40, 40), "⬤");
                        ui.label("Critical");
                    });
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(120, 120, 120), "⬤");
                        ui.label("Unknown");
                    });
                });
        }
        
        // Right panel - Audio information
        if self.show_audio_panel {
            egui::SidePanel::right("audio_panel")
                .default_width(380.0)
                .frame(egui::Frame::none().fill(egui::Color32::from_rgb(30, 30, 35)).inner_margin(12.0))
                .show(ctx, |ui| {
                    ui.heading(egui::RichText::new("🎵 Audio Representation").size(18.0));
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // Master volume control
                    let mut volume = self.audio_renderer.master_volume();
                    ui.horizontal(|ui| {
                        ui.label("Master Volume:");
                        if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0)).changed() {
                            self.audio_renderer.set_master_volume(volume);
                        }
                    });
                    
                    // Enable/disable toggle
                    let mut enabled = self.audio_renderer.is_enabled();
                    ui.horizontal(|ui| {
                        ui.label("Audio Enabled:");
                        if ui.checkbox(&mut enabled, "").changed() {
                            self.audio_renderer.set_enabled(enabled);
                        }
                    });
                    
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // Soundscape description
                    ui.heading(egui::RichText::new("🎼 Soundscape").size(16.0));
                    ui.add_space(4.0);
                    let description = self.audio_renderer.describe_soundscape();
                    ui.label(egui::RichText::new(description).size(13.0).color(egui::Color32::from_rgb(200, 200, 200)));
                    
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // Node-level audio info
                    if let Some(selected_id) = self.visual_renderer.selected_node() {
                        ui.heading(egui::RichText::new("🎯 Selected Node").size(16.0));
                        ui.add_space(4.0);
                        if let Some(node_desc) = self.audio_renderer.describe_node_audio(selected_id) {
                            ui.label(egui::RichText::new(node_desc).size(13.0).color(egui::Color32::from_rgb(255, 230, 150)));
                        }
                    } else {
                        ui.heading(egui::RichText::new("🎯 Selected Node").size(16.0).color(egui::Color32::GRAY));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Click a node to hear its audio representation").size(12.0).italics().color(egui::Color32::GRAY));
                    }
                    
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // Instrument legend
                    ui.heading(egui::RichText::new("🎹 Instrument Mapping").size(16.0));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("🐻 Security → Deep Bass").color(egui::Color32::from_rgb(100, 150, 255)));
                    ui.label(egui::RichText::new("🍄 Compute → Rhythmic Drums").color(egui::Color32::from_rgb(255, 200, 100)));
                    ui.label(egui::RichText::new("🐦 Discovery → Light Chimes").color(egui::Color32::from_rgb(150, 255, 150)));
                    ui.label(egui::RichText::new("🏠 Storage → Sustained Strings").color(egui::Color32::from_rgb(255, 150, 255)));
                    ui.label(egui::RichText::new("🐿️ AI → High Synth").color(egui::Color32::from_rgb(255, 100, 100)));
                });
        }
        
        // Central panel - Graph visualization
        egui::CentralPanel::default().show(ctx, |ui| {
            self.visual_renderer.render(ui);
        });
    }
}

