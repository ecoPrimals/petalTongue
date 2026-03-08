// SPDX-License-Identifier: AGPL-3.0-only
//! Graph Node
//!
//! Represents a single node in the execution graph.

use serde::{Deserialize, Serialize};

/// Graph Node - Single unit of execution
///
/// Represents a node in the collaborative intelligence graph.
/// Node types are discovered at runtime (no hardcoding).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    /// Unique identifier
    pub id: String,

    /// Node type (discovered at runtime from available capabilities)
    pub node_type: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Node properties (type-specific configuration)
    pub properties: serde_json::Value,

    /// Position on canvas (x, y)
    pub position: (f32, f32),

    /// Visual state
    pub state: NodeState,

    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Node execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeState {
    /// Not yet started
    Pending,

    /// Currently executing
    Running {
        /// Progress (0.0 - 1.0)
        progress: u8, // Store as 0-100 to maintain Eq
    },

    /// Completed successfully
    Completed,

    /// Failed with error
    Failed {
        /// Error message
        error: String,
    },

    /// Paused (waiting for user input or condition)
    Paused,
}

impl GraphNode {
    /// Create a new node
    #[must_use]
    pub fn new(id: String, node_type: String) -> Self {
        Self {
            id: id.clone(),
            node_type,
            name: id,
            description: String::new(),
            properties: serde_json::json!({}),
            position: (0.0, 0.0),
            state: NodeState::Pending,
            tags: Vec::new(),
        }
    }

    /// Create a node with name
    #[must_use]
    pub fn with_name(id: String, node_type: String, name: String) -> Self {
        Self {
            id,
            node_type,
            name,
            description: String::new(),
            properties: serde_json::json!({}),
            position: (0.0, 0.0),
            state: NodeState::Pending,
            tags: Vec::new(),
        }
    }

    /// Set position
    #[must_use]
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    /// Set properties
    #[must_use]
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = properties;
        self
    }

    /// Set description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Set tags
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Update state
    pub fn set_state(&mut self, state: NodeState) {
        self.state = state;
    }

    /// Check if node is running
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self.state, NodeState::Running { .. })
    }

    /// Check if node is completed
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self.state, NodeState::Completed)
    }

    /// Check if node is failed
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self.state, NodeState::Failed { .. })
    }

    /// Get progress (0.0 - 1.0)
    #[must_use]
    pub fn progress(&self) -> f32 {
        match &self.state {
            NodeState::Running { progress } => f32::from(*progress) / 100.0,
            NodeState::Completed => 1.0,
            _ => 0.0,
        }
    }

    /// Get display color based on state
    #[must_use]
    pub fn display_color(&self) -> [u8; 3] {
        match &self.state {
            NodeState::Pending => [128, 128, 128],      // Gray
            NodeState::Running { .. } => [0, 128, 255], // Blue
            NodeState::Completed => [0, 255, 0],        // Green
            NodeState::Failed { .. } => [255, 0, 0],    // Red
            NodeState::Paused => [255, 255, 0],         // Yellow
        }
    }

    /// Get display icon based on state
    #[must_use]
    pub fn display_icon(&self) -> &'static str {
        match &self.state {
            NodeState::Pending => "⚪",
            NodeState::Running { .. } => "🔵",
            NodeState::Completed => "✅",
            NodeState::Failed { .. } => "❌",
            NodeState::Paused => "⏸️",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
        assert_eq!(node.id, "node-1");
        assert_eq!(node.node_type, "test-type");
        assert_eq!(node.state, NodeState::Pending);
    }

    #[test]
    fn test_node_builder() {
        let node = GraphNode::new("node-1".to_string(), "test-type".to_string())
            .with_position(100.0, 200.0)
            .with_description("Test description".to_string())
            .with_tags(vec!["tag1".to_string(), "tag2".to_string()]);

        assert_eq!(node.position, (100.0, 200.0));
        assert_eq!(node.description, "Test description");
        assert_eq!(node.tags.len(), 2);
    }

    #[test]
    fn test_node_state() {
        let mut node = GraphNode::new("node-1".to_string(), "test-type".to_string());

        assert!(!node.is_running());
        assert!(!node.is_completed());
        assert!(!node.is_failed());

        node.set_state(NodeState::Running { progress: 50 });
        assert!(node.is_running());
        assert_eq!(node.progress(), 0.5);

        node.set_state(NodeState::Completed);
        assert!(node.is_completed());
        assert_eq!(node.progress(), 1.0);

        node.set_state(NodeState::Failed {
            error: "Test error".to_string(),
        });
        assert!(node.is_failed());
    }

    #[test]
    fn test_display_color() {
        let mut node = GraphNode::new("node-1".to_string(), "test-type".to_string());

        assert_eq!(node.display_color(), [128, 128, 128]); // Pending = Gray

        node.set_state(NodeState::Running { progress: 50 });
        assert_eq!(node.display_color(), [0, 128, 255]); // Running = Blue

        node.set_state(NodeState::Completed);
        assert_eq!(node.display_color(), [0, 255, 0]); // Completed = Green

        node.set_state(NodeState::Failed {
            error: "Test".to_string(),
        });
        assert_eq!(node.display_color(), [255, 0, 0]); // Failed = Red
    }
}
