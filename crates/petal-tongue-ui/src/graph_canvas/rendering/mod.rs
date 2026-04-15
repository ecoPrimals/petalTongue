// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph canvas rendering - grid, nodes, edges, selection box.

mod arrow;
mod canvas_impl;
mod geometry;

#[cfg(test)]
mod tests;

pub use geometry::node_colors;
