// SPDX-License-Identifier: AGPL-3.0-or-later
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
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails or response is invalid.
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
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails or response is invalid.
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
    ///
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails or response is invalid.
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
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails or response is invalid.
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
    ///
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails or response is invalid.
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
    ///
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails.
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
    ///
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails.
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
    ///
    /// # Errors
    /// Returns `DiscoveryError` if the API call fails.
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
#[path = "neural_graph_client_tests.rs"]
mod tests;
