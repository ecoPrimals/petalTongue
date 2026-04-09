// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::accessibility::ColorPalette;
use crate::accessibility::ColorScheme;
use petal_tongue_core::NodeType;
use petal_tongue_core::graph_builder::{GraphNode, Vec2, VisualGraph};

#[test]
fn test_panel_creation() {
    let panel = PropertyPanel::new();
    assert!(panel.editing_node.is_none());
    assert!(panel.temp_params.is_empty());
    assert!(panel.errors.is_empty());
}

#[test]
fn test_set_editing_node() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());

    let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    node.set_parameter("primal_name".to_string(), "sample_primal".to_string());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id.clone()), &graph);

    assert_eq!(panel.editing_node, Some(node_id));
    assert_eq!(
        panel.temp_params.get("primal_name"),
        Some(&"sample_primal".to_string())
    );
}

#[test]
fn test_apply_changes() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());

    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id.clone()), &graph);
    panel
        .temp_params
        .insert("primal_name".to_string(), "sample_primal".to_string());
    panel
        .temp_params
        .insert("family_id".to_string(), "nat0".to_string());

    panel.apply_changes(&mut graph);

    let node = graph.get_node(&node_id).unwrap();
    assert_eq!(
        node.get_parameter("primal_name"),
        Some(&"sample_primal".to_string())
    );
    assert_eq!(node.get_parameter("family_id"), Some(&"nat0".to_string()));
    assert!(!node.visual_state.has_error);
}

#[test]
fn test_validation_errors() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());

    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id.clone()), &graph);
    // Don't set required parameters
    panel.apply_changes(&mut graph);

    assert!(!panel.errors.is_empty());
    let node = graph.get_node(&node_id).unwrap();
    assert!(node.visual_state.has_error);
}

#[test]
fn test_reset_changes() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());

    let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    node.set_parameter("primal_name".to_string(), "original".to_string());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id), &graph);
    panel
        .temp_params
        .insert("primal_name".to_string(), "modified".to_string());

    panel.reset_changes(&graph);

    assert_eq!(
        panel.temp_params.get("primal_name"),
        Some(&"original".to_string())
    );
}

#[test]
fn test_has_unsaved_changes() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());

    let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    node.set_parameter("primal_name".to_string(), "original".to_string());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id), &graph);
    assert!(!panel.has_unsaved_changes(&graph));

    panel
        .temp_params
        .insert("primal_name".to_string(), "modified".to_string());
    assert!(panel.has_unsaved_changes(&graph));
}

#[test]
fn test_default_impl() {
    let panel = PropertyPanel::default();
    assert!(panel.editing_node.is_none());
    assert!(panel.temp_params.is_empty());
}

#[test]
fn test_set_editing_node_none_clears() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id), &graph);
    panel
        .temp_params
        .insert("extra".to_string(), "value".to_string());

    panel.set_editing_node(None, &graph);
    assert!(panel.editing_node.is_none());
    assert!(panel.temp_params.is_empty());
    assert!(panel.errors.is_empty());
}

#[test]
fn test_property_extraction_field_ordering() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    node.set_parameter("family_id".to_string(), "nat0".to_string());
    node.set_parameter("primal_name".to_string(), "sample_primal".to_string());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id), &graph);

    assert_eq!(
        panel.temp_params.get("primal_name"),
        Some(&"sample_primal".to_string())
    );
    assert_eq!(
        panel.temp_params.get("family_id"),
        Some(&"nat0".to_string())
    );
    let required = NodeType::PrimalStart.required_parameters();
    assert_eq!(required, &["primal_name", "family_id"]);
}

#[test]
fn test_verification_node_params() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let mut node = GraphNode::new(NodeType::Verification, Vec2::zero());
    node.set_parameter("primal_name".to_string(), "p1".to_string());
    node.set_parameter("timeout".to_string(), "30".to_string());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id.clone()), &graph);
    panel.apply_changes(&mut graph);

    let node = graph.get_node(&node_id).unwrap();
    assert!(!node.visual_state.has_error);
    assert_eq!(node.get_parameter("primal_name"), Some(&"p1".to_string()));
    assert_eq!(node.get_parameter("timeout"), Some(&"30".to_string()));
}

#[test]
fn test_apply_empty_string_validation() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id), &graph);
    panel
        .temp_params
        .insert("primal_name".to_string(), "  ".to_string());
    panel
        .temp_params
        .insert("family_id".to_string(), "nat0".to_string());

    panel.apply_changes(&mut graph);

    assert!(panel.errors.contains_key("primal_name"));
}

#[test]
fn test_get_editing_node() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node_id = node.id.clone();
    graph.add_node(node);

    assert!(panel.get_editing_node().is_none());
    panel.set_editing_node(Some(node_id.clone()), &graph);
    assert_eq!(panel.get_editing_node(), Some(&node_id));
}

#[test]
fn test_get_parameter_help() {
    assert_eq!(
        PropertyPanel::get_parameter_help("primal_name"),
        "Name of the primal to start (discovered at runtime)"
    );
    assert_eq!(
        PropertyPanel::get_parameter_help("family_id"),
        "Family ID for the primal (e.g., nat0)"
    );
    assert_eq!(
        PropertyPanel::get_parameter_help("timeout"),
        "Timeout in seconds (e.g., 30)"
    );
    assert_eq!(
        PropertyPanel::get_parameter_help("condition"),
        "Condition to wait for or evaluate"
    );
    assert_eq!(
        PropertyPanel::get_parameter_help("unknown_param"),
        "Enter value for this parameter"
    );
}

#[test]
fn test_render_no_node_selected() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let palette = ColorPalette::from_scheme(ColorScheme::Default);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            panel.render(ui, &mut graph, &palette);
        });
    });
}

#[test]
fn test_render_with_node_selected() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    node.set_parameter("primal_name".to_string(), "test_primal".to_string());
    node.set_parameter("family_id".to_string(), "nat0".to_string());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id), &graph);
    let palette = ColorPalette::from_scheme(ColorScheme::Default);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            panel.render(ui, &mut graph, &palette);
        });
    });
}

#[test]
fn test_render_with_validation_errors() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id), &graph);
    panel.apply_changes(&mut graph);

    let palette = ColorPalette::from_scheme(ColorScheme::Default);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            panel.render(ui, &mut graph, &palette);
        });
    });
}

#[test]
fn test_conditional_node_params() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let mut node = GraphNode::new(NodeType::Conditional, Vec2::zero());
    node.set_parameter("condition".to_string(), "health > 0".to_string());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id.clone()), &graph);
    panel.apply_changes(&mut graph);

    let node = graph.get_node(&node_id).unwrap();
    assert!(!node.visual_state.has_error);
    assert_eq!(
        node.get_parameter("condition"),
        Some(&"health > 0".to_string())
    );
}

#[test]
fn test_wait_for_node_params() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let mut node = GraphNode::new(NodeType::WaitFor, Vec2::zero());
    node.set_parameter("primal_name".to_string(), "p1".to_string());
    node.set_parameter("timeout".to_string(), "60".to_string());
    node.set_parameter("condition".to_string(), "ready".to_string());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id.clone()), &graph);
    panel.apply_changes(&mut graph);

    let node = graph.get_node(&node_id).unwrap();
    assert!(!node.visual_state.has_error);
}

#[test]
fn test_apply_missing_required_param() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id), &graph);
    panel
        .temp_params
        .insert("primal_name".to_string(), "p1".to_string());
    // Missing family_id

    panel.apply_changes(&mut graph);

    assert!(panel.errors.contains_key("family_id"));
}

#[test]
fn test_has_unsaved_changes_extra_param() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    node.set_parameter("primal_name".to_string(), "p1".to_string());
    node.set_parameter("family_id".to_string(), "nat0".to_string());
    let node_id = node.id.clone();
    graph.add_node(node);

    panel.set_editing_node(Some(node_id), &graph);
    assert!(!panel.has_unsaved_changes(&graph));

    panel
        .temp_params
        .insert("extra".to_string(), "value".to_string());
    assert!(panel.has_unsaved_changes(&graph));
}

#[test]
fn test_set_editing_node_nonexistent_clears() {
    let mut panel = PropertyPanel::new();
    let mut graph = VisualGraph::new("test".to_string());
    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    graph.add_node(node);

    panel.set_editing_node(Some("nonexistent-id".to_string()), &graph);
    assert_eq!(panel.editing_node.as_deref(), Some("nonexistent-id"));
    assert!(panel.temp_params.is_empty());
}
