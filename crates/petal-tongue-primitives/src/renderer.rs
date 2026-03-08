// SPDX-License-Identifier: AGPL-3.0-only
//! # Renderer Traits
//!
//! Capability-based renderer traits for UI primitives.
//!
//! ## Philosophy
//!
//! - **Capability-Based**: Renderers discovered at runtime, not hardcoded
//! - **Multi-Modal**: Same primitive renders to GUI, TUI, Audio, API
//! - **Generic**: Works with any data type
//! - **Zero Hardcoding**: No assumptions about specific renderer implementations
//!
//! ## Example
//!
//! ```rust,ignore
//! // Discover available renderers at runtime
//! let renderer = discover_tree_renderer().await?;
//!
//! // Render tree (works with any renderer)
//! renderer.render_tree(&tree)?;
//! ```

use crate::tree::TreeNode;
use anyhow::Result;
use async_trait::async_trait;

/// Capability-based tree renderer
///
/// This trait is implemented by different rendering backends (GUI, TUI, Audio, API).
/// The actual renderer is discovered at runtime based on available capabilities.
///
/// # Zero Hardcoding
///
/// Implementations do NOT know about each other. Each renderer only knows itself.
/// Discovery happens through capability system at runtime.
#[async_trait]
pub trait TreeRenderer<T>: Send + Sync {
    /// Render a tree
    ///
    /// The renderer adapts to its modality (GUI shows visually, TUI shows in terminal,
    /// Audio reads the structure, API returns JSON, etc.)
    async fn render_tree(&mut self, root: &TreeNode<T>) -> Result<()>
    where
        T: Send + Sync;

    /// Handle selection event
    ///
    /// Called when user selects a node. The renderer determines HOW selection happens
    /// (mouse click, keyboard, voice command, API call, etc.)
    async fn on_select<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(&T) + Send + Sync + 'static,
        T: Send + Sync;

    /// Handle expansion toggle
    ///
    /// Some modalities support expansion (GUI, TUI), others don't (API just returns all).
    /// Renderer decides if this applies.
    async fn on_toggle(&mut self, node_path: &[usize]) -> Result<()>;

    /// Get renderer capabilities
    ///
    /// Describes what this renderer can do (for capability-based routing)
    fn capabilities(&self) -> RendererCapabilities;

    /// Renderer name (for diagnostics only, not for hardcoding!)
    fn name(&self) -> &str;
}

/// Renderer capabilities (for capability-based discovery)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererCapabilities {
    /// Modality this renderer provides
    pub modality: Modality,

    /// Whether renderer supports expansion/collapse
    pub supports_expansion: bool,

    /// Whether renderer supports selection
    pub supports_selection: bool,

    /// Whether renderer supports icons
    pub supports_icons: bool,

    /// Whether renderer supports colors
    pub supports_colors: bool,

    /// Whether renderer supports filtering
    pub supports_filtering: bool,

    /// Whether renderer is interactive (vs. static export)
    pub is_interactive: bool,
}

/// Rendering modality (what kind of renderer)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modality {
    /// Visual GUI (egui, etc.)
    VisualGUI,

    /// Terminal UI (ratatui, etc.)
    TerminalTUI,

    /// Audio (sonification, screen reader)
    Audio,

    /// API (JSON-RPC, GraphQL, REST)
    API,

    /// Export (SVG, PNG, etc.)
    Export,

    /// Custom modality (discovered at runtime)
    Custom(String),
}

impl Modality {
    /// Check if this is an interactive modality
    #[must_use]
    pub fn is_interactive(&self) -> bool {
        matches!(self, Self::VisualGUI | Self::TerminalTUI | Self::Audio)
    }
}

/// Table renderer (Phase 2.2)
#[async_trait]
pub trait TableRenderer<T>: Send + Sync {
    /// Render a table
    async fn render_table(&mut self, table: &crate::table::Table<T>) -> Result<()>
    where
        T: Send + Sync;

    /// Get capabilities
    fn capabilities(&self) -> RendererCapabilities;
}

/// Panel renderer (Phase 2.3)
#[async_trait]
pub trait PanelRenderer<T>: Send + Sync {
    /// Render a panel layout
    async fn render_panel(&mut self, panel: &crate::panel::Panel<T>) -> Result<()>
    where
        T: Send + Sync;

    /// Get capabilities
    fn capabilities(&self) -> RendererCapabilities;
}

/// Command palette renderer (Phase 2.4)
#[async_trait]
pub trait CommandPaletteRenderer<T>: Send + Sync {
    /// Render a command palette
    async fn render_palette(
        &mut self,
        palette: &crate::command_palette::CommandPalette<T>,
    ) -> Result<()>
    where
        T: Send + Sync + Clone;

    /// Get capabilities
    fn capabilities(&self) -> RendererCapabilities;
}

/// Form renderer (Phase 2.5)
#[async_trait]
pub trait FormRenderer<T>: Send + Sync {
    /// Render a form
    async fn render_form(&mut self, form: &mut crate::form::Form<T>) -> Result<()>
    where
        T: Send + Sync;

    /// Get capabilities
    fn capabilities(&self) -> RendererCapabilities;
}

// Future: CodeRenderer, TimelineRenderer, ChatRenderer, DashboardRenderer, CanvasRenderer

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modality_is_interactive() {
        assert!(Modality::VisualGUI.is_interactive());
        assert!(Modality::TerminalTUI.is_interactive());
        assert!(Modality::Audio.is_interactive());
        assert!(!Modality::API.is_interactive());
        assert!(!Modality::Export.is_interactive());
    }

    #[test]
    fn test_renderer_capabilities() {
        let caps = RendererCapabilities {
            modality: Modality::VisualGUI,
            supports_expansion: true,
            supports_selection: true,
            supports_icons: true,
            supports_colors: true,
            supports_filtering: true,
            is_interactive: true,
        };

        assert_eq!(caps.modality, Modality::VisualGUI);
        assert!(caps.supports_expansion);
        assert!(caps.is_interactive);
    }
}
