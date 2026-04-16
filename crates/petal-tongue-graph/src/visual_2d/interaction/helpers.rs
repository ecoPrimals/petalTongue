// SPDX-License-Identifier: AGPL-3.0-or-later
//! Small helpers for interactive node naming and edge deduplication.

#[must_use]
pub(super) fn interactive_node_id(node_count: usize) -> String {
    format!("interactive-node-{}", node_count + 1)
}

#[must_use]
pub(super) fn interactive_node_name(node_count: usize) -> String {
    format!("Node {}", node_count + 1)
}

#[must_use]
pub(super) fn is_edge_duplicate(
    from: &str,
    to: &str,
    existing_from: &str,
    existing_to: &str,
) -> bool {
    (existing_from == from && existing_to == to) || (existing_from == to && existing_to == from)
}
