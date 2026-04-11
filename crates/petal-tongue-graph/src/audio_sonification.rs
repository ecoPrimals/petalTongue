// SPDX-License-Identifier: AGPL-3.0-or-later
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
    pub const fn new(graph: Arc<RwLock<GraphEngine>>) -> Self {
        Self {
            graph,
            master_volume: 0.7,
            enabled: true,
        }
    }

    /// Generate audio attributes for all nodes
    #[must_use]
    pub fn generate_audio_attributes(&self) -> Vec<(String, AudioAttributes)> {
        let Ok(graph) = self.graph.read() else {
            return Vec::new();
        };

        graph
            .nodes()
            .iter()
            .map(|node| {
                let attrs = self.node_to_audio(node);
                (node.info.id.as_str().to_string(), attrs)
            })
            .collect()
    }

    /// Convert a node to audio attributes
    fn node_to_audio(&self, node: &Node) -> AudioAttributes {
        AudioAttributes {
            instrument: self.map_primal_to_instrument(&node.info.primal_type),
            pitch: self.health_to_pitch(node.info.health),
            volume: self.activity_to_volume(node) * self.master_volume,
            pan: self.position_to_pan(node.position),
        }
    }

    /// Map primal type to instrument
    #[expect(clippy::unused_self, reason = "trait-style method for consistency")]
    fn map_primal_to_instrument(&self, primal_type: &str) -> Instrument {
        match primal_type.to_lowercase().as_str() {
            "security" => Instrument::Bass,
            "compute" => Instrument::Drums,
            "discovery" => Instrument::Chimes,
            "storage" => Instrument::Strings,
            "ai" => Instrument::Synth,
            _ => Instrument::Default,
        }
    }

    /// Map health status to pitch
    /// Healthy = harmonic (0.7-0.8), Warning = off-key (0.5-0.6), Critical = dissonant (0.2-0.3)
    #[expect(clippy::unused_self, reason = "trait-style method for consistency")]
    const fn health_to_pitch(&self, health: PrimalHealthStatus) -> f32 {
        match health {
            PrimalHealthStatus::Healthy => 0.75,  // Harmonic, pleasant
            PrimalHealthStatus::Warning => 0.55,  // Slightly off-key
            PrimalHealthStatus::Critical => 0.25, // Dissonant, harsh
            PrimalHealthStatus::Unknown => 0.5,   // Neutral
        }
    }

    /// Map activity to volume
    /// For now, we'll use a simple heuristic based on capabilities
    #[expect(clippy::unused_self, reason = "trait-style method for consistency")]
    fn activity_to_volume(&self, node: &Node) -> f32 {
        // More capabilities = more active = louder
        let capability_count = node.info.capabilities.len();
        #[expect(
            clippy::cast_precision_loss,
            reason = "normalization for audio volume, precision loss acceptable"
        )]
        let normalized = (capability_count as f32 / 10.0).min(1.0);

        // Base volume + activity
        0.3 + (normalized * 0.7)
    }

    /// Map 2D position to stereo pan
    /// Left side = -1.0, Center = 0.0, Right side = 1.0
    #[expect(clippy::unused_self, reason = "trait-style method for consistency")]
    fn position_to_pan(&self, position: Position) -> f32 {
        // Normalize x position to [-1, 1] range
        // Assuming positions are roughly in [-500, 500] range
        (position.x / 500.0).clamp(-1.0, 1.0)
    }

    /// Set master volume (0.0 to 1.0)
    pub const fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Get master volume
    #[must_use]
    pub const fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Enable/disable audio
    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if audio is enabled
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Generate a textual description of the soundscape (for AI narration)
    #[must_use]
    pub fn describe_soundscape(&self) -> String {
        let Ok(graph) = self.graph.read() else {
            return "Graph unavailable.".to_string();
        };
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
                "{healthy_count} healthy, {warning_count} warnings, {critical_count} critical. "
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
                format!("{count} {name}")
            })
            .collect();
        inst_desc.sort();
        description.push_str(&inst_desc.join(", "));
        description.push('.');

        description
    }

    /// Get detailed information about a specific node's audio
    #[must_use]
    pub fn describe_node_audio(&self, node_id: &str) -> Option<String> {
        let node = self.graph.read().ok()?.get_node(node_id)?.clone();
        let attrs = self.node_to_audio(&node);
        let name = node.info.name.clone();
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

        let inst_name = match attrs.instrument {
            Instrument::Bass => "deep bass",
            Instrument::Drums => "rhythmic drums",
            Instrument::Chimes => "light chimes",
            Instrument::Strings => "sustained strings",
            Instrument::Synth => "high synth",
            Instrument::Default => "default tone",
        };

        Some(format!(
            "{}: Playing {}. Status: {}. Volume: {:.0}%. {}.",
            name,
            inst_name,
            health_desc,
            attrs.volume * 100.0,
            position_desc,
        ))
    }
}

#[cfg(test)]
#[path = "audio_sonification_tests.rs"]
mod tests;
