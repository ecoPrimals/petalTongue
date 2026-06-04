// SPDX-License-Identifier: AGPL-3.0-or-later
//! NUCLEUS atomics composition scene and expand/collapse animation.

use petal_tongue_scene::animation::{Animation, AnimationTarget, Easing, Sequence};
use petal_tongue_scene::primitive::{
    AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle,
};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

struct NucleusLayer {
    id: &'static str,
    label: &'static str,
    components: &'static [&'static str],
    color: Color,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Build the NUCLEUS atomics composition scene.
#[expect(clippy::too_many_lines, reason = "single cohesive scene builder")]
pub fn build_nucleus_scene() -> SceneGraph {
    let mut scene = SceneGraph::new();

    let layers = [
        NucleusLayer {
            id: "full-nucleus",
            label: "Full NUCLEUS",
            components: &[
                "+ AI inference",
                "+ Orchestration layer",
                "+ Universal User Interface",
            ],
            color: Color::from_rgba8(203, 166, 247, 120),
            x: 50.0,
            y: 30.0,
            width: 700.0,
            height: 540.0,
        },
        NucleusLayer {
            id: "nest-atomic",
            label: "Nest Atomic",
            components: &["+ Content storage", "+ Provenance Trio"],
            color: Color::from_rgba8(148, 226, 213, 140),
            x: 80.0,
            y: 80.0,
            width: 640.0,
            height: 420.0,
        },
        NucleusLayer {
            id: "node-atomic",
            label: "Node Atomic",
            components: &["+ Compute engine", "+ GPU math pipeline", "+ GPU compiler"],
            color: Color::from_rgba8(249, 226, 175, 140),
            x: 110.0,
            y: 130.0,
            width: 580.0,
            height: 310.0,
        },
        NucleusLayer {
            id: "tower-atomic",
            label: "Tower Atomic",
            components: &[
                "Cryptographic security",
                "Network mesh",
                "= Pure Rust HTTPS",
            ],
            color: Color::from_rgba8(166, 227, 161, 160),
            x: 140.0,
            y: 180.0,
            width: 520.0,
            height: 200.0,
        },
    ];

    let composition_group = SceneNode::new("composition").with_label("NUCLEUS composition layers");
    scene.add_to_root(composition_group);

    for layer in &layers {
        let rect = Primitive::Rect {
            x: layer.x,
            y: layer.y,
            width: layer.width,
            height: layer.height,
            fill: Some(layer.color),
            stroke: Some(StrokeStyle {
                color: Color::from_rgba8(205, 214, 244, 180),
                width: 2.0,
                cap: LineCap::Butt,
                join: LineJoin::Miter,
            }),
            corner_radius: 8.0,
            data_id: Some(layer.id.to_owned()),
        };
        let title = Primitive::Text {
            x: layer.x + 15.0,
            y: layer.y + 18.0,
            content: layer.label.to_owned(),
            font_size: 14.0,
            color: Color::from_rgba8(205, 214, 244, 255),
            anchor: AnchorPoint::TopLeft,
            bold: true,
            italic: false,
            data_id: None,
        };

        let mut node = SceneNode::new(layer.id)
            .with_primitive(rect)
            .with_primitive(title)
            .with_label(layer.label);

        for (ci, comp) in layer.components.iter().enumerate() {
            node.primitives.push(Primitive::Text {
                x: layer.x + layer.width - 15.0,
                y: layer.y + 18.0 + (ci as f64 * 16.0),
                content: (*comp).to_owned(),
                font_size: 11.0,
                color: Color::from_rgba8(186, 194, 222, 220),
                anchor: AnchorPoint::TopRight,
                bold: false,
                italic: false,
                data_id: None,
            });
        }

        scene.add_node(node, "composition");
    }

    let tls_group = SceneNode::new("tls-detail").with_label("TLS 1.3 handshake");
    scene.add_to_root(tls_group);

    let tls_steps = [
        "1. ClientHello (TLS provider)",
        "2. ServerHello + Certificate (X.509)",
        "3. Key Exchange (X25519)",
        "4. Finished (HMAC verify)",
        "5. Application Data (encrypted channel)",
    ];

    for (i, step) in tls_steps.iter().enumerate() {
        let text = Primitive::Text {
            x: 180.0,
            y: 220.0 + (i as f64 * 28.0),
            content: (*step).to_owned(),
            font_size: 11.0,
            color: Color::from_rgba8(166, 227, 161, 240),
            anchor: AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: Some(format!("tls-step-{i}")),
        };
        let step_node = SceneNode::new(format!("tls-{i}")).with_primitive(text);
        scene.add_node(step_node, "tls-detail");
    }

    let flow_label = SceneNode::new("flow-label")
        .with_primitive(Primitive::Text {
            x: 400.0,
            y: 575.0,
            content: "Each layer adds capabilities. Tower is the minimum viable secure unit."
                .to_owned(),
            font_size: 11.0,
            color: Color::from_rgba8(186, 194, 222, 200),
            anchor: AnchorPoint::BottomCenter,
            bold: false,
            italic: true,
            data_id: None,
        })
        .with_label("Composition principle");
    scene.add_to_root(flow_label);

    scene
}

/// Cascading expand/collapse animation for NUCLEUS composition layers.
pub fn build_nucleus_expand_animation(layer_id: &str) -> Sequence {
    let layers = ["tower-atomic", "node-atomic", "nest-atomic", "full-nucleus"];
    let target_idx = layers.iter().position(|&l| l == layer_id).unwrap_or(0);

    let mut animations = Vec::new();

    for (i, &lid) in layers[..=target_idx].iter().rev().enumerate() {
        let delay = i as f64 * 0.3;
        animations.push(Animation {
            target: AnimationTarget::Scale {
                node_id: lid.to_owned(),
                from: 1.0,
                to: 1.03,
            },
            duration_secs: 0.4,
            easing: Easing::EaseInOut,
            delay_secs: delay,
        });
        animations.push(Animation {
            target: AnimationTarget::Opacity {
                node_id: lid.to_owned(),
                from: 0.6,
                to: 1.0,
            },
            duration_secs: 0.3,
            easing: Easing::EaseOut,
            delay_secs: delay,
        });
    }

    for (i, &lid) in layers[..=target_idx].iter().rev().enumerate() {
        let delay = (layers.len() as f64 * 0.3) + (i as f64 * 0.15);
        animations.push(Animation {
            target: AnimationTarget::Scale {
                node_id: lid.to_owned(),
                from: 1.03,
                to: 1.0,
            },
            duration_secs: 0.3,
            easing: Easing::EaseIn,
            delay_secs: delay,
        });
    }

    Sequence::Sequential(animations)
}
