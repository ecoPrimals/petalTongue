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
use crate::modality::{GUIModality, ModalityRegistry, NullModality};

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
pub struct UniversalRenderingEngine<M: GUIModality = NullModality> {
    /// Engine state
    state: Arc<RwLock<EngineState>>,

    /// Event bus
    events: Arc<EventBus>,

    /// Registered modalities
    modalities: Arc<RwLock<ModalityRegistry<M>>>,

    /// Compute providers
    compute: Arc<RwLock<ComputeRegistry>>,
}

impl<M: GUIModality + 'static> UniversalRenderingEngine<M> {
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
        Arc::clone(&self.state)
    }

    /// Get event bus
    #[must_use]
    pub fn events(&self) -> Arc<EventBus> {
        Arc::clone(&self.events)
    }

    /// Get modalities (read-only)
    #[must_use]
    pub fn modalities(&self) -> Arc<RwLock<ModalityRegistry<M>>> {
        Arc::clone(&self.modalities)
    }

    /// Get compute providers (read-only)
    #[must_use]
    pub fn compute(&self) -> Arc<RwLock<ComputeRegistry>> {
        Arc::clone(&self.compute)
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
        };

        self.render(modality_name).await
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

        let name_for_events = modality_name.to_string();

        // Initialize
        modality.initialize(Arc::clone(&self)).await?;

        // Broadcast start event
        self.events
            .broadcast(EngineEvent::ModalityStarted {
                name: name_for_events.clone(),
            })
            .await
            .map_err(PetalTongueError::EventBus)?;

        // Start rendering
        let result = modality.render().await;

        // Broadcast stop event
        self.events
            .broadcast(EngineEvent::ModalityStopped {
                name: name_for_events,
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
            let engine = Arc::clone(&self);
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
#[path = "engine_tests.rs"]
mod tests;
