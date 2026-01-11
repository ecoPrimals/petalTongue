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
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

/// Stream message - Real-time updates for graph execution
///
/// All messages are bidirectional: from biomeOS to petalTongue,
/// and from petalTongue to biomeOS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    /// Node status update (running, completed, failed, etc)
    NodeStatus {
        graph_id: String,
        node_id: String,
        status: NodeStatus,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Progress update (percentage complete)
    Progress {
        graph_id: String,
        node_id: String,
        progress: f32, // 0.0 - 1.0
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// AI reasoning update (why decisions are made)
    Reasoning {
        graph_id: String,
        reasoning: AIReasoning,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Resource usage update
    ResourceUsage {
        graph_id: String,
        node_id: String,
        resources: ResourceUsage,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Error update
    Error {
        graph_id: String,
        node_id: Option<String>,
        error: ErrorInfo,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Graph modification (from user)
    GraphModification {
        graph_id: String,
        modification: GraphModification,
        user_id: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Heartbeat (keep connection alive)
    Heartbeat {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Node status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Pending,
    Running { progress: u8 }, // 0-100
    Completed,
    Failed { error: String },
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
    pub description: String,
    pub confidence: f32,
    pub reason_not_chosen: String,
}

/// Historical pattern referenced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub description: String,
    pub source: String, // "user_history", "community", "system"
    pub relevance: f32,
}

/// Resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub disk_io_mbps: f32,
    pub network_mbps: f32,
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub error_type: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
    pub suggested_action: Option<String>,
}

/// Graph modification (from user)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum GraphModification {
    AddNode { node: serde_json::Value },
    RemoveNode { node_id: String },
    ModifyNode { node_id: String, changes: serde_json::Value },
    AddEdge { from: String, to: String },
    RemoveEdge { edge_id: String },
}

/// Stream handler - Manages WebSocket connections and message routing
///
/// This is the central hub for all real-time communication.
pub struct StreamHandler {
    /// Broadcast channel for sending messages to all subscribers
    tx: broadcast::Sender<StreamMessage>,

    /// Active graph executions (graph_id -> execution state)
    executions: Arc<RwLock<std::collections::HashMap<String, ExecutionState>>>,
}

/// Execution state for a graph
#[derive(Debug, Clone)]
pub struct ExecutionState {
    pub graph_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub current_node: Option<String>,
    pub completed_nodes: Vec<String>,
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
        match message {
            StreamMessage::NodeStatus {
                graph_id,
                node_id,
                status,
                ..
            } => {
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
            _ => {}
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
        let msg = rx.recv().await.unwrap();
        match msg {
            StreamMessage::NodeStatus { node_id, .. } => {
                assert_eq!(node_id, "node-1");
            }
            _ => panic!("Expected NodeStatus message"),
        }
    }

    #[tokio::test]
    async fn test_execution_tracking() {
        let handler = StreamHandler::new();
        let _rx = handler.subscribe(); // Keep subscriber alive

        // Start execution
        handler.start_execution("test-graph".to_string()).await.unwrap();

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

        let msg = rx.recv().await.unwrap();
        match msg {
            StreamMessage::Progress { progress, message, .. } => {
                assert_eq!(progress, 0.75);
                assert_eq!(message, "Processing...");
            }
            _ => panic!("Expected Progress message"),
        }
    }

    #[tokio::test]
    async fn test_ai_reasoning() {
        let handler = StreamHandler::new();
        let mut rx = handler.subscribe();

        let reasoning = AIReasoning {
            decision: "Execute node A next".to_string(),
            confidence: 0.87,
            rationale: vec!["Highest priority".to_string(), "Resources available".to_string()],
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

        let msg = rx.recv().await.unwrap();
        match msg {
            StreamMessage::Reasoning { reasoning: r, .. } => {
                assert_eq!(r.decision, reasoning.decision);
                assert_eq!(r.confidence, reasoning.confidence);
            }
            _ => panic!("Expected Reasoning message"),
        }
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
            .send_error("test-graph".to_string(), Some("node-1".to_string()), error.clone())
            .await
            .unwrap();

        let msg = rx.recv().await.unwrap();
        match msg {
            StreamMessage::Error { error: e, .. } => {
                assert_eq!(e.message, error.message);
                assert!(e.recoverable);
            }
            _ => panic!("Expected Error message"),
        }
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
        let msg1 = rx1.recv().await.unwrap();
        let msg2 = rx2.recv().await.unwrap();

        match (&msg1, &msg2) {
            (StreamMessage::NodeStatus { .. }, StreamMessage::NodeStatus { .. }) => {
                // Both received NodeStatus
            }
            _ => panic!("Both subscribers should receive NodeStatus"),
        }
    }
}

