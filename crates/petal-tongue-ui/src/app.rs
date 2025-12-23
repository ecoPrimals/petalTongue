//! Main application logic for petalTongue UI

use petal_tongue_core::{GraphEngine, LayoutAlgorithm, PrimalInfo, PrimalHealthStatus, TopologyEdge};
use petal_tongue_graph::{Visual2DRenderer, AudioSonificationRenderer};
use petal_tongue_api::BiomeOSClient;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// The main petalTongue UI application
pub struct PetalTongueApp {
    /// The graph engine (shared between renderers)
    graph: Arc<RwLock<GraphEngine>>,
    /// Visual renderer
    visual_renderer: Visual2DRenderer,
    /// Audio renderer
    audio_renderer: AudioSonificationRenderer,
    /// BiomeOS API client
    biomeos_client: BiomeOSClient,
    /// Current layout algorithm
    current_layout: LayoutAlgorithm,
    /// Show audio description panel
    show_audio_panel: bool,
    /// Show controls panel
    show_controls: bool,
    /// Last refresh time
    last_refresh: Instant,
    /// Auto-refresh enabled
    auto_refresh: bool,
    /// Refresh interval (seconds)
    refresh_interval: f32,
}

impl PetalTongueApp {
    /// Create a new application
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create BiomeOS client (try localhost:3000, fallback to mock)
        let biomeos_url = std::env::var("BIOMEOS_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let biomeos_client = BiomeOSClient::new(&biomeos_url).with_mock_mode(true);
        
        // Create graph engine
        let graph = GraphEngine::new();
        let graph = Arc::new(RwLock::new(graph));
        
        // Create renderers
        let visual_renderer = Visual2DRenderer::new(Arc::clone(&graph));
        let audio_renderer = AudioSonificationRenderer::new(Arc::clone(&graph));
        
        let mut app = Self {
            graph,
            visual_renderer,
            audio_renderer,
            biomeos_client,
            current_layout: LayoutAlgorithm::ForceDirected,
            show_audio_panel: true,
            show_controls: true,
            last_refresh: Instant::now(),
            auto_refresh: true,
            refresh_interval: 5.0,
        };
        
        // Initial data load (async, but we'll do it sync here for simplicity)
        // In production, this would be done in a background task
        app.refresh_graph_data();
        
        app
    }
    
    /// Refresh graph data from BiomeOS
    fn refresh_graph_data(&mut self) {
        // For now, we'll use blocking calls in the UI thread
        // TODO: Move to background task with channels
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            // Discover primals
            let primals = match self.biomeos_client.discover_primals().await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("Failed to discover primals: {}", e);
                    return;
                }
            };
            
            // Get topology
            let edges = match self.biomeos_client.get_topology().await {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Failed to get topology: {}", e);
                    vec![]
                }
            };
            
            // Update graph
            let mut graph = self.graph.write().unwrap();
            
            // Clear existing graph
            // (In production, we'd do a smart merge to preserve positions)
            *graph = GraphEngine::new();
            
            // Add primals
            for primal in primals {
                let _ = graph.add_node(primal);
            }
            
            // Add edges
            for edge in edges {
                let _ = graph.add_edge(edge);
            }
            
            // Apply layout
            graph.set_layout(self.current_layout);
            graph.layout(100);
        });
        
        self.last_refresh = Instant::now();
    }
    
    /// Populate graph with sample primals for demonstration (legacy)
    #[allow(dead_code)]
    fn populate_sample_graph_legacy(graph: &mut GraphEngine) {
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
                
                // Refresh button
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_graph_data();
                }
                
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
                    
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                    
                    // Refresh controls
                    ui.heading(egui::RichText::new("🔄 Auto-Refresh").size(16.0));
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.auto_refresh, "Enabled");
                    ui.add(egui::Slider::new(&mut self.refresh_interval, 1.0..=60.0).text("Interval (s)"));
                    
                    let elapsed = self.last_refresh.elapsed().as_secs_f32();
                    ui.label(format!("Last refresh: {:.1}s ago", elapsed));
                    
                    if ui.button("Refresh Now").clicked() {
                        self.refresh_graph_data();
                    }
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
        
        // Auto-refresh logic
        if self.auto_refresh {
            let elapsed = self.last_refresh.elapsed();
            if elapsed >= Duration::from_secs_f32(self.refresh_interval) {
                self.refresh_graph_data();
            }
            
            // Request repaint for next frame
            ctx.request_repaint();
        }
    }
}

