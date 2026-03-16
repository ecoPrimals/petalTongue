// SPDX-License-Identifier: AGPL-3.0-or-later

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_editor::streaming::{NodeStatus as PNodeStatus, StreamMessage};
    use chrono::Utc;

    fn make_state(graph_id: &str) -> ExecutionState {
        ExecutionState {
            graph_id: graph_id.to_string(),
            started_at: Utc::now(),
            current_node: None,
            completed_nodes: vec![],
            failed_nodes: vec![],
        }
    }

    #[test]
    fn running_sets_current_node() {
        let mut executions = std::collections::HashMap::new();
        executions.insert("g1".to_string(), make_state("g1"));

        let msg = StreamMessage::NodeStatus {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            status: PNodeStatus::Running { progress: 50 },
            timestamp: Utc::now(),
        };
        update_execution_state(&mut executions, &msg);

        let state = executions.get("g1").unwrap();
        assert_eq!(state.current_node, Some("n1".to_string()));
        assert!(state.completed_nodes.is_empty());
        assert!(state.failed_nodes.is_empty());
    }

    #[test]
    fn completed_clears_current_adds_to_completed() {
        let mut executions = std::collections::HashMap::new();
        let mut state = make_state("g1");
        state.current_node = Some("n0".to_string());
        executions.insert("g1".to_string(), state);

        let msg = StreamMessage::NodeStatus {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            status: PNodeStatus::Completed,
            timestamp: Utc::now(),
        };
        update_execution_state(&mut executions, &msg);

        let state = executions.get("g1").unwrap();
        assert!(state.current_node.is_none());
        assert_eq!(state.completed_nodes, vec!["n1"]);
    }

    #[test]
    fn failed_clears_current_adds_to_failed() {
        let mut executions = std::collections::HashMap::new();
        let mut state = make_state("g1");
        state.current_node = Some("n0".to_string());
        executions.insert("g1".to_string(), state);

        let msg = StreamMessage::NodeStatus {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            status: PNodeStatus::Failed {
                error: "err".to_string(),
            },
            timestamp: Utc::now(),
        };
        update_execution_state(&mut executions, &msg);

        let state = executions.get("g1").unwrap();
        assert!(state.current_node.is_none());
        assert_eq!(state.failed_nodes, vec!["n1"]);
    }

    #[test]
    fn pending_ignored() {
        let mut executions = std::collections::HashMap::new();
        let mut state = make_state("g1");
        state.current_node = Some("n0".to_string());
        executions.insert("g1".to_string(), state);

        let msg = StreamMessage::NodeStatus {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            status: PNodeStatus::Pending,
            timestamp: Utc::now(),
        };
        update_execution_state(&mut executions, &msg);

        let state = executions.get("g1").unwrap();
        assert_eq!(state.current_node, Some("n0".to_string()));
        assert!(state.completed_nodes.is_empty());
    }

    #[test]
    fn unknown_graph_id_ignored() {
        let mut executions = std::collections::HashMap::new();
        executions.insert("g1".to_string(), make_state("g1"));

        let msg = StreamMessage::NodeStatus {
            graph_id: "g2".to_string(),
            node_id: "n1".to_string(),
            status: PNodeStatus::Completed,
            timestamp: Utc::now(),
        };
        update_execution_state(&mut executions, &msg);

        let state = executions.get("g1").unwrap();
        assert!(state.completed_nodes.is_empty());
    }

    #[test]
    fn non_node_status_message_ignored() {
        let mut executions = std::collections::HashMap::new();
        executions.insert("g1".to_string(), make_state("g1"));

        let msg = StreamMessage::Heartbeat {
            timestamp: Utc::now(),
        };
        update_execution_state(&mut executions, &msg);

        let state = executions.get("g1").unwrap();
        assert!(state.current_node.is_none());
    }
}
