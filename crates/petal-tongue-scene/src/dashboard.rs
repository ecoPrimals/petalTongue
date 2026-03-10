// SPDX-License-Identifier: AGPL-3.0-only
//! Dashboard layout engine for multi-panel visualizations.
//!
//! Composes multiple compiled `SceneGraph`s into a single dashboard scene
//! with automatic grid layout, titles, and consistent spacing.

use crate::compiler::GrammarCompiler;
use crate::data_binding_compiler::DataBindingCompiler;
use crate::domain_palette::palette_for_domain;
use crate::primitive::{AnchorPoint, Color, Primitive, StrokeStyle};
use crate::scene_graph::{SceneGraph, SceneNode};
use crate::transform::Transform2D;

use petal_tongue_core::DataBinding;

/// Layout strategy for dashboard panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardLayout {
    /// Auto-fit into a grid with the given maximum number of columns.
    Grid { max_columns: usize },
    /// Single column, panels stacked vertically.
    Vertical,
    /// Single row, panels side by side.
    Horizontal,
}

impl Default for DashboardLayout {
    fn default() -> Self {
        Self::Grid { max_columns: 3 }
    }
}

/// Configuration for the dashboard.
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub layout: DashboardLayout,
    pub panel_width: f64,
    pub panel_height: f64,
    pub spacing: f64,
    pub title: Option<String>,
    pub domain: Option<String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            layout: DashboardLayout::default(),
            panel_width: 400.0,
            panel_height: 300.0,
            spacing: 20.0,
            title: None,
            domain: None,
        }
    }
}

impl DashboardConfig {
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    #[must_use]
    pub fn with_layout(mut self, layout: DashboardLayout) -> Self {
        self.layout = layout;
        self
    }

    #[must_use]
    pub fn with_panel_size(mut self, width: f64, height: f64) -> Self {
        self.panel_width = width;
        self.panel_height = height;
        self
    }
}

/// A compiled dashboard: a single `SceneGraph` containing all panels.
pub struct Dashboard {
    pub scene: SceneGraph,
    pub panel_count: usize,
    pub columns: usize,
    pub rows: usize,
}

/// Build a dashboard from a slice of `DataBinding`s.
///
/// Each binding is compiled through the Grammar of Graphics pipeline
/// and arranged according to the `DashboardConfig` layout.
#[must_use]
pub fn build_dashboard(bindings: &[DataBinding], config: &DashboardConfig) -> Dashboard {
    let domain = config.domain.as_deref();
    let compiler = GrammarCompiler::new();

    let panels: Vec<(String, SceneGraph)> = bindings
        .iter()
        .map(|binding| {
            let (expr, data) = DataBindingCompiler::compile(binding, domain);
            let title = expr.title.clone().unwrap_or_default();
            let scene = compiler.compile(&expr, &data);
            (title, scene)
        })
        .collect();

    compose_dashboard(&panels, config)
}

/// Compose pre-compiled panels into a dashboard.
#[must_use]
pub fn compose_dashboard(panels: &[(String, SceneGraph)], config: &DashboardConfig) -> Dashboard {
    let panel_count = panels.len();
    let (columns, rows) = grid_dimensions(panel_count, &config.layout);

    let palette = palette_for_domain(config.domain.as_deref().unwrap_or("measurement"));

    let title_offset = if config.title.is_some() { 40.0 } else { 0.0 };
    let total_width = columns as f64 * (config.panel_width + config.spacing) - config.spacing;
    let total_height =
        rows as f64 * (config.panel_height + config.spacing) - config.spacing + title_offset;

    let mut scene = SceneGraph::new();

    if let Some(root) = scene.get_mut("root") {
        root.label = Some("dashboard".to_string());
        root.primitives.push(Primitive::Rect {
            x: 0.0,
            y: 0.0,
            width: total_width.max(0.0),
            height: total_height.max(0.0),
            fill: Some(palette.chart_bg),
            stroke: None,
            corner_radius: 0.0,
            data_id: None,
        });

        if let Some(ref title_text) = config.title {
            root.primitives.push(Primitive::Text {
                x: total_width / 2.0,
                y: 24.0,
                content: title_text.clone(),
                font_size: 20.0,
                color: palette.primary,
                anchor: AnchorPoint::Center,
                bold: true,
                italic: false,
                data_id: None,
            });
        }
    }

    let panel_stroke = StrokeStyle {
        color: Color::rgba(
            palette.secondary.r,
            palette.secondary.g,
            palette.secondary.b,
            0.4,
        ),
        width: 1.0,
        cap: crate::primitive::LineCap::Butt,
        join: crate::primitive::LineJoin::Miter,
    };

    for (idx, (title, panel_scene)) in panels.iter().enumerate() {
        let col = idx % columns.max(1);
        let row = idx / columns.max(1);
        let x = col as f64 * (config.panel_width + config.spacing);
        let y = row as f64 * (config.panel_height + config.spacing) + title_offset;

        let panel_id = format!("panel_{idx}");
        let mut panel_node = SceneNode::new(&panel_id);
        panel_node.transform = Transform2D::translate(x, y);
        panel_node.label = Some(title.clone());

        panel_node.primitives.push(Primitive::Rect {
            x: 0.0,
            y: 0.0,
            width: config.panel_width,
            height: config.panel_height,
            fill: Some(Color::rgba(1.0, 1.0, 1.0, 0.12)),
            stroke: Some(panel_stroke),
            corner_radius: 4.0,
            data_id: None,
        });

        if !title.is_empty() {
            panel_node.primitives.push(Primitive::Text {
                x: config.panel_width / 2.0,
                y: 16.0,
                content: title.clone(),
                font_size: 13.0,
                color: Color::rgba(
                    palette.primary.r,
                    palette.primary.g,
                    palette.primary.b,
                    0.86,
                ),
                anchor: AnchorPoint::Center,
                bold: false,
                italic: false,
                data_id: None,
            });
        }

        scene.add_to_root(panel_node);

        let content_id = format!("panel_{idx}_content");
        let mut content_node = SceneNode::new(&content_id);
        for (_transform, primitive) in panel_scene.flatten() {
            content_node.primitives.push(primitive.clone());
        }
        scene.add_node(content_node, &panel_id);
    }

    Dashboard {
        scene,
        panel_count,
        columns,
        rows,
    }
}

fn grid_dimensions(n: usize, layout: &DashboardLayout) -> (usize, usize) {
    if n == 0 {
        return (0, 0);
    }
    match layout {
        DashboardLayout::Grid { max_columns } => {
            let cols = n.min(*max_columns);
            let rows = n.div_ceil(cols);
            (cols, rows)
        }
        DashboardLayout::Vertical => (1, n),
        DashboardLayout::Horizontal => (n, 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::DataBinding;

    fn sample_bindings() -> Vec<DataBinding> {
        vec![
            DataBinding::TimeSeries {
                id: "ts".to_string(),
                label: "Glucose".to_string(),
                x_label: "Time".to_string(),
                y_label: "mg/dL".to_string(),
                unit: "mg/dL".to_string(),
                x_values: vec![0.0, 1.0, 2.0],
                y_values: vec![90.0, 95.0, 88.0],
            },
            DataBinding::Gauge {
                id: "hr".to_string(),
                label: "Heart Rate".to_string(),
                value: 72.0,
                min: 40.0,
                max: 140.0,
                unit: "bpm".to_string(),
                normal_range: [60.0, 100.0],
                warning_range: [40.0, 60.0],
            },
            DataBinding::Bar {
                id: "bar".to_string(),
                label: "Labs".to_string(),
                categories: vec!["WBC".to_string(), "RBC".to_string()],
                values: vec![6.5, 4.2],
                unit: "k/uL".to_string(),
            },
            DataBinding::Spectrum {
                id: "hrv".to_string(),
                label: "HRV Spectrum".to_string(),
                frequencies: vec![0.0, 0.04, 0.15, 0.4],
                amplitudes: vec![100.0, 500.0, 300.0, 50.0],
                unit: "ms\u{b2}/Hz".to_string(),
            },
        ]
    }

    #[test]
    fn grid_dimensions_default() {
        let (c, r) = grid_dimensions(4, &DashboardLayout::default());
        assert_eq!((c, r), (3, 2));
    }

    #[test]
    fn grid_dimensions_vertical() {
        let (c, r) = grid_dimensions(3, &DashboardLayout::Vertical);
        assert_eq!((c, r), (1, 3));
    }

    #[test]
    fn grid_dimensions_horizontal() {
        let (c, r) = grid_dimensions(3, &DashboardLayout::Horizontal);
        assert_eq!((c, r), (3, 1));
    }

    #[test]
    fn grid_dimensions_empty() {
        let (c, r) = grid_dimensions(0, &DashboardLayout::default());
        assert_eq!((c, r), (0, 0));
    }

    #[test]
    fn build_dashboard_produces_scene() {
        let bindings = sample_bindings();
        let config = DashboardConfig::default().with_title("Health Dashboard");
        let dashboard = build_dashboard(&bindings, &config);
        assert_eq!(dashboard.panel_count, 4);
        assert_eq!(dashboard.columns, 3);
        assert_eq!(dashboard.rows, 2);
        assert!(dashboard.scene.node_count() > 4);
    }

    #[test]
    fn build_dashboard_with_domain() {
        let bindings = sample_bindings();
        let config = DashboardConfig::default()
            .with_title("Clinical View")
            .with_domain("health");
        let dashboard = build_dashboard(&bindings, &config);
        assert!(dashboard.scene.total_primitives() > 0);
    }

    #[test]
    fn build_dashboard_single_panel() {
        let bindings = vec![DataBinding::TimeSeries {
            id: "single".to_string(),
            label: "One Panel".to_string(),
            x_label: String::new(),
            y_label: String::new(),
            unit: String::new(),
            x_values: vec![0.0, 1.0],
            y_values: vec![1.0, 2.0],
        }];
        let config = DashboardConfig::default();
        let dashboard = build_dashboard(&bindings, &config);
        assert_eq!(dashboard.panel_count, 1);
        assert_eq!(dashboard.columns, 1);
        assert_eq!(dashboard.rows, 1);
    }

    #[test]
    fn build_dashboard_empty() {
        let config = DashboardConfig::default();
        let dashboard = build_dashboard(&[], &config);
        assert_eq!(dashboard.panel_count, 0);
        assert_eq!(dashboard.columns, 0);
        assert_eq!(dashboard.rows, 0);
    }

    #[test]
    fn dashboard_layout_custom_columns() {
        let config =
            DashboardConfig::default().with_layout(DashboardLayout::Grid { max_columns: 2 });
        let dashboard = build_dashboard(&sample_bindings(), &config);
        assert_eq!(dashboard.columns, 2);
        assert_eq!(dashboard.rows, 2);
    }

    #[test]
    fn dashboard_panel_size_custom() {
        let config = DashboardConfig::default().with_panel_size(800.0, 600.0);
        let dashboard = build_dashboard(&sample_bindings(), &config);
        assert!(dashboard.scene.total_primitives() > 0);
    }
}
