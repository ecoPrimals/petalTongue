// SPDX-License-Identifier: AGPL-3.0-or-later
//! Core scene graph unit tests (construction, graph ops, flatten, traversal, transforms).

use crate::primitive::Primitive;
use crate::transform::Transform2D;

use super::{SceneGraph, SceneNode};

const EPS: f64 = 1e-10;

#[test]
fn new_creates_root() {
    let g = SceneGraph::new();
    assert_eq!(g.root_id(), "root");
    assert_eq!(g.node_count(), 1);
    assert!(g.get("root").is_some());
}

#[test]
fn add_to_root_and_get() {
    let mut g = SceneGraph::new();
    let node = SceneNode::new("child1");
    g.add_to_root(node);
    assert_eq!(g.node_count(), 2);
    let child = g.get("child1").expect("child1 should exist");
    assert_eq!(child.id.as_str(), "child1");
    assert!(
        g.get("root")
            .expect("root should exist")
            .children
            .iter()
            .any(|c| c.as_str() == "child1"),
        "root should have child1"
    );
}

#[test]
fn add_node_with_parent() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("parent"));
    let child = SceneNode::new("child");
    let ok = g.add_node(child, "parent");
    assert!(ok);
    assert_eq!(g.node_count(), 3);
    assert!(
        g.get("parent")
            .expect("parent should exist")
            .children
            .iter()
            .any(|c| c.as_str() == "child"),
        "parent should have child"
    );
}

#[test]
fn add_node_with_invalid_parent_returns_false() {
    let mut g = SceneGraph::new();
    let node = SceneNode::new("orphan");
    let ok = g.add_node(node, "nonexistent");
    assert!(!ok);
    assert_eq!(g.node_count(), 1);
}

#[test]
fn remove_node_and_descendants() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("a"));
    g.add_node(SceneNode::new("b"), "a");
    g.add_node(SceneNode::new("c"), "b");
    assert_eq!(g.node_count(), 4);
    let removed = g.remove("a");
    assert!(removed.is_some());
    assert_eq!(g.node_count(), 1);
    assert!(g.get("a").is_none());
    assert!(g.get("b").is_none());
    assert!(g.get("c").is_none());
}

#[test]
fn cannot_remove_root() {
    let mut g = SceneGraph::new();
    let removed = g.remove("root");
    assert!(removed.is_none());
    assert_eq!(g.node_count(), 1);
}

#[test]
fn node_count() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("a"));
    g.add_to_root(SceneNode::new("b"));
    assert_eq!(g.node_count(), 3);
}

#[test]
fn total_primitives() {
    let mut g = SceneGraph::new();
    let prim = Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: None,
    };
    g.add_to_root(SceneNode::new("a").with_primitive(prim.clone()));
    g.add_to_root(
        SceneNode::new("b")
            .with_primitive(prim.clone())
            .with_primitive(prim),
    );
    assert_eq!(g.total_primitives(), 3);
}

#[test]
fn flatten_produces_correct_transforms() {
    let mut g = SceneGraph::new();
    let t_a = Transform2D::translate(10.0, 20.0);
    let t_b = Transform2D::translate(5.0, 0.0);
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
            .with_transform(t_a)
            .with_primitive(prim.clone()),
    );
    g.add_node(
        SceneNode::new("b").with_transform(t_b).with_primitive(prim),
        "a",
    );
    let flat = g.flatten();
    assert_eq!(flat.len(), 2);
    assert!((flat[0].0.tx - 10.0).abs() < EPS);
    assert!((flat[0].0.ty - 20.0).abs() < EPS);
    let composed = t_a.then(t_b);
    assert!((flat[1].0.tx - composed.tx).abs() < EPS);
    assert!((flat[1].0.ty - composed.ty).abs() < EPS);
}

#[test]
fn find_by_data_id() {
    let mut g = SceneGraph::new();
    let prim_foo = Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: Some("foo".to_string()),
    };
    let prim_bar = Primitive::Point {
        x: 1.0,
        y: 1.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: Some("bar".to_string()),
    };
    let prim_foo2 = Primitive::Point {
        x: 2.0,
        y: 2.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: Some("foo".to_string()),
    };
    g.add_to_root(SceneNode::new("a").with_primitive(prim_foo));
    g.add_to_root(SceneNode::new("b").with_primitive(prim_bar));
    g.add_to_root(SceneNode::new("c").with_primitive(prim_foo2));
    let found = g.find_by_data_id("foo");
    assert_eq!(found.len(), 2);
    let found_bar = g.find_by_data_id("bar");
    assert_eq!(found_bar.len(), 1);
    let found_none = g.find_by_data_id("baz");
    assert_eq!(found_none.len(), 0);
}

#[test]
fn flatten_skips_invisible_nodes() {
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
        SceneNode::new("visible")
            .with_visible(true)
            .with_primitive(prim.clone()),
    );
    g.add_to_root(
        SceneNode::new("hidden")
            .with_visible(false)
            .with_primitive(prim),
    );
    let flat = g.flatten();
    assert_eq!(flat.len(), 1);
}

#[test]
fn node_ids_iteration() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("a"));
    g.add_to_root(SceneNode::new("b"));
    let ids: Vec<_> = g.node_ids().collect();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"root"));
    assert!(ids.contains(&"a"));
    assert!(ids.contains(&"b"));
}

#[test]
fn remove_returns_removed_node() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("to_remove"));
    let removed = g.remove("to_remove");
    assert!(removed.is_some());
    assert_eq!(
        removed.expect("removed node should exist").id.as_str(),
        "to_remove"
    );
    assert!(g.get("to_remove").is_none());
}

#[test]
fn add_node_preserves_children_order() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("p"));
    g.add_node(SceneNode::new("c1"), "p");
    g.add_node(SceneNode::new("c2"), "p");
    let parent = g.get("p").expect("parent p should exist");
    assert_eq!(parent.children.len(), 2);
    assert_eq!(parent.children[0].as_str(), "c1");
    assert_eq!(parent.children[1].as_str(), "c2");
}

#[test]
fn scene_node_builder_chain() {
    let node = SceneNode::new("id")
        .with_label("test")
        .with_opacity(0.5)
        .with_visible(false);
    assert_eq!(node.id.as_str(), "id");
    assert_eq!(node.label.as_deref(), Some("test"));
    assert!((node.opacity - 0.5).abs() < f32::EPSILON);
    assert!(!node.visible);
}

#[test]
fn scene_node_primitive_count() {
    let prim = Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: None,
    };
    let node = SceneNode::new("n")
        .with_primitive(prim.clone())
        .with_primitive(prim);
    assert_eq!(node.primitive_count(), 2);
}

#[test]
fn flatten_empty_primitives() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("empty"));
    let flat = g.flatten();
    assert_eq!(flat.len(), 0);
}

#[test]
fn default_scene_graph() {
    let g = SceneGraph::default();
    assert_eq!(g.root_id(), "root");
    assert_eq!(g.node_count(), 1);
}

#[test]
fn scene_node_with_transform() {
    let t = Transform2D::translate(50.0, 100.0);
    let node = SceneNode::new("n").with_transform(t);
    assert!((node.transform.tx - 50.0).abs() < EPS);
    assert!((node.transform.ty - 100.0).abs() < EPS);
}

#[test]
fn flatten_missing_child_node_skipped() {
    let mut g = SceneGraph::new();
    let prim = Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: None,
    };
    let root = g.get_mut("root").expect("root exists");
    root.children.push("phantom".into());
    root.primitives.push(prim);
    let flat = g.flatten();
    assert_eq!(
        flat.len(),
        1,
        "phantom child does not exist, only root primitives"
    );
}

#[test]
fn remove_leaf_node() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("parent"));
    g.add_node(SceneNode::new("leaf"), "parent");
    assert_eq!(g.node_count(), 3);
    let removed = g.remove("leaf");
    assert!(removed.is_some());
    assert_eq!(g.node_count(), 2);
    assert!(g.get("leaf").is_none());
    let parent = g.get("parent").expect("parent exists");
    assert!(!parent.children.iter().any(|c| c.as_str() == "leaf"));
}

#[test]
fn get_mut_modifies_node() {
    let mut g = SceneGraph::new();
    g.add_to_root(SceneNode::new("n").with_opacity(0.5));
    {
        let node = g.get_mut("n").expect("n exists");
        node.opacity = 0.8;
    }
    let node = g.get("n").expect("n exists");
    assert!((node.opacity - 0.8).abs() < f32::EPSILON);
}

#[test]
fn get_nonexistent_returns_none() {
    let mut g = SceneGraph::new();
    assert!(g.get("nonexistent").is_none());
    assert!(g.get_mut("nonexistent").is_none());
}

#[test]
fn transform_composition_in_hierarchy() {
    let mut g = SceneGraph::new();
    let t1 = Transform2D::translate(10.0, 0.0);
    let t2 = Transform2D::scale(2.0, 2.0);
    let prim = Primitive::Point {
        x: 1.0,
        y: 1.0,
        radius: 1.0,
        fill: None,
        stroke: None,
        data_id: None,
    };
    g.add_to_root(
        SceneNode::new("a")
            .with_transform(t1)
            .with_primitive(prim.clone()),
    );
    g.add_node(
        SceneNode::new("b").with_transform(t2).with_primitive(prim),
        "a",
    );
    let flat = g.flatten();
    assert_eq!(flat.len(), 2, "should have 2 primitives from a and b");
    let (t_a, _) = &flat[0];
    let (t_ab, _) = &flat[1];
    let (wx1, wy1) = t_a.apply(1.0, 1.0);
    let (wx2, wy2) = t_ab.apply(1.0, 1.0);
    assert!(
        (wx1 - 11.0).abs() < EPS,
        "a: translate(10,0) + (1,1) = (11,1)"
    );
    assert!((wy1 - 1.0).abs() < EPS);
    assert!(
        (wx2 - 12.0).abs() < EPS,
        "a->b: composed transform scales then translates"
    );
    assert!((wy2 - 2.0).abs() < EPS);
}

#[test]
fn find_by_data_id_empty_graph() {
    let g = SceneGraph::new();
    let found = g.find_by_data_id("any");
    assert!(found.is_empty());
}
