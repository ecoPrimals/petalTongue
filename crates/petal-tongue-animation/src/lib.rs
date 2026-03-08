// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
//! # petal-tongue-animation
//!
//! Flow animation and data visualization for petalTongue.
//!
//! This crate provides animation capabilities for visualizing:
//! - Data flows between primals
//! - Message passing along edges
//! - Activity indicators (pulsing, flowing)
//! - Bandwidth usage visualization
//! - Temporal patterns (bursts, steady-state)
//! - Flower opening (awakening experience)

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

pub mod flower;
pub mod visual_flower;

use std::time::{Duration, Instant};

/// Represents a particle flowing along an edge
#[derive(Debug, Clone)]
pub struct FlowParticle {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Progress along edge (0.0 to 1.0)
    pub progress: f32,
    /// Particle color (RGB)
    pub color: (u8, u8, u8),
    /// Particle size
    pub size: f32,
    /// Speed multiplier
    pub speed: f32,
    /// Creation time
    pub created: Instant,
}

impl FlowParticle {
    /// Create a new flow particle
    #[must_use]
    pub fn new(source: String, target: String) -> Self {
        Self {
            source,
            target,
            progress: 0.0,
            color: (100, 200, 255), // Blue by default
            size: 3.0,
            speed: 1.0,
            created: Instant::now(),
        }
    }

    /// Update particle progress based on elapsed time
    pub fn update(&mut self, delta_time: Duration) {
        let delta_seconds = delta_time.as_secs_f32();
        self.progress += delta_seconds * self.speed * 0.5; // 0.5 = base speed

        // Wrap or remove if complete
        if self.progress > 1.0 {
            self.progress = 0.0; // Loop for now
        }
    }

    /// Check if particle has completed its journey
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }
}

/// Pulse animation for node activity
#[derive(Debug, Clone)]
pub struct NodePulse {
    /// Node ID
    pub node_id: String,
    /// Current pulse phase (0.0 to 1.0)
    pub phase: f32,
    /// Pulse frequency (pulses per second)
    pub frequency: f32,
    /// Pulse intensity (0.0 to 1.0)
    pub intensity: f32,
    /// Last update time
    pub last_update: Instant,
}

impl NodePulse {
    /// Create a new node pulse animation
    #[must_use]
    pub fn new(node_id: String, frequency: f32) -> Self {
        Self {
            node_id,
            phase: 0.0,
            frequency,
            intensity: 1.0,
            last_update: Instant::now(),
        }
    }

    /// Update pulse phase
    pub fn update(&mut self, delta_time: Duration) {
        let delta_seconds = delta_time.as_secs_f32();
        self.phase += delta_seconds * self.frequency;

        // Wrap phase at 1.0
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }
    }

    /// Get current pulse radius multiplier (oscillates between 1.0 and 1.0 + intensity)
    #[must_use]
    pub fn radius_multiplier(&self) -> f32 {
        1.0 + (self.phase * std::f32::consts::TAU).sin() * 0.5 * self.intensity
    }

    /// Get current pulse alpha (fades in and out)
    #[must_use]
    pub fn alpha(&self) -> f32 {
        (self.phase * std::f32::consts::TAU).sin().abs() * self.intensity
    }
}

/// Animation state for an edge (connection between nodes)
#[derive(Debug, Clone)]
pub struct EdgeAnimation {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Flow particles on this edge
    pub particles: Vec<FlowParticle>,
    /// Bandwidth usage (0.0 to 1.0)
    pub bandwidth: f32,
    /// Edge thickness multiplier based on activity
    pub thickness_multiplier: f32,
}

impl EdgeAnimation {
    /// Create new edge animation
    #[must_use]
    pub fn new(source: String, target: String) -> Self {
        Self {
            source,
            target,
            particles: Vec::new(),
            bandwidth: 0.0,
            thickness_multiplier: 1.0,
        }
    }

    /// Spawn a new particle
    pub fn spawn_particle(&mut self) {
        let particle = FlowParticle::new(self.source.clone(), self.target.clone());
        self.particles.push(particle);
    }

    /// Update all particles
    pub fn update(&mut self, delta_time: Duration) {
        // Update existing particles
        for particle in &mut self.particles {
            particle.update(delta_time);
        }

        // Remove completed particles (if using one-shot mode)
        // Currently particles loop, so this isn't needed
        // self.particles.retain(|p| !p.is_complete());

        // Update thickness based on bandwidth
        self.thickness_multiplier = 1.0 + self.bandwidth * 2.0;
    }
}

/// Main animation engine managing all flow animations
#[derive(Debug)]
pub struct AnimationEngine {
    /// Node pulse animations
    pub node_pulses: Vec<NodePulse>,
    /// Edge animations
    pub edge_animations: Vec<EdgeAnimation>,
    /// Last update time
    pub last_update: Instant,
    /// Particle spawn rate (particles per second per edge)
    pub spawn_rate: f32,
    /// Time accumulator for spawning
    spawn_accumulator: f32,
}

impl AnimationEngine {
    /// Create a new animation engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            node_pulses: Vec::new(),
            edge_animations: Vec::new(),
            last_update: Instant::now(),
            spawn_rate: 2.0, // 2 particles per second per edge
            spawn_accumulator: 0.0,
        }
    }

    /// Add or update a node pulse
    pub fn set_node_pulse(&mut self, node_id: String, frequency: f32) {
        if let Some(pulse) = self.node_pulses.iter_mut().find(|p| p.node_id == node_id) {
            pulse.frequency = frequency;
        } else {
            self.node_pulses.push(NodePulse::new(node_id, frequency));
        }
    }

    /// Remove a node pulse
    pub fn remove_node_pulse(&mut self, node_id: &str) {
        self.node_pulses.retain(|p| p.node_id != node_id);
    }

    /// Add or update an edge animation
    pub fn set_edge_animation(&mut self, source: String, target: String, bandwidth: f32) {
        let key = format!("{source}->{target}");

        if let Some(anim) = self
            .edge_animations
            .iter_mut()
            .find(|a| format!("{}->{}", a.source, a.target) == key)
        {
            anim.bandwidth = bandwidth;
        } else {
            let mut anim = EdgeAnimation::new(source, target);
            anim.bandwidth = bandwidth;
            self.edge_animations.push(anim);
        }
    }

    /// Remove an edge animation
    pub fn remove_edge_animation(&mut self, source: &str, target: &str) {
        self.edge_animations
            .retain(|a| !(a.source == source && a.target == target));
    }

    /// Update all animations
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update);
        let delta_seconds = delta_time.as_secs_f32();

        // Update node pulses
        for pulse in &mut self.node_pulses {
            pulse.update(delta_time);
        }

        // Update edge animations
        for anim in &mut self.edge_animations {
            anim.update(delta_time);
        }

        // Spawn new particles based on spawn rate
        self.spawn_accumulator += delta_seconds * self.spawn_rate;
        while self.spawn_accumulator >= 1.0 {
            // Spawn one particle on each active edge
            for anim in &mut self.edge_animations {
                if anim.bandwidth > 0.1 {
                    // Only spawn if there's activity
                    anim.spawn_particle();
                }
            }
            self.spawn_accumulator -= 1.0;
        }

        self.last_update = now;
    }

    /// Clear all animations
    pub fn clear(&mut self) {
        self.node_pulses.clear();
        self.edge_animations.clear();
    }
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export flower animation
pub use flower::{FlowerAnimation, FlowerFrame, FlowerState, generate_awakening_sequence};
pub use visual_flower::VisualFlowerRenderer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_particle_creation() {
        let particle = FlowParticle::new("node1".to_string(), "node2".to_string());
        assert_eq!(particle.source, "node1");
        assert_eq!(particle.target, "node2");
        assert!((particle.progress - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flow_particle_update() {
        let mut particle = FlowParticle::new("node1".to_string(), "node2".to_string());
        particle.update(Duration::from_secs(1));
        assert!(particle.progress > 0.0);
        assert!(particle.progress < 1.0);
    }

    #[test]
    fn test_node_pulse_creation() {
        let pulse = NodePulse::new("node1".to_string(), 1.0);
        assert_eq!(pulse.node_id, "node1");
        assert!((pulse.frequency - 1.0).abs() < f32::EPSILON);
        assert!((pulse.phase - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_node_pulse_update() {
        let mut pulse = NodePulse::new("node1".to_string(), 2.0);
        pulse.update(Duration::from_millis(250)); // 0.25 seconds
        assert!((pulse.phase - 0.5).abs() < 0.01); // Should be at 0.5 phase
    }

    #[test]
    fn test_edge_animation() {
        let mut edge = EdgeAnimation::new("node1".to_string(), "node2".to_string());
        assert_eq!(edge.particles.len(), 0);

        edge.spawn_particle();
        assert_eq!(edge.particles.len(), 1);

        edge.update(Duration::from_secs(1));
        assert!(edge.particles[0].progress > 0.0);
    }

    #[test]
    fn test_animation_engine() {
        let mut engine = AnimationEngine::new();

        engine.set_node_pulse("node1".to_string(), 1.0);
        assert_eq!(engine.node_pulses.len(), 1);

        engine.set_edge_animation("node1".to_string(), "node2".to_string(), 0.5);
        assert_eq!(engine.edge_animations.len(), 1);

        engine.update();
        // Verify no crashes
    }
}
