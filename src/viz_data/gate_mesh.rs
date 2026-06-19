// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gate mesh topology scene — WireGuard overlay, enrollment status, NUCLEUS health.
//!
//! Consumes shared topology data from `petal_tongue_core::gate_mesh` and renders
//! it as a scene graph with enrollment color coding and WireGuard link overlays.

use petal_tongue_core::gate_mesh::{self, GateEnrollment, MeshNode};
use petal_tongue_scene::animation::{Animation, AnimationTarget, Easing, Sequence};
use petal_tongue_scene::primitive::{
    AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle,
};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

fn enrollment_color(status: GateEnrollment) -> Color {
    match status {
        GateEnrollment::Enrolled => Color::from_rgba8(166, 227, 161, 255),
        GateEnrollment::MeshLive => Color::from_rgba8(137, 180, 250, 255),
        GateEnrollment::Sovereign => Color::from_rgba8(249, 226, 175, 255),
        GateEnrollment::Public => Color::from_rgba8(250, 179, 135, 255),
        GateEnrollment::Offline => Color::from_rgba8(88, 91, 112, 180),
    }
}

const fn enrollment_label(status: GateEnrollment) -> &'static str {
    match status {
        GateEnrollment::Enrolled => "Enrolled (13/13)",
        GateEnrollment::MeshLive => "Mesh Live",
        GateEnrollment::Sovereign => "Sovereign Relay",
        GateEnrollment::Public => "Public Relay",
        GateEnrollment::Offline => "Offline",
    }
}

/// Build the gate mesh topology scene.
#[expect(clippy::too_many_lines, reason = "single cohesive scene builder")]
pub fn build_gate_mesh_scene() -> SceneGraph {
    let mut scene = SceneGraph::new();
    let width = 900.0;
    let height = 600.0;

    // Background
    let bg = Primitive::Rect {
        x: 0.0,
        y: 0.0,
        width,
        height,
        fill: Some(Color::from_rgba8(30, 30, 46, 255)),
        stroke: None,
        corner_radius: 8.0,
        data_id: None,
    };
    scene.add_to_root(SceneNode::new("background").with_primitive(bg));

    // Zone regions
    let zones_group = SceneNode::new("zones").with_label("Cytoplasm zones");
    scene.add_to_root(zones_group);

    for (zone_id, zone_label, zx, zy, zw, zh) in [
        (
            "zone-backbone",
            "Backbone (Hub 1)",
            120.0,
            60.0,
            680.0,
            220.0,
        ),
        ("zone-wan", "WAN (Offsite)", 80.0, 350.0, 200.0, 130.0),
        ("zone-house2", "House 2 (Hub 2)", 330.0, 390.0, 480.0, 170.0),
    ] {
        let zone_rect = Primitive::Rect {
            x: zx,
            y: zy,
            width: zw,
            height: zh,
            fill: Some(Color::from_rgba8(49, 50, 68, 80)),
            stroke: Some(StrokeStyle {
                color: Color::from_rgba8(108, 112, 134, 120),
                width: 1.0,
                cap: LineCap::Butt,
                join: LineJoin::Miter,
            }),
            corner_radius: 12.0,
            data_id: Some(zone_id.to_owned()),
        };
        let zone_lbl = Primitive::Text {
            x: zx + 10.0,
            y: zy + 18.0,
            content: zone_label.to_owned(),
            font_size: 12.0,
            color: Color::from_rgba8(147, 153, 178, 200),
            anchor: AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: None,
        };
        let node = SceneNode::new(zone_id)
            .with_primitive(zone_rect)
            .with_primitive(zone_lbl)
            .with_label(zone_label);
        scene.add_node(node, "zones");
    }

    // WireGuard links (behind nodes)
    let links_group = SceneNode::new("wg-links").with_label("WireGuard overlay");
    scene.add_to_root(links_group);

    let all_nodes: Vec<&MeshNode> = gate_mesh::all_nodes().collect();

    for link in gate_mesh::WG_LINKS {
        let from_node = all_nodes.iter().find(|n| n.id == link.from);
        let to_node = all_nodes.iter().find(|n| n.id == link.to);
        if let (Some(from), Some(to)) = (from_node, to_node) {
            let line = Primitive::Line {
                points: vec![[from.x, from.y], [to.x, to.y]],
                stroke: StrokeStyle {
                    color: Color::from_rgba8(137, 180, 250, 80),
                    width: 1.5,
                    cap: LineCap::Round,
                    join: LineJoin::Round,
                },
                closed: false,
                data_id: Some(format!("wg-{}-{}", link.from, link.to)),
            };

            let mid_x = f64::midpoint(from.x, to.x);
            let mid_y = f64::midpoint(from.y, to.y);
            let latency_lbl = Primitive::Text {
                x: mid_x,
                y: mid_y - 6.0,
                content: format!("{}ms", link.latency_ms),
                font_size: 9.0,
                color: Color::from_rgba8(147, 153, 178, 160),
                anchor: AnchorPoint::BottomCenter,
                bold: false,
                italic: false,
                data_id: None,
            };

            let link_id = format!("wg-{}-{}", link.from, link.to);
            let link_node = SceneNode::new(link_id)
                .with_primitive(line)
                .with_primitive(latency_lbl);
            scene.add_node(link_node, "wg-links");
        }
    }

    // Gate nodes
    let gates_group = SceneNode::new("gates").with_label("Gate nodes");
    scene.add_to_root(gates_group);

    for gate in &all_nodes {
        let radius = if gate.nucleus_count == 13 { 22.0 } else { 16.0 };

        let circle = Primitive::Point {
            x: gate.x,
            y: gate.y,
            radius,
            fill: Some(enrollment_color(gate.enrollment)),
            stroke: Some(StrokeStyle {
                color: Color::from_rgba8(205, 214, 244, 180),
                width: if gate.nucleus_count == 13 { 2.5 } else { 1.5 },
                cap: LineCap::Round,
                join: LineJoin::Round,
            }),
            data_id: Some(gate.id.to_owned()),
        };

        let name_lbl = Primitive::Text {
            x: gate.x,
            y: gate.y + radius + 14.0,
            content: gate.label.to_owned(),
            font_size: 11.0,
            color: Color::from_rgba8(205, 214, 244, 220),
            anchor: AnchorPoint::TopCenter,
            bold: true,
            italic: false,
            data_id: None,
        };

        let mut node = SceneNode::new(gate.id)
            .with_primitive(circle)
            .with_primitive(name_lbl)
            .with_label(gate.label);

        if let Some(ip) = gate.wg_ip {
            node = node.with_primitive(Primitive::Text {
                x: gate.x,
                y: gate.y + radius + 26.0,
                content: ip.to_owned(),
                font_size: 9.0,
                color: Color::from_rgba8(137, 180, 250, 160),
                anchor: AnchorPoint::TopCenter,
                bold: false,
                italic: false,
                data_id: None,
            });
        }

        if gate.nucleus_count > 0 {
            node = node.with_primitive(Primitive::Text {
                x: gate.x,
                y: gate.y + 4.0,
                content: format!("{}/13", gate.nucleus_count),
                font_size: 10.0,
                color: Color::from_rgba8(30, 30, 46, 255),
                anchor: AnchorPoint::TopCenter,
                bold: true,
                italic: false,
                data_id: None,
            });
        }

        scene.add_node(node, "gates");
    }

    // Title
    let title = Primitive::Text {
        x: width / 2.0,
        y: 25.0,
        content: "Gate Mesh Topology — WireGuard Overlay".to_owned(),
        font_size: 16.0,
        color: Color::from_rgba8(205, 214, 244, 255),
        anchor: AnchorPoint::TopCenter,
        bold: true,
        italic: false,
        data_id: None,
    };
    scene.add_to_root(SceneNode::new("title").with_primitive(title));

    // Legend
    let legend_group = SceneNode::new("legend").with_label("Enrollment legend");
    scene.add_to_root(legend_group);

    let statuses = [
        GateEnrollment::Enrolled,
        GateEnrollment::MeshLive,
        GateEnrollment::Sovereign,
        GateEnrollment::Public,
        GateEnrollment::Offline,
    ];

    for (i, status) in statuses.iter().enumerate() {
        let lx = 30.0;
        #[expect(clippy::cast_precision_loss)]
        let ly = height - 100.0 + (i as f64) * 18.0;
        let dot = Primitive::Point {
            x: lx,
            y: ly,
            radius: 5.0,
            fill: Some(enrollment_color(*status)),
            stroke: None,
            data_id: None,
        };
        let lbl = Primitive::Text {
            x: lx + 12.0,
            y: ly + 3.0,
            content: enrollment_label(*status).to_owned(),
            font_size: 10.0,
            color: Color::from_rgba8(186, 194, 222, 200),
            anchor: AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: None,
        };
        let legend_node = SceneNode::new(format!("legend-{i}"))
            .with_primitive(dot)
            .with_primitive(lbl);
        scene.add_node(legend_node, "legend");
    }

    scene
}

/// Build an enrollment progression animation.
pub fn build_enrollment_animation() -> Sequence {
    let targets: Vec<&str> = gate_mesh::GATES
        .iter()
        .filter(|g| !matches!(g.enrollment, GateEnrollment::Enrolled))
        .map(|g| g.id)
        .collect();

    let mut animations = Vec::new();

    for (i, gate_id) in targets.iter().enumerate() {
        let delay = i as f64 * 1.5;

        animations.push(Animation {
            target: AnimationTarget::Opacity {
                node_id: (*gate_id).to_owned(),
                from: 0.4,
                to: 1.0,
            },
            duration_secs: 1.0,
            easing: Easing::EaseInOut,
            delay_secs: delay,
        });
        animations.push(Animation {
            target: AnimationTarget::Scale {
                node_id: (*gate_id).to_owned(),
                from: 1.0,
                to: 1.3,
            },
            duration_secs: 0.4,
            easing: Easing::EaseOut,
            delay_secs: delay + 0.5,
        });
        animations.push(Animation {
            target: AnimationTarget::Scale {
                node_id: (*gate_id).to_owned(),
                from: 1.3,
                to: 1.0,
            },
            duration_secs: 0.3,
            easing: Easing::EaseIn,
            delay_secs: delay + 0.9,
        });
    }

    Sequence::Sequential(animations)
}
