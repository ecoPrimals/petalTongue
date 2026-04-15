// SPDX-License-Identifier: AGPL-3.0-or-later

//! Primary compile path: grammar + tabular data → [`SceneGraph`](crate::scene_graph::SceneGraph).

use serde_json::Value;

use crate::domain_palette::palette_for_domain;
use crate::grammar::GrammarExpr;
use crate::math::{Axes, MathObject};
use crate::primitive::{AnchorPoint, Color, LineCap, LineJoin, Primitive, StrokeStyle};
use crate::scene_graph::{SceneGraph, SceneNode};

use super::GrammarCompiler;
use super::geometry::compile_geometry;
use super::utils::{x_field, y_field};

impl GrammarCompiler {
    /// Compile grammar expression and data into a scene graph.
    pub fn compile(&self, expr: &GrammarExpr, data: &[Value]) -> SceneGraph {
        let mut graph = SceneGraph::new();

        let x_field = x_field(expr);
        let y_field = y_field(expr);

        let palette = expr
            .domain
            .as_deref()
            .map_or_else(|| palette_for_domain("measurement"), palette_for_domain);

        let axes = Axes::default();
        let mut points: Vec<[f64; 2]> = Vec::new();
        for (i, obj) in data.iter().enumerate() {
            let obj = obj.as_object();
            #[expect(clippy::cast_precision_loss, reason = "fallback index: f64 sufficient")]
            let x = obj
                .and_then(|o| x_field.and_then(|f| super::utils::get_number(o, f)))
                .unwrap_or(i as f64);
            let y = obj
                .and_then(|o| y_field.and_then(|f| super::utils::get_number(o, f)))
                .unwrap_or(0.0);
            points.push([x, y]);
        }

        let stroke = StrokeStyle {
            color: palette.primary,
            width: 1.5,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        };

        let primitives = compile_geometry(expr, data, &points, &axes, palette, &stroke);

        let axes_prims = axes.to_primitives();
        let mut main_prims = primitives;
        main_prims.extend(axes_prims);

        let main_node = SceneNode::new("main").with_primitives(main_prims);
        graph.add_to_root(main_node);

        if let Some(ref title) = expr.title {
            let title_node = SceneNode::new("title").with_primitive(Primitive::Text {
                x: axes.origin.0 + axes.width / 2.0,
                y: axes.origin.1 - axes.height - 20.0,
                content: title.clone(),
                font_size: 16.0,
                color: Color::BLACK,
                anchor: AnchorPoint::BottomCenter,
                bold: true,
                italic: false,
                data_id: None,
            });
            graph.add_to_root(title_node);
        }

        graph
    }
}
