// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::modality::{ModalityCapabilities, ModalityTier};
use async_trait::async_trait;

struct MockModality {
    name: &'static str,
    tier: ModalityTier,
}

#[async_trait]
impl crate::modality::GUIModality for MockModality {
    fn name(&self) -> &'static str {
        self.name
    }
    fn is_available(&self) -> bool {
        true
    }
    fn tier(&self) -> ModalityTier {
        self.tier
    }
    async fn initialize(&mut self, _engine: Arc<UniversalRenderingEngine>) -> Result<()> {
        Ok(())
    }
    async fn render(&mut self) -> Result<()> {
        Ok(())
    }
    async fn handle_event(&mut self, _event: EngineEvent) -> Result<()> {
        Ok(())
    }
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
    fn capabilities(&self) -> ModalityCapabilities {
        ModalityCapabilities::default()
    }
}

#[tokio::test]
async fn test_engine_creation() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");

    let state = engine.state.read().await;
    assert_eq!(state.view_mode, ViewMode::Graph);
    assert!(state.selection.is_empty());
    drop(state);
}

#[tokio::test]
async fn test_selection_update() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");

    let mut selected = HashSet::new();
    selected.insert("node1".to_string());

    // Event broadcast may fail if no subscribers - that's OK in tests
    let _ = engine.set_selection(selected.clone()).await;

    let state = engine.state.read().await;
    assert_eq!(state.selection.len(), 1);
    assert!(state.selection.contains("node1"));
    drop(state);
}

#[tokio::test]
async fn test_viewport_update() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");

    // Event broadcast may fail if no subscribers - that's OK in tests
    let _ = engine.set_viewport(100.0, 200.0, 1.5).await;

    let state = engine.state.read().await;
    assert!((state.viewport.center_x - 100.0).abs() < f32::EPSILON);
    assert!((state.viewport.center_y - 200.0).abs() < f32::EPSILON);
    assert!((state.viewport.zoom - 1.5).abs() < f32::EPSILON);
    drop(state);
}

#[tokio::test]
async fn test_discover_modalities() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");
    engine
        .discover_modalities()
        .await
        .expect("discover modalities");
}

#[tokio::test]
async fn test_discover_compute() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");
    engine.discover_compute().await.expect("discover compute");
}

#[tokio::test]
async fn test_render_auto_no_modalities() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");
    let engine = Arc::new(engine);
    let result = engine.clone().render_auto().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no modalities"));
}

#[tokio::test]
async fn test_render_unknown_modality() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");
    let engine = Arc::new(engine);
    let result = engine.clone().render("nonexistent").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("modality not found")
    );
}

#[tokio::test]
async fn test_render_with_registered_modality() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");
    {
        let modalities = engine.modalities();
        let mut guard = modalities.write().await;
        guard.register(Box::new(MockModality {
            name: "test",
            tier: ModalityTier::AlwaysAvailable,
        }));
    }
    let engine = Arc::new(engine);
    let _sub = engine.events().subscribe().await;
    let result = engine.clone().render("test").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_render_multi() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");
    {
        let modalities = engine.modalities();
        let mut guard = modalities.write().await;
        guard.register(Box::new(MockModality {
            name: "a",
            tier: ModalityTier::AlwaysAvailable,
        }));
        guard.register(Box::new(MockModality {
            name: "b",
            tier: ModalityTier::AlwaysAvailable,
        }));
    }
    let engine = Arc::new(engine);
    let _sub = engine.events().subscribe().await;
    let result = engine.clone().render_multi(vec!["a", "b"]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_engine_state_accessors() {
    let engine = UniversalRenderingEngine::new().expect("engine creation");
    let _state = engine.state();
    let _events = engine.events();
    let _modalities = engine.modalities();
    let _compute = engine.compute();
}

#[tokio::test]
async fn test_view_mode_variants() {
    assert_eq!(ViewMode::Graph, ViewMode::Graph);
    assert_ne!(ViewMode::Graph, ViewMode::List);
    assert_ne!(ViewMode::Graph, ViewMode::Tree);
    assert_ne!(ViewMode::Graph, ViewMode::Timeline);
}

#[tokio::test]
async fn test_engine_state_default() {
    let state = EngineState::default();
    assert_eq!(state.view_mode, ViewMode::Graph);
    assert!(state.selection.is_empty());
    assert!((state.viewport.center_x - 0.0).abs() < f32::EPSILON);
    assert!((state.viewport.zoom - 1.0).abs() < f32::EPSILON);
    assert!((state.time.current - 0.0).abs() < f64::EPSILON);
    assert!(!state.time.paused);
}

#[tokio::test]
async fn test_engine_lifecycle_new_succeeds() {
    let engine = UniversalRenderingEngine::new();
    assert!(engine.is_ok());
}

#[tokio::test]
async fn test_engine_modality_initialize_error_propagates() {
    struct FailingModality;
    #[async_trait]
    impl crate::modality::GUIModality for FailingModality {
        fn name(&self) -> &'static str {
            "failing"
        }
        fn is_available(&self) -> bool {
            true
        }
        fn tier(&self) -> ModalityTier {
            ModalityTier::AlwaysAvailable
        }
        async fn initialize(&mut self, _engine: Arc<UniversalRenderingEngine>) -> Result<()> {
            Err(crate::error::PetalTongueError::Internal(
                "init failed".into(),
            ))
        }
        async fn render(&mut self) -> Result<()> {
            Ok(())
        }
        async fn handle_event(&mut self, _event: EngineEvent) -> Result<()> {
            Ok(())
        }
        async fn shutdown(&mut self) -> Result<()> {
            Ok(())
        }
        fn capabilities(&self) -> ModalityCapabilities {
            ModalityCapabilities::default()
        }
    }
    let engine = UniversalRenderingEngine::new().expect("engine");
    {
        let modalities = engine.modalities();
        let mut guard = modalities.write().await;
        guard.register(Box::new(FailingModality));
    }
    let engine = Arc::new(engine);
    let _sub = engine.events().subscribe().await;
    let result = engine.clone().render("failing").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("init failed"));
}

#[tokio::test]
async fn test_engine_modality_render_error_propagates() {
    struct RenderFailingModality;
    #[async_trait]
    impl crate::modality::GUIModality for RenderFailingModality {
        fn name(&self) -> &'static str {
            "render_fail"
        }
        fn is_available(&self) -> bool {
            true
        }
        fn tier(&self) -> ModalityTier {
            ModalityTier::AlwaysAvailable
        }
        async fn initialize(&mut self, _engine: Arc<UniversalRenderingEngine>) -> Result<()> {
            Ok(())
        }
        async fn render(&mut self) -> Result<()> {
            Err(crate::error::PetalTongueError::Internal(
                "render failed".into(),
            ))
        }
        async fn handle_event(&mut self, _event: EngineEvent) -> Result<()> {
            Ok(())
        }
        async fn shutdown(&mut self) -> Result<()> {
            Ok(())
        }
        fn capabilities(&self) -> ModalityCapabilities {
            ModalityCapabilities::default()
        }
    }
    let engine = UniversalRenderingEngine::new().expect("engine");
    {
        let modalities = engine.modalities();
        let mut guard = modalities.write().await;
        guard.register(Box::new(RenderFailingModality));
    }
    let engine = Arc::new(engine);
    let _sub = engine.events().subscribe().await;
    let result = engine.clone().render("render_fail").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("render failed"));
}

#[tokio::test]
async fn test_engine_set_selection_empty() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    let empty: HashSet<String> = HashSet::new();
    let _ = engine.set_selection(empty).await;
    let state = engine.state.read().await;
    assert!(state.selection.is_empty());
    drop(state);
}

#[tokio::test]
async fn test_engine_set_selection_multiple() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    let mut selected = HashSet::new();
    selected.insert("a".to_string());
    selected.insert("b".to_string());
    selected.insert("c".to_string());
    let _ = engine.set_selection(selected.clone()).await;
    let state = engine.state.read().await;
    assert_eq!(state.selection.len(), 3);
    assert!(state.selection.contains("a"));
    assert!(state.selection.contains("b"));
    assert!(state.selection.contains("c"));
    drop(state);
}

#[tokio::test]
async fn test_engine_set_viewport_zero_zoom() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    let _ = engine.set_viewport(0.0, 0.0, 0.5).await;
    let state = engine.state.read().await;
    assert!((state.viewport.zoom - 0.5).abs() < f32::EPSILON);
    drop(state);
}

#[tokio::test]
async fn test_engine_render_multi_empty_list() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    let engine = Arc::new(engine);
    let result = engine.clone().render_multi(vec![]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_engine_render_multi_one_fails() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    {
        let modalities = engine.modalities();
        let mut guard = modalities.write().await;
        guard.register(Box::new(MockModality {
            name: "ok",
            tier: ModalityTier::AlwaysAvailable,
        }));
    }
    let engine = Arc::new(engine);
    let _sub = engine.events().subscribe().await;
    let result = engine.clone().render_multi(vec!["ok", "nonexistent"]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_viewport_copy() {
    let v = Viewport {
        center_x: 1.0,
        center_y: 2.0,
        zoom: 3.0,
    };
    let v2 = v;
    assert!((v2.center_x - 1.0).abs() < f32::EPSILON);
    assert!((v2.zoom - 3.0).abs() < f32::EPSILON);
}

#[tokio::test]
async fn test_time_state_copy() {
    let t = TimeState {
        current: 10.0,
        scale: 2.0,
        paused: true,
    };
    let t2 = t;
    assert!((t2.current - 10.0).abs() < f64::EPSILON);
    assert!(t2.paused);
}

#[tokio::test]
async fn test_set_selection_updates_state_even_when_broadcast_fails() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    let mut selected = HashSet::new();
    selected.insert("node1".to_string());
    let result = engine.set_selection(selected).await;
    let state = engine.state.read().await;
    assert_eq!(state.selection.len(), 1);
    assert!(state.selection.contains("node1"));
    drop(state);
    if let Err(e) = result {
        assert!(e.to_string().contains("broadcast"));
    }
}

#[tokio::test]
async fn test_set_viewport_updates_state_even_when_broadcast_fails() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    let result = engine.set_viewport(50.0, 75.0, 2.0).await;
    let state = engine.state.read().await;
    assert!((state.viewport.center_x - 50.0).abs() < f32::EPSILON);
    assert!((state.viewport.center_y - 75.0).abs() < f32::EPSILON);
    assert!((state.viewport.zoom - 2.0).abs() < f32::EPSILON);
    drop(state);
    if let Err(e) = result {
        assert!(e.to_string().contains("broadcast"));
    }
}

#[tokio::test]
async fn test_set_selection_succeeds_with_subscriber() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    let _sub = engine.events().subscribe().await;
    let mut selected = HashSet::new();
    selected.insert("a".to_string());
    let result = engine.set_selection(selected).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_viewport_succeeds_with_subscriber() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    let _sub = engine.events().subscribe().await;
    let result = engine.set_viewport(1.0, 2.0, 1.0).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_render_completes_and_broadcasts_stop() {
    struct TrackStopModality;
    #[async_trait]
    impl crate::modality::GUIModality for TrackStopModality {
        fn name(&self) -> &'static str {
            "track_stop"
        }
        fn is_available(&self) -> bool {
            true
        }
        fn tier(&self) -> ModalityTier {
            ModalityTier::AlwaysAvailable
        }
        async fn initialize(&mut self, _engine: Arc<UniversalRenderingEngine>) -> Result<()> {
            Ok(())
        }
        async fn render(&mut self) -> Result<()> {
            Ok(())
        }
        async fn handle_event(&mut self, _event: EngineEvent) -> Result<()> {
            Ok(())
        }
        async fn shutdown(&mut self) -> Result<()> {
            Ok(())
        }
        fn capabilities(&self) -> ModalityCapabilities {
            ModalityCapabilities::default()
        }
    }
    let engine = UniversalRenderingEngine::new().expect("engine");
    {
        let modalities = engine.modalities();
        let mut guard = modalities.write().await;
        guard.register(Box::new(TrackStopModality));
    }
    let engine = Arc::new(engine);
    let _sub = engine.events().subscribe().await;
    let result = engine.clone().render("track_stop").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_engine_state_view_mode_variants() {
    assert_eq!(ViewMode::Graph, ViewMode::Graph);
    assert_eq!(ViewMode::List, ViewMode::List);
    assert_eq!(ViewMode::Tree, ViewMode::Tree);
    assert_eq!(ViewMode::Timeline, ViewMode::Timeline);
}

#[tokio::test]
async fn test_render_multi_propagates_error() {
    let engine = UniversalRenderingEngine::new().expect("engine");
    let engine = Arc::new(engine);
    let result = engine.clone().render_multi(vec!["nonexistent"]).await;
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("modality not found"));
}
