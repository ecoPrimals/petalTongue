// SPDX-License-Identifier: AGPL-3.0-only

use super::protocol::{NodeStatus, StreamMessage};

#[derive(Debug, Clone)]
pub struct ExecutionState {
    pub graph_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub current_node: Option<String>,
    pub completed_nodes: Vec<String>,
    pub failed_nodes: Vec<String>,
}

pub fn update_execution_state(
    executions: &mut std::collections::HashMap<String, ExecutionState>,
    message: &StreamMessage,
) {
    if let StreamMessage::NodeStatus {
        graph_id,
        node_id,
        status,
        ..
    } = message
        && let Some(state) = executions.get_mut(graph_id)
    {
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
