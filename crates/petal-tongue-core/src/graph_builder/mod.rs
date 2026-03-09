// SPDX-License-Identifier: AGPL-3.0-only
//! Graph Builder Core Types
//!
//! Visual graph representation for Neural API graph construction.
//! Provides data structures for nodes, edges, and graph manipulation.

mod builder;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    EdgeType, GraphEdge, GraphLayout, GraphNode, NodeType, NodeVisualState, Vec2, VisualGraph,
};
