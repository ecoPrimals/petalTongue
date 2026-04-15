// SPDX-License-Identifier: AGPL-3.0-or-later
//! Grammar compiler: transforms `GrammarExpr` + data into a `SceneGraph`.
//!
//! The compiler reads variable bindings from the grammar expression to map
//! data fields to x/y coordinates, applies scales, and produces primitives
//! for the requested geometry type.

mod core;
mod facets;
mod geometry;
mod plan;
mod utils;

#[cfg(test)]
mod tests;

use serde_json::Value;

use crate::grammar::GrammarExpr;
use crate::primitive::Primitive;
use crate::scene_graph::{SceneGraph, SceneNode};
use crate::tufte::{TufteConstraint, TufteReport};

/// Compiles `GrammarExpr` and data into a `SceneGraph`.
#[derive(Debug, Clone, Default)]
pub struct GrammarCompiler;

impl GrammarCompiler {
    /// Create a new grammar compiler.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Compile grammar expression and data, then evaluate Tufte constraints.
    pub fn compile_with_constraints(
        &self,
        expr: &GrammarExpr,
        data: &[Value],
        constraints: &[&dyn TufteConstraint],
    ) -> (SceneGraph, TufteReport) {
        let graph = self.compile(expr, data);
        let primitives: Vec<Primitive> = graph
            .flatten()
            .into_iter()
            .map(|(_, p)| p.clone())
            .collect();
        let report = TufteReport::evaluate_all(constraints, &primitives, expr, Some(data));
        (graph, report)
    }
}

impl SceneNode {
    /// Builder: add multiple primitives.
    fn with_primitives(mut self, primitives: Vec<Primitive>) -> Self {
        for p in primitives {
            self.primitives.push(p);
        }
        self
    }
}
