// SPDX-License-Identifier: AGPL-3.0-or-later
//! Serialization roundtrips, proptest property tests, and remaining scene graph coverage.

use proptest::prelude::*;

use crate::primitive::{Color, Primitive};
use crate::transform::Transform2D;

use super::{SceneGraph, SceneNode};

const EPS: f64 = 1e-10;

#[test]
fn scene_node_default_properties() {
    let node = SceneNode::new("id");
    assert_eq!(node.id.as_str(), "id");
    assert_eq!(node.transform, Transform2D::IDENTITY);
    assert!(node.primitives.is_empty());
    assert!(node.children.is_empty());
    assert!(node.visible);
    assert!((node.opacity - 1.0).abs() < f32::EPSILON);
    assert!(node.label.is_none());
    assert!(node.data_source.is_none());
}

#[test]
fn scene_node_serialization_roundtrip() {
    let node = SceneNode::new("test_node")
        .with_label("Test Label")
        .with_opacity(0.7)
        .with_visible(true)
        .with_primitive(Primitive::Point {
            x: 1.0,
            y: 2.0,
            radius: 3.0,
            fill: Some(Color::WHITE),
            stroke: None,
            data_id: Some("pt-1".to_string()),
        });
    let json = serde_json::to_string(&node).expect("serialization should succeed");
    let decoded: SceneNode = serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(node.id, decoded.id);
    assert_eq!(node.label, decoded.label);
    assert!((node.opacity - decoded.opacity).abs() < f32::EPSILON);
    assert_eq!(node.visible, decoded.visible);
    assert_eq!(node.primitives.len(), decoded.primitives.len());
}

#[test]
fn scene_graph_serialization_roundtrip() {
    let mut g = SceneGraph::new();
    g.add_to_root(
        SceneNode::new("child1")
            .with_label("Child")
            .with_primitive(Primitive::Point {
                x: 0.0,
                y: 0.0,
                radius: 1.0,
                fill: None,
                stroke: None,
                data_id: None,
            }),
    );
    let json = serde_json::to_string(&g).expect("serialization should succeed");
    let decoded: SceneGraph = serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(g.root_id(), decoded.root_id());
    assert_eq!(g.node_count(), decoded.node_count());
    assert!(decoded.get("child1").is_some());
}

#[test]
fn scene_node_with_data_source() {
    let mut node = SceneNode::new("n");
    node.data_source = Some("ds-1".to_string());
    assert_eq!(node.data_source.as_deref(), Some("ds-1"));
}

#[test]
fn scene_node_debug_formatting() {
    let node = SceneNode::new("debug_node");
    let s = format!("{node:?}");
    assert!(s.contains("debug_node"));
}

#[test]
fn scene_graph_debug_formatting() {
    let g = SceneGraph::new();
    let s = format!("{g:?}");
    assert!(!s.is_empty());
}

#[test]
fn scene_node_clone() {
    let node = SceneNode::new("orig").with_opacity(0.5);
    let cloned = node.clone();
    assert_eq!(node.id, cloned.id);
    assert!((node.opacity - cloned.opacity).abs() < f32::EPSILON);
}

#[test]
fn scene_graph_clone() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("a"));
    let cloned = g.clone();
    assert_eq!(g.node_count(), cloned.node_count());
    assert!(cloned.get("a").is_some());
}

#[test]
fn flatten_node_with_multiple_primitives() {
    let mut g = SceneGraph::new();
    let prim = Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: None,
    };
    g.add_to_root(
        SceneNode::new("multi")
            .with_primitive(prim.clone())
            .with_primitive(prim.clone())
            .with_primitive(prim),
    );
    let flat = g.flatten();
    assert_eq!(flat.len(), 3);
}

#[test]
fn add_node_to_root_multiple_times() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("a"));
    g.add_to_root(SceneNode::new("b"));
    g.add_to_root(SceneNode::new("c"));
    assert_eq!(g.node_count(), 4);
    let root = g.get("root").expect("root exists");
    assert_eq!(root.children.len(), 3);
}

#[test]
fn empty_graph_has_only_root() {
    let g = SceneGraph::new();
    assert_eq!(g.node_count(), 1);
    assert_eq!(g.root_id(), "root");
    let flat = g.flatten();
    assert!(flat.is_empty(), "root has no primitives");
}

#[test]
fn deeply_nested_hierarchy() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("l1"));
    g.add_node(SceneNode::new("l2"), "l1");
    g.add_node(SceneNode::new("l3"), "l2");
    g.add_node(SceneNode::new("l4"), "l3");
    g.add_node(SceneNode::new("l5"), "l4");
    assert_eq!(g.node_count(), 6);
    let l5 = g.get("l5").expect("l5 exists");
    assert!(l5.children.is_empty());
    let l1 = g.get("l1").expect("l1 exists");
    assert_eq!(l1.children.len(), 1);
    assert_eq!(l1.children[0].as_str(), "l2");
}

#[test]
fn circular_reference_prevention_flat_storage() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("a"));
    g.add_node(SceneNode::new("b"), "a");
    let root = g.get_mut("root").expect("root");
    root.children.push("a".into());
    let flat = g.flatten();
    assert!(
        flat.len() <= 2,
        "flatten traverses children; phantom refs to a from root don't create cycles in flat storage"
    );
}

#[test]
fn remove_deep_branch() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("p"));
    g.add_node(SceneNode::new("c1"), "p");
    g.add_node(SceneNode::new("c2"), "c1");
    g.add_node(SceneNode::new("c3"), "c2");
    assert_eq!(g.node_count(), 5);
    let removed = g.remove("c1");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id.as_str(), "c1");
    assert_eq!(g.node_count(), 2);
    assert!(g.get("c1").is_none());
    assert!(g.get("c2").is_none());
    assert!(g.get("c3").is_none());
}

#[test]
fn add_node_overwrites_existing_same_id() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("dup").with_opacity(0.5).with_label("first"));
    g.add_to_root(SceneNode::new("dup").with_opacity(0.9).with_label("second"));
    let node = g.get("dup").expect("dup exists");
    assert!((node.opacity - 0.9).abs() < f32::EPSILON);
    assert_eq!(node.label.as_deref(), Some("second"));
    let root = g.get("root").expect("root");
    assert_eq!(
        root.children.iter().filter(|c| c.as_str() == "dup").count(),
        2
    );
}

#[test]
fn flatten_order_depth_first() {
    let mut g = SceneGraph::new();
    let prim = Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: None,
    };
    g.add_to_root(
        SceneNode::new("a")
            .with_transform(Transform2D::translate(1.0, 0.0))
            .with_primitive(prim.clone()),
    );
    g.add_node(
        SceneNode::new("b")
            .with_transform(Transform2D::translate(2.0, 0.0))
            .with_primitive(prim),
        "a",
    );
    let flat = g.flatten();
    assert_eq!(flat.len(), 2);
    assert!((flat[0].0.tx - 1.0).abs() < EPS);
    assert!((flat[1].0.tx - 3.0).abs() < EPS);
}

#[test]
fn node_ids_no_duplicates_in_iteration() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("x"));
    let ids: Vec<_> = g.node_ids().collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"root"));
    assert!(ids.contains(&"x"));
}

// === Proptest ===

#[test]
fn prop_add_n_nodes_gives_n_plus_one_total() {
    fn prop(n: u8) -> Result<(), TestCaseError> {
        let n = n as usize;
        let mut g = SceneGraph::new();
        for i in 0..n {
            let id = format!("node-{i}");
            g.add_to_root(SceneNode::new(id));
        }
        prop_assert_eq!(
            g.node_count(),
            1 + n,
            "root + {} added = {} total",
            n,
            1 + n
        );
        Ok(())
    }
    proptest!(|(n in 0u8..50u8)| prop(n)?);
}

#[test]
fn prop_remove_node_removes_from_children() {
    fn prop(removed_id: String) -> Result<(), TestCaseError> {
        let removed_id = if removed_id.is_empty() || removed_id == "root" {
            "to_remove".to_string()
        } else {
            removed_id
        };
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("parent"));
        g.add_node(SceneNode::new(removed_id.clone()), "parent");
        g.add_node(SceneNode::new("sibling"), "parent");
        let _ = g.remove(&removed_id);
        for node in g.node_ids() {
            let n = g.get(node).unwrap();
            prop_assert!(
                !n.children.iter().any(|c| c.as_str() == removed_id),
                "removed node {:?} must not appear in any children list",
                removed_id
            );
        }
        Ok(())
    }
    proptest!(|(removed_id in "\\w{1,20}")| prop(removed_id)?);
}

#[test]
fn prop_scene_graph_serialization_roundtrip() {
    fn prop(node_ids: Vec<String>) -> Result<(), TestCaseError> {
        let mut g = SceneGraph::new();
        for (i, id) in node_ids.into_iter().enumerate().take(20) {
            let id = if id.is_empty() || id == "root" {
                format!("n{i}")
            } else {
                id
            };
            g.add_to_root(SceneNode::new(id));
        }
        let json = serde_json::to_string(&g).map_err(|_| TestCaseError::reject("serialize"))?;
        let decoded: SceneGraph =
            serde_json::from_str(&json).map_err(|_| TestCaseError::reject("deserialize"))?;
        prop_assert_eq!(g.root_id(), decoded.root_id());
        prop_assert_eq!(g.node_count(), decoded.node_count());
        for id in g.node_ids() {
            prop_assert!(
                decoded.get(id).is_some(),
                "node {} must exist after roundtrip",
                id
            );
        }
        Ok(())
    }
    proptest!(|(node_ids in prop::collection::vec("[a-zA-Z0-9_]{0,10}", 0..25))| prop(node_ids)?);
}
