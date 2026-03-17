// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::interaction::adapter::{InputModality, InteractionCapability, InteractionContext};
use crate::interaction::intent::{InteractionIntent, SelectionMode};
use crate::interaction::perspective::Perspective;
use crate::interaction::result::InteractionResult;
use crate::interaction::target::{DataObjectId, InteractionTarget};
use crate::sensor::{MouseButton, SensorEvent};
use std::time::Instant;

struct MockAdapter {
    translate_returns: Option<InteractionIntent>,
}

impl crate::interaction::InputAdapter for MockAdapter {
    fn name(&self) -> &'static str {
        "mock"
    }
    fn modality(&self) -> InputModality {
        InputModality::PointerMouse
    }
    fn capabilities(&self) -> &[InteractionCapability] {
        &[InteractionCapability::PointSelect]
    }
    fn translate(
        &self,
        _event: &SensorEvent,
        _context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        self.translate_returns.clone()
    }
    fn active_target(&self, _context: &InteractionContext) -> Option<InteractionTarget> {
        None
    }
    fn feedback(&mut self, _result: &InteractionResult) {}
}

/// Adapter that returns intents from a queue (for testing sequence of intents).
struct QueuedAdapter {
    intents: std::sync::Mutex<Vec<InteractionIntent>>,
}

impl crate::interaction::InputAdapter for QueuedAdapter {
    fn name(&self) -> &'static str {
        "queued"
    }
    fn modality(&self) -> InputModality {
        InputModality::PointerMouse
    }
    fn capabilities(&self) -> &[InteractionCapability] {
        &[InteractionCapability::PointSelect]
    }
    fn translate(
        &self,
        _event: &SensorEvent,
        _context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        let mut intents = self.intents.lock().expect("lock");
        if intents.is_empty() {
            None
        } else {
            Some(intents.remove(0))
        }
    }
    fn active_target(&self, _context: &InteractionContext) -> Option<InteractionTarget> {
        None
    }
    fn feedback(&mut self, _result: &InteractionResult) {}
}

#[test]
fn engine_lifecycle() {
    let mut engine = InteractionEngine::new();
    assert_eq!(engine.adapter_count(), 0);
    assert_eq!(engine.inverse_pipeline_count(), 0);

    let id = engine.add_perspective(Perspective::new(0));
    assert!(engine.perspective(id).is_some());
    assert_eq!(engine.perspectives().len(), 1);
}

#[test]
fn process_empty_events() {
    let mut engine = InteractionEngine::new();
    let ctx = InteractionContext::default_for_perspective(1);
    let results = engine.process_events(&[], &ctx);
    assert!(results.is_empty());
}

#[test]
fn selection_propagation() {
    use crate::interaction::perspective::PerspectiveSync;

    let mut engine = InteractionEngine::new();

    let mut p1 = Perspective::new(0);
    p1.sync_mode = PerspectiveSync::SharedSelection;
    let id1 = engine.add_perspective(p1);

    let mut p2 = Perspective::new(0);
    p2.sync_mode = PerspectiveSync::SharedSelection;
    let id2 = engine.add_perspective(p2);

    // Manually select in p1
    let obj = DataObjectId::new("test", serde_json::json!("row1"));
    engine.perspective_mut(id1).unwrap().select(obj.clone());

    // Propagate
    let mut changes = Vec::new();
    engine.propagate_selection(id1, &mut changes);

    // p2 should now have the same selection
    assert!(engine.perspective(id2).unwrap().is_selected(&obj));
}

#[test]
fn navigation_zoom() {
    use super::super::intent::NavigationDirection;
    use super::super::perspective::PerspectiveViewport;

    let mut vp = PerspectiveViewport::default();
    let initial_zoom = vp.zoom;

    super::apply_navigation(&mut vp, &NavigationDirection::In, 1.0);
    assert!(vp.zoom > initial_zoom);

    super::apply_navigation(&mut vp, &NavigationDirection::Out, 1.0);
    // Should be approximately back to initial (not exact due to asymmetric scaling)
    assert!(vp.zoom < initial_zoom * 1.2);
}

#[test]
fn ipc_event_injection() {
    let mut engine = InteractionEngine::new();
    let mut p = Perspective::new(0);
    p.sync_mode = super::super::perspective::PerspectiveSync::SharedSelection;
    let id = engine.add_perspective(p);

    let event = InteractionEvent {
        event_type: "select".into(),
        targets: vec![DataObjectId::new("remote", serde_json::json!("obj1"))],
        perspective_id: 99,
        grammar_id: "test".into(),
        timestamp: "2026-03-09T00:00:00Z".into(),
    };

    engine.inject_ipc_event(event);

    // Process should drain the pending result
    let ctx = InteractionContext::default_for_perspective(id);
    let results = engine.process_events(&[], &ctx);
    assert_eq!(results.len(), 1);

    // The local perspective should have the remote selection
    assert!(
        engine
            .perspective(id)
            .unwrap()
            .is_selected(&DataObjectId::new("remote", serde_json::json!("obj1")))
    );
}

#[test]
fn register_adapter_and_process() {
    let mut engine = InteractionEngine::new();
    let obj = DataObjectId::new("table", serde_json::json!("row1"));
    engine.register_adapter(Box::new(MockAdapter {
        translate_returns: Some(InteractionIntent::Select {
            target: InteractionTarget::DataRow {
                data_id: obj.clone(),
            },
            mode: SelectionMode::Replace,
        }),
    }));
    assert_eq!(engine.adapter_count(), 1);

    let id = engine.add_perspective(Perspective::new(0));
    let ctx = InteractionContext::default_for_perspective(id);
    let click = SensorEvent::Click {
        x: 100.0,
        y: 200.0,
        button: MouseButton::Left,
        timestamp: Instant::now(),
    };
    let results = engine.process_events(&[click], &ctx);
    assert_eq!(results.len(), 1);
    assert!(engine.perspective(id).unwrap().is_selected(&obj));
}

#[test]
fn apply_intent_selection_modes() {
    let mut engine = InteractionEngine::new();
    let id = engine.add_perspective(Perspective::new(0));
    let obj1 = DataObjectId::new("t", serde_json::json!("r1"));
    let obj2 = DataObjectId::new("t", serde_json::json!("r2"));

    engine.register_adapter(Box::new(QueuedAdapter {
        intents: std::sync::Mutex::new(vec![
            InteractionIntent::Select {
                target: InteractionTarget::DataRow {
                    data_id: obj1.clone(),
                },
                mode: SelectionMode::Replace,
            },
            InteractionIntent::Select {
                target: InteractionTarget::DataRow {
                    data_id: obj2.clone(),
                },
                mode: SelectionMode::Add,
            },
        ]),
    }));
    let ctx = InteractionContext::default_for_perspective(id);
    let click = SensorEvent::Click {
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
        timestamp: Instant::now(),
    };
    engine.process_events(std::slice::from_ref(&click), &ctx);
    assert!(engine.perspective(id).unwrap().is_selected(&obj1));

    engine.process_events(std::slice::from_ref(&click), &ctx);
    let p = engine.perspective(id).unwrap();
    assert!(p.is_selected(&obj1));
    assert!(p.is_selected(&obj2));
}

#[test]
fn apply_intent_focus() {
    let mut engine = InteractionEngine::new();
    let id = engine.add_perspective(Perspective::new(0));
    let obj = DataObjectId::new("t", serde_json::json!("r1"));
    engine.register_adapter(Box::new(MockAdapter {
        translate_returns: Some(InteractionIntent::Focus {
            target: InteractionTarget::DataRow {
                data_id: obj.clone(),
            },
        }),
    }));
    let ctx = InteractionContext::default_for_perspective(id);
    engine.process_events(
        &[SensorEvent::Position {
            x: 50.0,
            y: 50.0,
            timestamp: Instant::now(),
        }],
        &ctx,
    );
    assert_eq!(engine.perspective(id).unwrap().focus, Some(obj));
}

#[test]
fn apply_intent_dismiss() {
    let mut engine = InteractionEngine::new();
    let id = engine.add_perspective(Perspective::new(0));
    let obj = DataObjectId::new("t", serde_json::json!("r1"));
    engine.perspective_mut(id).unwrap().select(obj);
    engine.register_adapter(Box::new(MockAdapter {
        translate_returns: Some(InteractionIntent::Dismiss),
    }));
    let ctx = InteractionContext::default_for_perspective(id);
    engine.process_events(
        &[SensorEvent::KeyPress {
            key: crate::sensor::Key::Char('E'),
            modifiers: crate::sensor::Modifiers::none(),
            timestamp: Instant::now(),
        }],
        &ctx,
    );
    let p = engine.perspective(id).unwrap();
    assert!(p.selection.is_empty());
    assert!(p.focus.is_none());
}

#[test]
fn navigation_directions() {
    use super::super::intent::NavigationDirection;
    use super::super::perspective::PerspectiveViewport;

    let mut vp = PerspectiveViewport {
        center_x: 100.0,
        center_y: 50.0,
        zoom: 1.0,
        ..Default::default()
    };

    super::apply_navigation(&mut vp, &NavigationDirection::Left, 1.0);
    assert!(vp.center_x < 100.0);

    super::apply_navigation(&mut vp, &NavigationDirection::Right, 1.0);
    super::apply_navigation(&mut vp, &NavigationDirection::Up, 1.0);
    assert!(vp.center_y > 50.0);

    super::apply_navigation(&mut vp, &NavigationDirection::Down, 1.0);
    super::apply_navigation(&mut vp, &NavigationDirection::In, 0.5);
    assert!(vp.zoom >= 0.1 && vp.zoom <= 100.0);

    super::apply_navigation(&mut vp, &NavigationDirection::Out, 0.5);
    assert!(vp.zoom >= 0.1 && vp.zoom <= 100.0);
}

#[test]
fn ipc_event_focus() {
    let mut engine = InteractionEngine::new();
    let mut p = Perspective::new(0);
    p.sync_mode = super::super::perspective::PerspectiveSync::FullSync;
    let id = engine.add_perspective(p);

    let event = InteractionEvent {
        event_type: "focus".into(),
        targets: vec![DataObjectId::new("remote", serde_json::json!("f1"))],
        perspective_id: id,
        grammar_id: "test".into(),
        timestamp: "2026-03-09T00:00:00Z".into(),
    };
    engine.inject_ipc_event(event);
    assert_eq!(
        engine.perspective(id).unwrap().focus,
        Some(DataObjectId::new("remote", serde_json::json!("f1")))
    );
}

#[test]
fn default_impl() {
    let engine = InteractionEngine::default();
    assert_eq!(engine.adapter_count(), 0);
}

#[test]
fn apply_intent_selection_remove() {
    let mut engine = InteractionEngine::new();
    let id = engine.add_perspective(Perspective::new(0));
    let obj1 = DataObjectId::new("t", serde_json::json!("r1"));
    let obj2 = DataObjectId::new("t", serde_json::json!("r2"));

    engine.perspective_mut(id).unwrap().select(obj1.clone());
    engine
        .perspective_mut(id)
        .unwrap()
        .add_to_selection(obj2.clone());

    engine.register_adapter(Box::new(MockAdapter {
        translate_returns: Some(InteractionIntent::Select {
            target: InteractionTarget::DataRow {
                data_id: obj1.clone(),
            },
            mode: SelectionMode::Remove,
        }),
    }));
    let ctx = InteractionContext::default_for_perspective(id);
    engine.process_events(
        &[SensorEvent::Click {
            x: 0.0,
            y: 0.0,
            button: MouseButton::Left,
            timestamp: Instant::now(),
        }],
        &ctx,
    );

    let p = engine.perspective(id).unwrap();
    assert!(!p.is_selected(&obj1));
    assert!(p.is_selected(&obj2));
}

#[test]
fn apply_intent_selection_toggle() {
    let mut engine = InteractionEngine::new();
    let id = engine.add_perspective(Perspective::new(0));
    let obj = DataObjectId::new("t", serde_json::json!("r1"));

    engine.register_adapter(Box::new(QueuedAdapter {
        intents: std::sync::Mutex::new(vec![
            InteractionIntent::Select {
                target: InteractionTarget::DataRow {
                    data_id: obj.clone(),
                },
                mode: SelectionMode::Toggle,
            },
            InteractionIntent::Select {
                target: InteractionTarget::DataRow {
                    data_id: obj.clone(),
                },
                mode: SelectionMode::Toggle,
            },
        ]),
    }));
    let ctx = InteractionContext::default_for_perspective(id);
    let click = SensorEvent::Click {
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
        timestamp: Instant::now(),
    };
    engine.process_events(std::slice::from_ref(&click), &ctx);
    assert!(engine.perspective(id).unwrap().is_selected(&obj));

    engine.process_events(std::slice::from_ref(&click), &ctx);
    assert!(!engine.perspective(id).unwrap().is_selected(&obj));
}

#[test]
fn apply_intent_navigate() {
    use super::super::intent::NavigationDirection;
    use super::super::perspective::PerspectiveViewport;

    let mut vp = PerspectiveViewport {
        center_x: 0.0,
        center_y: 0.0,
        zoom: 1.0,
        ..Default::default()
    };
    super::apply_navigation(&mut vp, &NavigationDirection::Forward, 1.0);
    super::apply_navigation(&mut vp, &NavigationDirection::Backward, 1.0);
    assert!((vp.center_x - 0.0).abs() < f64::EPSILON);
    assert!((vp.center_y - 0.0).abs() < f64::EPSILON);
}
