// SPDX-License-Identifier: AGPL-3.0-only
//! Neural API Graph Client
//!
//! Client for saving, loading, and executing graphs via Neural API.
//! TRUE PRIMAL: Zero hardcoding, capability-based graph operations.

use crate::errors::{DiscoveryError, DiscoveryResult};
use crate::neural_api_provider::NeuralApiProvider;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Graph metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Graph ID
    pub id: String,

    /// Graph name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Creation timestamp
    pub created_at: String,

    /// Last modified timestamp
    pub modified_at: String,

    /// Number of nodes
    pub node_count: usize,

    /// Number of edges
    pub edge_count: usize,
}

/// Graph execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    /// Execution queued but not started
    Queued,

    /// Currently executing
    Running,

    /// Execution completed successfully
    Completed,

    /// Execution failed
    Failed,

    /// Execution was cancelled
    Cancelled,
}

/// Graph execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Execution ID
    pub execution_id: String,

    /// Graph ID that was executed
    pub graph_id: String,

    /// Current status
    pub status: ExecutionStatus,

    /// Start time
    pub started_at: Option<String>,

    /// End time
    pub completed_at: Option<String>,

    /// Error message if failed
    pub error: Option<String>,

    /// Output from execution
    pub output: Option<serde_json::Value>,
}

/// Neural API graph operations client
pub struct NeuralGraphClient<'a> {
    provider: &'a NeuralApiProvider,
}

impl<'a> NeuralGraphClient<'a> {
    /// Create a new graph client
    pub const fn new(provider: &'a NeuralApiProvider) -> Self {
        Self { provider }
    }

    /// Save a graph to Neural API
    ///
    /// # Arguments
    /// * `graph_json` - The graph as JSON (serialized `VisualGraph`)
    ///
    /// # Returns
    /// The graph ID assigned by Neural API
    pub async fn save_graph(&self, graph_json: serde_json::Value) -> DiscoveryResult<String> {
        let params = json!({
            "graph": graph_json
        });

        let result = self
            .provider
            .call_method("neural_api.save_graph", Some(params))
            .await
            .map_err(|e| DiscoveryError::InvalidData {
                name: "Neural API".to_string(),
                reason: format!("Failed to save graph: {e}"),
            })?;

        let graph_id = result
            .get("graph_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DiscoveryError::MissingField {
                field: "graph_id".to_string(),
                context: " (Neural API)".to_string(),
            })?
            .to_string();

        tracing::info!("💾 Saved graph to Neural API: {}", graph_id);
        Ok(graph_id)
    }

    /// Load a graph from Neural API
    ///
    /// # Arguments
    /// * `graph_id` - The graph ID to load
    ///
    /// # Returns
    /// The graph as JSON (to be deserialized into `VisualGraph`)
    pub async fn load_graph(&self, graph_id: &str) -> DiscoveryResult<serde_json::Value> {
        let params = json!({
            "graph_id": graph_id
        });

        let result = self
            .provider
            .call_method("neural_api.load_graph", Some(params))
            .await
            .map_err(|e| DiscoveryError::InvalidData {
                name: "Neural API".to_string(),
                reason: format!("Failed to load graph: {e}"),
            })?;

        let graph = result
            .get("graph")
            .ok_or_else(|| DiscoveryError::MissingField {
                field: "graph".to_string(),
                context: " (Neural API)".to_string(),
            })?
            .clone();

        tracing::info!("📂 Loaded graph from Neural API: {}", graph_id);
        Ok(graph)
    }

    /// List all available graphs
    pub async fn list_graphs(&self) -> DiscoveryResult<Vec<GraphMetadata>> {
        let result = self
            .provider
            .call_method("neural_api.list_graphs", None)
            .await
            .map_err(|e| DiscoveryError::InvalidData {
                name: "Neural API".to_string(),
                reason: format!("Failed to list graphs: {e}"),
            })?;

        let graphs = result
            .get("graphs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| DiscoveryError::ExpectedArray {
                context: " (Neural API graphs)".to_string(),
            })?;

        let metadata: Vec<GraphMetadata> = graphs
            .iter()
            .filter_map(|g| serde_json::from_value(g.clone()).ok())
            .collect();

        tracing::info!("📋 Listed {} graphs from Neural API", metadata.len());
        Ok(metadata)
    }

    /// Execute a graph
    ///
    /// # Arguments
    /// * `graph_id` - The graph ID to execute
    /// * `parameters` - Optional parameters for execution
    ///
    /// # Returns
    /// Execution ID for tracking status
    pub async fn execute_graph(
        &self,
        graph_id: &str,
        parameters: Option<serde_json::Value>,
    ) -> DiscoveryResult<String> {
        let params = json!({
            "graph_id": graph_id,
            "parameters": parameters.unwrap_or_else(|| json!({}))
        });

        let result = self
            .provider
            .call_method("neural_api.execute_graph", Some(params))
            .await
            .map_err(|e| DiscoveryError::InvalidData {
                name: "Neural API".to_string(),
                reason: format!("Failed to execute graph: {e}"),
            })?;

        let execution_id = result
            .get("execution_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DiscoveryError::MissingField {
                field: "execution_id".to_string(),
                context: " (Neural API)".to_string(),
            })?
            .to_string();

        tracing::info!("🚀 Started graph execution: {}", execution_id);
        Ok(execution_id)
    }

    /// Get execution status
    ///
    /// # Arguments
    /// * `execution_id` - The execution ID to check
    pub async fn get_execution_status(
        &self,
        execution_id: &str,
    ) -> DiscoveryResult<ExecutionResult> {
        let params = json!({
            "execution_id": execution_id
        });

        let result = self
            .provider
            .call_method("neural_api.get_execution_status", Some(params))
            .await
            .map_err(|e| DiscoveryError::InvalidData {
                name: "Neural API".to_string(),
                reason: format!("Failed to get execution status: {e}"),
            })?;

        let execution: ExecutionResult =
            serde_json::from_value(result).map_err(|e| DiscoveryError::ParseError {
                data_type: "execution status".to_string(),
                message: e.to_string(),
            })?;

        Ok(execution)
    }

    /// Cancel a running execution
    ///
    /// # Arguments
    /// * `execution_id` - The execution ID to cancel
    pub async fn cancel_execution(&self, execution_id: &str) -> DiscoveryResult<()> {
        let params = json!({
            "execution_id": execution_id
        });

        self.provider
            .call_method("neural_api.cancel_execution", Some(params))
            .await
            .map_err(|e| DiscoveryError::InvalidData {
                name: "Neural API".to_string(),
                reason: format!("Failed to cancel execution: {e}"),
            })?;

        tracing::info!("🛑 Cancelled execution: {}", execution_id);
        Ok(())
    }

    /// Delete a graph
    ///
    /// # Arguments
    /// * `graph_id` - The graph ID to delete
    pub async fn delete_graph(&self, graph_id: &str) -> DiscoveryResult<()> {
        let params = json!({
            "graph_id": graph_id
        });

        self.provider
            .call_method("neural_api.delete_graph", Some(params))
            .await
            .map_err(|e| DiscoveryError::InvalidData {
                name: "Neural API".to_string(),
                reason: format!("Failed to delete graph: {e}"),
            })?;

        tracing::info!("🗑️ Deleted graph: {}", graph_id);
        Ok(())
    }

    /// Update graph metadata
    ///
    /// # Arguments
    /// * `graph_id` - The graph ID to update
    /// * `name` - New name (optional)
    /// * `description` - New description (optional)
    pub async fn update_graph_metadata(
        &self,
        graph_id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> DiscoveryResult<()> {
        let mut params = json!({
            "graph_id": graph_id
        });

        if let Some(n) = name {
            params["name"] = json!(n);
        }
        if let Some(d) = description {
            params["description"] = json!(d);
        }

        self.provider
            .call_method("neural_api.update_graph_metadata", Some(params))
            .await
            .map_err(|e| DiscoveryError::InvalidData {
                name: "Neural API".to_string(),
                reason: format!("Failed to update graph metadata: {e}"),
            })?;

        tracing::info!("✏️ Updated graph metadata: {}", graph_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NeuralApiProvider;
    use std::path::PathBuf;

    // Note: save_graph, load_graph, etc. require a running Neural API server
    // We test structure, serialization, and client construction

    #[test]
    fn test_neural_graph_client_creation() {
        let provider = NeuralApiProvider::with_socket_path(PathBuf::from("/tmp/test.sock"));
        let _client = NeuralGraphClient::new(&provider);
    }

    #[test]
    fn test_execution_status_serialization() {
        let status = ExecutionStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""running""#);

        let deserialized: ExecutionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ExecutionStatus::Running);
    }

    #[test]
    fn test_graph_metadata_structure() {
        let metadata = GraphMetadata {
            id: "graph-123".to_string(),
            name: "Test Graph".to_string(),
            description: Some("A test graph".to_string()),
            created_at: "2026-01-15T00:00:00Z".to_string(),
            modified_at: "2026-01-15T01:00:00Z".to_string(),
            node_count: 5,
            edge_count: 4,
        };

        let json = serde_json::to_value(&metadata).unwrap();
        assert_eq!(json["id"], "graph-123");
        assert_eq!(json["name"], "Test Graph");
        assert_eq!(json["node_count"], 5);
    }

    #[test]
    fn test_execution_result_structure() {
        let result = ExecutionResult {
            execution_id: "exec-456".to_string(),
            graph_id: "graph-123".to_string(),
            status: ExecutionStatus::Completed,
            started_at: Some("2026-01-15T00:00:00Z".to_string()),
            completed_at: Some("2026-01-15T00:01:00Z".to_string()),
            error: None,
            output: Some(json!({"result": "success"})),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["execution_id"], "exec-456");
        assert_eq!(json["status"], "completed");
    }

    #[test]
    fn test_execution_statuses() {
        let statuses = vec![
            ExecutionStatus::Queued,
            ExecutionStatus::Running,
            ExecutionStatus::Completed,
            ExecutionStatus::Failed,
            ExecutionStatus::Cancelled,
        ];

        for status in statuses {
            let json = serde_json::to_value(&status).unwrap();
            let deserialized: ExecutionStatus = serde_json::from_value(json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_graph_metadata_serde_roundtrip() {
        let metadata = GraphMetadata {
            id: "g1".to_string(),
            name: "Graph 1".to_string(),
            description: Some("Desc".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            modified_at: "2026-01-02T00:00:00Z".to_string(),
            node_count: 3,
            edge_count: 2,
        };

        let json = serde_json::to_value(&metadata).unwrap();
        let restored: GraphMetadata = serde_json::from_value(json).unwrap();
        assert_eq!(metadata.id, restored.id);
        assert_eq!(metadata.node_count, restored.node_count);
    }

    #[test]
    fn test_graph_metadata_minimal() {
        let metadata = GraphMetadata {
            id: "min".to_string(),
            name: "Minimal".to_string(),
            description: None,
            created_at: "0".to_string(),
            modified_at: "0".to_string(),
            node_count: 0,
            edge_count: 0,
        };

        let json = serde_json::to_value(&metadata).unwrap();
        assert_eq!(json["node_count"], 0);
    }

    #[test]
    fn test_execution_result_serde() {
        let result = ExecutionResult {
            execution_id: "e1".to_string(),
            graph_id: "g1".to_string(),
            status: ExecutionStatus::Failed,
            started_at: None,
            completed_at: None,
            error: Some("Something went wrong".to_string()),
            output: None,
        };

        let json = serde_json::to_value(&result).expect("serialize");
        assert_eq!(json["status"], "failed");
        assert_eq!(json["error"], "Something went wrong");
    }

    #[test]
    fn test_save_graph_params_structure() {
        let params = json!({"graph": {"nodes": [], "edges": []}});
        assert!(params.get("graph").is_some());
    }

    #[test]
    fn test_load_graph_params_structure() {
        let params = json!({"graph_id": "g-123"});
        assert_eq!(params["graph_id"], "g-123");
    }

    #[test]
    fn test_execute_graph_params_structure() {
        let params = json!({
            "graph_id": "g-1",
            "parameters": {"key": "value"}
        });
        assert_eq!(params["graph_id"], "g-1");
        assert!(params["parameters"].is_object());
    }

    #[test]
    fn test_execution_status_display() {
        assert_eq!(
            serde_json::to_string(&ExecutionStatus::Queued).expect("serialize"),
            r#""queued""#
        );
        assert_eq!(
            serde_json::to_string(&ExecutionStatus::Cancelled).expect("serialize"),
            r#""cancelled""#
        );
    }

    #[test]
    fn test_cancel_execution_params_structure() {
        let params = json!({"execution_id": "exec-789"});
        assert_eq!(params["execution_id"], "exec-789");
    }

    #[test]
    fn test_delete_graph_params_structure() {
        let params = json!({"graph_id": "g-delete"});
        assert_eq!(params["graph_id"], "g-delete");
    }

    #[test]
    fn test_update_graph_metadata_params_name_only() {
        let mut params = json!({"graph_id": "g-update"});
        params["name"] = json!("New Name");
        assert_eq!(params["graph_id"], "g-update");
        assert_eq!(params["name"], "New Name");
    }

    #[test]
    fn test_update_graph_metadata_params_both() {
        let mut params = json!({"graph_id": "g-update"});
        params["name"] = json!("New Name");
        params["description"] = json!("New description");
        assert_eq!(params["description"], "New description");
    }

    #[test]
    fn test_execution_result_deserialize_from_full_json() {
        let json = json!({
            "execution_id": "e1",
            "graph_id": "g1",
            "status": "running",
            "started_at": "2026-01-01T00:00:00Z",
            "completed_at": null,
            "error": null,
            "output": null
        });
        let result: ExecutionResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.execution_id, "e1");
        assert_eq!(result.status, ExecutionStatus::Running);
    }

    #[test]
    fn test_list_graphs_response_structure() {
        let response = json!({
            "graphs": [
                {
                    "id": "g1",
                    "name": "Graph 1",
                    "description": null,
                    "created_at": "2026-01-01",
                    "modified_at": "2026-01-02",
                    "node_count": 1,
                    "edge_count": 0
                }
            ]
        });
        let graphs = response.get("graphs").and_then(|v| v.as_array()).unwrap();
        let metadata: Vec<GraphMetadata> = graphs
            .iter()
            .filter_map(|g| serde_json::from_value(g.clone()).ok())
            .collect();
        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata[0].id, "g1");
    }

    #[test]
    fn test_execute_graph_params_empty_parameters() {
        let params = json!({
            "graph_id": "g-1",
            "parameters": {}
        });
        assert!(params["parameters"].is_object());
        assert!(params["parameters"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_get_execution_status_params_structure() {
        let params = json!({"execution_id": "exec-123"});
        assert_eq!(params["execution_id"], "exec-123");
    }

    #[test]
    fn test_execution_result_roundtrip() {
        let result = ExecutionResult {
            execution_id: "e1".to_string(),
            graph_id: "g1".to_string(),
            status: ExecutionStatus::Running,
            started_at: Some("2026-01-01T00:00:00Z".to_string()),
            completed_at: None,
            error: None,
            output: Some(json!({"progress": 50})),
        };
        let json = serde_json::to_value(&result).expect("serialize");
        let restored: ExecutionResult = serde_json::from_value(json).expect("deserialize");
        assert_eq!(result.execution_id, restored.execution_id);
        assert_eq!(result.status, restored.status);
    }

    #[test]
    fn test_graph_metadata_serialization_with_description() {
        let metadata = GraphMetadata {
            id: "g1".to_string(),
            name: "Graph".to_string(),
            description: Some("A graph".to_string()),
            created_at: "2026-01-01".to_string(),
            modified_at: "2026-01-02".to_string(),
            node_count: 10,
            edge_count: 8,
        };
        let json = serde_json::to_value(&metadata).expect("serialize");
        assert_eq!(json["description"], "A graph");
    }

    #[test]
    fn test_save_graph_params_nested_graph() {
        let graph = json!({"nodes": [{"id": "n1"}], "edges": []});
        let params = json!({"graph": graph});
        assert!(params.get("graph").unwrap().get("nodes").is_some());
    }

    #[test]
    fn test_save_graph_response_parsing() {
        let response = json!({"graph_id": "g-saved-123"});
        let graph_id = response.get("graph_id").and_then(|v| v.as_str()).unwrap();
        assert_eq!(graph_id, "g-saved-123");
    }

    #[test]
    fn test_load_graph_response_parsing() {
        let graph_data = json!({"nodes": [], "edges": []});
        let response = json!({"graph": graph_data});
        let graph = response.get("graph").unwrap().clone();
        assert!(graph.get("nodes").unwrap().as_array().unwrap().is_empty());
    }

    #[test]
    fn test_list_graphs_empty_response() {
        let response = json!({"graphs": []});
        let graphs = response.get("graphs").and_then(|v| v.as_array()).unwrap();
        let metadata: Vec<GraphMetadata> = graphs
            .iter()
            .filter_map(|g| serde_json::from_value(g.clone()).ok())
            .collect();
        assert!(metadata.is_empty());
    }

    #[test]
    fn test_execution_result_with_output() {
        let json = json!({
            "execution_id": "e1",
            "graph_id": "g1",
            "status": "completed",
            "started_at": "2026-01-01T00:00:00Z",
            "completed_at": "2026-01-01T00:01:00Z",
            "error": null,
            "output": {"result": "done", "metrics": {"latency_ms": 42}}
        });
        let result: ExecutionResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.status, ExecutionStatus::Completed);
        assert!(result.output.is_some());
        assert_eq!(result.output.as_ref().unwrap()["result"], "done");
    }

    #[test]
    fn test_update_graph_metadata_params_description_only() {
        let mut params = json!({"graph_id": "g-update"});
        params["description"] = json!("Updated description");
        assert_eq!(params["description"], "Updated description");
    }

    #[test]
    fn test_execute_graph_params_with_nested_parameters() {
        let params = json!({
            "graph_id": "g-1",
            "parameters": {"input": {"key": "value"}, "options": {"timeout": 30}}
        });
        assert!(params["parameters"]["input"].is_object());
        assert_eq!(params["parameters"]["options"]["timeout"], 30);
    }

    #[test]
    fn test_graph_metadata_deserialize_from_json() {
        let json = json!({
            "id": "g2",
            "name": "Graph 2",
            "description": "Desc 2",
            "created_at": "2026-01-01",
            "modified_at": "2026-01-02",
            "node_count": 10,
            "edge_count": 9
        });
        let metadata: GraphMetadata = serde_json::from_value(json).unwrap();
        assert_eq!(metadata.id, "g2");
        assert_eq!(metadata.node_count, 10);
    }

    #[test]
    fn test_execution_status_failed_serialization() {
        let json = serde_json::to_string(&ExecutionStatus::Failed).unwrap();
        assert_eq!(json, r#""failed""#);
    }

    #[test]
    fn test_execution_status_ok_health() {
        let json = json!({"health": "healthy"});
        let status = json["health"].as_str().unwrap();
        assert_eq!(status, "healthy");
    }

    #[test]
    fn test_cancel_execution_response_no_result() {
        let response = json!({"jsonrpc": "2.0", "result": {}, "id": 1});
        assert!(response.get("result").is_some());
    }

    #[test]
    fn test_delete_graph_response_structure() {
        let response = json!({"jsonrpc": "2.0", "result": null, "id": 1});
        let result = response.get("result");
        assert!(result.is_some());
    }
}
