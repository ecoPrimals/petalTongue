// SPDX-License-Identifier: AGPL-3.0-only
//! Graph engine types: nodes, positions, layout algorithms, and statistics.

use crate::types::PrimalInfo;
use serde::{Deserialize, Serialize};

/// Position in 2D or 3D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Position {
    /// X coordinate in graph space
    pub x: f32,
    /// Y coordinate in graph space
    pub y: f32,
    /// Optional Z coordinate for 3D positioning
    pub z: Option<f32>,
}

impl Position {
    /// Create a 2D position
    #[must_use]
    pub const fn new_2d(x: f32, y: f32) -> Self {
        Self { x, y, z: None }
    }

    /// Create a 3D position
    #[must_use]
    pub const fn new_3d(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z: Some(z) }
    }

    /// Convert to 3D (uses z=0 if 2D)
    #[must_use]
    pub fn to_3d(self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: Some(self.z.unwrap_or(0.0)),
        }
    }

    /// Calculate distance to another position (2D)
    #[must_use]
    pub fn distance_to(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx.hypot(dy)
    }

    /// Calculate distance to another position (3D if available)
    #[must_use]
    pub fn distance_to_3d(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z.unwrap_or(0.0) - other.z.unwrap_or(0.0);
        (dx.mul_add(dx, dy * dy) + dz * dz).sqrt()
    }
}

/// A node in the graph (represents a primal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Node properties (health, capabilities, etc.)
    pub info: PrimalInfo,
    /// Current position (set by layout algorithm)
    pub position: Position,
    /// Velocity (used by physics-based layouts)
    #[serde(skip)]
    pub velocity: Position,
}

impl Node {
    /// Create a new node with default position
    #[must_use]
    pub const fn new(info: PrimalInfo) -> Self {
        Self {
            info,
            position: Position::new_2d(0.0, 0.0),
            velocity: Position::new_2d(0.0, 0.0),
        }
    }

    /// Create a new node with specific position
    #[must_use]
    pub const fn with_position(info: PrimalInfo, position: Position) -> Self {
        Self {
            info,
            position,
            velocity: Position::new_2d(0.0, 0.0),
        }
    }
}

/// Graph statistics
#[derive(Debug, Clone, Copy)]
pub struct GraphStats {
    /// Number of nodes in the graph
    pub node_count: usize,
    /// Number of edges in the graph
    pub edge_count: usize,
    /// Average node degree (connections per node)
    pub avg_degree: f32,
}

/// Layout algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutAlgorithm {
    /// Force-directed layout (physics-based)
    ForceDirected,
    /// Hierarchical layout (tree-like)
    Hierarchical,
    /// Circular layout (nodes in a circle)
    Circular,
    /// Random layout (for testing)
    Random,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PrimalHealthStatus, PrimalInfo};

    fn test_primal_info(id: &str) -> PrimalInfo {
        PrimalInfo::new(
            id,
            "Test",
            "compute",
            "http://localhost:8080",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        )
    }

    #[test]
    fn position_default() {
        let p = Position::default();
        assert!((p.x - 0.0).abs() < f32::EPSILON);
        assert!((p.y - 0.0).abs() < f32::EPSILON);
        assert!(p.z.is_none());
    }

    #[test]
    fn position_distance_2d_same_point() {
        let p = Position::new_2d(1.0, 2.0);
        assert!((p.distance_to(p) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn position_distance_3d_with_2d_fallback() {
        let a = Position::new_2d(0.0, 0.0);
        let b = Position::new_2d(3.0, 4.0);
        assert!((a.distance_to_3d(b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn position_distance_3d_mixed_z() {
        let a = Position::new_2d(0.0, 0.0);
        let b = Position::new_3d(0.0, 0.0, 1.0);
        assert!((a.distance_to_3d(b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn node_new_default_position() {
        let info = test_primal_info("n1");
        let node = Node::new(info);
        assert_eq!(node.info.id.as_str(), "n1");
        assert!((node.position.x - 0.0).abs() < f32::EPSILON);
        assert!((node.position.y - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_algorithm_serialization_roundtrip() {
        for alg in [
            LayoutAlgorithm::ForceDirected,
            LayoutAlgorithm::Hierarchical,
            LayoutAlgorithm::Circular,
            LayoutAlgorithm::Random,
        ] {
            let json = serde_json::to_string(&alg).expect("serialize");
            let restored: LayoutAlgorithm = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(alg, restored);
        }
    }

    #[test]
    fn position_serialization_roundtrip() {
        let pos = Position::new_3d(1.5, 2.5, 3.5);
        let json = serde_json::to_string(&pos).expect("serialize");
        let restored: Position = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(pos, restored);
    }

    #[test]
    fn graph_stats_fields() {
        let stats = GraphStats {
            node_count: 10,
            edge_count: 15,
            avg_degree: 3.0,
        };
        assert_eq!(stats.node_count, 10);
        assert_eq!(stats.edge_count, 15);
        assert!((stats.avg_degree - 3.0).abs() < f32::EPSILON);
    }
}
