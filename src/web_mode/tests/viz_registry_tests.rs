// ── Viz data: registry + scene builders ────────────────────────────────

// SPDX-License-Identifier: AGPL-3.0-or-later

#[test]
fn test_viz_registry_discover_without_static() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let available = reg.available();
    assert!(available.contains(&"kderm-topology"), "should have kderm");
    assert!(
        available.contains(&"nucleus-composition"),
        "should have nucleus"
    );
    assert!(
        !available.contains(&"entity-graph"),
        "entity-graph needs static dir"
    );
}

#[test]
fn test_viz_registry_discover_with_entity_graph() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph_dir = tmp.path().join("graph");
    std::fs::create_dir(&graph_dir).unwrap();
    std::fs::write(
        graph_dir.join("entity-graph.json"),
        r#"{"nodes":[],"edges":[]}"#,
    )
    .unwrap();

    let reg = crate::viz_data::VizRegistry::discover(Some(tmp.path()));
    assert!(
        reg.available().contains(&"entity-graph"),
        "should discover entity-graph with JSON file"
    );
    assert_eq!(reg.list().len(), 4, "should have 4 visualizations total");
}

#[test]
fn test_viz_registry_get_and_list() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let entry = reg.get("kderm-topology").unwrap();
    assert_eq!(entry.slug, "kderm-topology");
    assert!(entry.has_animation);
    assert!(entry.data_source.is_none());

    let nucleus = reg.get("nucleus-composition").unwrap();
    assert_eq!(nucleus.title, "NUCLEUS Atomics Composition");
    assert!(nucleus.has_animation);

    assert!(reg.get("nonexistent").is_none());
}

#[test]
fn test_viz_registry_build_kderm_scene() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let scene = reg.build_scene("kderm-topology");
    assert!(scene.is_some(), "kderm scene should build");
    let scene = scene.unwrap();
    assert!(
        scene.node_count() > 1,
        "scene should have nodes beyond root"
    );
}

#[test]
fn test_viz_registry_build_nucleus_scene() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let scene = reg.build_scene("nucleus-composition");
    assert!(scene.is_some(), "nucleus scene should build");
    let scene = scene.unwrap();
    assert!(
        scene.node_count() > 1,
        "scene should have nodes beyond root"
    );
}

#[test]
fn test_viz_registry_build_unknown_returns_none() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    assert!(reg.build_scene("no-such-viz").is_none());
}

#[test]
fn test_viz_registry_build_kderm_animation() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let anim = reg.build_animation("kderm-topology");
    assert!(anim.is_some(), "kderm should have animation");
}

#[test]
fn test_viz_registry_build_nucleus_animation() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let anim = reg.build_animation("nucleus-composition");
    assert!(anim.is_some(), "nucleus should have animation");
}

#[test]
fn test_viz_registry_no_animation_for_entity_graph() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph_dir = tmp.path().join("graph");
    std::fs::create_dir(&graph_dir).unwrap();
    std::fs::write(
        graph_dir.join("entity-graph.json"),
        r#"{"nodes":[],"edges":[]}"#,
    )
    .unwrap();

    let reg = crate::viz_data::VizRegistry::discover(Some(tmp.path()));
    let anim = reg.build_animation("entity-graph");
    assert!(anim.is_none(), "entity-graph should not have animation");
}

#[test]
fn test_build_nucleus_scene_directly() {
    let scene = crate::viz_data::build_nucleus_scene();
    assert!(scene.node_count() > 1);
}

#[test]
fn test_build_nucleus_expand_animation() {
    let anim = crate::viz_data::build_nucleus_expand_animation("tower-atomic");
    match anim {
        petal_tongue_scene::animation::Sequence::Sequential(anims) => {
            assert!(!anims.is_empty(), "animation should have keyframes");
        }
        _ => panic!("expected Sequential"),
    }
}

#[test]
fn test_build_kderm_scene_directly() {
    let scene = crate::viz_data::build_kderm_scene();
    assert!(scene.node_count() > 1);
}

#[test]
fn test_build_kderm_relay_animation() {
    let anim = crate::viz_data::build_kderm_relay_animation();
    match anim {
        petal_tongue_scene::animation::Sequence::Sequential(anims) => {
            assert!(!anims.is_empty(), "animation should have keyframes");
        }
        _ => panic!("expected Sequential"),
    }
}

#[test]
fn test_build_gate_mesh_scene() {
    let scene = crate::viz_data::build_gate_mesh_scene();
    assert!(scene.node_count() > 10, "gate mesh should have many nodes");
}

#[test]
fn test_build_gate_mesh_enrollment_animation() {
    let anim = crate::viz_data::build_enrollment_animation();
    match anim {
        petal_tongue_scene::animation::Sequence::Sequential(anims) => {
            assert!(anims.len() >= 5, "should animate at least 5 gates");
        }
        _ => panic!("expected Sequential"),
    }
}

#[test]
fn test_viz_registry_build_gate_mesh_scene() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let scene = reg.build_scene("gate-mesh");
    assert!(scene.is_some(), "gate-mesh should build a scene");
}

#[test]
fn test_viz_registry_build_gate_mesh_animation() {
    let reg = crate::viz_data::VizRegistry::discover(None);
    let anim = reg.build_animation("gate-mesh");
    assert!(anim.is_some(), "gate-mesh should have an animation");
}
