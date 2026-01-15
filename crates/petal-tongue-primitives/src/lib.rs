//! # petalTongue Primitives
//!
//! Universal UI rendering primitives for the petalTongue UI infrastructure.
//!
//! ## Philosophy
//!
//! - **Zero Hardcoding**: All primitives are capability-based and data-driven
//! - **100% Safe Rust**: No unsafe code in UI primitives
//! - **Modern Idiomatic**: Async/await, zero-cost abstractions
//! - **Multi-Modal**: Every primitive renders across GUI, TUI, Audio, API
//! - **Generic**: Works with any data type via traits
//!
//! ## Primitives
//!
//! - **Tree**: Hierarchical data (files, categories, org charts)
//! - **Table**: Tabular data (logs, metrics, search results)
//! - **Form**: Editable fields (settings, config)
//! - **Code**: Syntax-highlighted text (editor, diff viewer)
//! - **Timeline**: Temporal data (history, events)
//! - **Chat**: Message streams (chat, comments)
//! - **Dashboard**: Metrics and KPIs (monitoring)
//! - **Canvas**: Free-form drawing (diagrams, sketches)
//!
//! ## Example
//!
//! ```rust
//! use petal_tongue_primitives::tree::TreeNode;
//!
//! // Build a tree (zero hardcoding - works with ANY data)
//! let root = TreeNode::new("root");
//! // Add children via builder pattern or direct mutation
//! // Render to GUI, TUI, or API (capability-based)
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(unsafe_code)] // 100% safe Rust in primitives

/// Command palette primitive
pub mod command_palette;
/// Form primitive with validation
pub mod form;
/// Panel layout system
pub mod panel;
/// Capability-based renderer traits
pub mod renderer;
/// Renderer implementations
pub mod renderers;
/// Generic table primitive
pub mod table;
/// Tree primitive
pub mod tree;

// pub mod form;     // Coming in Phase 2.5
// pub mod form;     // Coming in Phase 2.5
// pub mod code;     // Coming in Phase 3.1
// pub mod timeline; // Future
// pub mod chat;     // Future
// pub mod dashboard; // Future
// pub mod canvas;   // Future

/// Common types used across all primitives
pub mod common {
    use serde::{Deserialize, Serialize};

    /// Icon representation (capability-based)
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Icon {
        /// Unicode emoji
        Emoji(String),
        /// Nerd font icon
        NerdFont(String),
        /// Custom icon (capability discovery will determine support)
        Custom(String),
        /// No icon
        None,
    }

    impl std::fmt::Display for Icon {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Icon::Emoji(s) | Icon::NerdFont(s) | Icon::Custom(s) => write!(f, "{}", s),
                Icon::None => Ok(()),
            }
        }
    }

    /// Color representation (multi-modal)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Color {
        /// Red component (0-255)
        pub r: u8,
        /// Green component (0-255)
        pub g: u8,
        /// Blue component (0-255)
        pub b: u8,
        /// Alpha component (0-255)
        pub a: u8,
    }

    impl Color {
        /// Create a new color
        pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
            Self { r, g, b, a: 255 }
        }

        /// Create a new color with alpha
        pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
            Self { r, g, b, a }
        }
    }

    /// Common color constants
    impl Color {
        /// Transparent
        pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
        /// White
        pub const WHITE: Self = Self::rgb(255, 255, 255);
        /// Black
        pub const BLACK: Self = Self::rgb(0, 0, 0);
        /// Red
        pub const RED: Self = Self::rgb(255, 0, 0);
        /// Green
        pub const GREEN: Self = Self::rgb(0, 255, 0);
        /// Blue
        pub const BLUE: Self = Self::rgb(0, 0, 255);
    }

    /// Size representation
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Size {
        /// Width
        pub width: f32,
        /// Height
        pub height: f32,
    }

    /// Point in 2D space
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Point {
        /// X coordinate
        pub x: f32,
        /// Y coordinate
        pub y: f32,
    }
}
