// SPDX-License-Identifier: AGPL-3.0-only
//! Graph Builder Types
//!
//! Visual graph representation types for Neural API graph construction.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A visual graph representation for Neural API graphs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualGraph {
    /// Unique graph ID
    pub id: String,

    /// Human-readable graph name
    pub name: String,

    /// Graph description
    pub description: Option<String>,

    /// All nodes in the graph
    pub nodes: Vec<GraphNode>,

    /// Edges connecting nodes
    pub edges: Vec<GraphEdge>,

    /// Layout metadata (positions, zoom, etc.)
    pub layout: GraphLayout,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
}

/// A node in the visual graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node ID (within this graph)
    pub id: String,

    /// Node type (`primal_start`, verification, `wait_for`, conditional)
    pub node_type: NodeType,

    /// Display position on canvas
    pub position: Vec2,

    /// Node-specific parameters
    pub parameters: HashMap<String, String>,

    /// Visual state (transient, not persisted)
    #[serde(skip)]
    pub visual_state: NodeVisualState,
}

/// Node types supported by Neural API
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Start a primal
    PrimalStart,

    /// Verify primal health
    Verification,

    /// Wait for condition
    WaitFor,

    /// Conditional branch
    Conditional,
}

/// Edge connecting two nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,

    /// Target node ID
    pub to: String,

    /// Edge type (dependency, data flow, etc.)
    pub edge_type: EdgeType,
}

/// Type of relationship between graph nodes
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    /// Node B depends on Node A (execution order)
    Dependency,

    /// Data flows from A to B
    DataFlow,
}

/// Layout metadata for the graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphLayout {
    /// Camera position
    pub camera_position: Vec2,

    /// Zoom level (1.0 = 100%)
    pub zoom: f32,

    /// Grid enabled
    pub grid_enabled: bool,

    /// Grid size
    pub grid_size: f32,

    /// Snap to grid
    pub snap_to_grid: bool,
}

impl Default for GraphLayout {
    fn default() -> Self {
        Self {
            camera_position: Vec2 { x: 0.0, y: 0.0 },
            zoom: 1.0,
            grid_enabled: true,
            grid_size: 50.0,
            snap_to_grid: true,
        }
    }
}

/// 2D vector for positions and offsets
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vec2 {
    /// X coordinate (horizontal position)
    pub x: f32,
    /// Y coordinate (vertical position)
    pub y: f32,
}

impl Vec2 {
    /// Create a new 2D vector with the given coordinates
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a zero vector (origin point)
    #[must_use]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Distance to another point
    #[must_use]
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx.hypot(dy)
    }

    /// Snap to grid
    #[must_use]
    pub fn snap(&self, grid_size: f32) -> Self {
        Self {
            x: (self.x / grid_size).round() * grid_size,
            y: (self.y / grid_size).round() * grid_size,
        }
    }
}

/// Visual state for a node (transient)
#[derive(Clone, Debug, Default)]
pub struct NodeVisualState {
    /// Node is selected
    pub selected: bool,

    /// Node has validation errors
    pub has_error: bool,

    /// Error message (if any)
    pub error_message: Option<String>,

    /// Node is being hovered
    pub hovered: bool,

    /// Node is being dragged
    pub dragging: bool,
}

impl GraphNode {
    /// Create a new node
    #[must_use]
    pub fn new(node_type: NodeType, position: Vec2) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            node_type,
            position,
            parameters: HashMap::new(),
            visual_state: NodeVisualState::default(),
        }
    }

    /// Set a parameter value
    pub fn set_parameter(&mut self, key: String, value: String) {
        self.parameters.insert(key, value);
    }

    /// Get a parameter value
    #[must_use]
    pub fn get_parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }

    /// Check if all required parameters are set
    #[must_use]
    pub fn has_all_required_parameters(&self) -> bool {
        let required = self.node_type.required_parameters();
        required.iter().all(|k| self.parameters.contains_key(*k))
    }

    /// Get missing required parameters
    #[must_use]
    pub fn missing_parameters(&self) -> Vec<&'static str> {
        let required = self.node_type.required_parameters();
        required
            .iter()
            .filter(|k| !self.parameters.contains_key(**k))
            .copied()
            .collect()
    }
}

impl NodeType {
    /// Get human-readable name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::PrimalStart => "Start Primal",
            Self::Verification => "Verify Health",
            Self::WaitFor => "Wait For",
            Self::Conditional => "Conditional",
        }
    }

    /// Get icon for node type
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::PrimalStart => "🚀",
            Self::Verification => "✅",
            Self::WaitFor => "⏳",
            Self::Conditional => "❓",
        }
    }

    /// Get description
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::PrimalStart => "Start a primal service",
            Self::Verification => "Verify primal health status",
            Self::WaitFor => "Wait for condition to be met",
            Self::Conditional => "Branch based on condition",
        }
    }

    /// Get required parameters
    #[must_use]
    pub const fn required_parameters(&self) -> &'static [&'static str] {
        match self {
            Self::PrimalStart => &["primal_name", "family_id"],
            Self::Verification => &["primal_name", "timeout"],
            Self::WaitFor => &["condition", "timeout"],
            Self::Conditional => &["condition"],
        }
    }
}

impl GraphEdge {
    /// Create a new edge
    #[must_use]
    pub const fn new(from: String, to: String, edge_type: EdgeType) -> Self {
        Self {
            from,
            to,
            edge_type,
        }
    }

    /// Create a dependency edge
    #[must_use]
    pub const fn dependency(from: String, to: String) -> Self {
        Self::new(from, to, EdgeType::Dependency)
    }
}
