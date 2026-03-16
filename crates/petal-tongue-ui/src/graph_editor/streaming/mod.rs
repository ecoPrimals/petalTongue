// SPDX-License-Identifier: AGPL-3.0-only

mod execution_state;
mod protocol;

use crate::error::{GraphEditorError, Result};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, info};

pub use execution_state::ExecutionState;
pub use protocol::*;

pub struct StreamHandler {
    tx: broadcast::Sender<StreamMessage>,
    executions: Arc<RwLock<std::collections::HashMap<String, ExecutionState>>>,
}

impl StreamHandler {
    #[must_use]
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1000);

        Self {
            tx,
            executions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<StreamMessage> {
        self.tx.subscribe()
    }

    pub async fn send(&self, message: StreamMessage) -> Result<()> {
        debug!("Sending stream message: {:?}", message);

        {
            let mut executions = self.executions.write().await;
            execution_state::update_execution_state(&mut executions, &message);
        }

        self.tx
            .send(message)
            .map_err(|_| GraphEditorError::StreamBroadcastFailed)?;

        Ok(())
    }

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

    pub async fn stop_execution(&self, graph_id: &str) -> Result<()> {
        info!("Stopping execution tracking for graph '{}'", graph_id);

        let mut executions = self.executions.write().await;
        executions.remove(graph_id);

        Ok(())
    }

    pub async fn get_execution_state(&self, graph_id: &str) -> Option<ExecutionState> {
        let executions = self.executions.read().await;
        executions.get(graph_id).cloned()
    }

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

    pub async fn send_reasoning(&self, graph_id: String, reasoning: AIReasoning) -> Result<()> {
        self.send(StreamMessage::Reasoning {
            graph_id,
            reasoning,
            timestamp: chrono::Utc::now(),
        })
        .await
    }

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

        handler
            .send_node_status(
                "test-graph".to_string(),
                "node-1".to_string(),
                NodeStatus::Running { progress: 50 },
            )
            .await
            .unwrap();

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
        let _rx = handler.subscribe();

        handler
            .start_execution("test-graph".to_string())
            .await
            .unwrap();

        handler
            .send_node_status(
                "test-graph".to_string(),
                "node-1".to_string(),
                NodeStatus::Running { progress: 50 },
            )
            .await
            .unwrap();

        let state = handler.get_execution_state("test-graph").await.unwrap();
        assert_eq!(state.current_node, Some("node-1".to_string()));
        assert_eq!(state.completed_nodes.len(), 0);

        handler
            .send_node_status(
                "test-graph".to_string(),
                "node-1".to_string(),
                NodeStatus::Completed,
            )
            .await
            .unwrap();

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
                1.5,
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

    #[tokio::test]
    async fn test_stream_handler_default() {
        let handler = StreamHandler::default();
        assert_eq!(handler.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_progress_clamp_negative() {
        let handler = StreamHandler::new();
        let mut rx = handler.subscribe();

        handler
            .send_progress(
                "test-graph".to_string(),
                "node-1".to_string(),
                -0.5,
                "Negative".to_string(),
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
                StreamMessage::Progress { progress, .. } if progress == 0.0
            ),
            "Progress should be clamped to 0.0"
        );
    }
}
