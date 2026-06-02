// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ecosystem visualization scene builders.
//!
//! Each submodule builds a specific visualization as a `SceneGraph`.
//! The `VizRegistry` provides capability-based discovery of available
//! visualizations at runtime.

pub mod entity_graph;
pub mod kderm;
pub mod nucleus;

use petal_tongue_scene::animation::Sequence;
use petal_tongue_scene::scene_graph::SceneGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use entity_graph::{build_entity_graph_scene, load_entity_graph};
pub use kderm::{build_kderm_relay_animation, build_kderm_scene};
pub use nucleus::{build_nucleus_expand_animation, build_nucleus_scene};

// ── Visualization Registry ──────────────────────────────────────────────

/// A registered visualization builder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VizEntry {
    /// URL-safe identifier (e.g. "entity-graph").
    pub slug: String,
    /// Human-readable display name.
    pub title: String,
    /// Short summary of what this visualization shows.
    pub description: String,
    /// Optional path to external data feeding the visualization.
    pub data_source: Option<PathBuf>,
    /// Whether this visualization supports animated transitions.
    pub has_animation: bool,
}

/// Registry of available visualizations, discoverable at runtime.
#[derive(Debug, Clone)]
pub struct VizRegistry {
    entries: HashMap<String, VizEntry>,
}

impl VizRegistry {
    /// Build the registry by discovering capabilities from the filesystem.
    pub fn discover(static_dir: Option<&Path>) -> Self {
        let mut entries = HashMap::new();

        let graph_path = static_dir.map(|s| s.join("graph/entity-graph.json"));
        let graph_exists = graph_path.as_ref().is_some_and(|p| p.is_file());

        if graph_exists {
            entries.insert(
                "entity-graph".to_owned(),
                VizEntry {
                    slug: "entity-graph".to_owned(),
                    title: "Entity Graph Explorer".to_owned(),
                    description: "Force-directed graph of ecosystem entities and relationships"
                        .to_owned(),
                    data_source: graph_path,
                    has_animation: false,
                },
            );
        }

        entries.insert(
            "kderm-topology".to_owned(),
            VizEntry {
                slug: "kderm-topology".to_owned(),
                title: "K-Derm Diderm Topology".to_owned(),
                description: "5-layer cross-section of sovereign infrastructure with relay chain"
                    .to_owned(),
                data_source: None,
                has_animation: true,
            },
        );

        entries.insert(
            "nucleus-composition".to_owned(),
            VizEntry {
                slug: "nucleus-composition".to_owned(),
                title: "NUCLEUS Atomics Composition".to_owned(),
                description: "Nested composition layers from Tower to Full NUCLEUS".to_owned(),
                data_source: None,
                has_animation: true,
            },
        );

        Self { entries }
    }

    /// List all available visualization slugs.
    pub fn available(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    /// Get a viz entry by slug.
    pub fn get(&self, slug: &str) -> Option<&VizEntry> {
        self.entries.get(slug)
    }

    /// List all registered visualizations.
    pub fn list(&self) -> Vec<&VizEntry> {
        self.entries.values().collect()
    }

    /// Build a scene for the given slug, or None if unknown.
    pub fn build_scene(&self, slug: &str) -> Option<SceneGraph> {
        let entry = self.get(slug)?;
        match slug {
            "entity-graph" => {
                let path = entry.data_source.as_ref()?;
                let graph = load_entity_graph(path)?;
                Some(build_entity_graph_scene(&graph))
            }
            "kderm-topology" => Some(build_kderm_scene()),
            "nucleus-composition" => Some(build_nucleus_scene()),
            _ => None,
        }
    }

    /// Build an animation sequence for the given slug, or None.
    pub fn build_animation(&self, slug: &str) -> Option<Sequence> {
        let entry = self.get(slug)?;
        if !entry.has_animation {
            return None;
        }
        match slug {
            "kderm-topology" => Some(build_kderm_relay_animation()),
            "nucleus-composition" => Some(build_nucleus_expand_animation("nest-atomic")),
            _ => None,
        }
    }
}
