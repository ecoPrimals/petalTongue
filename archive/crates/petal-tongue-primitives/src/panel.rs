// SPDX-License-Identifier: AGPL-3.0-only
//! # Panel Layout Primitive
//!
//! Flexible panel layout system with splits, tabs, and docking.
//!
//! ## Philosophy
//!
//! - **Flexible**: Support splits (horizontal/vertical), tabs, and floating panels
//! - **Generic Content**: Panels can contain ANY content type
//! - **Resizable**: Support for resizing split panels
//! - **Dockable**: Panels can be docked/undocked (future)
//! - **Safe**: 100% safe Rust, no unsafe code
//!
//! ## Example
//!
//! ```rust,ignore
//! use petal_tongue_primitives::panel::{Panel, Direction};
//!
//! // Create a split layout: editor on left, file tree on right
//! let layout = Panel::split(
//!     Direction::Horizontal,
//!     0.7, // 70% for left panel
//!     Panel::leaf("editor", "Editor", editor_content),
//!     Panel::leaf("tree", "File Tree", tree_content),
//! );
//! ```

use serde::{Deserialize, Serialize};

/// Panel ID (unique identifier)
pub type PanelId = String;

/// Split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Horizontal split (left/right)
    Horizontal,
    /// Vertical split (top/bottom)
    Vertical,
}

/// Panel content
///
/// This is generic to support any content type.
/// In practice, content might be a Tree, Table, or custom widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelContent<T> {
    /// Panel unique identifier
    pub id: PanelId,

    /// Panel title (for tabs, headers)
    pub title: String,

    /// The actual content
    pub content: T,

    /// Whether panel is visible
    pub visible: bool,

    /// Whether panel is focused
    pub focused: bool,
}

impl<T> PanelContent<T> {
    /// Create new panel content
    pub fn new(id: impl Into<PanelId>, title: impl Into<String>, content: T) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content,
            visible: true,
            focused: false,
        }
    }

    /// Set visibility
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set focused state
    #[must_use]
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

/// Panel layout node
///
/// A panel can be:
/// - A leaf (single content panel)
/// - A split (two child panels with a divider)
/// - A tab group (multiple panels in tabs)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Panel<T> {
    /// Single content panel
    Leaf(PanelContent<T>),

    /// Split panel (horizontal or vertical)
    Split {
        /// Split direction
        direction: Direction,

        /// Split ratio (0.0 - 1.0)
        /// Represents the size of the first panel
        ratio: f32,

        /// First panel
        first: Box<Panel<T>>,

        /// Second panel
        second: Box<Panel<T>>,
    },

    /// Tab group (multiple panels)
    Tabs {
        /// Tab panels
        panels: Vec<PanelContent<T>>,

        /// Active tab index
        active_index: usize,
    },
}

impl<T> Panel<T> {
    /// Create a leaf panel
    pub fn leaf(id: impl Into<PanelId>, title: impl Into<String>, content: T) -> Self {
        Panel::Leaf(PanelContent::new(id, title, content))
    }

    /// Create a split panel
    pub fn split(direction: Direction, ratio: f32, first: Panel<T>, second: Panel<T>) -> Self {
        Panel::Split {
            direction,
            ratio: ratio.clamp(0.0, 1.0),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Create a tab group
    #[must_use]
    pub fn tabs(panels: Vec<PanelContent<T>>, active_index: usize) -> Self {
        Panel::Tabs {
            panels,
            active_index,
        }
    }

    /// Get panel by ID (depth-first search)
    pub fn find_panel(&self, id: &str) -> Option<&PanelContent<T>> {
        match self {
            Panel::Leaf(content) => {
                if content.id == id {
                    Some(content)
                } else {
                    None
                }
            }
            Panel::Split { first, second, .. } => {
                first.find_panel(id).or_else(|| second.find_panel(id))
            }
            Panel::Tabs { panels, .. } => panels.iter().find(|p| p.id == id),
        }
    }

    /// Get mutable panel by ID
    pub fn find_panel_mut(&mut self, id: &str) -> Option<&mut PanelContent<T>> {
        match self {
            Panel::Leaf(content) => {
                if content.id == id {
                    Some(content)
                } else {
                    None
                }
            }
            Panel::Split { first, second, .. } => first
                .find_panel_mut(id)
                .or_else(|| second.find_panel_mut(id)),
            Panel::Tabs { panels, .. } => panels.iter_mut().find(|p| p.id == id),
        }
    }

    /// Count total panels
    pub fn count_panels(&self) -> usize {
        match self {
            Panel::Leaf(_) => 1,
            Panel::Split { first, second, .. } => first.count_panels() + second.count_panels(),
            Panel::Tabs { panels, .. } => panels.len(),
        }
    }

    /// Get all panel IDs
    pub fn panel_ids(&self) -> Vec<PanelId> {
        match self {
            Panel::Leaf(content) => vec![content.id.clone()],
            Panel::Split { first, second, .. } => {
                let mut ids = first.panel_ids();
                ids.extend(second.panel_ids());
                ids
            }
            Panel::Tabs { panels, .. } => panels.iter().map(|p| p.id.clone()).collect(),
        }
    }

    /// Visit all panels (depth-first)
    pub fn visit<F>(&self, f: &mut F)
    where
        F: FnMut(&PanelContent<T>),
    {
        match self {
            Panel::Leaf(content) => f(content),
            Panel::Split { first, second, .. } => {
                first.visit(f);
                second.visit(f);
            }
            Panel::Tabs { panels, .. } => {
                for panel in panels {
                    f(panel);
                }
            }
        }
    }
}

impl<T: Clone> Panel<T> {
    /// Map panel content to a new type
    pub fn map<U, F>(self, f: &F) -> Panel<U>
    where
        F: Fn(T) -> U,
    {
        match self {
            Panel::Leaf(content) => Panel::Leaf(PanelContent {
                id: content.id,
                title: content.title,
                content: f(content.content),
                visible: content.visible,
                focused: content.focused,
            }),
            Panel::Split {
                direction,
                ratio,
                first,
                second,
            } => Panel::Split {
                direction,
                ratio,
                first: Box::new(first.map(f)),
                second: Box::new(second.map(f)),
            },
            Panel::Tabs {
                panels,
                active_index,
            } => Panel::Tabs {
                panels: panels
                    .into_iter()
                    .map(|p| PanelContent {
                        id: p.id,
                        title: p.title,
                        content: f(p.content),
                        visible: p.visible,
                        focused: p.focused,
                    })
                    .collect(),
                active_index,
            },
        }
    }
}

/// Panel layout manager
///
/// Manages the overall panel layout and operations
pub struct PanelLayout<T> {
    /// Root panel
    root: Panel<T>,

    /// Currently focused panel ID
    focused_id: Option<PanelId>,
}

impl<T> PanelLayout<T> {
    /// Create a new panel layout
    pub fn new(root: Panel<T>) -> Self {
        Self {
            root,
            focused_id: None,
        }
    }

    /// Get root panel
    pub fn root(&self) -> &Panel<T> {
        &self.root
    }

    /// Get mutable root panel
    pub fn root_mut(&mut self) -> &mut Panel<T> {
        &mut self.root
    }

    /// Get focused panel ID
    pub fn focused_id(&self) -> Option<&str> {
        self.focused_id.as_deref()
    }

    /// Set focused panel
    pub fn focus_panel(&mut self, id: impl Into<PanelId>) {
        let id = id.into();

        // Unfocus all panels
        self.root.visit(&mut |_panel: &PanelContent<T>| {
            // Note: We can't mutate here, need a separate method
        });

        // Set new focus
        if let Some(panel) = self.root.find_panel_mut(&id) {
            panel.focused = true;
            self.focused_id = Some(id);
        }
    }

    /// Find panel by ID
    pub fn find_panel(&self, id: &str) -> Option<&PanelContent<T>> {
        self.root.find_panel(id)
    }

    /// Find mutable panel by ID
    pub fn find_panel_mut(&mut self, id: &str) -> Option<&mut PanelContent<T>> {
        self.root.find_panel_mut(id)
    }

    /// Count total panels
    pub fn count_panels(&self) -> usize {
        self.root.count_panels()
    }

    /// Get all panel IDs
    pub fn panel_ids(&self) -> Vec<PanelId> {
        self.root.panel_ids()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_panel() {
        let panel = Panel::leaf("editor", "Editor", "Hello, World!");

        match panel {
            Panel::Leaf(content) => {
                assert_eq!(content.id, "editor");
                assert_eq!(content.title, "Editor");
                assert_eq!(content.content, "Hello, World!");
                assert!(content.visible);
                assert!(!content.focused);
            }
            _ => panic!("Expected leaf panel"),
        }
    }

    #[test]
    fn test_split_panel() {
        let panel = Panel::split(
            Direction::Horizontal,
            0.7,
            Panel::leaf("left", "Left", "Left content"),
            Panel::leaf("right", "Right", "Right content"),
        );

        match panel {
            Panel::Split {
                direction, ratio, ..
            } => {
                assert_eq!(direction, Direction::Horizontal);
                assert_eq!(ratio, 0.7);
            }
            _ => panic!("Expected split panel"),
        }
    }

    #[test]
    fn test_tab_panel() {
        let panels = vec![
            PanelContent::new("tab1", "Tab 1", "Content 1"),
            PanelContent::new("tab2", "Tab 2", "Content 2"),
        ];

        let panel = Panel::tabs(panels, 0);

        match panel {
            Panel::Tabs {
                panels,
                active_index,
            } => {
                assert_eq!(panels.len(), 2);
                assert_eq!(active_index, 0);
            }
            _ => panic!("Expected tabs panel"),
        }
    }

    #[test]
    fn test_find_panel() {
        let layout = Panel::split(
            Direction::Horizontal,
            0.5,
            Panel::leaf("left", "Left", "Left"),
            Panel::split(
                Direction::Vertical,
                0.5,
                Panel::leaf("top", "Top", "Top"),
                Panel::leaf("bottom", "Bottom", "Bottom"),
            ),
        );

        assert!(layout.find_panel("left").is_some());
        assert!(layout.find_panel("top").is_some());
        assert!(layout.find_panel("bottom").is_some());
        assert!(layout.find_panel("nonexistent").is_none());
    }

    #[test]
    fn test_count_panels() {
        let layout = Panel::split(
            Direction::Horizontal,
            0.5,
            Panel::leaf("left", "Left", ""),
            Panel::split(
                Direction::Vertical,
                0.5,
                Panel::leaf("top", "Top", ""),
                Panel::leaf("bottom", "Bottom", ""),
            ),
        );

        assert_eq!(layout.count_panels(), 3);
    }

    #[test]
    fn test_panel_ids() {
        let layout = Panel::split(
            Direction::Horizontal,
            0.5,
            Panel::leaf("left", "Left", ""),
            Panel::leaf("right", "Right", ""),
        );

        let ids = layout.panel_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"left".to_string()));
        assert!(ids.contains(&"right".to_string()));
    }

    #[test]
    fn test_panel_map() {
        let layout = Panel::split(
            Direction::Horizontal,
            0.5,
            Panel::leaf("left", "Left", 10),
            Panel::leaf("right", "Right", 20),
        );

        let mapped = layout.map(&|x: i32| x.to_string());

        let left = mapped.find_panel("left").unwrap();
        assert_eq!(left.content, "10");

        let right = mapped.find_panel("right").unwrap();
        assert_eq!(right.content, "20");
    }

    #[test]
    fn test_panel_layout() {
        let root = Panel::split(
            Direction::Horizontal,
            0.5,
            Panel::leaf("left", "Left", ""),
            Panel::leaf("right", "Right", ""),
        );

        let mut layout = PanelLayout::new(root);

        assert_eq!(layout.count_panels(), 2);
        assert_eq!(layout.focused_id(), None);

        layout.focus_panel("left");
        assert_eq!(layout.focused_id(), Some("left"));
    }

    #[test]
    fn test_split_ratio_clamping() {
        let panel = Panel::split(
            Direction::Horizontal,
            1.5, // Out of range
            Panel::leaf("left", "Left", ""),
            Panel::leaf("right", "Right", ""),
        );

        match panel {
            Panel::Split { ratio, .. } => {
                assert_eq!(ratio, 1.0); // Clamped to max
            }
            _ => panic!("Expected split panel"),
        }
    }

    #[test]
    fn test_panel_visibility() {
        let panel = PanelContent::new("test", "Test", "content").visible(false);

        assert!(!panel.visible);
    }

    #[test]
    fn test_visit_panels() {
        let layout = Panel::split(
            Direction::Horizontal,
            0.5,
            Panel::leaf("left", "Left", 1),
            Panel::leaf("right", "Right", 2),
        );

        let mut sum = 0;
        layout.visit(&mut |panel| {
            sum += panel.content;
        });

        assert_eq!(sum, 3);
    }
}
