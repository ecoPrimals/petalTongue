// SPDX-License-Identifier: AGPL-3.0-or-later
//! K-Derm diderm topology cross-section scene and relay animation.

use petal_tongue_scene::animation::{Animation, AnimationTarget, Easing, Sequence};
use petal_tongue_scene::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

struct KDermLayer {
    name: &'static str,
    label: &'static str,
    color: Color,
    y: f64,
    height: f64,
}

struct VpsNode {
    name: &'static str,
    layer: &'static str,
    x: f64,
    color: Color,
}

/// Build the K-Derm diderm cross-section scene.
#[expect(clippy::too_many_lines, reason = "single cohesive scene builder")]
pub fn build_kderm_scene() -> SceneGraph {
    let mut scene = SceneGraph::new();
    let width = 800.0;

    let layers = [
        KDermLayer { name: "extracellular", label: "Extracellular (Dark Forest)", color: Color::from_rgba8(49, 50, 68, 180), y: 10.0, height: 80.0 },
        KDermLayer { name: "outer", label: "Outer Membrane (Signal/Relay/Surface)", color: Color::from_rgba8(88, 91, 112, 200), y: 100.0, height: 80.0 },
        KDermLayer { name: "periplasm", label: "Periplasm (VPS relay, routing, telemetry)", color: Color::from_rgba8(108, 112, 134, 200), y: 190.0, height: 90.0 },
        KDermLayer { name: "plasma", label: "Plasma Membrane (Gate firewall)", color: Color::from_rgba8(137, 180, 250, 180), y: 290.0, height: 70.0 },
        KDermLayer { name: "cytoplasm", label: "Cytoplasm (Full NUCLEUS, 13 primals, UDS IPC)", color: Color::from_rgba8(166, 227, 161, 150), y: 370.0, height: 120.0 },
    ];

    let layers_group = SceneNode::new("layers").with_label("K-Derm membrane layers");
    scene.add_to_root(layers_group);

    for layer in &layers {
        let rect = Primitive::Rect {
            x: 30.0, y: layer.y, width: width - 60.0, height: layer.height,
            fill: Some(layer.color),
            stroke: Some(StrokeStyle { color: Color::from_rgba8(205, 214, 244, 100), width: 1.0, cap: LineCap::Butt, join: LineJoin::Miter }),
            corner_radius: 4.0, data_id: Some(layer.name.to_string()),
        };
        let label = Primitive::Text {
            x: 50.0, y: layer.y + 16.0, content: layer.label.to_string(),
            font_size: 12.0, color: Color::from_rgba8(205, 214, 244, 255),
            anchor: AnchorPoint::TopLeft, bold: true, italic: false, data_id: None,
        };
        let node = SceneNode::new(format!("layer-{}", layer.name))
            .with_primitive(rect).with_primitive(label).with_label(layer.label);
        scene.add_node(node, "layers");
    }

    let vps_nodes = [
        VpsNode { name: "golgiBody-ext", layer: "outer", x: 550.0, color: Color::from_rgba8(250, 179, 135, 255) },
        VpsNode { name: "peptidoglycan", layer: "periplasm", x: 400.0, color: Color::from_rgba8(249, 226, 175, 255) },
        VpsNode { name: "golgiBody", layer: "periplasm", x: 250.0, color: Color::from_rgba8(245, 194, 231, 255) },
        VpsNode { name: "eastGate", layer: "cytoplasm", x: 200.0, color: Color::from_rgba8(166, 227, 161, 255) },
        VpsNode { name: "flockGate", layer: "cytoplasm", x: 400.0, color: Color::from_rgba8(166, 227, 161, 255) },
        VpsNode { name: "GitHub", layer: "extracellular", x: 600.0, color: Color::from_rgba8(186, 194, 222, 200) },
    ];

    let vps_group = SceneNode::new("vps-nodes").with_label("Physical nodes");
    scene.add_to_root(vps_group);

    for vps in &vps_nodes {
        let layer_y = layers.iter().find(|l| l.name == vps.layer).map_or(300.0, |l| l.y + l.height / 2.0);
        let circle = Primitive::Point {
            x: vps.x, y: layer_y, radius: 14.0, fill: Some(vps.color),
            stroke: Some(StrokeStyle { color: Color::from_rgba8(30, 30, 46, 255), width: 2.0, cap: LineCap::Round, join: LineJoin::Round }),
            data_id: Some(vps.name.to_string()),
        };
        let label = Primitive::Text {
            x: vps.x, y: layer_y + 22.0, content: vps.name.to_string(),
            font_size: 10.0, color: Color::from_rgba8(205, 214, 244, 255),
            anchor: AnchorPoint::TopCenter, bold: true, italic: false, data_id: None,
        };
        let node = SceneNode::new(format!("vps-{}", vps.name))
            .with_primitive(circle).with_primitive(label).with_label(vps.name);
        scene.add_node(node, "vps-nodes");
    }

    let relay_group = SceneNode::new("relay-path").with_label("Git push relay chain");
    scene.add_to_root(relay_group);

    let relay_steps: Vec<(&str, f64, f64, &str, f64, f64)> = vec![
        ("flockGate", 400.0, 430.0, "golgiBody", 250.0, 235.0),
        ("golgiBody", 250.0, 235.0, "peptidoglycan", 400.0, 235.0),
        ("peptidoglycan", 400.0, 235.0, "golgiBody-ext", 550.0, 140.0),
        ("golgiBody-ext", 550.0, 140.0, "GitHub", 600.0, 50.0),
    ];

    for (i, (from, fx, fy, to, tx, ty)) in relay_steps.iter().enumerate() {
        let arrow = Primitive::Line {
            points: vec![[*fx, *fy], [*tx, *ty]],
            stroke: StrokeStyle { color: Color::from_rgba8(243, 139, 168, 220), width: 2.5, cap: LineCap::Round, join: LineJoin::Round },
            closed: false, data_id: Some(format!("relay-{i}")),
        };
        let step_label = Primitive::Text {
            x: (fx + tx) / 2.0, y: (fy + ty) / 2.0 - 8.0,
            content: format!("{from} → {to}"), font_size: 8.0,
            color: Color::from_rgba8(243, 139, 168, 200),
            anchor: AnchorPoint::BottomCenter, bold: false, italic: true, data_id: None,
        };
        let step_node = SceneNode::new(format!("relay-step-{i}"))
            .with_primitive(arrow).with_primitive(step_label)
            .with_label(format!("Relay step {}: {from} → {to}", i + 1));
        scene.add_node(step_node, "relay-path");
    }

    let bonds_group = SceneNode::new("bonds").with_label("Bond types");
    scene.add_to_root(bonds_group);

    let bond_labels = [
        ("Covalent (local)", 700.0, 430.0, Color::from_rgba8(166, 227, 161, 255)),
        ("Metallic (relay)", 700.0, 235.0, Color::from_rgba8(249, 226, 175, 255)),
        ("Ionic (cloud)", 700.0, 140.0, Color::from_rgba8(137, 180, 250, 255)),
        ("Weak (public)", 700.0, 50.0, Color::from_rgba8(186, 194, 222, 200)),
    ];

    for (label, x, y, color) in &bond_labels {
        let text = Primitive::Text {
            x: *x, y: *y, content: label.to_string(), font_size: 9.0, color: *color,
            anchor: AnchorPoint::CenterLeft, bold: false, italic: false, data_id: None,
        };
        let node = SceneNode::new(format!("bond-{}", label.split(' ').next().unwrap_or("x")))
            .with_primitive(text);
        scene.add_node(node, "bonds");
    }

    let title = SceneNode::new("title").with_primitive(Primitive::Text {
        x: width / 2.0, y: 580.0,
        content: "K-Derm Diderm Architecture — Sovereign Infrastructure Topology".to_string(),
        font_size: 14.0, color: Color::from_rgba8(205, 214, 244, 255),
        anchor: AnchorPoint::BottomCenter, bold: true, italic: false, data_id: None,
    });
    scene.add_to_root(title);

    scene
}

/// Build the relay animation: flockGate -> golgiBody -> peptidoglycan -> golgiBody-ext -> GitHub.
pub fn build_kderm_relay_animation() -> Sequence {
    let steps = [
        ("relay-step-0", "vps-flockGate", "vps-golgiBody"),
        ("relay-step-1", "vps-golgiBody", "vps-peptidoglycan"),
        ("relay-step-2", "vps-peptidoglycan", "vps-golgiBody-ext"),
        ("relay-step-3", "vps-golgiBody-ext", "vps-GitHub"),
    ];

    let mut animations = Vec::new();

    for (i, (relay_id, from_node, to_node)) in steps.iter().enumerate() {
        let delay = i as f64 * 1.2;

        animations.push(Animation {
            target: AnimationTarget::Scale { node_id: from_node.to_string(), from: 1.0, to: 1.3 },
            duration_secs: 0.3, easing: Easing::EaseOut, delay_secs: delay,
        });
        animations.push(Animation {
            target: AnimationTarget::StrokeDraw { node_id: relay_id.to_string() },
            duration_secs: 0.6, easing: Easing::EaseInOut, delay_secs: delay + 0.2,
        });
        animations.push(Animation {
            target: AnimationTarget::Opacity { node_id: relay_id.to_string(), from: 0.0, to: 1.0 },
            duration_secs: 0.5, easing: Easing::EaseOut, delay_secs: delay + 0.1,
        });
        animations.push(Animation {
            target: AnimationTarget::Scale { node_id: to_node.to_string(), from: 1.0, to: 1.3 },
            duration_secs: 0.3, easing: Easing::EaseOut, delay_secs: delay + 0.7,
        });
        animations.push(Animation {
            target: AnimationTarget::Scale { node_id: from_node.to_string(), from: 1.3, to: 1.0 },
            duration_secs: 0.2, easing: Easing::EaseIn, delay_secs: delay + 0.8,
        });
    }

    animations.push(Animation {
        target: AnimationTarget::Scale { node_id: "vps-GitHub".to_string(), from: 1.3, to: 1.0 },
        duration_secs: 0.3, easing: Easing::EaseIn, delay_secs: 5.0,
    });

    Sequence::Sequential(animations)
}
