// SPDX-License-Identifier: AGPL-3.0-only
//! Graph Edge
//!
//! Represents dependencies between nodes in the execution graph.

use serde::{Deserialize, Serialize};

/// Graph Edge - Dependency between nodes
///
/// Represents a directed edge (dependency) from one node to another.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    /// Unique identifier
    pub id: String,

    /// Source node ID
    pub from: String,

    /// Target node ID
    pub to: String,

    /// Type of dependency
    pub dependency_type: DependencyType,

    /// Optional label
    pub label: Option<String>,
}

/// Type of dependency between nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    /// Sequential execution (A must complete before B starts)
    Sequential,

    /// Data dependency (B needs data from A)
    DataFlow,

    /// Optional dependency (B prefers A, but can run without it)
    Optional,

    /// Conditional dependency (B runs only if A meets condition)
    Conditional {
        /// Condition to evaluate
        condition: String,
    },
}

impl GraphEdge {
    /// Create a new edge
    #[must_use]
    pub fn new(id: String, from: String, to: String, dependency_type: DependencyType) -> Self {
        Self {
            id,
            from,
            to,
            dependency_type,
            label: None,
        }
    }

    /// Create edge with label
    #[must_use]
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Check if this is a required dependency
    #[must_use]
    pub fn is_required(&self) -> bool {
        !matches!(self.dependency_type, DependencyType::Optional)
    }

    /// Get display color for edge type
    #[must_use]
    pub fn display_color(&self) -> [u8; 3] {
        match &self.dependency_type {
            DependencyType::Sequential => [0, 0, 0],             // Black
            DependencyType::DataFlow => [0, 0, 255],             // Blue
            DependencyType::Optional => [128, 128, 128],         // Gray
            DependencyType::Conditional { .. } => [255, 165, 0], // Orange
        }
    }

    /// Get display style (solid, dashed, etc)
    #[must_use]
    pub fn display_style(&self) -> EdgeStyle {
        match &self.dependency_type {
            DependencyType::Sequential => EdgeStyle::Solid,
            DependencyType::DataFlow => EdgeStyle::Solid,
            DependencyType::Optional => EdgeStyle::Dashed,
            DependencyType::Conditional { .. } => EdgeStyle::Dotted,
        }
    }
}

/// Edge display style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeStyle {
    /// Solid line
    Solid,
    /// Dashed line
    Dashed,
    /// Dotted line
    Dotted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_edge() {
        let edge = GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        );

        assert_eq!(edge.id, "edge-1");
        assert_eq!(edge.from, "node-1");
        assert_eq!(edge.to, "node-2");
        assert!(edge.is_required());
    }

    #[test]
    fn test_edge_with_label() {
        let edge = GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::DataFlow,
        )
        .with_label("data-output".to_string());

        assert_eq!(edge.label, Some("data-output".to_string()));
    }

    #[test]
    fn test_optional_dependency() {
        let edge = GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Optional,
        );

        assert!(!edge.is_required());
        assert_eq!(edge.display_style(), EdgeStyle::Dashed);
    }

    #[test]
    fn test_conditional_dependency() {
        let edge = GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Conditional {
                condition: "status == 'success'".to_string(),
            },
        );

        assert!(edge.is_required());
        assert_eq!(edge.display_style(), EdgeStyle::Dotted);
    }
}
