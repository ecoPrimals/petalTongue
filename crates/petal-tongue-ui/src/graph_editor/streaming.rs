// SPDX-License-Identifier: AGPL-3.0-only
//! Real-Time Streaming for Graph Execution
//!
//! Provides WebSocket-based streaming for live graph execution updates,
//! AI reasoning, and collaborative editing.
//!
//! # Philosophy
//!
//! TRUE PRIMAL streaming:
//! - No hardcoded endpoints (discover at runtime)
//! - Graceful degradation (works without WebSocket)
//! - Self-stable (can function standalone)
//! - Bidirectional (human and AI communicate as equals)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, info};

/// Stream message - Real-time updates for graph execution
///
/// All messages are bidirectional: from biomeOS to petalTongue,
/// and from petalTongue to biomeOS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    /// Node status update (running, completed, failed, etc)
    NodeStatus {
        /// Graph identifier
        graph_id: String,
        /// Node identifier
        node_id: String,
        /// Current node status
        status: NodeStatus,
        /// When status changed
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Progress update (percentage complete)
    Progress {
        /// Graph identifier
        graph_id: String,
        /// Node identifier
        node_id: String,
        /// Progress fraction (0.0 - 1.0)
        progress: f32,
        /// Progress description
        message: String,
        /// When progress updated
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// AI reasoning update (why decisions are made)
    Reasoning {
        /// Graph identifier
        graph_id: String,
        /// AI reasoning explanation
        reasoning: AIReasoning,
        /// When reasoning generated
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Resource usage update
    ResourceUsage {
        /// Graph identifier
        graph_id: String,
        /// Node identifier
        node_id: String,
        /// Resource usage metrics
        resources: ResourceUsage,
        /// When measured
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Error update
    Error {
        /// Graph identifier
        graph_id: String,
        /// Node that errored (if specific)
        node_id: Option<String>,
        /// Error details
        error: ErrorInfo,
        /// When error occurred
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Graph modification (from user)
    GraphModification {
        /// Graph identifier
        graph_id: String,
        /// Modification details
        modification: GraphModification,
        /// User who made modification
        user_id: Option<String>,
        /// When modified
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Heartbeat (keep connection alive)
    Heartbeat {
        /// Heartbeat timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Node status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    /// Node is pending execution
    Pending,
    /// Node is currently running
    Running {
        /// Progress percentage (0-100)
        progress: u8,
    },
    /// Node completed successfully
    Completed,
    /// Node failed with error
    Failed {
        /// Error message
        error: String,
    },
    /// Node execution paused
    Paused,
}

/// AI reasoning - Transparent decision explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReasoning {
    /// Decision made
    pub decision: String,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Why this decision?
    pub rationale: Vec<String>,

    /// Alternative options considered
    pub alternatives: Vec<Alternative>,

    /// Data sources used
    pub data_sources: Vec<String>,

    /// Historical patterns referenced
    pub patterns: Vec<Pattern>,
}

/// Alternative option considered by AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    /// Description of alternative
    pub description: String,
    /// Confidence in this alternative
    pub confidence: f32,
    /// Why this wasn't chosen
    pub reason_not_chosen: String,
}

/// Historical pattern referenced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Pattern description
    pub description: String,
    /// Pattern source (`user_history`, community, system)
    pub source: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
}

/// Resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f32,
    /// Memory usage in megabytes
    pub memory_mb: u64,
    /// Disk I/O in MB/s
    pub disk_io_mbps: f32,
    /// Network usage in MB/s
    pub network_mbps: f32,
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error type identifier
    pub error_type: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    pub details: Option<String>,
    /// Whether error is recoverable
    pub recoverable: bool,
    /// Suggested action to resolve error
    pub suggested_action: Option<String>,
}

/// Graph modification (from user)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum GraphModification {
    /// Add new node to graph
    AddNode {
        /// Node data
        node: serde_json::Value,
    },
    /// Remove node from graph
    RemoveNode {
        /// Node to remove
        node_id: String,
    },
    /// Modify existing node
    ModifyNode {
        /// Node to modify
        node_id: String,
        /// Changes to apply
        changes: serde_json::Value,
    },
    /// Add edge between nodes
    AddEdge {
        /// Source node
        from: String,
        /// Target node
        to: String,
    },
    /// Remove edge from graph
    RemoveEdge {
        /// Edge to remove
        edge_id: String,
    },
}

/// Stream handler - Manages WebSocket connections and message routing
///
/// This is the central hub for all real-time communication.
pub struct StreamHandler {
    /// Broadcast channel for sending messages to all subscribers
    tx: broadcast::Sender<StreamMessage>,

    /// Active graph executions (`graph_id` -> execution state)
    executions: Arc<RwLock<std::collections::HashMap<String, ExecutionState>>>,
}

/// Execution state for a graph
#[derive(Debug, Clone)]
pub struct ExecutionState {
    /// Unique graph identifier
    pub graph_id: String,
    /// Timestamp when execution started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Currently executing node (if any)
    pub current_node: Option<String>,
    /// Nodes that have completed successfully
    pub completed_nodes: Vec<String>,
    /// Nodes that have failed
    pub failed_nodes: Vec<String>,
}

impl StreamHandler {
    /// Create a new stream handler
    #[must_use]
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1000); // Buffer 1000 messages

        Self {
            tx,
            executions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Subscribe to stream updates
    ///
    /// Returns a receiver that will receive all future messages.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<StreamMessage> {
        self.tx.subscribe()
    }

    /// Send a message to all subscribers
    pub async fn send(&self, message: StreamMessage) -> Result<()> {
        debug!("Sending stream message: {:?}", message);

        // Update execution state based on message
        self.update_execution_state(&message).await?;

        // Broadcast to all subscribers
        self.tx
            .send(message)
            .context("Failed to broadcast message")?;

        Ok(())
    }

    /// Start tracking a graph execution
    pub async fn start_execution(&self, graph_id: String) -> Result<()> {
        info!("Starting execution tracking for graph '{}'", graph_id);

        let state = ExecutionState {
            graph_id: graph_id.clone(),
            started_at: chrono::Utc::now(),
            current_node: None,
            completed_nodes: Vec::new(),
            failed_nodes: Vec::new(),
        };

        let mut executions = self.executions.write().await;
        executions.insert(graph_id, state);

        Ok(())
    }

    /// Stop tracking a graph execution
    pub async fn stop_execution(&self, graph_id: &str) -> Result<()> {
        info!("Stopping execution tracking for graph '{}'", graph_id);

        let mut executions = self.executions.write().await;
        executions.remove(graph_id);

        Ok(())
    }

    /// Get execution state for a graph
    pub async fn get_execution_state(&self, graph_id: &str) -> Option<ExecutionState> {
        let executions = self.executions.read().await;
        executions.get(graph_id).cloned()
    }

    /// Update execution state based on message
    async fn update_execution_state(&self, message: &StreamMessage) -> Result<()> {
        if let StreamMessage::NodeStatus {
            graph_id,
            node_id,
            status,
            ..
        } = message
        {
            let mut executions = self.executions.write().await;
            if let Some(state) = executions.get_mut(graph_id) {
                match status {
                    NodeStatus::Running { .. } => {
                        state.current_node = Some(node_id.clone());
                    }
                    NodeStatus::Completed => {
                        state.completed_nodes.push(node_id.clone());
                        state.current_node = None;
                    }
                    NodeStatus::Failed { .. } => {
                        state.failed_nodes.push(node_id.clone());
                        state.current_node = None;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Send node status update
    pub async fn send_node_status(
        &self,
        graph_id: String,
        node_id: String,
        status: NodeStatus,
    ) -> Result<()> {
        self.send(StreamMessage::NodeStatus {
            graph_id,
            node_id,
            status,
            timestamp: chrono::Utc::now(),
        })
        .await
    }

    /// Send progress update
    pub async fn send_progress(
        &self,
        graph_id: String,
        node_id: String,
        progress: f32,
        message: String,
    ) -> Result<()> {
        self.send(StreamMessage::Progress {
            graph_id,
            node_id,
            progress: progress.clamp(0.0, 1.0),
            message,
            timestamp: chrono::Utc::now(),
        })
        .await
    }

    /// Send AI reasoning update
    pub async fn send_reasoning(&self, graph_id: String, reasoning: AIReasoning) -> Result<()> {
        self.send(StreamMessage::Reasoning {
            graph_id,
            reasoning,
            timestamp: chrono::Utc::now(),
        })
        .await
    }

    /// Send error update
    pub async fn send_error(
        &self,
        graph_id: String,
        node_id: Option<String>,
        error: ErrorInfo,
    ) -> Result<()> {
        self.send(StreamMessage::Error {
            graph_id,
            node_id,
            error,
            timestamp: chrono::Utc::now(),
        })
        .await
    }

    /// Get subscriber count
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for StreamHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_stream_handler_creation() {
        let handler = StreamHandler::new();
        assert_eq!(handler.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_subscribe_and_send() {
        let handler = StreamHandler::new();
        let mut rx = handler.subscribe();

        assert_eq!(handler.subscriber_count(), 1);

        // Send a message
        handler
            .send_node_status(
                "test-graph".to_string(),
                "node-1".to_string(),
                NodeStatus::Running { progress: 50 },
            )
            .await
            .unwrap();

        // Receive message
        let msg = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");
        assert!(
            matches!(msg, StreamMessage::NodeStatus { node_id, .. } if node_id == "node-1"),
            "Expected NodeStatus message with node_id node-1"
        );
    }

    #[tokio::test]
    async fn test_execution_tracking() {
        let handler = StreamHandler::new();
        let _rx = handler.subscribe(); // Keep subscriber alive

        // Start execution
        handler
            .start_execution("test-graph".to_string())
            .await
            .unwrap();

        // Send node status
        handler
            .send_node_status(
                "test-graph".to_string(),
                "node-1".to_string(),
                NodeStatus::Running { progress: 50 },
            )
            .await
            .unwrap();

        // Check execution state
        let state = handler.get_execution_state("test-graph").await.unwrap();
        assert_eq!(state.current_node, Some("node-1".to_string()));
        assert_eq!(state.completed_nodes.len(), 0);

        // Complete node
        handler
            .send_node_status(
                "test-graph".to_string(),
                "node-1".to_string(),
                NodeStatus::Completed,
            )
            .await
            .unwrap();

        // Check execution state
        let state = handler.get_execution_state("test-graph").await.unwrap();
        assert_eq!(state.current_node, None);
        assert_eq!(state.completed_nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_progress_update() {
        let handler = StreamHandler::new();
        let mut rx = handler.subscribe();

        handler
            .send_progress(
                "test-graph".to_string(),
                "node-1".to_string(),
                0.75,
                "Processing...".to_string(),
            )
            .await
            .unwrap();

        let msg = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");
        assert!(
            matches!(
                msg,
                StreamMessage::Progress {
                    progress,
                    message,
                    ..
                } if progress == 0.75 && message == "Processing..."
            ),
            "Expected Progress message with progress 0.75 and message Processing..."
        );
    }

    #[tokio::test]
    async fn test_ai_reasoning() {
        let handler = StreamHandler::new();
        let mut rx = handler.subscribe();

        let reasoning = AIReasoning {
            decision: "Execute node A next".to_string(),
            confidence: 0.87,
            rationale: vec![
                "Highest priority".to_string(),
                "Resources available".to_string(),
            ],
            alternatives: vec![Alternative {
                description: "Execute node B".to_string(),
                confidence: 0.73,
                reason_not_chosen: "Lower priority".to_string(),
            }],
            data_sources: vec!["user_history".to_string()],
            patterns: vec![Pattern {
                description: "User prefers A before B".to_string(),
                source: "user_history".to_string(),
                relevance: 0.9,
            }],
        };

        handler
            .send_reasoning("test-graph".to_string(), reasoning.clone())
            .await
            .unwrap();

        let msg = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");
        assert!(
            matches!(
                msg,
                StreamMessage::Reasoning { reasoning: r, .. }
                    if r.decision == reasoning.decision && r.confidence == reasoning.confidence
            ),
            "Expected Reasoning message"
        );
    }

    #[tokio::test]
    async fn test_error_handling() {
        let handler = StreamHandler::new();
        let mut rx = handler.subscribe();

        let error = ErrorInfo {
            error_type: "ExecutionError".to_string(),
            message: "Node failed".to_string(),
            details: Some("Out of memory".to_string()),
            recoverable: true,
            suggested_action: Some("Increase memory limit".to_string()),
        };

        handler
            .send_error(
                "test-graph".to_string(),
                Some("node-1".to_string()),
                error.clone(),
            )
            .await
            .unwrap();

        let msg = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");
        assert!(
            matches!(
                msg,
                StreamMessage::Error { error: e, .. }
                    if e.message == error.message && e.recoverable
            ),
            "Expected Error message"
        );
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let handler = StreamHandler::new();
        let mut rx1 = handler.subscribe();
        let mut rx2 = handler.subscribe();

        assert_eq!(handler.subscriber_count(), 2);

        handler
            .send_node_status(
                "test-graph".to_string(),
                "node-1".to_string(),
                NodeStatus::Running { progress: 50 },
            )
            .await
            .unwrap();

        // Both subscribers should receive the message
        let msg1 = tokio::time::timeout(Duration::from_secs(1), rx1.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");
        let msg2 = tokio::time::timeout(Duration::from_secs(1), rx2.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");

        assert!(
            matches!(
                (&msg1, &msg2),
                (
                    StreamMessage::NodeStatus { .. },
                    StreamMessage::NodeStatus { .. }
                )
            ),
            "Both subscribers should receive NodeStatus"
        );
    }

    #[tokio::test]
    async fn test_stop_execution() {
        let handler = StreamHandler::new();
        let _rx = handler.subscribe();

        handler
            .start_execution("test-graph".to_string())
            .await
            .unwrap();
        assert!(handler.get_execution_state("test-graph").await.is_some());

        handler.stop_execution("test-graph").await.unwrap();
        assert!(handler.get_execution_state("test-graph").await.is_none());
    }

    #[tokio::test]
    async fn test_progress_clamping() {
        let handler = StreamHandler::new();
        let mut rx = handler.subscribe();

        handler
            .send_progress(
                "test-graph".to_string(),
                "node-1".to_string(),
                1.5, // Should clamp to 1.0
                "Over 100%".to_string(),
            )
            .await
            .unwrap();

        let msg = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("recv timed out")
            .expect("recv failed");
        assert!(
            matches!(
                msg,
                StreamMessage::Progress { progress, .. } if progress == 1.0
            ),
            "Progress should be clamped to 1.0"
        );
    }
}
