use super::*;
use crate::accessibility::ColorPalette;
use crate::accessibility::ColorScheme;

#[test]
fn test_panel_creation() {
    let panel = GraphManagerPanel::new();
    assert!(panel.available_graphs.is_empty());
    assert!(panel.selected_graph_id.is_none());
    assert!(!panel.show_save_dialog);
}

#[test]
fn test_add_remove_graph() {
    let mut panel = GraphManagerPanel::new();

    let metadata = GraphMetadata {
        id: "test-123".to_string(),
        name: "Test Graph".to_string(),
        description: Some("A test".to_string()),
        created_at: "2026-01-15T00:00:00Z".to_string(),
        modified_at: "2026-01-15T01:00:00Z".to_string(),
        node_count: 5,
        edge_count: 4,
    };

    panel.add_graph(metadata);
    assert_eq!(panel.available_graphs.len(), 1);

    panel.remove_graph("test-123");
    assert_eq!(panel.available_graphs.len(), 0);
}

#[test]
fn test_set_error() {
    let mut panel = GraphManagerPanel::new();
    assert!(panel.error_message.is_none());

    panel.set_error(Some("Test error".to_string()));
    assert_eq!(panel.error_message, Some("Test error".to_string()));

    panel.set_error(None);
    assert!(panel.error_message.is_none());
}

#[test]
fn test_set_execution_status() {
    let mut panel = GraphManagerPanel::new();
    assert!(panel.execution_status.is_none());

    panel.set_execution_status(Some("Running...".to_string()));
    assert_eq!(panel.execution_status, Some("Running...".to_string()));

    panel.set_execution_status(None);
    assert!(panel.execution_status.is_none());
}

#[test]
fn test_remove_graph_clears_selected() {
    let mut panel = GraphManagerPanel::new();
    let metadata = GraphMetadata {
        id: "graph-a".to_string(),
        name: "Graph A".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-01-01T01:00:00Z".to_string(),
        node_count: 2,
        edge_count: 1,
    };
    panel.add_graph(metadata);
    panel.selected_graph_id = Some("graph-a".to_string());

    panel.remove_graph("graph-a");
    assert_eq!(panel.available_graphs.len(), 0);
    assert!(panel.selected_graph_id.is_none());
}

#[test]
fn test_remove_graph_keeps_others() {
    let mut panel = GraphManagerPanel::new();
    panel.add_graph(GraphMetadata {
        id: "g1".to_string(),
        name: "G1".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-01-01T01:00:00Z".to_string(),
        node_count: 1,
        edge_count: 0,
    });
    panel.add_graph(GraphMetadata {
        id: "g2".to_string(),
        name: "G2".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-01-01T01:00:00Z".to_string(),
        node_count: 2,
        edge_count: 1,
    });

    panel.remove_graph("g1");
    assert_eq!(panel.available_graphs.len(), 1);
    assert_eq!(panel.available_graphs[0].id, "g2");
}

#[test]
fn test_graph_manager_action_variants() {
    let save = GraphManagerAction::Save {
        name: "test".to_string(),
        description: Some("desc".to_string()),
    };
    match &save {
        GraphManagerAction::Save { name, description } => {
            assert_eq!(name, "test");
            assert_eq!(description.as_deref(), Some("desc"));
        }
        _ => panic!("expected Save"),
    }

    let load = GraphManagerAction::Load("id-123".to_string());
    match &load {
        GraphManagerAction::Load(id) => assert_eq!(id, "id-123"),
        _ => panic!("expected Load"),
    }

    let _ = GraphManagerAction::Execute;
    let delete = GraphManagerAction::Delete("del-id".to_string());
    match &delete {
        GraphManagerAction::Delete(id) => assert_eq!(id, "del-id"),
        _ => panic!("expected Delete"),
    }
}

#[test]
fn test_panel_default() {
    let panel = GraphManagerPanel::default();
    assert!(panel.available_graphs.is_empty());
    assert!(panel.save_name.is_empty());
    assert!(panel.save_description.is_empty());
}

#[test]
fn test_save_action_empty_description() {
    let save = GraphManagerAction::Save {
        name: "graph".to_string(),
        description: None,
    };
    match &save {
        GraphManagerAction::Save { name, description } => {
            assert_eq!(name, "graph");
            assert!(description.is_none());
        }
        _ => panic!("expected Save"),
    }
}

#[test]
fn test_remove_graph_nonexistent_id() {
    let mut panel = GraphManagerPanel::new();
    panel.add_graph(GraphMetadata {
        id: "g1".to_string(),
        name: "G1".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-01-01T01:00:00Z".to_string(),
        node_count: 1,
        edge_count: 0,
    });
    panel.remove_graph("nonexistent");
    assert_eq!(panel.available_graphs.len(), 1);
}

#[test]
fn test_add_multiple_graphs() {
    let mut panel = GraphManagerPanel::new();
    for i in 0..5 {
        panel.add_graph(GraphMetadata {
            id: format!("g{i}"),
            name: format!("Graph {i}"),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            modified_at: "2026-01-01T01:00:00Z".to_string(),
            node_count: i,
            edge_count: i.saturating_sub(1),
        });
    }
    assert_eq!(panel.available_graphs.len(), 5);
}

#[test]
fn test_graph_manager_action_clone() {
    let save = GraphManagerAction::Save {
        name: "x".to_string(),
        description: Some("y".to_string()),
    };
    let cloned = save.clone();
    match (&save, &cloned) {
        (GraphManagerAction::Save { name: a, .. }, GraphManagerAction::Save { name: b, .. }) => {
            assert_eq!(a, b);
        }
        _ => panic!(),
    }

    let load = GraphManagerAction::Load("id".to_string());
    let _ = load;

    let _ = GraphManagerAction::Execute;
    let del = GraphManagerAction::Delete("d".to_string());
    let _ = del;
}

#[test]
fn test_graph_manager_action_debug() {
    let s = format!("{:?}", GraphManagerAction::Execute);
    assert!(s.contains("Execute"));

    let s = format!("{:?}", GraphManagerAction::Load("x".to_string()));
    assert!(s.contains("Load"));
    assert!(s.contains('x'));
}

#[test]
fn test_panel_initial_state() {
    let panel = GraphManagerPanel::new();
    assert!(panel.last_refresh.is_none());
    assert!(panel.execution_status.is_none());
    assert!(panel.error_message.is_none());
    assert!(!panel.show_save_dialog);
}

#[test]
fn test_is_graph_selected() {
    assert!(is_graph_selected(Some("a".to_string()).as_ref(), "a"));
    assert!(!is_graph_selected(Some("a".to_string()).as_ref(), "b"));
    assert!(!is_graph_selected(None, "a"));
}

#[test]
fn test_save_description_opt() {
    assert_eq!(save_description_opt(""), None);
    assert_eq!(save_description_opt("desc"), Some("desc".to_string()));
}

#[test]
fn test_format_graph_stats() {
    let (n, e) = format_graph_stats(5, 3);
    assert_eq!(n, "📊 5 nodes");
    assert_eq!(e, "🔗 3 edges");
}

#[test]
fn test_format_modified_at() {
    assert_eq!(format_modified_at("2026-01-01"), "Modified: 2026-01-01");
}

#[test]
fn test_format_execution_status() {
    assert!(format_execution_status("Running").contains("Execution:"));
}

#[test]
fn test_format_error_display() {
    assert!(format_error_display("fail").contains("Error:"));
}

#[test]
fn test_remove_graph_does_not_clear_other_selection() {
    let mut panel = GraphManagerPanel::new();
    panel.add_graph(GraphMetadata {
        id: "g1".to_string(),
        name: "G1".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-01-01T01:00:00Z".to_string(),
        node_count: 1,
        edge_count: 0,
    });
    panel.add_graph(GraphMetadata {
        id: "g2".to_string(),
        name: "G2".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-01-01T01:00:00Z".to_string(),
        node_count: 2,
        edge_count: 1,
    });
    panel.selected_graph_id = Some("g2".to_string());
    panel.remove_graph("g1");
    assert_eq!(panel.selected_graph_id.as_deref(), Some("g2"));
}

#[test]
fn test_render_no_provider() {
    let mut panel = GraphManagerPanel::new();
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let runtime = tokio::runtime::Runtime::new().expect("create runtime");

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let action = panel.render(ui, &palette, None, None, &runtime);
            assert!(action.is_none());
        });
    });
}

#[test]
fn test_render_with_error_message() {
    let mut panel = GraphManagerPanel::new();
    panel.set_error(Some("Test error message".to_string()));
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let runtime = tokio::runtime::Runtime::new().expect("create runtime");

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let _ = panel.render(ui, &palette, None, None, &runtime);
        });
    });
}

#[test]
fn test_render_with_execution_status() {
    let mut panel = GraphManagerPanel::new();
    panel.set_execution_status(Some("Running...".to_string()));
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let runtime = tokio::runtime::Runtime::new().expect("create runtime");

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let _ = panel.render(ui, &palette, None, None, &runtime);
        });
    });
}

#[test]
fn test_format_graph_stats_zero() {
    let (n, e) = format_graph_stats(0, 0);
    assert_eq!(n, "📊 0 nodes");
    assert_eq!(e, "🔗 0 edges");
}

#[test]
fn test_format_error_display_empty() {
    let s = format_error_display("");
    assert!(s.contains("Error:"));
}

#[test]
fn test_is_graph_selected_empty_string() {
    assert!(!is_graph_selected(Some(String::new()).as_ref(), "a"));
    assert!(is_graph_selected(Some(String::new()).as_ref(), ""));
}

#[test]
fn test_format_graph_stats_large_numbers() {
    let (n, e) = format_graph_stats(999, 500);
    assert_eq!(n, "📊 999 nodes");
    assert_eq!(e, "🔗 500 edges");
}

#[test]
fn test_format_execution_status_empty() {
    let s = format_execution_status("");
    assert!(s.contains("Execution:"));
}

#[test]
fn test_format_modified_at_empty() {
    assert_eq!(format_modified_at(""), "Modified: ");
}

#[test]
fn test_save_description_opt_whitespace_only() {
    assert_eq!(save_description_opt("   "), Some("   ".to_string()));
}

#[test]
fn test_graph_metadata_with_all_fields() {
    let meta = GraphMetadata {
        id: "full-id".to_string(),
        name: "Full Name".to_string(),
        description: Some("Full description".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-01-16T12:00:00Z".to_string(),
        node_count: 100,
        edge_count: 99,
    };
    let (n, e) = format_graph_stats(meta.node_count, meta.edge_count);
    assert_eq!(n, "📊 100 nodes");
    assert_eq!(e, "🔗 99 edges");
}

#[test]
#[cfg(feature = "mock")]
fn test_render_with_provider_and_graphs() {
    use petal_tongue_core::graph_builder::VisualGraph;
    use petal_tongue_discovery::NeuralApiProvider;
    use std::path::PathBuf;

    let mut panel = GraphManagerPanel::new();
    panel.add_graph(GraphMetadata {
        id: "g1".to_string(),
        name: "Graph One".to_string(),
        description: Some("First graph".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-01-01T01:00:00Z".to_string(),
        node_count: 3,
        edge_count: 2,
    });
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let runtime = tokio::runtime::Runtime::new().expect("create runtime");
    let provider = Arc::new(NeuralApiProvider::with_socket_path(PathBuf::from(
        "/nonexistent/socket",
    )));
    let current_graph = VisualGraph::new("Current");

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let _ = panel.render(
                ui,
                &palette,
                Some(&provider),
                Some(&current_graph),
                &runtime,
            );
        });
    });
}

#[test]
#[cfg(feature = "mock")]
fn test_render_with_provider_no_graph() {
    use petal_tongue_discovery::NeuralApiProvider;
    use std::path::PathBuf;

    let mut panel = GraphManagerPanel::new();
    panel.add_graph(GraphMetadata {
        id: "g1".to_string(),
        name: "Graph One".to_string(),
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        modified_at: "2026-01-01T01:00:00Z".to_string(),
        node_count: 1,
        edge_count: 0,
    });
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let runtime = tokio::runtime::Runtime::new().expect("create runtime");
    let provider = Arc::new(NeuralApiProvider::with_socket_path(PathBuf::from(
        "/nonexistent/socket",
    )));

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let _ = panel.render(ui, &palette, Some(&provider), None, &runtime);
        });
    });
}
