//! Graph Editor - Interactive Collaborative Intelligence
//!
//! Enables human-AI collaboration through interactive graph editing.
//!
//! # Philosophy
//!
//! This is TRUE PRIMAL human-AI collaboration:
//! - Human and AI as equals (not subservient)
//! - Transparent reasoning (AI explains "why")
//! - User always in control (can override)
//! - Learn together (mutual improvement)
//! - Bootstrap fast (10x faster deployments)
//!
//! # Architecture
//!
//! The graph editor is built on capability discovery:
//! - No hardcoded node types (discover at runtime)
//! - No hardcoded primals (query Songbird)
//! - Graceful degradation (works without AI)
//! - Self-stable (no external dependencies required)

pub mod canvas;
pub mod edge;
pub mod graph;
pub mod node;
pub mod rpc_methods;
pub mod streaming;
pub mod ui_components;
pub mod validation;

pub use canvas::GraphCanvas;
pub use edge::{DependencyType, GraphEdge};
pub use graph::Graph;
pub use node::GraphNode;
pub use rpc_methods::GraphEditorService;
pub use streaming::{StreamHandler, StreamMessage};
pub use ui_components::{ConflictResolution, ConflictResolutionChoice, ReasoningDisplay, StatusDisplay};
pub use validation::GraphValidator;

