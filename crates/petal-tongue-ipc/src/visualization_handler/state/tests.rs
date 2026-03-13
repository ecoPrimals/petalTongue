// SPDX-License-Identifier: AGPL-3.0-only
//! Tests for visualization state handlers.

use petal_tongue_core::DataBinding;
use petal_tongue_scene::grammar::{GeometryType, GrammarExpr};

use crate::visualization_handler::VisualizationState;
use crate::visualization_handler::types::{
    BackpressureConfig, DashboardRenderRequest, DismissRequest, ExportRequest,
    GrammarRenderRequest, SessionStatusRequest, StreamOperation, StreamUpdateRequest, UiConfig,
    ValidateRequest, VisualizationRenderRequest,
};

fn make_timeseries(id: &str) -> DataBinding {
    DataBinding::TimeSeries {
        id: id.to_string(),
        label: "Test".to_string(),
        x_label: "X".to_string(),
        y_label: "Y".to_string(),
        unit: String::new(),
        x_values: vec![0.0, 1.0, 2.0],
        y_values: vec![10.0, 20.0, 30.0],
    }
}

fn make_gauge(id: &str, value: f64) -> DataBinding {
    DataBinding::Gauge {
        id: id.to_string(),
        label: "Gauge".to_string(),
        value,
        min: 0.0,
        max: 100.0,
        unit: "%".to_string(),
        normal_range: [20.0, 80.0],
        warning_range: [10.0, 90.0],
    }
}

#[test]
fn test_visualization_state_new_and_default() {
    let state = VisualizationState::new();
    assert!(state.sessions().is_empty());
    assert!(state.all_bindings().is_empty());

    let default = VisualizationState::default();
    assert!(default.sessions().is_empty());
}

#[test]
fn test_handle_render_empty_bindings() {
    let mut state = VisualizationState::new();
    let req = VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "Empty".to_string(),
        bindings: vec![],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    };
    let resp = state.handle_render(req);
    assert_eq!(resp.session_id, "s1");
    assert_eq!(resp.bindings_accepted, 0);
    assert_eq!(resp.status, "rendering");
    assert_eq!(state.sessions().len(), 1);
    assert!(state.grammar_scene("s1").is_none());
}

#[test]
fn test_handle_render_with_bindings() {
    let mut state = VisualizationState::new();
    let bindings = vec![make_timeseries("b1"), make_gauge("b2", 50.0)];
    let req = VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "Dashboard".to_string(),
        bindings,
        thresholds: vec![],
        domain: Some("health".to_string()),
        ui_config: None,
    };
    let resp = state.handle_render(req);
    assert_eq!(resp.bindings_accepted, 2);
    let session = state.sessions().get("s1").expect("session exists");
    assert_eq!(session.bindings.len(), 2);
    assert_eq!(session.domain.as_deref(), Some("health"));
    assert!(state.grammar_scene("s1").is_none());
    assert!(state.grammar_scene("s1:b1").is_some());
    assert!(state.grammar_scene("s1:b2").is_some());
}

#[test]
fn test_session_bindings_and_all_bindings() {
    let mut state = VisualizationState::new();
    let req = VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "T".to_string(),
        bindings: vec![make_timeseries("b1")],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    };
    state.handle_render(req);
    assert_eq!(state.session_bindings("s1").map(<[_]>::len), Some(1));
    assert!(state.session_bindings("nonexistent").is_none());
    assert_eq!(state.all_bindings().len(), 1);
}

#[test]
fn test_handle_stream_update_session_not_found() {
    let mut state = VisualizationState::new();
    let req = StreamUpdateRequest {
        session_id: "nonexistent".to_string(),
        binding_id: "b1".to_string(),
        operation: StreamOperation::Append {
            x_values: vec![3.0],
            y_values: vec![40.0],
        },
    };
    let resp = state.handle_stream_update(req);
    assert!(!resp.accepted);
}

#[test]
fn test_handle_stream_update_binding_not_found() {
    let mut state = VisualizationState::new();
    state.handle_render(VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "T".to_string(),
        bindings: vec![make_timeseries("b1")],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    });
    let req = StreamUpdateRequest {
        session_id: "s1".to_string(),
        binding_id: "b99".to_string(),
        operation: StreamOperation::Append {
            x_values: vec![3.0],
            y_values: vec![40.0],
        },
    };
    let resp = state.handle_stream_update(req);
    assert!(!resp.accepted);
}

#[test]
fn test_handle_stream_update_append_timeseries() {
    let mut state = VisualizationState::new();
    state.handle_render(VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "T".to_string(),
        bindings: vec![make_timeseries("b1")],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    });
    let req = StreamUpdateRequest {
        session_id: "s1".to_string(),
        binding_id: "b1".to_string(),
        operation: StreamOperation::Append {
            x_values: vec![3.0],
            y_values: vec![40.0],
        },
    };
    let resp = state.handle_stream_update(req);
    assert!(resp.accepted);
    let bindings = state.session_bindings("s1").expect("session exists");
    let b = &bindings[0];
    if let DataBinding::TimeSeries {
        x_values, y_values, ..
    } = b
    {
        assert_eq!(x_values.len(), 4);
        assert_eq!(y_values.len(), 4);
        assert!((y_values[3] - 40.0).abs() < f64::EPSILON);
    } else {
        panic!("expected TimeSeries");
    }
}

#[test]
fn test_handle_stream_update_set_value_gauge() {
    let mut state = VisualizationState::new();
    state.handle_render(VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "T".to_string(),
        bindings: vec![make_gauge("g1", 50.0)],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    });
    let req = StreamUpdateRequest {
        session_id: "s1".to_string(),
        binding_id: "g1".to_string(),
        operation: StreamOperation::SetValue { value: 75.0 },
    };
    let resp = state.handle_stream_update(req);
    assert!(resp.accepted);
    let bindings = state.session_bindings("s1").expect("session exists");
    if let DataBinding::Gauge { value, .. } = &bindings[0] {
        assert!((*value - 75.0).abs() < f64::EPSILON);
    } else {
        panic!("expected Gauge");
    }
}

#[test]
fn test_handle_grammar_render() {
    let mut state = VisualizationState::new();
    let grammar = GrammarExpr::new("data", GeometryType::Point);
    let data = vec![
        serde_json::json!({"x": 1.0, "y": 2.0}),
        serde_json::json!({"x": 3.0, "y": 4.0}),
    ];
    let req = GrammarRenderRequest {
        session_id: "gram-s1".to_string(),
        grammar,
        data,
        modality: "svg".to_string(),
        validate_tufte: true,
        domain: None,
    };
    let resp = state.handle_grammar_render(req);
    assert_eq!(resp.session_id, "gram-s1");
    assert_eq!(resp.modality, "svg");
    assert!(resp.tufte_report.is_some());
    assert!(state.grammar_scene("gram-s1").is_some());
}

#[test]
fn test_handle_grammar_render_no_tufte() {
    let mut state = VisualizationState::new();
    let grammar = GrammarExpr::new("data", GeometryType::Point);
    let req = GrammarRenderRequest {
        session_id: "g2".to_string(),
        grammar,
        data: vec![serde_json::json!({"x": 1.0, "y": 2.0})],
        modality: "description".to_string(),
        validate_tufte: false,
        domain: None,
    };
    let resp = state.handle_grammar_render(req);
    assert!(resp.tufte_report.is_none());
    assert_eq!(resp.modality, "description");
}

#[test]
fn test_handle_dashboard_render() {
    let mut state = VisualizationState::new();
    let req = DashboardRenderRequest {
        session_id: "dash1".to_string(),
        title: "Dashboard".to_string(),
        bindings: vec![make_timeseries("b1"), make_gauge("b2", 60.0)],
        domain: None,
        modality: "svg".to_string(),
        max_columns: 2,
    };
    let resp = state.handle_dashboard_render(req);
    assert_eq!(resp.session_id, "dash1");
    assert_eq!(resp.panel_count, 2);
    assert!(state.grammar_scene("dash1").is_some());
}

#[test]
fn test_handle_validate() {
    let state = VisualizationState::new();
    let grammar = GrammarExpr::new("data", GeometryType::Point);
    let req = ValidateRequest {
        grammar,
        data: vec![serde_json::json!({"x": 1.0, "y": 2.0})],
    };
    let resp = state.handle_validate(&req);
    assert!(!resp.constraints.is_empty());
    assert!(resp.score >= 0.0 && resp.score <= 1.0);
}

#[test]
fn test_handle_export_session_not_found() {
    let state = VisualizationState::new();
    let req = ExportRequest {
        session_id: "missing".to_string(),
        format: "svg".to_string(),
    };
    let resp = state.handle_export(req);
    assert_eq!(resp.session_id, "missing");
    assert!(resp.content.is_empty());
}

#[test]
fn test_handle_export_session_found() {
    let mut state = VisualizationState::new();
    state.handle_grammar_render(GrammarRenderRequest {
        session_id: "ex1".to_string(),
        grammar: GrammarExpr::new("data", GeometryType::Point),
        data: vec![serde_json::json!({"x": 1.0, "y": 2.0})],
        modality: "svg".to_string(),
        validate_tufte: false,
        domain: None,
    });
    let req = ExportRequest {
        session_id: "ex1".to_string(),
        format: "svg".to_string(),
    };
    let resp = state.handle_export(req);
    assert!(!resp.content.is_empty());
    assert!(resp.content.contains("svg") || resp.content.contains('<'));
}

#[test]
fn test_handle_dismiss_session_exists() {
    let mut state = VisualizationState::new();
    state.handle_render(VisualizationRenderRequest {
        session_id: "d1".to_string(),
        title: "T".to_string(),
        bindings: vec![make_timeseries("b1")],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    });
    let req = DismissRequest {
        session_id: "d1".to_string(),
    };
    let resp = state.handle_dismiss(req);
    assert!(resp.dismissed);
    assert!(state.sessions().get("d1").is_none());
}

#[test]
fn test_handle_dismiss_session_not_found() {
    let mut state = VisualizationState::new();
    let req = DismissRequest {
        session_id: "nonexistent".to_string(),
    };
    let resp = state.handle_dismiss(req);
    assert!(!resp.dismissed);
}

#[test]
fn test_handle_render_replaces_existing_session() {
    let mut state = VisualizationState::new();
    state.handle_render(VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "First".to_string(),
        bindings: vec![make_timeseries("b1")],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    });
    let resp = state.handle_render(VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "Replaced".to_string(),
        bindings: vec![make_gauge("g1", 25.0)],
        thresholds: vec![],
        domain: Some("physics".to_string()),
        ui_config: None,
    });
    assert_eq!(resp.bindings_accepted, 1);
    let session = state.sessions().get("s1").expect("session exists");
    assert_eq!(session.title, "Replaced");
    assert_eq!(session.bindings.len(), 1);
    assert_eq!(session.domain.as_deref(), Some("physics"));
}

#[test]
fn test_handle_stream_update_replace_operation() {
    let mut state = VisualizationState::new();
    state.handle_render(VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "T".to_string(),
        bindings: vec![make_gauge("g1", 50.0)],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    });
    let new_gauge = make_gauge("g1", 99.0);
    let req = StreamUpdateRequest {
        session_id: "s1".to_string(),
        binding_id: "g1".to_string(),
        operation: StreamOperation::Replace { binding: new_gauge },
    };
    let resp = state.handle_stream_update(req);
    assert!(resp.accepted);
    let bindings = state.session_bindings("s1").expect("session exists");
    if let DataBinding::Gauge { value, .. } = &bindings[0] {
        assert!((*value - 99.0).abs() < f64::EPSILON);
    } else {
        panic!("expected Gauge");
    }
}

#[test]
fn test_handle_export_json_format() {
    let mut state = VisualizationState::new();
    state.handle_grammar_render(GrammarRenderRequest {
        session_id: "ex2".to_string(),
        grammar: GrammarExpr::new("data", GeometryType::Point),
        data: vec![serde_json::json!({"x": 1.0, "y": 2.0})],
        modality: "json".to_string(),
        validate_tufte: false,
        domain: None,
    });
    let req = ExportRequest {
        session_id: "ex2".to_string(),
        format: "json".to_string(),
    };
    let resp = state.handle_export(req);
    assert_eq!(resp.session_id, "ex2");
    assert!(!resp.content.is_empty());
}

#[test]
fn test_handle_dismiss_grammar_scene_only() {
    let mut state = VisualizationState::new();
    state.handle_grammar_render(GrammarRenderRequest {
        session_id: "gram-only".to_string(),
        grammar: GrammarExpr::new("data", GeometryType::Point),
        data: vec![serde_json::json!({"x": 1.0})],
        modality: "svg".to_string(),
        validate_tufte: false,
        domain: None,
    });
    let req = DismissRequest {
        session_id: "gram-only".to_string(),
    };
    let resp = state.handle_dismiss(req);
    assert!(resp.dismissed, "dismiss should remove grammar scene");
    assert!(state.grammar_scene("gram-only").is_none());
}

#[test]
fn test_handle_validate_passed_threshold() {
    let state = VisualizationState::new();
    let grammar = GrammarExpr::new("data", GeometryType::Point);
    let req = ValidateRequest {
        grammar,
        data: vec![
            serde_json::json!({"x": 1.0, "y": 2.0}),
            serde_json::json!({"x": 3.0, "y": 4.0}),
        ],
    };
    let resp = state.handle_validate(&req);
    assert!(resp.score >= 0.0 && resp.score <= 1.0);
    assert_eq!(resp.passed, resp.score >= 0.5);
    for c in &resp.constraints {
        assert!(!c.name.is_empty());
    }
}

#[test]
fn test_render_session_has_updated_at() {
    let mut state = VisualizationState::new();
    state.handle_render(VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "T".to_string(),
        bindings: vec![make_timeseries("b1")],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    });
    let session = state.sessions().get("s1").expect("session exists");
    let _ = session.updated_at;
}

#[test]
fn test_handle_render_with_ui_config() {
    let mut config = std::collections::HashMap::new();
    config.insert("left_sidebar".to_string(), true);
    let mut state = VisualizationState::new();
    let req = VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "With Config".to_string(),
        bindings: vec![],
        thresholds: vec![],
        domain: None,
        ui_config: Some(UiConfig {
            show_panels: config,
            mode: Some("clinical".to_string()),
            initial_zoom: Some("1.0".to_string()),
            awakening_enabled: Some(true),
            theme: Some("clinical-dark".to_string()),
        }),
    };
    let resp = state.handle_render(req);
    assert_eq!(resp.session_id, "s1");
    let session = state.sessions().get("s1").expect("session exists");
    assert_eq!(
        session.ui_config.as_ref().expect("config").mode.as_deref(),
        Some("clinical")
    );
}

#[test]
fn test_handle_session_status_nonexistent() {
    let state = VisualizationState::new();
    let req = SessionStatusRequest {
        session_id: "nonexistent".to_string(),
    };
    let resp = state.handle_session_status(&req);
    assert!(!resp.exists);
    assert_eq!(resp.session_id, "nonexistent");
    assert_eq!(resp.frame_count, 0);
    assert_eq!(resp.binding_count, 0);
    assert!(!resp.backpressure_active);
}

#[test]
fn test_handle_session_status_exists() {
    let mut state = VisualizationState::new();
    state.handle_render(VisualizationRenderRequest {
        session_id: "s1".to_string(),
        title: "T".to_string(),
        bindings: vec![make_timeseries("b1")],
        thresholds: vec![],
        domain: Some("health".to_string()),
        ui_config: None,
    });
    let req = SessionStatusRequest {
        session_id: "s1".to_string(),
    };
    let resp = state.handle_session_status(&req);
    assert!(resp.exists);
    assert_eq!(resp.session_id, "s1");
    assert_eq!(resp.binding_count, 1);
    assert_eq!(resp.domain.as_deref(), Some("health"));
}

#[test]
fn test_with_backpressure_config() {
    let config = BackpressureConfig {
        max_updates_per_sec: 10,
        cooldown: std::time::Duration::from_millis(100),
        burst_tolerance: 2,
    };
    let mut state = VisualizationState::new().with_backpressure(config);
    state.handle_render(VisualizationRenderRequest {
        session_id: "bp".to_string(),
        title: "BP".to_string(),
        bindings: vec![make_timeseries("b1")],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    });
    for _ in 0..15 {
        let _ = state.handle_stream_update(StreamUpdateRequest {
            session_id: "bp".to_string(),
            binding_id: "b1".to_string(),
            operation: StreamOperation::Append {
                x_values: vec![1.0],
                y_values: vec![1.0],
            },
        });
    }
    let req = SessionStatusRequest {
        session_id: "bp".to_string(),
    };
    let resp = state.handle_session_status(&req);
    assert!(resp.exists);
}

#[test]
fn test_handle_dismiss_removes_binding_scenes() {
    let mut state = VisualizationState::new();
    state.handle_render(VisualizationRenderRequest {
        session_id: "ds".to_string(),
        title: "T".to_string(),
        bindings: vec![make_timeseries("b1"), make_gauge("b2", 50.0)],
        thresholds: vec![],
        domain: None,
        ui_config: None,
    });
    assert!(state.grammar_scene("ds:b1").is_some());
    assert!(state.grammar_scene("ds:b2").is_some());
    let req = DismissRequest {
        session_id: "ds".to_string(),
    };
    let resp = state.handle_dismiss(req);
    assert!(resp.dismissed);
    assert!(state.grammar_scene("ds:b1").is_none());
    assert!(state.grammar_scene("ds:b2").is_none());
    assert!(state.sessions().get("ds").is_none());
}
