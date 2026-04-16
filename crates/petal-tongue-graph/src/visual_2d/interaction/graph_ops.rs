// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph mutations for interactive painting (nodes, edges).

use egui::Pos2;
use petal_tongue_core::graph_engine::Position;
use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo, Properties, PropertyValue};

use crate::capability_validator::{ValidationResult, validate_connection};

use super::super::Visual2DRenderer;
use super::helpers::{interactive_node_id, interactive_node_name, is_edge_duplicate};

/// Try to complete an edge when drag is released (drop on target node).
pub(super) fn try_complete_edge_on_drag_release(
    renderer: &mut Visual2DRenderer,
    response: &egui::Response,
    screen_center: Pos2,
) {
    if let Some(edge_draft) = renderer.drawing_edge.take()
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let world_pos = renderer.screen_to_world(mouse_pos, screen_center);

        let target_id = {
            let Ok(graph) = renderer.graph.read() else {
                tracing::error!("graph lock poisoned");
                return;
            };
            graph
                .nodes()
                .iter()
                .find(|node| {
                    let distance = node.position.distance_to(world_pos);
                    distance < 20.0 && node.info.id.as_str() != edge_draft.from.as_str()
                })
                .map(|node| node.info.id.clone())
        };

        if let Some(target) = target_id {
            create_edge(renderer, edge_draft.from, target);
        }
    }
    renderer.dragging_node = None;
}

/// Create a new node at the given world position (interactive mode)
pub(super) fn create_node_at(renderer: &mut Visual2DRenderer, world_pos: Position) {
    let Ok(mut graph) = renderer.graph.write() else {
        tracing::error!("graph lock poisoned");
        return;
    };

    let node_count = graph.nodes().len();
    let new_id = interactive_node_id(node_count);

    let mut properties = Properties::new();
    properties.insert(
        "created_by".to_string(),
        PropertyValue::String("interactive-paint".to_string()),
    );
    let new_primal = PrimalInfo {
        id: PrimalId::from(new_id.clone()),
        name: interactive_node_name(node_count),
        primal_type: "custom".to_string(),
        endpoint: format!("interactive://{new_id}"),
        capabilities: vec!["interactive".to_string()],
        health: PrimalHealthStatus::Healthy,
        last_seen: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        endpoints: None,
        metadata: None,
        properties,
    }
    .with_family_id("interactive");

    graph.add_node(new_primal);

    if let Some(node) = graph.get_node_mut(&new_id) {
        node.position = world_pos;
    }

    drop(graph);
    renderer.selected_node = Some(PrimalId::from(new_id));
}

/// Create an edge between two nodes (interactive mode)
pub(super) fn create_edge(renderer: &Visual2DRenderer, from: PrimalId, to: PrimalId) {
    use petal_tongue_core::TopologyEdge;

    let Ok(graph) = renderer.graph.read() else {
        tracing::error!("graph lock poisoned");
        return;
    };

    let edge_exists = graph
        .edges()
        .iter()
        .any(|e| is_edge_duplicate(from.as_str(), to.as_str(), e.from.as_str(), e.to.as_str()));

    if edge_exists {
        return;
    }

    let from_node = graph.get_node(from.as_str());
    let to_node = graph.get_node(to.as_str());

    if let (Some(from_primal), Some(to_primal)) = (from_node, to_node) {
        let validation = validate_connection(&from_primal.info, &to_primal.info);

        match validation {
            ValidationResult::Invalid(reason) => {
                tracing::warn!("❌ Connection invalid: {}", reason);
                return;
            }
            ValidationResult::Warning(reason) => {
                tracing::info!("⚠️ Connection warning: {}", reason);
            }
            ValidationResult::Valid => {
                tracing::info!("✅ Connection validated");
            }
        }
    }

    drop(graph);

    let Ok(mut graph) = renderer.graph.write() else {
        tracing::error!("graph lock poisoned");
        return;
    };
    graph.add_edge(TopologyEdge {
        from,
        to,
        edge_type: "interactive".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });
}

/// Delete a node (interactive mode)
pub(super) fn delete_node(renderer: &Visual2DRenderer, node_id: &str) {
    let Ok(mut graph) = renderer.graph.write() else {
        tracing::error!("graph lock poisoned");
        return;
    };
    graph.remove_node(node_id);
}
