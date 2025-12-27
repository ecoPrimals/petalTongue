//! Audio Sonification Renderer
//!
//! Renders graph topology as audio soundscape.
//! Maps primals to instruments, health to pitch, activity to volume, position to stereo.

use petal_tongue_core::graph_engine::Node;
use petal_tongue_core::graph_engine::Position;
use petal_tongue_core::{GraphEngine, PrimalHealthStatus};
use std::sync::{Arc, RwLock};

/// Instrument types for sonification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instrument {
    /// Deep bass (security primals like `BearDog`)
    Bass,
    /// Rhythmic drums (compute primals like `ToadStool`)
    Drums,
    /// Light chimes (discovery primals like Songbird)
    Chimes,
    /// Sustained strings (storage primals like `NestGate`)
    Strings,
    /// High synth (AI primals like Squirrel)
    Synth,
    /// Default instrument
    Default,
}

/// Audio attributes for a node
#[derive(Debug, Clone)]
pub struct AudioAttributes {
    /// Instrument to play
    pub instrument: Instrument,
    /// Pitch (0.0 = low, 1.0 = high)
    pub pitch: f32,
    /// Volume (0.0 = silent, 1.0 = max)
    pub volume: f32,
    /// Stereo pan (-1.0 = left, 0.0 = center, 1.0 = right)
    pub pan: f32,
}

/// Audio Sonification Renderer
pub struct AudioSonificationRenderer {
    /// Shared graph engine
    graph: Arc<RwLock<GraphEngine>>,
    /// Master volume
    master_volume: f32,
    /// Is audio enabled
    enabled: bool,
}

impl AudioSonificationRenderer {
    /// Create a new audio sonification renderer
    pub fn new(graph: Arc<RwLock<GraphEngine>>) -> Self {
        Self {
            graph,
            master_volume: 0.7,
            enabled: true,
        }
    }

    /// Generate audio attributes for all nodes
    pub fn generate_audio_attributes(&self) -> Vec<(String, AudioAttributes)> {
        let graph = self.graph.read().expect("graph lock poisoned");

        graph
            .nodes()
            .iter()
            .map(|node| {
                let attrs = self.node_to_audio(node);
                (node.info.id.clone(), attrs)
            })
            .collect()
    }

    /// Convert a node to audio attributes
    fn node_to_audio(&self, node: &Node) -> AudioAttributes {
        AudioAttributes {
            instrument: self.map_primal_to_instrument(&node.info.primal_type),
            pitch: self.health_to_pitch(&node.info.health),
            volume: self.activity_to_volume(node) * self.master_volume,
            pan: self.position_to_pan(node.position),
        }
    }

    /// Map primal type to instrument
    fn map_primal_to_instrument(&self, primal_type: &str) -> Instrument {
        match primal_type.to_lowercase().as_str() {
            "security" | "beardog" => Instrument::Bass,
            "compute" | "toadstool" => Instrument::Drums,
            "discovery" | "songbird" => Instrument::Chimes,
            "storage" | "nestgate" => Instrument::Strings,
            "ai" | "squirrel" => Instrument::Synth,
            _ => Instrument::Default,
        }
    }

    /// Map health status to pitch
    /// Healthy = harmonic (0.7-0.8), Warning = off-key (0.5-0.6), Critical = dissonant (0.2-0.3)
    fn health_to_pitch(&self, health: &PrimalHealthStatus) -> f32 {
        match health {
            PrimalHealthStatus::Healthy => 0.75,  // Harmonic, pleasant
            PrimalHealthStatus::Warning => 0.55,  // Slightly off-key
            PrimalHealthStatus::Critical => 0.25, // Dissonant, harsh
            PrimalHealthStatus::Unknown => 0.5,   // Neutral
        }
    }

    /// Map activity to volume
    /// For now, we'll use a simple heuristic based on capabilities
    fn activity_to_volume(&self, node: &Node) -> f32 {
        // More capabilities = more active = louder
        let capability_count = node.info.capabilities.len();
        let normalized = (capability_count as f32 / 10.0).min(1.0);

        // Base volume + activity
        0.3 + (normalized * 0.7)
    }

    /// Map 2D position to stereo pan
    /// Left side = -1.0, Center = 0.0, Right side = 1.0
    fn position_to_pan(&self, position: Position) -> f32 {
        // Normalize x position to [-1, 1] range
        // Assuming positions are roughly in [-500, 500] range
        (position.x / 500.0).clamp(-1.0, 1.0)
    }

    /// Set master volume (0.0 to 1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Get master volume
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Enable/disable audio
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if audio is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Generate a textual description of the soundscape (for AI narration)
    pub fn describe_soundscape(&self) -> String {
        let graph = self.graph.read().expect("graph lock poisoned");
        let stats = graph.stats();

        if stats.node_count == 0 {
            return "Ecosystem is silent. No primals detected.".to_string();
        }

        let mut description = format!("Ecosystem soundscape with {} primals. ", stats.node_count);

        // Count by health
        let healthy_count = graph
            .nodes()
            .iter()
            .filter(|n| matches!(n.info.health, PrimalHealthStatus::Healthy))
            .count();
        let warning_count = graph
            .nodes()
            .iter()
            .filter(|n| matches!(n.info.health, PrimalHealthStatus::Warning))
            .count();
        let critical_count = graph
            .nodes()
            .iter()
            .filter(|n| matches!(n.info.health, PrimalHealthStatus::Critical))
            .count();

        if healthy_count == stats.node_count {
            description.push_str("All primals are playing in harmony. ");
        } else {
            description.push_str(&format!(
                "{} healthy, {} warnings, {} critical. ",
                healthy_count, warning_count, critical_count
            ));

            if warning_count > 0 {
                description.push_str("Some notes are slightly off-key. ");
            }
            if critical_count > 0 {
                description.push_str("Dissonant tones detected. ");
            }
        }

        // Describe instruments
        let instruments: std::collections::HashMap<Instrument, usize> = graph
            .nodes()
            .iter()
            .map(|n| self.map_primal_to_instrument(&n.info.primal_type))
            .fold(std::collections::HashMap::new(), |mut acc, inst| {
                *acc.entry(inst).or_insert(0) += 1;
                acc
            });

        description.push_str("Instruments playing: ");
        let mut inst_desc: Vec<String> = instruments
            .iter()
            .map(|(inst, count)| {
                let name = match inst {
                    Instrument::Bass => "bass",
                    Instrument::Drums => "drums",
                    Instrument::Chimes => "chimes",
                    Instrument::Strings => "strings",
                    Instrument::Synth => "synth",
                    Instrument::Default => "default",
                };
                format!("{} {}", count, name)
            })
            .collect();
        inst_desc.sort();
        description.push_str(&inst_desc.join(", "));
        description.push('.');

        description
    }

    /// Get detailed information about a specific node's audio
    pub fn describe_node_audio(&self, node_id: &str) -> Option<String> {
        let graph = self.graph.read().expect("graph lock poisoned");
        let node = graph.get_node(node_id)?;

        let attrs = self.node_to_audio(node);
        let inst_name = match attrs.instrument {
            Instrument::Bass => "deep bass",
            Instrument::Drums => "rhythmic drums",
            Instrument::Chimes => "light chimes",
            Instrument::Strings => "sustained strings",
            Instrument::Synth => "high synth",
            Instrument::Default => "default tone",
        };

        let health_desc = match node.info.health {
            PrimalHealthStatus::Healthy => "harmonic, in-key",
            PrimalHealthStatus::Warning => "slightly off-key",
            PrimalHealthStatus::Critical => "dissonant, harsh",
            PrimalHealthStatus::Unknown => "neutral tone",
        };

        let position_desc = if attrs.pan < -0.3 {
            "positioned to the left"
        } else if attrs.pan > 0.3 {
            "positioned to the right"
        } else {
            "centered"
        };

        Some(format!(
            "{}: Playing {}. Status: {}. Volume: {:.0}%. {}.",
            node.info.name,
            inst_name,
            health_desc,
            attrs.volume * 100.0,
            position_desc
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::{LayoutAlgorithm, PrimalInfo, TopologyEdge};

    fn create_test_graph() -> Arc<RwLock<GraphEngine>> {
        let mut graph = GraphEngine::new();

        // Add BearDog (Security)
        let mut beardog = petal_tongue_core::test_fixtures::primals::test_primal_with_type("beardog-1", "Security");
        beardog.name = "BearDog Security".to_string();
        beardog.capabilities = vec!["auth".to_string(), "encryption".to_string()];
        beardog.health = PrimalHealthStatus::Healthy;
        graph.add_node(beardog);

        // Add ToadStool (Compute)
        let mut toadstool = petal_tongue_core::test_fixtures::primals::test_primal_with_type("toadstool-1", "Compute");
        toadstool.name = "ToadStool Compute".to_string();
        toadstool.capabilities = vec!["runtime".to_string(), "execution".to_string()];
        toadstool.health = PrimalHealthStatus::Warning;
        graph.add_node(toadstool);

        // Add Songbird (Discovery)
        let mut songbird = petal_tongue_core::test_fixtures::primals::test_primal_with_type("songbird-1", "Discovery");
        songbird.name = "Songbird Discovery".to_string();
        songbird.capabilities = vec!["discovery".to_string()];
        songbird.health = PrimalHealthStatus::Healthy;
        graph.add_node(songbird);

        graph.add_edge(TopologyEdge {
            from: "beardog-1".to_string(),
            to: "toadstool-1".to_string(),
            edge_type: "api".to_string(),
            label: None,
        });

        graph.set_layout(LayoutAlgorithm::Circular);
        graph.layout(1);

        Arc::new(RwLock::new(graph))
    }

    #[test]
    fn test_renderer_creation() {
        let graph = create_test_graph();
        let renderer = AudioSonificationRenderer::new(graph);
        assert!((renderer.master_volume() - 0.7).abs() < 0.001);
        assert!(renderer.is_enabled());
    }

    #[test]
    fn test_instrument_mapping() {
        let graph = create_test_graph();
        let renderer = AudioSonificationRenderer::new(graph);

        assert_eq!(
            renderer.map_primal_to_instrument("Security"),
            Instrument::Bass
        );
        assert_eq!(
            renderer.map_primal_to_instrument("Compute"),
            Instrument::Drums
        );
        assert_eq!(
            renderer.map_primal_to_instrument("Discovery"),
            Instrument::Chimes
        );
        assert_eq!(
            renderer.map_primal_to_instrument("Storage"),
            Instrument::Strings
        );
        assert_eq!(renderer.map_primal_to_instrument("AI"), Instrument::Synth);
    }

    #[test]
    fn test_health_to_pitch() {
        let graph = create_test_graph();
        let renderer = AudioSonificationRenderer::new(graph);

        assert_eq!(renderer.health_to_pitch(&PrimalHealthStatus::Healthy), 0.75);
        assert_eq!(renderer.health_to_pitch(&PrimalHealthStatus::Warning), 0.55);
        assert_eq!(
            renderer.health_to_pitch(&PrimalHealthStatus::Critical),
            0.25
        );
        assert_eq!(renderer.health_to_pitch(&PrimalHealthStatus::Unknown), 0.5);
    }

    #[test]
    fn test_position_to_pan() {
        let graph = create_test_graph();
        let renderer = AudioSonificationRenderer::new(graph);

        // Left side
        let left_pos = Position::new_2d(-500.0, 0.0);
        assert_eq!(renderer.position_to_pan(left_pos), -1.0);

        // Center
        let center_pos = Position::new_2d(0.0, 0.0);
        assert_eq!(renderer.position_to_pan(center_pos), 0.0);

        // Right side
        let right_pos = Position::new_2d(500.0, 0.0);
        assert_eq!(renderer.position_to_pan(right_pos), 1.0);
    }

    #[test]
    fn test_generate_audio_attributes() {
        let graph = create_test_graph();
        let renderer = AudioSonificationRenderer::new(graph);

        let attrs = renderer.generate_audio_attributes();
        assert_eq!(attrs.len(), 3);

        // Find BearDog
        let beardog_attrs = attrs.iter().find(|(id, _)| id == "beardog-1").unwrap();
        assert_eq!(beardog_attrs.1.instrument, Instrument::Bass);
        assert_eq!(beardog_attrs.1.pitch, 0.75); // Healthy
    }

    #[test]
    fn test_master_volume() {
        let graph = create_test_graph();
        let mut renderer = AudioSonificationRenderer::new(graph);

        renderer.set_master_volume(0.5);
        assert!((renderer.master_volume() - 0.5).abs() < 0.001);

        // Test clamping
        renderer.set_master_volume(1.5);
        assert!((renderer.master_volume() - 1.0).abs() < 0.001);

        renderer.set_master_volume(-0.5);
        assert!((renderer.master_volume() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_enable_disable() {
        let graph = create_test_graph();
        let mut renderer = AudioSonificationRenderer::new(graph);

        assert!(renderer.is_enabled());

        renderer.set_enabled(false);
        assert!(!renderer.is_enabled());

        renderer.set_enabled(true);
        assert!(renderer.is_enabled());
    }

    #[test]
    fn test_describe_soundscape() {
        let graph = create_test_graph();
        let renderer = AudioSonificationRenderer::new(graph);

        let description = renderer.describe_soundscape();
        assert!(description.contains("3 primals"));
        assert!(description.contains("2 healthy, 1 warnings"));
        assert!(description.contains("bass"));
        assert!(description.contains("drums"));
        assert!(description.contains("chimes"));
    }

    #[test]
    fn test_describe_node_audio() {
        let graph = create_test_graph();
        let renderer = AudioSonificationRenderer::new(graph);

        let description = renderer.describe_node_audio("beardog-1").unwrap();
        assert!(description.contains("BearDog Security"));
        assert!(description.contains("deep bass"));
        assert!(description.contains("harmonic"));

        let description = renderer.describe_node_audio("toadstool-1").unwrap();
        assert!(description.contains("ToadStool Compute"));
        assert!(description.contains("drums"));
        assert!(description.contains("off-key"));

        assert!(renderer.describe_node_audio("nonexistent").is_none());
    }
}
