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
pub mod graph;
pub mod node;
pub mod edge;
pub mod validation;

pub use canvas::GraphCanvas;
pub use graph::Graph;
pub use node::GraphNode;
pub use edge::{GraphEdge, DependencyType};
pub use validation::GraphValidator;

