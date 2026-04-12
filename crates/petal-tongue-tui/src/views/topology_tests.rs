// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo, TopologyEdge};

fn line_text(line: &ratatui::text::Line) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

#[test]
fn render_ascii_graph_empty_primals() {
    let primals: Vec<PrimalInfo> = vec![];
    let topology: Vec<TopologyEdge> = vec![];
    let lines = render_ascii_graph(&primals, &topology);
    assert_eq!(lines.len(), 1);
    assert_eq!(line_text(&lines[0]), "");
}

#[test]
fn render_ascii_graph_one_primal_no_edges() {
    let primals = vec![PrimalInfo::new(
        PrimalId::from("test-primal"),
        "TestPrimal",
        "Compute",
        format!(
            "http://{}",
            petal_tongue_core::constants::default_biomeos_connection_target()
        ),
        vec![],
        PrimalHealthStatus::Healthy,
        0,
    )];
    let topology: Vec<TopologyEdge> = vec![];
    let lines = render_ascii_graph(&primals, &topology);
    assert!(lines.len() >= 5);
    let all_text: String = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
    assert!(all_text.contains("TestPrimal"));
    assert!(all_text.contains("Compute"));
}

#[test]
fn render_ascii_graph_with_topology_edges() {
    let primals = vec![
        PrimalInfo::new(
            PrimalId::from("a"),
            "PrimalA",
            "Compute",
            "http://a",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            PrimalId::from("b"),
            "PrimalB",
            "Storage",
            "http://b",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
    ];
    let topology = vec![TopologyEdge {
        from: PrimalId::from("a"),
        to: PrimalId::from("b"),
        edge_type: "connection".to_string(),
        label: None,
        capability: None,
        metrics: None,
    }];
    let lines = render_ascii_graph(&primals, &topology);
    assert!(lines.len() >= 10);
    let all_text: String = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
    assert!(all_text.contains("PrimalA"));
    assert!(all_text.contains("PrimalB"));
    assert!(all_text.contains("connection"));
}

#[test]
fn render_ascii_graph_health_icons() {
    let primals = vec![
        PrimalInfo::new(
            PrimalId::from("h"),
            "Healthy",
            "T",
            "http://h",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            PrimalId::from("w"),
            "Warning",
            "T",
            "http://w",
            vec![],
            PrimalHealthStatus::Warning,
            0,
        ),
    ];
    let topology: Vec<TopologyEdge> = vec![];
    let lines = render_ascii_graph(&primals, &topology);
    let all_text: String = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
    assert!(all_text.contains("Healthy"));
    assert!(all_text.contains("Warning"));
    assert!(all_text.contains("✅"));
    assert!(all_text.contains("⚠️"));
}

#[test]
fn health_icon_for_status_mapping() {
    assert_eq!(health_icon_for_status(PrimalHealthStatus::Healthy), "✅");
    assert_eq!(health_icon_for_status(PrimalHealthStatus::Warning), "⚠️");
    assert_eq!(health_icon_for_status(PrimalHealthStatus::Critical), "❌");
    assert_eq!(health_icon_for_status(PrimalHealthStatus::Unknown), "❓");
}

#[test]
fn count_edge_types_empty() {
    let topology: Vec<TopologyEdge> = vec![];
    let counts = count_edge_types(&topology);
    assert!(counts.is_empty());
}

#[test]
fn count_edge_types_single_type() {
    let topology = vec![
        TopologyEdge {
            from: PrimalId::from("a"),
            to: PrimalId::from("b"),
            edge_type: "connection".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
        TopologyEdge {
            from: PrimalId::from("b"),
            to: PrimalId::from("c"),
            edge_type: "connection".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
    ];
    let counts = count_edge_types(&topology);
    assert_eq!(counts.get("connection"), Some(&2));
}

#[test]
fn count_edge_types_multiple_types() {
    let topology = vec![
        TopologyEdge {
            from: PrimalId::from("a"),
            to: PrimalId::from("b"),
            edge_type: "connection".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
        TopologyEdge {
            from: PrimalId::from("b"),
            to: PrimalId::from("c"),
            edge_type: "data".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
        TopologyEdge {
            from: PrimalId::from("c"),
            to: PrimalId::from("a"),
            edge_type: "connection".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
    ];
    let counts = count_edge_types(&topology);
    assert_eq!(counts.get("connection"), Some(&2));
    assert_eq!(counts.get("data"), Some(&1));
}

#[test]
fn force_directed_layout_empty() {
    let primals: Vec<PrimalInfo> = vec![];
    let topology: Vec<TopologyEdge> = vec![];
    let positions = force_directed_layout(&primals, &topology, 70.0, 20.0, 10);
    assert!(positions.is_empty());
}

#[test]
fn render_ascii_graph_critical_unknown_health() {
    let primals = vec![
        PrimalInfo::new(
            PrimalId::from("c"),
            "Critical",
            "T",
            "http://c",
            vec![],
            PrimalHealthStatus::Critical,
            0,
        ),
        PrimalInfo::new(
            PrimalId::from("u"),
            "Unknown",
            "T",
            "http://u",
            vec![],
            PrimalHealthStatus::Unknown,
            0,
        ),
    ];
    let topology: Vec<TopologyEdge> = vec![];
    let lines = render_ascii_graph(&primals, &topology);
    let all_text: String = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
    assert!(all_text.contains("Critical"));
    assert!(all_text.contains("Unknown"));
    assert!(all_text.contains("❌"));
    assert!(all_text.contains("❓"));
}

#[test]
fn force_directed_layout_two_nodes_one_edge() {
    let primals = vec![
        PrimalInfo::new(
            "a",
            "A",
            "Compute",
            "http://a",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            "b",
            "B",
            "Storage",
            "http://b",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
    ];
    let topology = vec![TopologyEdge {
        from: PrimalId::from("a"),
        to: PrimalId::from("b"),
        edge_type: "conn".to_string(),
        label: None,
        capability: None,
        metrics: None,
    }];
    let positions = force_directed_layout(&primals, &topology, 70.0, 20.0, 20);
    assert_eq!(positions.len(), 2);
    let id_a = PrimalId::from("a");
    let id_b = PrimalId::from("b");
    assert!(positions.contains_key(&id_a));
    assert!(positions.contains_key(&id_b));
    let (x1, y1) = positions.get(&id_a).unwrap();
    let (x2, y2) = positions.get(&id_b).unwrap();
    assert!(*x1 >= 0.0 && *x1 <= 70.0);
    assert!(*y1 >= 0.0 && *y1 <= 20.0);
    assert!(*x2 >= 0.0 && *x2 <= 70.0);
    assert!(*y2 >= 0.0 && *y2 <= 20.0);
}

#[test]
fn force_directed_layout_three_nodes() {
    let primals = vec![
        PrimalInfo::new(
            "a",
            "A",
            "T",
            "http://a",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            "b",
            "B",
            "T",
            "http://b",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            "c",
            "C",
            "T",
            "http://c",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
    ];
    let topology = vec![
        TopologyEdge {
            from: PrimalId::from("a"),
            to: PrimalId::from("b"),
            edge_type: "e1".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
        TopologyEdge {
            from: PrimalId::from("b"),
            to: PrimalId::from("c"),
            edge_type: "e2".to_string(),
            label: None,
            capability: None,
            metrics: None,
        },
    ];
    let positions = force_directed_layout(&primals, &topology, 100.0, 50.0, 30);
    assert_eq!(positions.len(), 3);
    for (x, y) in positions.values() {
        assert!(*x >= 0.0 && *x <= 100.0);
        assert!(*y >= 0.0 && *y <= 50.0);
    }
}
