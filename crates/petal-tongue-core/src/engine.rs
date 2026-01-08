//! # Universal Rendering Engine
//! 
//! Core engine that manages topology state and coordinates modalities.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashSet;

use crate::compute::ComputeRegistry;
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
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: Arc::new(RwLock::new(EngineState::default())),
            events: Arc::new(EventBus::new()),
            modalities: Arc::new(RwLock::new(ModalityRegistry::new())),
            compute: Arc::new(RwLock::new(ComputeRegistry::new())),
        })
    }
    
    /// Get state (read-only)
    pub fn state(&self) -> Arc<RwLock<EngineState>> {
        self.state.clone()
    }
    
    /// Get event bus
    pub fn events(&self) -> Arc<EventBus> {
        self.events.clone()
    }
    
    /// Get modalities (read-only)
    pub fn modalities(&self) -> Arc<RwLock<ModalityRegistry>> {
        self.modalities.clone()
    }
    
    /// Get compute providers (read-only)
    pub fn compute(&self) -> Arc<RwLock<ComputeRegistry>> {
        self.compute.clone()
    }
    
    /// Update selection
    pub async fn set_selection(&self, selected: HashSet<String>) -> Result<()> {
        // Update state
        {
            let mut state = self.state.write().await;
            state.selection = selected.clone();
        }
        
        // Broadcast event
        self.events
            .broadcast(EngineEvent::SelectionChanged { selected })
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(())
    }
    
    /// Update viewport
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
            .map_err(|e| anyhow::anyhow!(e))?;
        
        Ok(())
    }
    
    /// Discover and register available modalities
    pub async fn discover_modalities(&self) -> Result<()> {
        // This will be implemented when we extract the actual modalities
        // For now, it's a placeholder
        tracing::info!("Discovering modalities...");
        Ok(())
    }
    
    /// Discover and register available compute providers
    pub async fn discover_compute(&self) -> Result<()> {
        // Use universal_discovery to find compute providers
        tracing::info!("Discovering compute providers...");
        Ok(())
    }
    
    /// Start rendering in best available modality
    pub async fn render_auto(self: Arc<Self>) -> Result<()> {
        let modality_name = {
            let registry = self.modalities.read().await;
            registry
                .auto_select()
                .ok_or_else(|| anyhow::anyhow!("No modalities available"))?
                .to_string()
        };
        
        self.render(&modality_name).await
    }
    
    /// Start rendering in specific modality
    pub async fn render(self: Arc<Self>, modality_name: &str) -> Result<()> {
        let mut registry = self.modalities.write().await;
        
        let modality = registry
            .get_mut(modality_name)
            .ok_or_else(|| anyhow::anyhow!("Modality not found: {}", modality_name))?;
        
        // Initialize
        modality.initialize(self.clone()).await?;
        
        // Broadcast start event
        self.events
            .broadcast(EngineEvent::ModalityStarted {
                name: modality_name.to_string(),
            })
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        
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
    pub async fn render_multi(self: Arc<Self>, modality_names: Vec<&str>) -> Result<()> {
        let mut handles = Vec::new();
        
        for name in modality_names {
            let engine = self.clone();
            let name = name.to_string();
            
            let handle = tokio::spawn(async move {
                engine.render(&name).await
            });
            
            handles.push(handle);
        }
        
        // Wait for all to complete (or first error)
        for handle in handles {
            handle.await??;
        }
        
        Ok(())
    }
}

impl Default for UniversalRenderingEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_engine_creation() {
        let engine = UniversalRenderingEngine::new().unwrap();
        
        let state = engine.state.read().await;
        assert_eq!(state.view_mode, ViewMode::Graph);
        assert!(state.selection.is_empty());
    }
    
    #[tokio::test]
    async fn test_selection_update() {
        let engine = UniversalRenderingEngine::new().unwrap();
        
        let mut selected = HashSet::new();
        selected.insert("node1".to_string());
        
        // Event broadcast may fail if no subscribers - that's OK in tests
        let _ = engine.set_selection(selected.clone()).await;
        
        let state = engine.state.read().await;
        assert_eq!(state.selection.len(), 1);
        assert!(state.selection.contains("node1"));
    }
    
    #[tokio::test]
    async fn test_viewport_update() {
        let engine = UniversalRenderingEngine::new().unwrap();
        
        // Event broadcast may fail if no subscribers - that's OK in tests
        let _ = engine.set_viewport(100.0, 200.0, 1.5).await;
        
        let state = engine.state.read().await;
        assert_eq!(state.viewport.center_x, 100.0);
        assert_eq!(state.viewport.center_y, 200.0);
        assert_eq!(state.viewport.zoom, 1.5);
    }
}

