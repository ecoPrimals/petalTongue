// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hormesis scene (biphasic dose-response).

use petal_tongue_scene::math::{Axes, FunctionPlot, MathObject};
use petal_tongue_scene::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

/// Hormesis model parameters.
pub(super) struct HormesisParams {
    pub baseline: f64,
    pub stimulation_peak: f64,
    pub peak_dose: f64,
    pub inhibition_ic50: f64,
    pub hill_n: f64,
}

impl HormesisParams {
    /// Biphasic response: stimulation at low dose, inhibition at high dose.
    /// Model: `baseline + stim_peak * (x/peak) * e^(1 - x/peak) - inhibition(x)`
    pub fn response(&self, log_conc: f64) -> f64 {
        let x = 10.0_f64.powf(log_conc);
        let ratio = x / self.peak_dose;

        // Stimulation component (log-normal shaped peak)
        let stim = self.stimulation_peak * ratio * (1.0 - ratio).exp();

        // Inhibition component (Hill equation)
        let inhibition = self.baseline / (1.0 + (self.inhibition_ic50 / x).powf(self.hill_n));

        (self.baseline + stim - inhibition).max(0.0)
    }
}

/// Build the hormesis scene (biphasic dose-response).
///
/// X-axis: log10\[Dose\] (M), Y-axis: % Response relative to control.
/// Shows characteristic J-shaped curve: stimulation at low doses,
/// inhibition at high doses.
#[expect(clippy::too_many_lines, reason = "single cohesive scene builder")]
pub fn build_hormesis_scene() -> SceneGraph {
    let mut scene = SceneGraph::new();

    let params = HormesisParams {
        baseline: 100.0,
        stimulation_peak: 25.0,
        peak_dose: 1e-8,       // 10 nM peak stimulation
        inhibition_ic50: 1e-5, // 10 µM inhibition
        hill_n: 1.5,
    };

    let axes = Axes {
        x_range: (-11.0, -3.0, 1.0),
        y_range: (0.0, 140.0, 20.0),
        origin: (80.0, 380.0),
        width: 500.0,
        height: 320.0,
        color: Color::from_rgba8(30, 30, 46, 255),
        show_labels: true,
        label_font_size: 11.0,
    };

    // Axes
    let axes_prims = axes.to_primitives();
    let mut axes_node = SceneNode::new("hormesis-axes").with_label("Hormesis Axes");
    for p in axes_prims {
        axes_node = axes_node.with_primitive(p);
    }
    scene.add_to_root(axes_node);

    // Baseline reference (100% dashed line)
    let (sx_left, sy_base) = axes.data_to_screen(-11.0, 100.0);
    let (sx_right, _) = axes.data_to_screen(-3.0, 100.0);
    let mut baseline_node = SceneNode::new("hormesis-baseline").with_label("Baseline");
    baseline_node = baseline_node.with_primitive(Primitive::Line {
        points: vec![[sx_left, sy_base], [sx_right, sy_base]],
        stroke: StrokeStyle {
            color: Color::from_rgba8(108, 112, 134, 150),
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        },
        closed: false,
        data_id: Some("baseline-100".to_owned()),
    });
    baseline_node = baseline_node.with_primitive(Primitive::Text {
        x: sx_right + 5.0,
        y: sy_base,
        content: "control".to_owned(),
        font_size: 9.0,
        color: Color::from_rgba8(108, 112, 134, 200),
        anchor: AnchorPoint::CenterLeft,
        bold: false,
        italic: true,
        data_id: None,
    });
    scene.add_to_root(baseline_node);

    // Hormesis curve
    let curve_stroke = StrokeStyle {
        color: Color::from_rgba8(166, 227, 161, 255),
        width: 2.5,
        cap: LineCap::Round,
        join: LineJoin::Round,
    };
    let plot = FunctionPlot::sample(&axes, |x| params.response(x), curve_stroke);
    let mut curve_node = SceneNode::new("hormesis-curve").with_label("Hormesis Response");
    for p in plot.to_primitives() {
        curve_node = curve_node.with_primitive(p);
    }
    scene.add_to_root(curve_node);

    // Stimulation zone annotation
    let (peak_screen_x, peak_screen_y) = axes.data_to_screen(params.peak_dose.log10(), 120.0);
    let mut zone_node = SceneNode::new("hormesis-zones").with_label("Response Zones");
    zone_node = zone_node.with_primitive(Primitive::Text {
        x: peak_screen_x,
        y: peak_screen_y - 25.0,
        content: "stimulation".to_owned(),
        font_size: 10.0,
        color: Color::from_rgba8(166, 227, 161, 200),
        anchor: AnchorPoint::BottomCenter,
        bold: false,
        italic: true,
        data_id: None,
    });
    // Inhibition zone
    let (sx_inhib, _) = axes.data_to_screen(params.inhibition_ic50.log10(), 50.0);
    zone_node = zone_node.with_primitive(Primitive::Text {
        x: sx_inhib,
        y: 380.0 - 50.0,
        content: "inhibition".to_owned(),
        font_size: 10.0,
        color: Color::from_rgba8(243, 139, 168, 200),
        anchor: AnchorPoint::TopCenter,
        bold: false,
        italic: true,
        data_id: None,
    });
    scene.add_to_root(zone_node);

    // Title
    let title_node = SceneNode::new("hormesis-title")
        .with_label("Title")
        .with_primitive(Primitive::Text {
            x: 330.0,
            y: 20.0,
            content: "Hormesis \u{2014} Biphasic Dose-Response".to_owned(),
            font_size: 16.0,
            color: Color::from_rgba8(205, 214, 244, 255),
            anchor: AnchorPoint::TopCenter,
            bold: true,
            italic: false,
            data_id: None,
        });
    scene.add_to_root(title_node);

    // Axis labels
    let mut labels_node = SceneNode::new("hormesis-labels").with_label("Axis Labels");
    labels_node = labels_node.with_primitive(Primitive::Text {
        x: 330.0,
        y: 400.0,
        content: "log\u{2081}\u{2080}[Dose] (M)".to_owned(),
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
