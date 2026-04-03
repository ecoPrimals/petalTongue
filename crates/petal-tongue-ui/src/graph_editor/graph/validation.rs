// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph structural validation: cycles, topological order, depth.

use crate::error::{GraphEditorError, Result};
use std::collections::{HashMap, HashSet, VecDeque};

use super::super::edge::GraphEdge;
use super::Graph;

impl Graph {
    /// Build a borrowed adjacency list from the graph's edges.
    pub(super) fn adjacency_list(&self) -> HashMap<&str, Vec<&str>> {
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }
        adj
    }

    /// Check if adding an edge would create a cycle
    pub(super) fn would_create_cycle<'a>(&'a self, new_edge: &'a GraphEdge) -> Result<bool> {
        let mut adj = self.adjacency_list();
        adj.entry(new_edge.from.as_str())
            .or_default()
            .push(new_edge.to.as_str());

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        Ok(Self::has_cycle_dfs(
            new_edge.to.as_str(),
            &adj,
            &mut visited,
            &mut rec_stack,
        ))
    }

    /// DFS helper for cycle detection
    fn has_cycle_dfs<'a>(
        node: &'a str,
        adj: &HashMap<&str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
    ) -> bool {
        if rec_stack.contains(node) {
            return true; // Cycle detected
        }

        if visited.contains(node) {
            return false; // Already processed
        }

        visited.insert(node);
        rec_stack.insert(node);

        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                if Self::has_cycle_dfs(neighbor, adj, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Get topological sort of nodes (execution order)
    ///
    /// Returns nodes in execution order (dependencies first).
    ///
    /// # Errors
    ///
    /// Returns an error if the graph contains cycles.
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let adj = self.adjacency_list();

        for node_id in self.nodes.keys().map(String::as_str) {
            in_degree.insert(node_id, 0);
        }
        for edge in &self.edges {
            *in_degree.entry(edge.to.as_str()).or_default() += 1;
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|&(_, degree)| *degree == 0)
            .map(|(id, _)| *id)
            .collect();

        let mut result = Vec::with_capacity(self.nodes.len());

        while let Some(node) = queue.pop_front() {
            result.push(node);

            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err(GraphEditorError::GraphContainsCycles.into());
        }

        Ok(result.into_iter().map(String::from).collect())
    }

    /// Calculate maximum depth of the graph
    pub(super) fn max_depth(&self) -> usize {
        let adj = self.adjacency_list();
        self.nodes
            .keys()
            .map(|node| Self::calculate_depth(node.as_str(), &adj, &mut HashSet::new()))
            .max()
            .unwrap_or(0)
    }

    /// Calculate depth for a node (DFS)
    fn calculate_depth<'a>(
        node: &'a str,
        adj: &HashMap<&'a str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
    ) -> usize {
        if visited.contains(node) {
            return 0; // Cycle or already visited
        }

        visited.insert(node);

        let depth = adj.get(node).map_or(1, |neighbors| {
            neighbors
                .iter()
                .map(|&neighbor| Self::calculate_depth(neighbor, adj, visited))
                .max()
                .unwrap_or(0)
                + 1
        });

        visited.remove(node);
        depth
    }
}
