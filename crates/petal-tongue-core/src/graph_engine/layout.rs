// SPDX-License-Identifier: AGPL-3.0-or-later
//! Layout algorithms for positioning graph nodes.

use crate::types::TopologyEdge;
use std::collections::HashMap;

use super::types::Node;

/// Force-directed layout using Fruchterman-Reingold algorithm
pub(super) fn force_directed_layout(nodes: &mut [Node], edges: &[TopologyEdge], iterations: usize) {
    const K: f32 = 100.0; // Optimal distance between nodes
    const AREA: f32 = 1000.0; // Layout area
    const COOLING_FACTOR: f32 = 0.95;

    // Initialize nodes with random positions if they're all at origin
    let all_at_origin = nodes
        .iter()
        .all(|n| n.position.x == 0.0 && n.position.y == 0.0);
    if all_at_origin {
        random_layout(nodes);
    }

    let mut temperature = AREA / 10.0;

    for _ in 0..iterations {
        // Calculate repulsive forces (all pairs)
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let delta_x = nodes[i].position.x - nodes[j].position.x;
                let delta_y = nodes[i].position.y - nodes[j].position.y;
                let distance = delta_x.hypot(delta_y).max(0.01);

                let repulsion = K * K / distance;
                let force_x = (delta_x / distance) * repulsion;
                let force_y = (delta_y / distance) * repulsion;

                nodes[i].velocity.x += force_x;
                nodes[i].velocity.y += force_y;
                nodes[j].velocity.x -= force_x;
                nodes[j].velocity.y -= force_y;
            }
        }

        // Calculate attractive forces (edges)
        for edge in edges {
            if let (Some(from_idx), Some(to_idx)) = (
                nodes.iter().position(|n| n.info.id == edge.from),
                nodes.iter().position(|n| n.info.id == edge.to),
            ) {
                let delta_x = nodes[from_idx].position.x - nodes[to_idx].position.x;
                let delta_y = nodes[from_idx].position.y - nodes[to_idx].position.y;
                let distance = delta_x.hypot(delta_y).max(0.01);

                let attraction = distance * distance / K;
                let force_x = (delta_x / distance) * attraction;
                let force_y = (delta_y / distance) * attraction;

                nodes[from_idx].velocity.x -= force_x;
                nodes[from_idx].velocity.y -= force_y;
                nodes[to_idx].velocity.x += force_x;
                nodes[to_idx].velocity.y += force_y;
            }
        }

        // Apply velocities with cooling
        for node in nodes.iter_mut() {
            let v_len = node.velocity.x.hypot(node.velocity.y);
            if v_len > 0.0 {
                let displacement = v_len.min(temperature);
                node.position.x += (node.velocity.x / v_len) * displacement;
                node.position.y += (node.velocity.y / v_len) * displacement;
            }

            // Reset velocity
            node.velocity.x = 0.0;
            node.velocity.y = 0.0;
        }

        temperature *= COOLING_FACTOR;
    }
}

/// Hierarchical layout (simple tree-like layout)
///
/// OPTIMIZATION: Uses node indices instead of cloning IDs repeatedly
pub(super) fn hierarchical_layout(nodes: &mut [Node], edges: &[TopologyEdge]) {
    if nodes.is_empty() {
        return;
    }

    // Build ID -> index mapping
    let id_to_index: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.info.id.as_str(), idx))
        .collect();

    // Find root nodes (nodes with no incoming edges)
    let mut incoming_counts: HashMap<usize, usize> = HashMap::new();
    for edge in edges {
        if let Some(&to_idx) = id_to_index.get(edge.to.as_str()) {
            *incoming_counts.entry(to_idx).or_insert(0) += 1;
        }
    }

    let root_indices: Vec<usize> = (0..nodes.len())
        .filter(|&idx| !incoming_counts.contains_key(&idx))
        .collect();

    // Assign levels using BFS (using indices, not IDs)
    let mut levels: HashMap<usize, usize> = HashMap::new();
    let mut queue = root_indices.clone();
    for &root_idx in &root_indices {
        levels.insert(root_idx, 0);
    }

    while let Some(current_idx) = queue.pop() {
        let current_level = levels[&current_idx];
        let current_id = &nodes[current_idx].info.id;

        for edge in edges {
            if edge.from.as_str() == current_id.as_str()
                && let Some(&to_idx) = id_to_index.get(edge.to.as_str())
                && !levels.contains_key(&to_idx)
            {
                levels.insert(to_idx, current_level + 1);
                queue.push(to_idx);
            }
        }
    }

    // Position nodes by level
    let mut level_counts: HashMap<usize, usize> = HashMap::new();
    for (idx, node) in nodes.iter_mut().enumerate() {
        let level = levels.get(&idx).copied().unwrap_or(0);
        let count = level_counts.entry(level).or_insert(0);

        #[expect(clippy::cast_precision_loss)]
        {
            node.position.x = (*count as f32) * 150.0;
            node.position.y = (level as f32) * 150.0;
        }

        *count += 1;
    }
}

/// Circular layout (nodes arranged in a circle)
#[expect(clippy::cast_precision_loss)] // Precision loss acceptable for layout
pub(super) fn circular_layout(nodes: &mut [Node]) {
    let radius = 300.0;
    let angle_step = 2.0 * std::f32::consts::PI / nodes.len() as f32;

    for (i, node) in nodes.iter_mut().enumerate() {
        let angle = (i as f32) * angle_step;
        node.position.x = angle.cos() * radius;
        node.position.y = angle.sin() * radius;
    }
}

/// Random layout (for testing)
#[expect(clippy::cast_precision_loss)] // Precision loss acceptable for layout
pub(super) fn random_layout(nodes: &mut [Node]) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    for node in nodes.iter_mut() {
        // Use node ID as seed for deterministic "random" layout
        let mut hasher = DefaultHasher::new();
        node.info.id.hash(&mut hasher);
        let hash = hasher.finish();

        let x = ((hash % 1000) as f32 - 500.0) * 2.0;
        let y = (((hash / 1000) % 1000) as f32 - 500.0) * 2.0;

        node.position.x = x;
        node.position.y = y;
    }
}
