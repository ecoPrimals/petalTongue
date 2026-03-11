// SPDX-License-Identifier: AGPL-3.0-only
//! SAME DAVE Channel Model — Neuroanatomy-based signal pathways
//!
//! SAME DAVE is neuroanatomy, not AI — the channel model maps specialized
//! unidirectional pathways analogous to the spinal cord's dorsal/ventral roots.
//! Sensory Afferent pathways carry input TO the proprioception core. Motor
//! Efferent pathways carry commands FROM the core to effectors. Classification
//! nodes along each channel act like nodes of Ranvier, enabling saltatory
//! signal routing.
//!
//! # Neuroanatomy Model
//!
//! - **SAME**: **S**ensory **A**fferent, **M**otor **E**fferent
//! - **DAVE**: **D**orsal **A**fferent, **V**entral **E**fferent
//!
//! Channels are **unidirectional** pathways (axons):
//!
//! - **Afferent** channels carry sensory input TO the proprioception core
//!   (user events, IPC commands, config changes, sensor data).
//! - **Efferent** channels carry motor commands FROM the core to effectors
//!   (panel visibility, zoom, layout, mode changes, display updates).
//!
//! Bidirectional control emerges from paired afferent + efferent channels,
//! not from bidirectional channels.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Direction of signal flow through a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelDirection {
    /// Sensory pathway — signals traveling TO the proprioception core.
    /// Dorsal root (DAVE: Dorsal Afferent).
    Afferent,
    /// Motor pathway — commands traveling FROM the core to effectors.
    /// Ventral root (DAVE: Ventral Efferent).
    Efferent,
}

/// What a channel carries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelModality {
    /// Visual output (display, rendering)
    Visual,
    /// Audio output/input (sonification, microphone)
    Audio,
    /// Pointer input (mouse, touch)
    Pointer,
    /// Keyboard input
    Keyboard,
    /// Inter-process commands (IPC, JSON-RPC)
    Ipc,
    /// Configuration changes (scenario, profile, runtime)
    Config,
    /// UI state control (panels, zoom, layout, mode)
    UiControl,
    /// Custom modality
    Custom(String),
}

/// A classification node along a channel — the Schwann cell / Node of Ranvier.
///
/// Signals jump between these nodes via saltatory conduction.
/// Each node can inspect, filter, or transform signals passing through.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationNode {
    /// Unique identifier for this node.
    pub id: String,
    /// What this node classifies / handles.
    pub classifier: SignalClassifier,
    /// How many signals have passed through this node.
    #[serde(default)]
    pub signals_processed: u64,
    /// How many signals were filtered (dropped) at this node.
    #[serde(default)]
    pub signals_filtered: u64,
}

/// What a classification node handles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalClassifier {
    /// Passes all signals (transparent relay)
    PassAll,
    /// Only passes signals matching a specific category
    Category(String),
    /// Rate limiter — drops signals exceeding a threshold
    RateLimit {
        /// Maximum signals per second
        max_per_second: u32,
    },
    /// Priority filter — only passes signals at or above priority
    Priority {
        /// Minimum priority to pass (0 = all, higher = more selective)
        min_priority: u8,
    },
}

/// A unidirectional signal pathway.
#[derive(Debug, Clone)]
pub struct Channel {
    /// Unique identifier.
    pub id: String,
    /// Direction of signal flow.
    pub direction: ChannelDirection,
    /// What this channel carries.
    pub modality: ChannelModality,
    /// Classification nodes along the pathway (ordered from source to sink).
    pub classification_nodes: Vec<ClassificationNode>,
    /// Total signals that entered this channel.
    pub signals_in: u64,
    /// Total signals that reached the end of this channel.
    pub signals_out: u64,
    /// When this channel was created.
    pub created: Instant,
    /// When the last signal passed through.
    pub last_signal: Option<Instant>,
}

impl Channel {
    /// Create a new channel with no classification nodes.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        direction: ChannelDirection,
        modality: ChannelModality,
    ) -> Self {
        Self {
            id: id.into(),
            direction,
            modality,
            classification_nodes: Vec::new(),
            signals_in: 0,
            signals_out: 0,
            created: Instant::now(),
            last_signal: None,
        }
    }

    /// Add a classification node to the pipeline.
    pub fn add_node(&mut self, node: ClassificationNode) {
        self.classification_nodes.push(node);
    }

    /// Record that a signal entered this channel.
    pub fn record_signal_in(&mut self) {
        self.signals_in += 1;
        self.last_signal = Some(Instant::now());
    }

    /// Record that a signal exited this channel (reached the end).
    pub const fn record_signal_out(&mut self) {
        self.signals_out += 1;
    }

    /// Signal throughput: fraction of input signals that reach the output.
    #[must_use]
    pub fn throughput(&self) -> f32 {
        if self.signals_in == 0 {
            return 0.0;
        }
        #[expect(clippy::cast_precision_loss)]
        let ratio = self.signals_out as f32 / self.signals_in as f32;
        ratio
    }

    /// Whether this channel has seen any activity.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.signals_in > 0
    }

    /// Take a serializable snapshot of channel health.
    #[must_use]
    pub fn snapshot(&self) -> ChannelSnapshot {
        ChannelSnapshot {
            id: self.id.clone(),
            direction: self.direction,
            modality: self.modality.clone(),
            node_count: self.classification_nodes.len(),
            signals_in: self.signals_in,
            signals_out: self.signals_out,
            throughput: self.throughput(),
            active: self.is_active(),
        }
    }
}

/// Serializable snapshot of channel state (for proprioception data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSnapshot {
    /// Channel identifier.
    pub id: String,
    /// Direction of flow.
    pub direction: ChannelDirection,
    /// What this channel carries.
    pub modality: ChannelModality,
    /// Number of classification nodes in the pipeline.
    pub node_count: usize,
    /// Total signals entered.
    pub signals_in: u64,
    /// Total signals exited.
    pub signals_out: u64,
    /// Throughput ratio (0.0 – 1.0).
    pub throughput: f32,
    /// Whether the channel has seen any activity.
    pub active: bool,
}

/// Registry of all afferent and efferent channels.
#[derive(Debug)]
pub struct ChannelRegistry {
    channels: Vec<Channel>,
}

impl ChannelRegistry {
    /// Create an empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            channels: Vec::new(),
        }
    }

    /// Register a new channel.
    pub fn register(&mut self, channel: Channel) {
        self.channels.push(channel);
    }

    /// Get a mutable reference to a channel by ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Channel> {
        self.channels.iter_mut().find(|c| c.id == id)
    }

    /// Get a reference to a channel by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Channel> {
        self.channels.iter().find(|c| c.id == id)
    }

    /// All afferent (sensory) channels.
    pub fn afferent(&self) -> impl Iterator<Item = &Channel> {
        self.channels
            .iter()
            .filter(|c| c.direction == ChannelDirection::Afferent)
    }

    /// All efferent (motor) channels.
    pub fn efferent(&self) -> impl Iterator<Item = &Channel> {
        self.channels
            .iter()
            .filter(|c| c.direction == ChannelDirection::Efferent)
    }

    /// Snapshot all channels.
    #[must_use]
    pub fn snapshots(&self) -> Vec<ChannelSnapshot> {
        self.channels.iter().map(Channel::snapshot).collect()
    }

    /// Number of registered channels.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.channels.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the standard channel set for a petalTongue instance.
#[must_use]
pub fn standard_channels() -> ChannelRegistry {
    let mut registry = ChannelRegistry::new();

    // Afferent (sensory) channels
    registry.register(Channel::new(
        "keyboard-afferent",
        ChannelDirection::Afferent,
        ChannelModality::Keyboard,
    ));
    registry.register(Channel::new(
        "pointer-afferent",
        ChannelDirection::Afferent,
        ChannelModality::Pointer,
    ));
    registry.register(Channel::new(
        "ipc-afferent",
        ChannelDirection::Afferent,
        ChannelModality::Ipc,
    ));
    registry.register(Channel::new(
        "config-afferent",
        ChannelDirection::Afferent,
        ChannelModality::Config,
    ));

    // Efferent (motor) channels
    registry.register(Channel::new(
        "visual-efferent",
        ChannelDirection::Efferent,
        ChannelModality::Visual,
    ));
    registry.register(Channel::new(
        "audio-efferent",
        ChannelDirection::Efferent,
        ChannelModality::Audio,
    ));
    registry.register(Channel::new(
        "ui-control-efferent",
        ChannelDirection::Efferent,
        ChannelModality::UiControl,
    ));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_direction_serde() {
        let json = serde_json::to_string(&ChannelDirection::Afferent).unwrap();
        let back: ChannelDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ChannelDirection::Afferent);
    }

    #[test]
    fn channel_throughput() {
        let mut ch = Channel::new(
            "test",
            ChannelDirection::Afferent,
            ChannelModality::Keyboard,
        );
        assert!((ch.throughput() - 0.0).abs() < f32::EPSILON);

        for _ in 0..10 {
            ch.record_signal_in();
        }
        for _ in 0..8 {
            ch.record_signal_out();
        }
        let t = ch.throughput();
        assert!((t - 0.8).abs() < 0.01);
    }

    #[test]
    fn standard_channels_coverage() {
        let reg = standard_channels();
        assert_eq!(reg.afferent().count(), 4);
        assert_eq!(reg.efferent().count(), 3);
        assert_eq!(reg.len(), 7);
    }

    #[test]
    fn snapshot_roundtrip() {
        let ch = Channel::new(
            "snap-test",
            ChannelDirection::Efferent,
            ChannelModality::Visual,
        );
        let snap = ch.snapshot();
        let json = serde_json::to_string(&snap).unwrap();
        let back: ChannelSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "snap-test");
        assert_eq!(back.direction, ChannelDirection::Efferent);
    }

    #[test]
    fn classification_node_filtering() {
        let mut ch = Channel::new("filtered", ChannelDirection::Afferent, ChannelModality::Ipc);
        ch.add_node(ClassificationNode {
            id: "rate-limiter".into(),
            classifier: SignalClassifier::RateLimit { max_per_second: 60 },
            signals_processed: 0,
            signals_filtered: 0,
        });
        assert_eq!(ch.classification_nodes.len(), 1);
        assert_eq!(ch.snapshot().node_count, 1);
    }

    #[test]
    fn registry_lookup() {
        let reg = standard_channels();
        assert!(reg.get("keyboard-afferent").is_some());
        assert!(reg.get("ui-control-efferent").is_some());
        assert!(reg.get("nonexistent").is_none());
    }
}
