// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entity graph scene builder — force-directed layout from JSON data.

use petal_tongue_scene::primitive::{
    AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle,
};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};
use petal_tongue_scene::transform::Transform2D;
use serde::Deserialize;
use std::collections::HashMap;

/// A node in the entity-graph.json format.
#[derive(Debug, Deserialize)]
pub struct GraphNode {
    /// Canonical lowercase identifier (e.g. "beardog").
    pub id: String,
    /// Human-readable label shown in the visualization.
    pub display: String,
    /// Entity category (primal, spring, product, infra, garden).
    pub kind: String,
    /// Unicode glyph representing this entity.
    pub emoji: String,
}

/// An edge in the entity-graph.json format.
#[derive(Debug, Deserialize)]
pub struct GraphEdge {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Relationship label (e.g. "validates", "depends_on").
    pub relation: String,
    /// Whether this edge is an inverse/back-reference.
    pub inverse: bool,
}

/// The full entity graph JSON structure.
#[derive(Debug, Deserialize)]
pub struct EntityGraph {
    /// All entities participating in the graph.
    pub nodes: Vec<GraphNode>,
    /// Relationships between entities.
    pub edges: Vec<GraphEdge>,
}

/// Color palette for entity kinds (Catppuccin Mocha).
fn kind_color(kind: &str) -> Color {
    match kind {
        "primal" => Color::from_rgba8(166, 227, 161, 255),
        "spring" => Color::from_rgba8(137, 180, 250, 255),
        "product" => Color::from_rgba8(245, 194, 231, 255),
        "composition" => Color::from_rgba8(249, 226, 175, 255),
        "concept" => Color::from_rgba8(203, 166, 247, 255),
        "infra" => Color::from_rgba8(148, 226, 213, 255),
        "org" => Color::from_rgba8(250, 179, 135, 255),
        _ => Color::from_rgba8(186, 194, 222, 255),
    }
}

fn relation_color(relation: &str) -> Color {
    match relation {
        "composes into" => Color::from_rgba8(166, 227, 161, 200),
        "validated by" | "validates" => Color::from_rgba8(137, 180, 250, 200),
        "dispatches" | "dispatched by" => Color::from_rgba8(249, 226, 175, 200),
        "stores for" | "stored by" => Color::from_rgba8(148, 226, 213, 200),
        "compiled by" | "compiles" => Color::from_rgba8(245, 194, 231, 200),
        "discovers" | "discovered by" => Color::from_rgba8(203, 166, 247, 200),
        "derived from" | "extends" => Color::from_rgba8(250, 179, 135, 200),
        _ => Color::from_rgba8(186, 194, 222, 150),
    }
}

/// Fruchterman-Reingold-style force-directed layout.
fn force_layout(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    width: f64,
    height: f64,
) -> HashMap<String, (f64, f64)> {
    let n = nodes.len();
    let mut positions: Vec<(f64, f64)> = Vec::with_capacity(n);
    let id_to_idx: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
        let r = width.min(height) * 0.35;
        let cx = width / 2.0;
        let cy = height / 2.0;
        positions.push((cx + r * angle.cos(), cy + r * angle.sin()));
    }

    let k = (width * height / n as f64).sqrt();
    let iterations = 80;

    for iter in 0..iterations {
        let temp = 0.1 * width * (1.0 - iter as f64 / iterations as f64);
        let mut displacements = vec![(0.0f64, 0.0f64); n];

        for i in 0..n {
            for j in (i + 1)..n {
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                let dist = dx.hypot(dy).max(1.0);
                let force = k * k / dist;
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;
                displacements[i].0 += fx;
                displacements[i].1 += fy;
                displacements[j].0 -= fx;
                displacements[j].1 -= fy;
            }
        }

        for edge in edges {
            if edge.inverse {
                continue;
            }
            let Some(&si) = id_to_idx.get(edge.source.as_str()) else {
                continue;
            };
            let Some(&ti) = id_to_idx.get(edge.target.as_str()) else {
                continue;
            };
            let dx = positions[si].0 - positions[ti].0;
            let dy = positions[si].1 - positions[ti].1;
            let dist = dx.hypot(dy).max(1.0);
            let force = dist * dist / k;
            let fx = (dx / dist) * force;
            let fy = (dy / dist) * force;
            displacements[si].0 -= fx;
            displacements[si].1 -= fy;
            displacements[ti].0 += fx;
            displacements[ti].1 += fy;
        }

        for i in 0..n {
            let (dx, dy) = displacements[i];
            let dist = dx.hypot(dy).max(0.001);
            let scale = temp.min(dist) / dist;
            positions[i].0 = (positions[i].0 + dx * scale).clamp(40.0, width - 40.0);
            positions[i].1 = (positions[i].1 + dy * scale).clamp(40.0, height - 40.0);
        }
    }

    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), positions[i]))
        .collect()
}

/// Build the entity graph scene from parsed JSON data.
pub fn build_entity_graph_scene(graph: &EntityGraph) -> SceneGraph {
    let width = 800.0;
    let height = 600.0;
    let mut scene = SceneGraph::new();

    let positions = force_layout(&graph.nodes, &graph.edges, width, height);

    let edges_node = SceneNode::new("edges").with_label("Entity relationships");
    scene.add_to_root(edges_node);

    for (i, edge) in graph.edges.iter().enumerate() {
        if edge.inverse {
            continue;
        }
        let Some(&(sx, sy)) = positions.get(&edge.source) else {
            continue;
        };
        let Some(&(tx, ty)) = positions.get(&edge.target) else {
            continue;
        };

        let edge_prim = Primitive::Line {
            points: vec![[sx, sy], [tx, ty]],
            stroke: StrokeStyle {
                color: relation_color(&edge.relation),
                width: 1.2,
                cap: LineCap::Round,
                join: LineJoin::Round,
            },
            closed: false,
            data_id: Some(format!("edge-{i}")),
        };

        let edge_node = SceneNode::new(format!("edge-{i}"))
            .with_primitive(edge_prim)
            .with_opacity(0.7);
        scene.add_node(edge_node, "edges");
    }

    let nodes_group = SceneNode::new("nodes").with_label("Ecosystem entities");
    scene.add_to_root(nodes_group);

    for gn in &graph.nodes {
        let Some(&(x, y)) = positions.get(&gn.id) else {
            continue;
        };
        let color = kind_color(&gn.kind);
        let radius = match gn.kind.as_str() {
            "primal" => 10.0,
            "spring" => 8.0,
            "composition" => 9.0,
            _ => 7.0,
        };

        let circle = Primitive::Point {
            x,
            y,
            radius,
            fill: Some(color),
            stroke: Some(StrokeStyle {
                color: Color::from_rgba8(30, 30, 46, 200),
                width: 1.5,
                cap: LineCap::Round,
                join: LineJoin::Round,
            }),
            data_id: Some(gn.id.clone()),
        };

        let label = Primitive::Text {
            x,
            y: y + radius + 12.0,
            content: gn.display.clone(),
            font_size: 9.0,
            color: Color::from_rgba8(205, 214, 244, 255),
            anchor: AnchorPoint::TopCenter,
            bold: false,
            italic: false,
            data_id: None,
        };

        let node = SceneNode::new(format!("node-{}", gn.id))
            .with_primitive(circle)
            .with_primitive(label)
            .with_label(format!("{} {} ({})", gn.emoji, gn.display, gn.kind));
        scene.add_node(node, "nodes");
    }

    let legend = build_legend(width);
    scene.add_to_root(legend);

    scene
}

fn build_legend(width: f64) -> SceneNode {
    let kinds = [
        ("primal", "Primals"),
        ("spring", "Springs"),
        ("product", "Products"),
        ("composition", "Compositions"),
        ("concept", "Concepts"),
        ("infra", "Infrastructure"),
        ("org", "Organizations"),
    ];

    let mut legend = SceneNode::new("legend")
        .with_transform(Transform2D::translate(width - 140.0, 10.0))
        .with_label("Legend");

    for (i, (kind, label)) in kinds.iter().enumerate() {
        let y = i as f64 * 18.0 + 5.0;
        legend.primitives.push(Primitive::Point {
            x: 8.0,
            y: y + 4.0,
            radius: 5.0,
            fill: Some(kind_color(kind)),
            stroke: None,
            data_id: None,
        });
        legend.primitives.push(Primitive::Text {
            x: 20.0,
            y,
            content: (*label).to_owned(),
            font_size: 10.0,
            color: Color::from_rgba8(186, 194, 222, 255),
            anchor: AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: None,
        });
    }

    legend
}

/// Load entity graph from a JSON file path.
pub fn load_entity_graph(path: &std::path::Path) -> Option<EntityGraph> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test code")]
    use super::*;

    fn sample_graph() -> EntityGraph {
        EntityGraph {
            nodes: vec![
                GraphNode {
                    id: "beardog".into(),
                    display: "BearDog".into(),
                    kind: "primal".into(),
                    emoji: "🐻🐕".into(),
                },
                GraphNode {
                    id: "songbird".into(),
                    display: "SongBird".into(),
                    kind: "primal".into(),
                    emoji: "🐦".into(),
                },
                GraphNode {
                    id: "petaltongue".into(),
                    display: "PetalTongue".into(),
                    kind: "primal".into(),
                    emoji: "🌸👅".into(),
                },
            ],
            edges: vec![
                GraphEdge {
                    source: "beardog".into(),
                    target: "songbird".into(),
                    relation: "validates".into(),
                    inverse: false,
                },
                GraphEdge {
                    source: "songbird".into(),
                    target: "petaltongue".into(),
                    relation: "discovers".into(),
                    inverse: false,
                },
            ],
        }
    }

    #[test]
    fn build_scene_with_populated_graph() {
        let graph = sample_graph();
        let scene = build_entity_graph_scene(&graph);

        assert!(
            scene.node_count() >= 4,
            "scene should have root + edges + nodes groups + legend, got {} nodes",
            scene.node_count()
        );
        assert!(
            scene.total_primitives() > 0,
            "scene should contain primitives"
        );
    }

    #[test]
    fn force_layout_positions_all_nodes() {
        let graph = sample_graph();
        let positions = force_layout(&graph.nodes, &graph.edges, 800.0, 600.0);

        assert_eq!(positions.len(), 3);
        assert!(positions.contains_key("beardog"));
        assert!(positions.contains_key("songbird"));
        assert!(positions.contains_key("petaltongue"));

        for (x, y) in positions.values() {
            assert!(*x >= 0.0 && *x <= 800.0, "x out of bounds: {x}");
            assert!(*y >= 0.0 && *y <= 600.0, "y out of bounds: {y}");
        }
    }

    #[test]
    fn force_layout_single_node() {
        let graph = EntityGraph {
            nodes: vec![GraphNode {
                id: "solo".into(),
                display: "Solo".into(),
                kind: "infra".into(),
                emoji: "🏗️".into(),
            }],
            edges: vec![],
        };
        let positions = force_layout(&graph.nodes, &graph.edges, 800.0, 600.0);
        assert_eq!(positions.len(), 1);
    }

    #[test]
    fn kind_color_known_and_default() {
        let primal = kind_color("primal");
        let unknown = kind_color("nonexistent");
        assert_ne!(primal, unknown, "known kinds should differ from default");
    }

    #[test]
    fn relation_color_known_and_default() {
        let validates = relation_color("validates");
        let unknown = relation_color("nonexistent");
        assert_ne!(
            validates, unknown,
            "known relations should differ from default"
        );
    }

    #[test]
    fn inverse_edges_skipped_in_scene() {
        let graph = EntityGraph {
            nodes: vec![
                GraphNode {
                    id: "a".into(),
                    display: "A".into(),
                    kind: "primal".into(),
                    emoji: "🅰️".into(),
                },
                GraphNode {
                    id: "b".into(),
                    display: "B".into(),
                    kind: "primal".into(),
                    emoji: "🅱️".into(),
                },
            ],
            edges: vec![
                GraphEdge {
                    source: "a".into(),
                    target: "b".into(),
                    relation: "validates".into(),
                    inverse: false,
                },
                GraphEdge {
                    source: "b".into(),
                    target: "a".into(),
                    relation: "validated by".into(),
                    inverse: true,
                },
            ],
        };
        let scene = build_entity_graph_scene(&graph);
        let edges_node = scene.get("edges").expect("edges group exists");
        assert_eq!(
            edges_node.children.len(),
            1,
            "inverse edges should be skipped"
        );
    }

    #[test]
    fn load_entity_graph_nonexistent_returns_none() {
        assert!(load_entity_graph(std::path::Path::new("/nonexistent/graph.json")).is_none());
    }

    #[test]
    fn load_entity_graph_invalid_json_returns_none() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "not json").unwrap();
        assert!(load_entity_graph(temp.path()).is_none());
    }
}
