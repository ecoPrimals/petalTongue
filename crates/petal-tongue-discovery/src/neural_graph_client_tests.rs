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
