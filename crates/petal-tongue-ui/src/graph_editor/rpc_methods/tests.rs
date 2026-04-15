// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::graph_editor::edge::{DependencyType, GraphEdge};
use crate::graph_editor::graph::Graph;
use crate::graph_editor::node::GraphNode;

#[tokio::test]
async fn test_editor_open() {
    let service = GraphEditorService::new();
    let req = EditorOpenRequest {
        graph_id: "test-graph".to_string(),
    };

    let resp = service.editor_open(req).await.unwrap();
    assert_eq!(resp.graph.id, "test-graph");
    assert!(resp.template_info.is_none());
}

#[tokio::test]
async fn test_add_node() {
    let service = GraphEditorService::new();
    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let req = AddNodeRequest {
        graph_id: "test-graph".to_string(),
        node,
    };

    let resp = service.add_node(req).await.unwrap();
    assert_eq!(resp.node_id, "node-1");
    assert!(resp.validation.valid);
}

#[tokio::test]
async fn test_remove_node() {
    let service = GraphEditorService::new();

    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let add_req = AddNodeRequest {
        graph_id: "test-graph".to_string(),
        node,
    };
    service.add_node(add_req).await.unwrap();

    let remove_req = RemoveNodeRequest {
        graph_id: "test-graph".to_string(),
        node_id: "node-1".to_string(),
    };
    let resp = service.remove_node(remove_req).await.unwrap();
    assert!(resp.success);
}

#[tokio::test]
async fn test_add_edge() {
    let service = GraphEditorService::new();

    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());

    service
        .add_node(AddNodeRequest {
            graph_id: "test-graph".to_string(),
            node: node1,
        })
        .await
        .unwrap();

    service
        .add_node(AddNodeRequest {
            graph_id: "test-graph".to_string(),
            node: node2,
        })
        .await
        .unwrap();

    let req = AddEdgeRequest {
        graph_id: "test-graph".to_string(),
        from: "node-1".to_string(),
        to: "node-2".to_string(),
        edge_type: DependencyType::Sequential,
    };

    let resp = service.add_edge(req).await.unwrap();
    assert!(resp.validation.valid);
}

#[tokio::test]
async fn test_save_and_apply_template() {
    let service = GraphEditorService::new();

    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
    service
        .add_node(AddNodeRequest {
            graph_id: "test-graph".to_string(),
            node,
        })
        .await
        .unwrap();

    let save_req = SaveTemplateRequest {
        graph_id: "test-graph".to_string(),
        name: "Test Template".to_string(),
        description: "A test template".to_string(),
        tags: vec!["test".to_string()],
    };
    let save_resp = service.save_template(save_req).await.unwrap();

    let apply_req = ApplyTemplateRequest {
        template_id: save_resp.template_id,
        merge: false,
    };
    let apply_resp = service.apply_template(apply_req).await.unwrap();
    assert_eq!(apply_resp.nodes_added, 1);
}

#[tokio::test]
async fn test_get_preview() {
    let service = GraphEditorService::new();

    let mut graph = Graph::new("test-graph".to_string(), "Test".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();

    let req = GetPreviewRequest { graph };
    let resp = service.get_preview(req).await.unwrap();

    assert_eq!(resp.execution_order.len(), 2);
    assert!(resp.validation_warnings.is_empty());
}

#[tokio::test]
async fn test_modify_node() {
    let service = GraphEditorService::new();

    let node = GraphNode::new("node-1".to_string(), "test-type".to_string())
        .with_properties(serde_json::json!({"key": "original"}));
    service
        .add_node(AddNodeRequest {
            graph_id: "test-graph".to_string(),
            node,
        })
        .await
        .unwrap();

    let changes = serde_json::json!({"key": "modified", "new_key": "value"});
    let req = ModifyNodeRequest {
        graph_id: "test-graph".to_string(),
        node_id: "node-1".to_string(),
        changes,
    };
    let resp = service.modify_node(req).await.unwrap();
    assert!(resp.success);
    assert!(resp.validation.valid);
}

#[tokio::test]
async fn test_list_templates() {
    let service = GraphEditorService::new();
    let templates = service.list_templates().await;
    assert!(templates.is_empty());
}

#[tokio::test]
async fn test_editor_open_existing_graph() {
    let service = GraphEditorService::new();
    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
    service
        .add_node(AddNodeRequest {
            graph_id: "g1".to_string(),
            node,
        })
        .await
        .unwrap();

    let resp = service
        .editor_open(EditorOpenRequest {
            graph_id: "g1".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(resp.graph.nodes.len(), 1);
}

#[tokio::test]
async fn test_remove_node_not_found() {
    let service = GraphEditorService::new();
    let resp = service
        .remove_node(RemoveNodeRequest {
            graph_id: "g1".to_string(),
            node_id: "nonexistent".to_string(),
        })
        .await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_add_node_duplicate_validation() {
    let service = GraphEditorService::new();
    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
    service
        .add_node(AddNodeRequest {
            graph_id: "g1".to_string(),
            node: node.clone(),
        })
        .await
        .unwrap();
    let resp = service
        .add_node(AddNodeRequest {
            graph_id: "g1".to_string(),
            node,
        })
        .await
        .unwrap();
    assert!(!resp.validation.valid);
}

#[tokio::test]
async fn test_modify_node_not_found() {
    let service = GraphEditorService::new();
    let resp = service
        .modify_node(ModifyNodeRequest {
            graph_id: "g1".to_string(),
            node_id: "nonexistent".to_string(),
            changes: serde_json::json!({}),
        })
        .await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_add_edge_graph_not_found() {
    let service = GraphEditorService::new();
    let resp = service
        .add_edge(AddEdgeRequest {
            graph_id: "nonexistent".to_string(),
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: DependencyType::Sequential,
        })
        .await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_save_template_graph_not_found() {
    let service = GraphEditorService::new();
    let resp = service
        .save_template(SaveTemplateRequest {
            graph_id: "nonexistent".to_string(),
            name: "T".to_string(),
            description: "D".to_string(),
            tags: vec![],
        })
        .await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_apply_template_not_found() {
    let service = GraphEditorService::new();
    let resp = service
        .apply_template(ApplyTemplateRequest {
            template_id: "nonexistent".to_string(),
            merge: false,
        })
        .await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_get_preview_large_graph_warning() {
    let service = GraphEditorService::new();
    let mut graph = Graph::new("g".to_string(), "Large".to_string());
    for i in 0..105 {
        let node = GraphNode::new(format!("node-{}", i), "t".to_string());
        graph.add_node(node).unwrap();
    }
    for i in 0..104 {
        graph
            .add_edge(GraphEdge::new(
                format!("e-{}", i),
                format!("node-{}", i),
                format!("node-{}", i + 1),
                DependencyType::Sequential,
            ))
            .unwrap();
    }
    let req = GetPreviewRequest { graph };
    let resp = service.get_preview(req).await.unwrap();
    assert!(!resp.validation_warnings.is_empty());
}

#[tokio::test]
async fn test_get_graph() {
    let service = GraphEditorService::new();
    assert!(service.get_graph("nonexistent").await.is_none());
    service
        .add_node(AddNodeRequest {
            graph_id: "g1".to_string(),
            node: GraphNode::new("n1".to_string(), "t".to_string()),
        })
        .await
        .unwrap();
    let g = service.get_graph("g1").await;
    assert!(g.is_some());
    assert_eq!(g.unwrap().nodes.len(), 1);
}

#[tokio::test]
async fn test_modify_node_non_object_changes() {
    let service = GraphEditorService::new();
    service
        .add_node(AddNodeRequest {
            graph_id: "g1".to_string(),
            node: GraphNode::new("n1".to_string(), "t".to_string()),
        })
        .await
        .unwrap();
    let resp = service
        .modify_node(ModifyNodeRequest {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            changes: serde_json::json!("not an object"),
        })
        .await
        .unwrap();
    assert!(resp.success);
}
