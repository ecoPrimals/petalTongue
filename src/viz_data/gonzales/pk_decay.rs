// SPDX-License-Identifier: AGPL-3.0-or-later
//! PK decay scene (two-compartment pharmacokinetic elimination).

use petal_tongue_scene::math::{Axes, FunctionPlot, MathObject};
use petal_tongue_scene::primitive::{
    AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle,
};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

/// Two-compartment PK parameters.
pub(super) struct PkParams {
    pub a_coeff: f64,
    pub alpha: f64,
    pub b_coeff: f64,
    pub beta: f64,
}

impl PkParams {
    /// C(t) = A·e^(-α·t) + B·e^(-β·t)
    pub fn concentration(&self, t: f64) -> f64 {
        self.a_coeff.mul_add(
            (-self.alpha * t).exp(),
            self.b_coeff * (-self.beta * t).exp(),
        )
    }
}

/// Build the PK decay scene (two-compartment elimination).
///
/// X-axis: Time (hours), Y-axis: Plasma concentration (ng/mL).
/// Shows rapid distribution phase (α) and slower elimination phase (β).
#[expect(clippy::too_many_lines, reason = "single cohesive scene builder")]
pub fn build_pk_decay_scene() -> SceneGraph {
    let mut scene = SceneGraph::new();

    let params = PkParams {
        a_coeff: 80.0, // ng/mL (distribution amplitude)
        alpha: 1.5,    // h⁻¹ (distribution rate)
        b_coeff: 20.0, // ng/mL (elimination amplitude)
        beta: 0.15,    // h⁻¹ (elimination rate, t½ ≈ 4.6h)
    };

    let axes = Axes {
        x_range: (0.0, 24.0, 4.0),
        y_range: (0.0, 110.0, 20.0),
        origin: (80.0, 380.0),
        width: 500.0,
        height: 320.0,
        color: Color::from_rgba8(30, 30, 46, 255),
        show_labels: true,
        label_font_size: 11.0,
    };

    // Axes
    let axes_prims = axes.to_primitives();
    let mut axes_node = SceneNode::new("pk-axes").with_label("PK Axes");
    for p in axes_prims {
        axes_node = axes_node.with_primitive(p);
    }
    scene.add_to_root(axes_node);

    // Total concentration curve
    let total_stroke = StrokeStyle {
        color: Color::from_rgba8(148, 226, 213, 255),
        width: 2.5,
        cap: LineCap::Round,
        join: LineJoin::Round,
    };
    let total_plot = FunctionPlot::sample(&axes, |t| params.concentration(t), total_stroke);
    let mut total_node = SceneNode::new("pk-total").with_label("Total Concentration");
    for p in total_plot.to_primitives() {
        total_node = total_node.with_primitive(p);
    }
    scene.add_to_root(total_node);

    // Distribution phase (α component) — fainter
    let alpha_stroke = StrokeStyle {
        color: Color::from_rgba8(137, 180, 250, 120),
        width: 1.5,
        cap: LineCap::Round,
        join: LineJoin::Round,
    };
    let alpha_plot = FunctionPlot::sample(
        &axes,
        |t| params.a_coeff * (-params.alpha * t).exp(),
        alpha_stroke,
    );
    let mut alpha_node = SceneNode::new("pk-alpha").with_label("α Phase (Distribution)");
    for p in alpha_plot.to_primitives() {
        alpha_node = alpha_node.with_primitive(p);
    }
    scene.add_to_root(alpha_node);

    // Elimination phase (β component) — fainter
    let beta_stroke = StrokeStyle {
        color: Color::from_rgba8(245, 194, 231, 120),
        width: 1.5,
        cap: LineCap::Round,
        join: LineJoin::Round,
    };
    let beta_plot = FunctionPlot::sample(
        &axes,
        |t| params.b_coeff * (-params.beta * t).exp(),
        beta_stroke,
    );
    let mut beta_node = SceneNode::new("pk-beta").with_label("β Phase (Elimination)");
    for p in beta_plot.to_primitives() {
        beta_node = beta_node.with_primitive(p);
    }
    scene.add_to_root(beta_node);

    // Half-life annotation
    let t_half_beta = 0.693 / params.beta;
    let c_at_half = params.concentration(t_half_beta);
    let (half_screen_x, half_screen_y) = axes.data_to_screen(t_half_beta, c_at_half);
    let mut anno_node = SceneNode::new("pk-annotation").with_label("Half-Life Marker");
    anno_node = anno_node.with_primitive(Primitive::Point {
        x: half_screen_x,
        y: half_screen_y,
        radius: 5.0,
        fill: Some(Color::from_rgba8(249, 226, 175, 255)),
        stroke: None,
        data_id: Some("pk-t-half".to_owned()),
    });
    anno_node = anno_node.with_primitive(Primitive::Text {
        x: half_screen_x + 10.0,
        y: half_screen_y - 8.0,
        content: format!("t\u{00BD}\u{03B2} = {t_half_beta:.1}h"),
        font_size: 11.0,
        color: Color::from_rgba8(249, 226, 175, 255),
        anchor: AnchorPoint::BottomLeft,
        bold: true,
        italic: false,
        data_id: None,
    });
    scene.add_to_root(anno_node);

    // Title
    let title_node = SceneNode::new("pk-title")
        .with_label("Title")
        .with_primitive(Primitive::Text {
            x: 330.0,
            y: 20.0,
            content: "Two-Compartment PK Decay".to_owned(),
            font_size: 16.0,
            color: Color::from_rgba8(205, 214, 244, 255),
            anchor: AnchorPoint::TopCenter,
            bold: true,
            italic: false,
            data_id: None,
        });
    scene.add_to_root(title_node);

    // Axis labels
    let mut labels_node = SceneNode::new("pk-labels").with_label("Axis Labels");
    labels_node = labels_node.with_primitive(Primitive::Text {
        x: 330.0,
        y: 400.0,
        content: "Time (hours)".to_owned(),
        font_size: 12.0,
        color: Color::from_rgba8(166, 173, 200, 255),
        anchor: AnchorPoint::TopCenter,
        bold: false,
        italic: false,
        data_id: None,
    });
    labels_node = labels_node.with_primitive(Primitive::Text {
        x: 20.0,
        y: 220.0,
        content: "C (ng/mL)".to_owned(),
        font_size: 12.0,
        color: Color::from_rgba8(166, 173, 200, 255),
        anchor: AnchorPoint::CenterLeft,
        bold: false,
        italic: false,
        data_id: None,
    });
    scene.add_to_root(labels_node);

    // Legend
    let mut legend_node = SceneNode::new("pk-legend").with_label("Legend");
    let legend_entries = [
        ("C(t) total", Color::from_rgba8(148, 226, 213, 255)),
        ("α (distribution)", Color::from_rgba8(137, 180, 250, 180)),
        ("β (elimination)", Color::from_rgba8(245, 194, 231, 180)),
    ];
    #[expect(clippy::cast_precision_loss, reason = "legend index: f64 sufficient")]
    for (i, (label, color)) in legend_entries.iter().enumerate() {
        let ly = (i as f64).mul_add(18.0, 50.0);
        legend_node = legend_node.with_primitive(Primitive::Line {
            points: vec![[490.0, ly], [510.0, ly]],
            stroke: StrokeStyle {
                color: *color,
                width: 2.0,
                cap: LineCap::Round,
                join: LineJoin::Miter,
            },
            closed: false,
            data_id: None,
        });
        legend_node = legend_node.with_primitive(Primitive::Text {
            x: 515.0,
            y: ly,
            content: (*label).to_owned(),
            font_size: 10.0,
            color: Color::from_rgba8(166, 173, 200, 255),
            anchor: AnchorPoint::CenterLeft,
            bold: false,
            italic: false,
            data_id: None,
        });
    }
    scene.add_to_root(legend_node);

    scene
}
