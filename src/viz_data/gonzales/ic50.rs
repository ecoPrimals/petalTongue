// SPDX-License-Identifier: AGPL-3.0-or-later
//! IC50 dose-response scene (4-parameter logistic Hill equation).

use petal_tongue_scene::math::{Axes, FunctionPlot, MathObject};
use petal_tongue_scene::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

/// Hill equation parameters for IC50 visualization.
pub(super) struct HillParams {
    pub top: f64,
    pub bottom: f64,
    pub ic50: f64,
    pub hill_coeff: f64,
}

impl HillParams {
    /// Evaluate the 4PL Hill equation: response at a given log-concentration.
    pub fn response(&self, log_conc: f64) -> f64 {
        let x = 10.0_f64.powf(log_conc);
        self.bottom + (self.top - self.bottom) / (1.0 + (x / self.ic50).powf(self.hill_coeff))
    }
}

/// Build the IC50 dose-response scene (sigmoidal Hill curve).
///
/// X-axis: log10\[Inhibitor\] (M), Y-axis: % Response (0–100).
/// Shows the characteristic sigmoidal transition with IC50 annotation.
#[expect(clippy::too_many_lines, reason = "single cohesive scene builder")]
pub fn build_ic50_scene() -> SceneGraph {
    let mut scene = SceneGraph::new();

    let params = HillParams {
        top: 100.0,
        bottom: 5.0,
        ic50: 1e-7, // 100 nM
        hill_coeff: 1.2,
    };

    let axes = Axes {
        x_range: (-10.0, -4.0, 1.0),
        y_range: (0.0, 110.0, 20.0),
        origin: (80.0, 380.0),
        width: 500.0,
        height: 320.0,
        color: Color::from_rgba8(30, 30, 46, 255),
        show_labels: true,
        label_font_size: 11.0,
    };

    // Axes node
    let axes_prims = axes.to_primitives();
    let mut axes_node = SceneNode::new("ic50-axes").with_label("IC50 Axes");
    for p in axes_prims {
        axes_node = axes_node.with_primitive(p);
    }
    scene.add_to_root(axes_node);

    // Dose-response curve
    let curve_stroke = StrokeStyle {
        color: Color::from_rgba8(137, 180, 250, 255),
        width: 2.5,
        cap: LineCap::Round,
        join: LineJoin::Round,
    };
    let plot = FunctionPlot::sample(&axes, |x| params.response(x), curve_stroke);
    let curve_prims = plot.to_primitives();
    let mut curve_node = SceneNode::new("ic50-curve").with_label("Dose-Response Curve");
    for p in curve_prims {
        curve_node = curve_node.with_primitive(p);
    }
    scene.add_to_root(curve_node);

    // IC50 annotation: horizontal dashed line at 50% response
    let mid_response = f64::midpoint(params.top, params.bottom);
    let (sx_left, sy_mid) = axes.data_to_screen(-10.0, mid_response);
    let (sx_ic50, _) = axes.data_to_screen(params.ic50.log10(), mid_response);
    let dashed_stroke = StrokeStyle {
        color: Color::from_rgba8(166, 173, 200, 180),
        width: 1.0,
        cap: LineCap::Butt,
        join: LineJoin::Miter,
    };
    let mut anno_node = SceneNode::new("ic50-annotation").with_label("IC50 Marker");
    anno_node = anno_node.with_primitive(Primitive::Line {
        points: vec![[sx_left, sy_mid], [sx_ic50, sy_mid]],
        stroke: dashed_stroke,
        closed: false,
        data_id: Some("ic50-hline".to_owned()),
    });
    // Vertical line down from IC50 point
    let (_, sy_bottom) = axes.data_to_screen(params.ic50.log10(), 0.0);
    anno_node = anno_node.with_primitive(Primitive::Line {
        points: vec![[sx_ic50, sy_mid], [sx_ic50, sy_bottom]],
        stroke: dashed_stroke,
        closed: false,
        data_id: Some("ic50-vline".to_owned()),
    });
    // IC50 label
    anno_node = anno_node.with_primitive(Primitive::Text {
        x: sx_ic50 + 8.0,
        y: sy_mid - 12.0,
        content: format!("IC\u{2085}\u{2080} = {:.0} nM", params.ic50 * 1e9),
        font_size: 12.0,
        color: Color::from_rgba8(245, 194, 231, 255),
        anchor: AnchorPoint::BottomLeft,
        bold: true,
        italic: false,
        data_id: None,
    });
    scene.add_to_root(anno_node);

    // Title
    let title_node = SceneNode::new("ic50-title")
        .with_label("Title")
        .with_primitive(Primitive::Text {
            x: 330.0,
            y: 20.0,
            content: "IC\u{2085}\u{2080} Dose-Response (Hill Equation)".to_owned(),
            font_size: 16.0,
            color: Color::from_rgba8(205, 214, 244, 255),
            anchor: AnchorPoint::TopCenter,
            bold: true,
            italic: false,
            data_id: None,
        });
    scene.add_to_root(title_node);

    // Axis labels
    let mut labels_node = SceneNode::new("ic50-labels").with_label("Axis Labels");
    labels_node = labels_node.with_primitive(Primitive::Text {
        x: 330.0,
        y: 400.0,
        content: "log\u{2081}\u{2080}[Inhibitor] (M)".to_owned(),
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
        content: "% Response".to_owned(),
        font_size: 12.0,
        color: Color::from_rgba8(166, 173, 200, 255),
        anchor: AnchorPoint::CenterLeft,
        bold: false,
        italic: false,
        data_id: None,
    });
    scene.add_to_root(labels_node);

    scene
}
