// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use petal_tongue_core::{GraphEngine, LayoutAlgorithm, TopologyEdge};

fn create_test_graph() -> Arc<RwLock<GraphEngine>> {
    let mut graph = GraphEngine::new();

    // Add BearDog (Security)
    let mut beardog =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("beardog-1", "Security");
    beardog.name = "BearDog Security".to_string();
    beardog.capabilities = vec!["auth".to_string(), "encryption".to_string()];
    beardog.health = PrimalHealthStatus::Healthy;
    graph.add_node(beardog);

    // Add ToadStool (Compute)
    let mut toadstool =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("toadstool-1", "Compute");
    toadstool.name = "ToadStool Compute".to_string();
    toadstool.capabilities = vec!["runtime".to_string(), "execution".to_string()];
    toadstool.health = PrimalHealthStatus::Warning;
    graph.add_node(toadstool);

    // Add Songbird (Discovery)
    let mut songbird =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("songbird-1", "Discovery");
    songbird.name = "Songbird Discovery".to_string();
    songbird.capabilities = vec!["discovery".to_string()];
    songbird.health = PrimalHealthStatus::Healthy;
    graph.add_node(songbird);

    graph.add_edge(TopologyEdge {
        from: "beardog-1".into(),
        to: "toadstool-1".into(),
        edge_type: "api".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });

    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);

    Arc::new(RwLock::new(graph))
}

#[test]
fn test_renderer_creation() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);
    assert!((renderer.master_volume() - 0.7).abs() < 0.001);
    assert!(renderer.is_enabled());
}

#[test]
fn test_instrument_mapping() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);

    assert_eq!(
        renderer.map_primal_to_instrument("Security"),
        Instrument::Bass
    );
    assert_eq!(
        renderer.map_primal_to_instrument("Compute"),
        Instrument::Drums
    );
    assert_eq!(
        renderer.map_primal_to_instrument("Discovery"),
        Instrument::Chimes
    );
    assert_eq!(
        renderer.map_primal_to_instrument("Storage"),
        Instrument::Strings
    );
    assert_eq!(renderer.map_primal_to_instrument("AI"), Instrument::Synth);
}

#[test]
fn test_health_to_pitch() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);

    assert!((renderer.health_to_pitch(PrimalHealthStatus::Healthy) - 0.75).abs() < f32::EPSILON);
    assert!((renderer.health_to_pitch(PrimalHealthStatus::Warning) - 0.55).abs() < f32::EPSILON);
    assert!((renderer.health_to_pitch(PrimalHealthStatus::Critical) - 0.25).abs() < f32::EPSILON);
    assert!((renderer.health_to_pitch(PrimalHealthStatus::Unknown) - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_position_to_pan() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);

    // Left side
    let left_pos = Position::new_2d(-500.0, 0.0);
    assert!((renderer.position_to_pan(left_pos) - (-1.0)).abs() < f32::EPSILON);

    // Center
    let center_pos = Position::new_2d(0.0, 0.0);
    assert!((renderer.position_to_pan(center_pos) - 0.0).abs() < f32::EPSILON);

    // Right side
    let right_pos = Position::new_2d(500.0, 0.0);
    assert!((renderer.position_to_pan(right_pos) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_generate_audio_attributes() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);

    let attrs = renderer.generate_audio_attributes();
    assert_eq!(attrs.len(), 3);

    // Find BearDog
    let beardog_attrs = attrs.iter().find(|(id, _)| id == "beardog-1").unwrap();
    assert_eq!(beardog_attrs.1.instrument, Instrument::Bass);
    assert!((beardog_attrs.1.pitch - 0.75).abs() < f32::EPSILON);
}

#[test]
fn test_master_volume() {
    let graph = create_test_graph();
    let mut renderer = AudioSonificationRenderer::new(graph);

    renderer.set_master_volume(0.5);
    assert!((renderer.master_volume() - 0.5).abs() < 0.001);

    // Test clamping
    renderer.set_master_volume(1.5);
    assert!((renderer.master_volume() - 1.0).abs() < 0.001);

    renderer.set_master_volume(-0.5);
    assert!((renderer.master_volume() - 0.0).abs() < 0.001);
}

#[test]
fn test_enable_disable() {
    let graph = create_test_graph();
    let mut renderer = AudioSonificationRenderer::new(graph);

    assert!(renderer.is_enabled());

    renderer.set_enabled(false);
    assert!(!renderer.is_enabled());

    renderer.set_enabled(true);
    assert!(renderer.is_enabled());
}

#[test]
fn test_describe_soundscape() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);

    let description = renderer.describe_soundscape();
    assert!(description.contains("3 primals"));
    assert!(description.contains("2 healthy, 1 warnings"));
    assert!(description.contains("bass"));
    assert!(description.contains("drums"));
    assert!(description.contains("chimes"));
}

#[test]
fn test_describe_node_audio() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);

    let description = renderer.describe_node_audio("beardog-1").unwrap();
    assert!(description.contains("BearDog Security"));
    assert!(description.contains("deep bass"));
    assert!(description.contains("harmonic"));

    let description = renderer.describe_node_audio("toadstool-1").unwrap();
    assert!(description.contains("ToadStool Compute"));
    assert!(description.contains("drums"));
    assert!(description.contains("off-key"));

    assert!(renderer.describe_node_audio("nonexistent").is_none());
}

#[test]
fn test_instrument_default_unknown_type() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);
    assert_eq!(
        renderer.map_primal_to_instrument("unknown"),
        Instrument::Default
    );
    assert_eq!(renderer.map_primal_to_instrument(""), Instrument::Default);
    assert_eq!(
        renderer.map_primal_to_instrument("FOO"),
        Instrument::Default
    );
}

#[test]
fn test_instrument_case_insensitive() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);
    assert_eq!(
        renderer.map_primal_to_instrument("security"),
        Instrument::Bass
    );
    assert_eq!(
        renderer.map_primal_to_instrument("SECURITY"),
        Instrument::Bass
    );
    assert_eq!(
        renderer.map_primal_to_instrument("compute"),
        Instrument::Drums
    );
    assert_eq!(renderer.map_primal_to_instrument("AI"), Instrument::Synth);
}

#[test]
fn test_position_to_pan_clamp() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);
    let far_left = Position::new_2d(-1000.0, 0.0);
    assert!((renderer.position_to_pan(far_left) - (-1.0)).abs() < f32::EPSILON);
    let far_right = Position::new_2d(1000.0, 0.0);
    assert!((renderer.position_to_pan(far_right) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_activity_to_volume_via_attributes() {
    let mut graph = GraphEngine::new();
    let mut low_cap =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("low", "Compute");
    low_cap.capabilities = vec!["a".to_string()];
    graph.add_node(low_cap);
    let mut high_cap =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("high", "Compute");
    high_cap.capabilities = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
        "e".to_string(),
        "f".to_string(),
        "g".to_string(),
        "h".to_string(),
        "i".to_string(),
        "j".to_string(),
    ];
    graph.add_node(high_cap);
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);
    let renderer = AudioSonificationRenderer::new(Arc::new(RwLock::new(graph)));
    let attrs = renderer.generate_audio_attributes();
    let low_vol = attrs.iter().find(|(id, _)| id == "low").unwrap().1.volume;
    let high_vol = attrs.iter().find(|(id, _)| id == "high").unwrap().1.volume;
    assert!(high_vol > low_vol);
}

#[test]
fn test_audio_attributes_structure() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);
    let attrs = renderer.generate_audio_attributes();
    for (_, a) in &attrs {
        assert!(a.pitch >= 0.0 && a.pitch <= 1.0);
        assert!(a.volume >= 0.0 && a.volume <= 1.0);
        assert!(a.pan >= -1.0 && a.pan <= 1.0);
    }
}

#[test]
fn test_describe_soundscape_empty() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let renderer = AudioSonificationRenderer::new(graph);
    let desc = renderer.describe_soundscape();
    assert!(desc.contains("silent"));
    assert!(desc.contains("No primals"));
}

#[test]
fn test_master_volume_zero_silence() {
    let graph = create_test_graph();
    let mut renderer = AudioSonificationRenderer::new(graph);
    renderer.set_master_volume(0.0);
    let attrs = renderer.generate_audio_attributes();
    for (_, a) in &attrs {
        assert!((a.volume - 0.0).abs() < f32::EPSILON);
    }
}

#[test]
fn test_master_volume_max() {
    let graph = create_test_graph();
    let mut renderer = AudioSonificationRenderer::new(graph);
    renderer.set_master_volume(1.0);
    assert!((renderer.master_volume() - 1.0).abs() < f32::EPSILON);
    let attrs = renderer.generate_audio_attributes();
    for (_, a) in &attrs {
        assert!(a.volume <= 1.0);
    }
}

#[test]
fn test_activity_to_volume_zero_capabilities() {
    let mut graph = GraphEngine::new();
    let mut node =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("zero-cap", "Compute");
    node.capabilities = vec![];
    graph.add_node(node);
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);
    let renderer = AudioSonificationRenderer::new(Arc::new(RwLock::new(graph)));
    let attrs = renderer.generate_audio_attributes();
    let vol = attrs
        .iter()
        .find(|(id, _)| *id == "zero-cap")
        .unwrap()
        .1
        .volume;
    assert!((0.0..=1.0).contains(&vol));
    assert!(vol < 0.5, "Zero capabilities should have lower volume");
}

#[test]
fn test_describe_soundscape_all_critical() {
    let mut graph = GraphEngine::new();
    let mut n1 = petal_tongue_core::test_fixtures::primals::test_primal_with_type("n1", "Compute");
    n1.health = PrimalHealthStatus::Critical;
    graph.add_node(n1);
    let mut n2 = petal_tongue_core::test_fixtures::primals::test_primal_with_type("n2", "Storage");
    n2.health = PrimalHealthStatus::Critical;
    graph.add_node(n2);
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);
    let renderer = AudioSonificationRenderer::new(Arc::new(RwLock::new(graph)));
    let desc = renderer.describe_soundscape();
    assert!(desc.contains("critical"));
    assert!(desc.contains("Dissonant"));
}

#[test]
fn test_describe_soundscape_all_healthy() {
    let mut graph = GraphEngine::new();
    let mut n1 = petal_tongue_core::test_fixtures::primals::test_primal_with_type("n1", "AI");
    n1.health = PrimalHealthStatus::Healthy;
    graph.add_node(n1);
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);
    let renderer = AudioSonificationRenderer::new(Arc::new(RwLock::new(graph)));
    let desc = renderer.describe_soundscape();
    assert!(desc.contains("harmony"));
}

#[test]
fn test_position_to_pan_fractional() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);
    let pos = Position::new_2d(250.0, 0.0);
    let pan = renderer.position_to_pan(pos);
    assert!((pan - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_audio_attributes_pan_range() {
    let graph = create_test_graph();
    let renderer = AudioSonificationRenderer::new(graph);
    let attrs = renderer.generate_audio_attributes();
    for (_, a) in &attrs {
        assert!(a.pan >= -1.0 && a.pan <= 1.0, "Pan must be in [-1, 1]");
    }
}

#[test]
fn test_describe_node_audio_critical() {
    let mut graph = GraphEngine::new();
    let mut node =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("critical-node", "AI");
    node.name = "Critical AI".to_string();
    node.health = PrimalHealthStatus::Critical;
    graph.add_node(node);
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);
    let renderer = AudioSonificationRenderer::new(Arc::new(RwLock::new(graph)));
    let desc = renderer.describe_node_audio("critical-node").unwrap();
    assert!(desc.contains("dissonant"));
}

#[test]
fn test_describe_node_audio_unknown_health() {
    let mut graph = GraphEngine::new();
    let mut node =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("unknown-node", "AI");
    node.name = "Unknown AI".to_string();
    node.health = PrimalHealthStatus::Unknown;
    graph.add_node(node);
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);
    let renderer = AudioSonificationRenderer::new(Arc::new(RwLock::new(graph)));
    let desc = renderer.describe_node_audio("unknown-node").unwrap();
    assert!(desc.contains("neutral"));
}

#[test]
fn test_describe_node_audio_position_left() {
    let mut graph = GraphEngine::new();
    let node =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("left-node", "Compute");
    graph.add_node(node);
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);
    if let Some(n) = graph.get_node_mut("left-node") {
        n.position = Position::new_2d(-400.0, 0.0);
    }
    let renderer = AudioSonificationRenderer::new(Arc::new(RwLock::new(graph)));
    let desc = renderer.describe_node_audio("left-node").unwrap();
    assert!(desc.contains("left"));
}

#[test]
fn test_describe_node_audio_position_right() {
    let mut graph = GraphEngine::new();
    let node =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("right-node", "Compute");
    graph.add_node(node);
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);
    if let Some(n) = graph.get_node_mut("right-node") {
        n.position = Position::new_2d(400.0, 0.0);
    }
    let renderer = AudioSonificationRenderer::new(Arc::new(RwLock::new(graph)));
    let desc = renderer.describe_node_audio("right-node").unwrap();
    assert!(desc.contains("right"));
}

#[test]
fn test_describe_node_audio_position_center() {
    let mut graph = GraphEngine::new();
    let node =
        petal_tongue_core::test_fixtures::primals::test_primal_with_type("center-node", "Compute");
    graph.add_node(node);
    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);
    if let Some(n) = graph.get_node_mut("center-node") {
        n.position = Position::new_2d(0.0, 0.0);
    }
    let renderer = AudioSonificationRenderer::new(Arc::new(RwLock::new(graph)));
    let desc = renderer.describe_node_audio("center-node").unwrap();
    assert!(desc.contains("centered"));
}

#[test]
fn test_activity_to_volume_formula() {
    let activity = |cap_count: usize| {
        let normalized = (cap_count as f32 / 10.0).min(1.0);
        0.3 + (normalized * 0.7)
    };
    assert!((activity(0) - 0.3).abs() < f32::EPSILON);
    assert!((activity(10) - 1.0).abs() < f32::EPSILON);
    assert!(activity(5) > activity(0));
    assert!(activity(5) < activity(10));
}

#[test]
fn test_health_to_pitch_all_variants() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let renderer = AudioSonificationRenderer::new(graph);
    let h = renderer.health_to_pitch(PrimalHealthStatus::Healthy);
    let w = renderer.health_to_pitch(PrimalHealthStatus::Warning);
    let c = renderer.health_to_pitch(PrimalHealthStatus::Critical);
    let u = renderer.health_to_pitch(PrimalHealthStatus::Unknown);
    assert!(c < u && u < w && w < h);
}

#[test]
fn test_instrument_enum_variants() {
    assert_ne!(Instrument::Bass, Instrument::Drums);
    assert_ne!(Instrument::Chimes, Instrument::Strings);
    assert_eq!(Instrument::Default, Instrument::Default);
}

#[test]
fn test_audio_attributes_clone() {
    let attrs = AudioAttributes {
        instrument: Instrument::Synth,
        pitch: 0.75,
        volume: 0.8,
        pan: 0.0,
    };
    let cloned = attrs.clone();
    assert_eq!(cloned.instrument, attrs.instrument);
    assert!((cloned.pitch - attrs.pitch).abs() < f32::EPSILON);
}

#[test]
fn test_generate_audio_attributes_empty_graph() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let renderer = AudioSonificationRenderer::new(graph);
    let attrs = renderer.generate_audio_attributes();
    assert!(attrs.is_empty());
}
