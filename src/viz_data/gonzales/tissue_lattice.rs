// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tissue lattice scene (spatial cell-state viability grid).

use petal_tongue_scene::primitive::{
    AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle,
};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

/// Build the tissue lattice scene (cell viability heat map).
///
/// Renders a 12×12 lattice where each cell's color encodes viability state
/// (green=viable, yellow=stressed, red=apoptotic). Simulates spatial
/// heterogeneity in drug response across a tissue section.
#[expect(clippy::too_many_lines, reason = "single cohesive scene builder")]
pub fn build_tissue_lattice_scene() -> SceneGraph {
    let mut scene = SceneGraph::new();

    let grid_size = 12_usize;
    let cell_size = 28.0;
    let gap = 3.0;
    let offset_x = 100.0;
    let offset_y = 70.0;

    // Simulate viability pattern: radial gradient from a drug diffusion source
    // at top-left corner (higher drug concentration → lower viability).
    #[expect(clippy::cast_precision_loss, reason = "grid_size is 12, fits f64")]
    let max_dist = (grid_size as f64).hypot(grid_size as f64);

    let mut lattice_node = SceneNode::new("tissue-grid").with_label("Tissue Lattice");

    #[expect(
        clippy::cast_precision_loss,
        reason = "grid indices: f64 sufficient for 12×12"
    )]
    for row in 0..grid_size {
        for col in 0..grid_size {
            let dist = (row as f64).hypot(col as f64);
            let viability = (dist / max_dist).clamp(0.0, 1.0);

            // Add biological noise (deterministic pseudo-noise from position)
            let noise = ((row * 7 + col * 13) % 17) as f64 / 170.0;
            let v = (viability + noise).clamp(0.0, 1.0);

            let color = viability_color(v);
            let x = (col as f64).mul_add(cell_size + gap, offset_x);
            let y = (row as f64).mul_add(cell_size + gap, offset_y);

            lattice_node = lattice_node.with_primitive(Primitive::Rect {
                x,
                y,
                width: cell_size,
                height: cell_size,
                fill: Some(color),
                stroke: Some(StrokeStyle {
                    color: Color::from_rgba8(69, 71, 90, 200),
                    width: 0.5,
                    cap: LineCap::Butt,
                    join: LineJoin::Miter,
                }),
                corner_radius: 3.0,
                data_id: Some(format!("cell-{row}-{col}")),
            });
        }
    }
    scene.add_to_root(lattice_node);

    // Color scale legend
    let mut legend_node = SceneNode::new("tissue-legend").with_label("Viability Scale");
    let legend_x = 520.0;
    let legend_height = 200.0;
    let legend_steps = 20_usize;
    #[expect(clippy::cast_precision_loss, reason = "legend steps: f64 sufficient")]
    for i in 0..legend_steps {
        let t = i as f64 / (legend_steps - 1) as f64;
        let y = 100.0 + t * legend_height;
        let color = viability_color(1.0 - t);
        legend_node = legend_node.with_primitive(Primitive::Rect {
            x: legend_x,
            y,
            width: 20.0,
            height: legend_height / legend_steps as f64 + 1.0,
            fill: Some(color),
            stroke: None,
            corner_radius: 0.0,
            data_id: None,
        });
    }
    // Scale labels
    legend_node = legend_node.with_primitive(Primitive::Text {
        x: legend_x + 25.0,
        y: 100.0,
        content: "100% viable".to_owned(),
        font_size: 10.0,
        color: Color::from_rgba8(166, 173, 200, 255),
        anchor: AnchorPoint::CenterLeft,
        bold: false,
        italic: false,
        data_id: None,
    });
    legend_node = legend_node.with_primitive(Primitive::Text {
        x: legend_x + 25.0,
        y: 100.0 + legend_height,
        content: "0% (apoptotic)".to_owned(),
        font_size: 10.0,
        color: Color::from_rgba8(166, 173, 200, 255),
        anchor: AnchorPoint::CenterLeft,
        bold: false,
        italic: false,
        data_id: None,
    });
    scene.add_to_root(legend_node);

    // Drug source indicator
    let source_node = SceneNode::new("tissue-source")
        .with_label("Drug Source")
        .with_primitive(Primitive::Text {
            x: offset_x - 10.0,
            y: offset_y - 15.0,
            content: "\u{2197} Drug source".to_owned(),
            font_size: 10.0,
            color: Color::from_rgba8(243, 139, 168, 255),
            anchor: AnchorPoint::BottomLeft,
            bold: true,
            italic: false,
            data_id: None,
        });
    scene.add_to_root(source_node);

    // Title
    let title_node = SceneNode::new("tissue-title")
        .with_label("Title")
        .with_primitive(Primitive::Text {
            x: 330.0,
            y: 20.0,
            content: "Tissue Lattice \u{2014} Spatial Drug Response".to_owned(),
            font_size: 16.0,
            color: Color::from_rgba8(205, 214, 244, 255),
            anchor: AnchorPoint::TopCenter,
            bold: true,
            italic: false,
            data_id: None,
        });
    scene.add_to_root(title_node);

    scene
}

/// Map viability (0.0 = dead, 1.0 = fully viable) to a color.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "color channels computed in f64 then clamped to [0, 255]"
)]
pub(super) fn viability_color(v: f64) -> Color {
    if v > 0.6 {
        // Green — viable
        let t = (v - 0.6) / 0.4;
        let g = (80.0 + t * 127.0).clamp(0.0, 255.0) as u8;
        let b = (40.0 + t * 200.0).clamp(0.0, 255.0) as u8;
        Color::from_rgba8(40, g, b, 230)
    } else if v > 0.3 {
        // Yellow — stressed
        let t = (v - 0.3) / 0.3;
        let g = (140.0 + t * 60.0).clamp(0.0, 255.0) as u8;
        Color::from_rgba8(200, g, 30, 230)
    } else {
        // Red — apoptotic
        let t = v / 0.3;
        let r = (220.0 - t * 40.0).clamp(0.0, 255.0) as u8;
        let g = (30.0 + t * 30.0).clamp(0.0, 255.0) as u8;
        Color::from_rgba8(r, g, 30, 230)
    }
}
