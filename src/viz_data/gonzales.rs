// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gonzales Interactive Explorer chart scenes.
//!
//! Four pharmacological visualization scenes:
//! - **IC50**: Sigmoidal dose-response (4-parameter logistic Hill equation)
//! - **PK decay**: Two-compartment pharmacokinetic elimination
//! - **Tissue lattice**: Spatial cell-state grid (viability heat map)
//! - **Hormesis**: Biphasic dose-response (low-dose stimulation, high-dose inhibition)
//!
//! Each scene uses the petal-tongue-scene math layer (`Axes`, `FunctionPlot`)
//! and renders to a `SceneGraph` compatible with all modalities.

use petal_tongue_scene::animation::{Animation, AnimationTarget, Easing, Sequence};
use petal_tongue_scene::math::{Axes, FunctionPlot, MathObject};
use petal_tongue_scene::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

// ── IC50: 4-Parameter Logistic (Hill Equation) ─────────────────────────────

/// Hill equation parameters for IC50 visualization.
struct HillParams {
    top: f64,
    bottom: f64,
    ic50: f64,
    hill_coeff: f64,
}

impl HillParams {
    /// Evaluate the 4PL Hill equation: response at a given log-concentration.
    fn response(&self, log_conc: f64) -> f64 {
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

// ── PK Decay: Two-Compartment Pharmacokinetic Model ─────────────────────────

/// Two-compartment PK parameters.
struct PkParams {
    a_coeff: f64,
    alpha: f64,
    b_coeff: f64,
    beta: f64,
}

impl PkParams {
    /// C(t) = A·e^(-α·t) + B·e^(-β·t)
    fn concentration(&self, t: f64) -> f64 {
        self.a_coeff
            .mul_add((-self.alpha * t).exp(), self.b_coeff * (-self.beta * t).exp())
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
        a_coeff: 80.0,  // ng/mL (distribution amplitude)
        alpha: 1.5,     // h⁻¹ (distribution rate)
        b_coeff: 20.0,  // ng/mL (elimination amplitude)
        beta: 0.15,     // h⁻¹ (elimination rate, t½ ≈ 4.6h)
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

// ── Tissue Lattice: Spatial Cell-State Viability Grid ────────────────────────

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
    #[expect(
        clippy::cast_precision_loss,
        reason = "legend steps: f64 sufficient"
    )]
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
            content: "Tissue Lattice — Spatial Drug Response".to_owned(),
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
fn viability_color(v: f64) -> Color {
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

// ── Hormesis: Biphasic Dose-Response ────────────────────────────────────────

/// Hormesis model parameters.
struct HormesisParams {
    baseline: f64,
    stimulation_peak: f64,
    peak_dose: f64,
    inhibition_ic50: f64,
    hill_n: f64,
}

impl HormesisParams {
    /// Biphasic response: stimulation at low dose, inhibition at high dose.
    /// Model: `baseline + stim_peak * (x/peak) * e^(1 - x/peak) - inhibition(x)`
    fn response(&self, log_conc: f64) -> f64 {
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
        peak_dose: 1e-8,    // 10 nM peak stimulation
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
            content: "Hormesis — Biphasic Dose-Response".to_owned(),
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

// ── Animations ──────────────────────────────────────────────────────────────

/// Build a dose-sweep animation for IC50 (traces the curve left to right).
pub fn build_ic50_sweep_animation() -> Sequence {
    Sequence::Sequential(vec![Animation {
        target: AnimationTarget::Opacity {
            node_id: "ic50-curve".to_owned(),
            from: 0.0,
            to: 1.0,
        },
        duration_secs: 2.0,
        delay_secs: 0.5,
        easing: Easing::EaseInOut,
    }])
}

/// Build a time-lapse animation for PK decay (reveals concentration drop).
pub fn build_pk_decay_animation() -> Sequence {
    Sequence::Sequential(vec![
        Animation {
            target: AnimationTarget::Opacity {
                node_id: "pk-total".to_owned(),
                from: 0.0,
                to: 1.0,
            },
            duration_secs: 1.5,
            delay_secs: 0.0,
            easing: Easing::EaseOut,
        },
        Animation {
            target: AnimationTarget::Opacity {
                node_id: "pk-alpha".to_owned(),
                from: 0.0,
                to: 1.0,
            },
            duration_secs: 1.0,
            delay_secs: 0.3,
            easing: Easing::EaseIn,
        },
        Animation {
            target: AnimationTarget::Opacity {
                node_id: "pk-beta".to_owned(),
                from: 0.0,
                to: 1.0,
            },
            duration_secs: 1.0,
            delay_secs: 0.3,
            easing: Easing::EaseIn,
        },
    ])
}

/// Build a drug-diffusion animation for the tissue lattice.
pub fn build_tissue_diffusion_animation() -> Sequence {
    Sequence::Sequential(vec![Animation {
        target: AnimationTarget::Opacity {
            node_id: "tissue-grid".to_owned(),
            from: 0.0,
            to: 1.0,
        },
        duration_secs: 3.0,
        delay_secs: 0.3,
        easing: Easing::EaseInOut,
    }])
}

/// Build a dose-sweep animation for hormesis.
pub fn build_hormesis_sweep_animation() -> Sequence {
    Sequence::Sequential(vec![Animation {
        target: AnimationTarget::Opacity {
            node_id: "hormesis-curve".to_owned(),
            from: 0.0,
            to: 1.0,
        },
        duration_secs: 2.5,
        delay_secs: 0.5,
        easing: Easing::EaseInOut,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ic50_scene_has_nodes_and_primitives() {
        let scene = build_ic50_scene();
        // root + axes + curve + annotation + title + labels = 6 nodes minimum
        assert!(scene.node_count() > 3);
        assert!(scene.total_primitives() > 10);
    }

    #[test]
    fn ic50_hill_equation_bounds() {
        let params = HillParams {
            top: 100.0,
            bottom: 5.0,
            ic50: 1e-7,
            hill_coeff: 1.2,
        };
        let high = params.response(-12.0);
        assert!(high > 95.0, "low conc should give high response: {high}");
        let low = params.response(-3.0);
        assert!(low < 15.0, "high conc should give low response: {low}");
        let mid = params.response(params.ic50.log10());
        let expected_mid = f64::midpoint(params.top, params.bottom);
        assert!(
            (mid - expected_mid).abs() < 5.0,
            "at IC50 should be near midpoint: {mid} vs {expected_mid}"
        );
    }

    #[test]
    fn pk_decay_scene_has_multiple_curves() {
        let scene = build_pk_decay_scene();
        assert!(scene.node_count() > 5);
        assert!(scene.total_primitives() > 10);
    }

    #[test]
    fn pk_two_compartment_decay() {
        let params = PkParams {
            a_coeff: 80.0,
            alpha: 1.5,
            b_coeff: 20.0,
            beta: 0.15,
        };
        let c0 = params.concentration(0.0);
        assert!((c0 - 100.0).abs() < 1e-10, "C(0) = A + B = 100: {c0}");
        let c1 = params.concentration(1.0);
        let c5 = params.concentration(5.0);
        let c24 = params.concentration(24.0);
        assert!(c0 > c1 && c1 > c5 && c5 > c24);
        assert!(c24 < 5.0, "C(24h) should be near zero: {c24}");
    }

    #[test]
    fn tissue_lattice_scene_has_grid() {
        let scene = build_tissue_lattice_scene();
        // 12×12 = 144 cell rects in the grid node
        let grid_node = scene.get("tissue-grid").expect("grid node exists");
        assert_eq!(grid_node.primitives.len(), 144);
    }

    #[test]
    fn tissue_viability_color_range() {
        let dead = viability_color(0.0);
        let alive = viability_color(1.0);
        assert!(dead.r > 0.5, "dead cells should be reddish");
        assert!(alive.g > alive.r, "alive cells should be greenish");
    }

    #[test]
    fn hormesis_scene_has_baseline_and_curve() {
        let scene = build_hormesis_scene();
        assert!(scene.node_count() > 4);
        assert!(scene.total_primitives() > 10);
    }

    #[test]
    fn hormesis_biphasic_shape() {
        let params = HormesisParams {
            baseline: 100.0,
            stimulation_peak: 25.0,
            peak_dose: 1e-8,
            inhibition_ic50: 1e-5,
            hill_n: 1.5,
        };
        let low = params.response(-11.0);
        assert!(
            (low - 100.0).abs() < 10.0,
            "very low dose near baseline: {low}"
        );
        let peak = params.response(params.peak_dose.log10());
        assert!(peak > 100.0, "peak should exceed baseline: {peak}");
        let high = params.response(-3.0);
        assert!(high < 100.0, "high dose should be inhibited: {high}");
    }

    #[test]
    fn animations_produce_valid_sequences() {
        let ic50_anim = build_ic50_sweep_animation();
        assert!(ic50_anim.total_duration() > 0.0);
        let pk_anim = build_pk_decay_animation();
        assert!(pk_anim.total_duration() > 2.0);
        let tissue_anim = build_tissue_diffusion_animation();
        assert!(tissue_anim.total_duration() > 0.0);
        let hormesis_anim = build_hormesis_sweep_animation();
        assert!(hormesis_anim.total_duration() > 0.0);
    }
}
