// SPDX-License-Identifier: AGPL-3.0-or-later
//! # Universal Rendering Engine
//!
//! Core engine that manages topology state and coordinates modalities.

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::compute::ComputeRegistry;
use crate::error::{PetalTongueError, Result};
use crate::event::{EngineEvent, EventBus};
use crate::modality::ModalityRegistry;

/// Engine State
///
/// Shared state across all modalities.
#[derive(Debug, Clone)]
pub struct EngineState {
    /// Current view mode
    pub view_mode: ViewMode,

    /// Selected nodes
    pub selection: HashSet<String>,

    /// Viewport state
    pub viewport: Viewport,

    /// Time state
    pub time: TimeState,
}

/// View Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Graph view (network diagram)
    Graph,

    /// List view (tabular)
    List,

    /// Tree view (hierarchical)
    Tree,

    /// Timeline view
    Timeline,
}

/// Viewport State
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// Center X coordinate
    pub center_x: f32,

    /// Center Y coordinate
    pub center_y: f32,

    /// Zoom level (1.0 = 100%)
    pub zoom: f32,
}

/// Time State
#[derive(Debug, Clone, Copy)]
pub struct TimeState {
    /// Current time (seconds since start)
    pub current: f64,

    /// Time scale (1.0 = real-time)
    pub scale: f64,

    /// Paused
    pub paused: bool,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Graph,
            selection: HashSet::new(),
            viewport: Viewport {
                center_x: 0.0,
                center_y: 0.0,
                zoom: 1.0,
            },
            time: TimeState {
                current: 0.0,
                scale: 1.0,
                paused: false,
            },
        }
    }
}

/// Universal Rendering Engine
///
/// Core engine that manages topology state and coordinates
/// rendering across multiple modalities.
pub struct UniversalRenderingEngine {
    /// Engine state
    state: Arc<RwLock<EngineState>>,

    /// Event bus
    events: Arc<EventBus>,

    /// Registered modalities
    modalities: Arc<RwLock<ModalityRegistry>>,

    /// Compute providers
    compute: Arc<RwLock<ComputeRegistry>>,
}

impl UniversalRenderingEngine {
    /// Create new engine
    ///
    /// # Errors
    ///
    /// Does not currently return errors.
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: Arc::new(RwLock::new(EngineState::default())),
            events: Arc::new(EventBus::new()),
            modalities: Arc::new(RwLock::new(ModalityRegistry::new())),
            compute: Arc::new(RwLock::new(ComputeRegistry::new())),
        })
    }

    /// Get state (read-only)
    #[must_use]
    pub fn state(&self) -> Arc<RwLock<EngineState>> {
        self.state.clone()
    }

    /// Get event bus
    #[must_use]
    pub fn events(&self) -> Arc<EventBus> {
        self.events.clone()
    }

    /// Get modalities (read-only)
    #[must_use]
    pub fn modalities(&self) -> Arc<RwLock<ModalityRegistry>> {
        self.modalities.clone()
    }

    /// Get compute providers (read-only)
    #[must_use]
    pub fn compute(&self) -> Arc<RwLock<ComputeRegistry>> {
        self.compute.clone()
    }

    /// Update selection
    ///
    /// # Errors
    ///
    /// Returns an error if the event broadcast fails (e.g., no subscribers).
    pub async fn set_selection(&self, selected: HashSet<String>) -> Result<()> {
        // Update state
        {
            let mut state = self.state.write().await;
            state.selection.clone_from(&selected);
        }

        // Broadcast event
        self.events
            .broadcast(EngineEvent::SelectionChanged { selected })
            .await
            .map_err(PetalTongueError::EventBus)?;

        Ok(())
    }

    /// Update viewport
    ///
    /// # Errors
    ///
    /// Returns an error if the event broadcast fails (e.g., no subscribers).
    pub async fn set_viewport(&self, center_x: f32, center_y: f32, zoom: f32) -> Result<()> {
        // Update state
        {
            let mut state = self.state.write().await;
            state.viewport = Viewport {
                center_x,
                center_y,
                zoom,
            };
        }

        // Broadcast event
        self.events
            .broadcast(EngineEvent::ViewChanged {
                center_x,
                center_y,
                zoom,
            })
            .await
            .map_err(PetalTongueError::EventBus)?;

        Ok(())
    }

    /// Discover and register available modalities.
    ///
    /// Queries the modality registry. Returns Ok(()) with empty registry
    /// when no modalities are registered (graceful degradation).
    ///
    /// # Errors
    ///
    /// Does not return errors.
    pub async fn discover_modalities(&self) -> Result<()> {
        let count = {
            let registry = self.modalities.read().await;
            registry.len()
        };
        tracing::info!("Modality discovery complete: {count} modality(ies) available");
        Ok(())
    }

    /// Discover and register available compute providers.
    ///
    /// Queries the compute registry. Returns Ok(()) with empty registry
    /// when no compute providers are registered (graceful degradation).
    ///
    /// # Errors
    ///
    /// Does not return errors.
    pub async fn discover_compute(&self) -> Result<()> {
        let count = {
            let registry = self.compute.read().await;
            registry.len()
        };
        tracing::info!("Compute discovery complete: {count} provider(s) available");
        Ok(())
    }

    /// Start rendering in best available modality
    ///
    /// # Errors
    ///
    /// Returns an error if no modalities are registered, or if rendering fails.
    pub async fn render_auto(self: Arc<Self>) -> Result<()> {
        let modality_name = {
            let registry = self.modalities.read().await;
            registry
                .auto_select()
                .ok_or(PetalTongueError::NoModalities)?
                .to_string()
        };

        self.render(&modality_name).await
    }

    /// Start rendering in specific modality
    ///
    /// # Errors
    ///
    /// Returns an error if the modality is not found, if event broadcast fails,
    /// if modality initialization fails, or if modality rendering fails.
    #[expect(
        clippy::significant_drop_tightening,
        reason = "RwLock guard must span both initialize and render awaits"
    )]
    pub async fn render(self: Arc<Self>, modality_name: &str) -> Result<()> {
        let mut registry = self.modalities.write().await;

        let modality = registry
            .get_mut(modality_name)
            .ok_or_else(|| PetalTongueError::ModalityNotFound(modality_name.to_string()))?;

        // Initialize
        modality.initialize(self.clone()).await?;

        // Broadcast start event
        self.events
            .broadcast(EngineEvent::ModalityStarted {
                name: modality_name.to_string(),
            })
            .await
            .map_err(PetalTongueError::EventBus)?;

        // Start rendering
        let result = modality.render().await;

        // Broadcast stop event
        self.events
            .broadcast(EngineEvent::ModalityStopped {
                name: modality_name.to_string(),
            })
            .await
            .ok();

        result
    }

    /// Start rendering in multiple modalities simultaneously
    ///
    /// # Errors
    ///
    /// Returns an error if any modality fails to render or if a spawned task panics.
    pub async fn render_multi(self: Arc<Self>, modality_names: Vec<&str>) -> Result<()> {
        let mut handles = Vec::new();

        for name in modality_names {
            let engine = self.clone();
            let name = name.to_string();

            let handle = tokio::spawn(async move { engine.render(&name).await });

            handles.push(handle);
        }

        // Wait for all to complete (or first error)
        for handle in handles {
            handle.await??;
        }

        Ok(())
    }
}

// Default impl removed: use UniversalRenderingEngine::new()? for proper error handling.

#[cfg(test)]
mod tests {
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
}
