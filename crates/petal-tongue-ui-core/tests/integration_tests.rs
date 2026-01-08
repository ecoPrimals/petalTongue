//! Integration tests for petal-tongue-ui-core

use anyhow::Result;
use petal_tongue_core::{
    GraphEngine, LayoutAlgorithm, PrimalHealthStatus, PrimalInfo, TopologyEdge,
};
use petal_tongue_ui_core::{
    CanvasUI, ExportFormat, SvgUI, TerminalUI, TextUI, UIMode, UniversalUI, detect_best_ui_mode,
};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Create a test graph with sample data
fn create_test_graph() -> Arc<RwLock<GraphEngine>> {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let mut g = graph.write().unwrap();

    // Add test primals
    let primals = vec![
        PrimalInfo::new(
            "test-1",
            "Test Primal 1",
            "Compute",
            "http://localhost:8001",
            vec!["compute".to_string(), "test".to_string()],
            PrimalHealthStatus::Healthy,
            1704571200,
        ),
        PrimalInfo::new(
            "test-2",
            "Test Primal 2",
            "Storage",
            "http://localhost:8002",
            vec!["storage".to_string(), "test".to_string()],
            PrimalHealthStatus::Warning,
            1704571200,
        ),
        PrimalInfo::new(
            "test-3",
            "Test Primal 3",
            "Security",
            "http://localhost:8003",
            vec!["security".to_string(), "test".to_string()],
            PrimalHealthStatus::Critical,
            1704571200,
        ),
    ];

    for primal in primals {
        g.add_node(primal);
    }

    // Add test edges
    g.add_edge(TopologyEdge {
        from: "test-1".to_string(),
        to: "test-2".to_string(),
        edge_type: "depends_on".to_string(),
        label: Some("Dependency".to_string()),
    });
    g.add_edge(TopologyEdge {
        from: "test-2".to_string(),
        to: "test-3".to_string(),
        edge_type: "secures".to_string(),
        label: Some("Security".to_string()),
    });

    // Apply layout
    g.layout(5);

    drop(g);
    graph
}

#[test]
fn test_svg_ui_integration() -> Result<()> {
    let graph = create_test_graph();
    let ui = SvgUI::new(graph, 800, 600);

    // Test rendering to string
    let svg = ui.render_to_string()?;
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("Test Primal 1"));
    assert!(svg.contains("Test Primal 2"));

    Ok(())
}

#[test]
fn test_terminal_ui_integration() -> Result<()> {
    let graph = create_test_graph();
    let ui = TerminalUI::new(graph);

    // Test rendering to string
    let output = ui.render_to_string()?;
    assert!(output.contains("petalTongue Topology"));
    assert!(output.contains("PRIMALS:"));
    assert!(output.contains("CONNECTIONS:"));
    assert!(output.contains("Test Primal 1"));
    assert!(output.contains("Test Primal 2"));

    Ok(())
}

#[test]
fn test_text_ui_json_integration() -> Result<()> {
    let graph = create_test_graph();
    let ui = TextUI::new(graph).with_format(ExportFormat::Json);

    // Test rendering to string
    let json = ui.render_to_string()?;
    let parsed: serde_json::Value = serde_json::from_str(&json)?;

    assert!(parsed["topology"]["primals"].is_array());
    assert_eq!(parsed["topology"]["primals"].as_array().unwrap().len(), 3);
    assert!(parsed["topology"]["connections"].is_array());
    assert_eq!(
        parsed["topology"]["connections"].as_array().unwrap().len(),
        2
    );

    Ok(())
}

#[test]
fn test_text_ui_dot_integration() -> Result<()> {
    let graph = create_test_graph();
    let ui = TextUI::new(graph).with_format(ExportFormat::Dot);

    // Test rendering to string
    let dot = ui.render_to_string()?;
    assert!(dot.contains("digraph PetalTongue"));
    assert!(dot.contains("\"test-1\""));
    assert!(dot.contains("\"test-2\""));
    assert!(dot.contains("->"));

    Ok(())
}

#[test]
fn test_canvas_ui_integration() -> Result<()> {
    let graph = create_test_graph();
    let ui = CanvasUI::new(graph, 800, 600);

    // Test capabilities
    assert!(ui.supports(petal_tongue_ui_core::UICapability::Export));

    Ok(())
}

#[test]
fn test_svg_export_to_file() -> Result<()> {
    let graph = create_test_graph();
    let ui = SvgUI::new(graph, 800, 600);

    let temp_file = std::env::temp_dir().join("test_topology.svg");
    ui.export(&temp_file, ExportFormat::Svg)?;

    // Verify file exists and contains valid SVG
    assert!(temp_file.exists());
    let content = std::fs::read_to_string(&temp_file)?;
    assert!(content.contains("<svg"));
    assert!(content.contains("</svg>"));

    // Cleanup
    std::fs::remove_file(temp_file)?;

    Ok(())
}

#[test]
fn test_json_export_to_file() -> Result<()> {
    let graph = create_test_graph();
    let ui = TextUI::new(graph).with_format(ExportFormat::Json);

    let temp_file = std::env::temp_dir().join("test_topology.json");
    ui.export(&temp_file, ExportFormat::Json)?;

    // Verify file exists and contains valid JSON
    assert!(temp_file.exists());
    let content = std::fs::read_to_string(&temp_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    assert!(parsed["topology"].is_object());

    // Cleanup
    std::fs::remove_file(temp_file)?;

    Ok(())
}

#[test]
fn test_dot_export_to_file() -> Result<()> {
    let graph = create_test_graph();
    let ui = TextUI::new(graph).with_format(ExportFormat::Dot);

    let temp_file = std::env::temp_dir().join("test_topology.dot");
    ui.export(&temp_file, ExportFormat::Dot)?;

    // Verify file exists and contains valid DOT
    assert!(temp_file.exists());
    let content = std::fs::read_to_string(&temp_file)?;
    assert!(content.contains("digraph"));

    // Cleanup
    std::fs::remove_file(temp_file)?;

    Ok(())
}

#[test]
fn test_ui_mode_detection() {
    // Test that detection doesn't crash
    let mode = detect_best_ui_mode();

    // In CI/test environment, should detect headless or terminal
    assert!(matches!(
        mode,
        UIMode::Headless | UIMode::Terminal | UIMode::Display
    ));
}

#[test]
fn test_multiple_formats_from_same_graph() -> Result<()> {
    let graph = create_test_graph();

    // Render to multiple formats
    let svg_ui = SvgUI::new(graph.clone(), 800, 600);
    let svg = svg_ui.render_to_string()?;
    assert!(svg.contains("<svg"));

    let terminal_ui = TerminalUI::new(graph.clone());
    let terminal = terminal_ui.render_to_string()?;
    assert!(terminal.contains("petalTongue"));

    let json_ui = TextUI::new(graph.clone()).with_format(ExportFormat::Json);
    let json = json_ui.render_to_string()?;
    let _parsed: serde_json::Value = serde_json::from_str(&json)?;

    Ok(())
}

#[test]
fn test_empty_graph_rendering() -> Result<()> {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // SVG should render empty graph without crashing
    let svg_ui = SvgUI::new(graph.clone(), 800, 600);
    let svg = svg_ui.render_to_string()?;
    assert!(svg.contains("<svg"));

    // Terminal should render empty graph
    let terminal_ui = TerminalUI::new(graph.clone());
    let terminal = terminal_ui.render_to_string()?;
    assert!(terminal.contains("petalTongue"));

    // JSON should render empty arrays
    let json_ui = TextUI::new(graph).with_format(ExportFormat::Json);
    let json = json_ui.render_to_string()?;
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    assert_eq!(parsed["topology"]["primals"].as_array().unwrap().len(), 0);

    Ok(())
}

#[test]
fn test_large_graph_rendering() -> Result<()> {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let mut g = graph.write().unwrap();

    // Create 20 primals
    for i in 0..20 {
        let primal = PrimalInfo::new(
            format!("primal-{}", i),
            format!("Primal {}", i),
            "Test",
            format!("http://localhost:800{}", i),
            vec!["test".to_string()],
            if i % 3 == 0 {
                PrimalHealthStatus::Healthy
            } else if i % 3 == 1 {
                PrimalHealthStatus::Warning
            } else {
                PrimalHealthStatus::Critical
            },
            1704571200,
        );
        g.add_node(primal);
    }

    // Create edges
    for i in 0..19 {
        g.add_edge(TopologyEdge {
            from: format!("primal-{}", i),
            to: format!("primal-{}", i + 1),
            edge_type: "connects".to_string(),
            label: Some(format!("Connection {}", i)),
        });
    }

    g.layout(10);
    drop(g);

    // Test rendering large graph
    let svg_ui = SvgUI::new(graph.clone(), 1920, 1080);
    let svg = svg_ui.render_to_string()?;
    assert!(svg.contains("<svg"));

    let json_ui = TextUI::new(graph).with_format(ExportFormat::Json);
    let json = json_ui.render_to_string()?;
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    assert_eq!(parsed["topology"]["primals"].as_array().unwrap().len(), 20);

    Ok(())
}

#[test]
fn test_health_status_colors() -> Result<()> {
    use petal_tongue_ui_core::{health_to_color, health_to_emoji, health_to_percentage};

    // Test all health statuses
    assert_eq!(health_to_percentage(&PrimalHealthStatus::Healthy), 100);
    assert_eq!(health_to_percentage(&PrimalHealthStatus::Warning), 75);
    assert_eq!(health_to_percentage(&PrimalHealthStatus::Critical), 25);
    assert_eq!(health_to_percentage(&PrimalHealthStatus::Unknown), 50);

    assert_eq!(health_to_color(&PrimalHealthStatus::Healthy), "#4ade80");
    assert_eq!(health_to_color(&PrimalHealthStatus::Warning), "#facc15");
    assert_eq!(health_to_color(&PrimalHealthStatus::Critical), "#f87171");
    assert_eq!(health_to_color(&PrimalHealthStatus::Unknown), "#9ca3af");

    assert_eq!(health_to_emoji(&PrimalHealthStatus::Healthy), "🟢");
    assert_eq!(health_to_emoji(&PrimalHealthStatus::Warning), "🟡");
    assert_eq!(health_to_emoji(&PrimalHealthStatus::Critical), "🔴");
    assert_eq!(health_to_emoji(&PrimalHealthStatus::Unknown), "⚪");

    Ok(())
}
