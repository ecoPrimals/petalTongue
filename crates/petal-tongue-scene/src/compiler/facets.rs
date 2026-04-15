// SPDX-License-Identifier: AGPL-3.0-or-later

//! Faceted layout (small multiples): partition, compile sub-scenes, offset primitives.

use serde_json::Value;

use crate::grammar::{FacetLayout, GrammarExpr};
use crate::primitive::{AnchorPoint, Color, Primitive};
use crate::scene_graph::{SceneGraph, SceneNode};

use super::GrammarCompiler;
use super::utils::{offset_primitive, partition_by_field};

impl GrammarCompiler {
    /// Compile with faceting support (small multiples).
    ///
    /// When `expr.facets` is set, partitions data by the facet variable and
    /// compiles each group as a sub-scene. Groups are arranged according to
    /// the `FacetLayout` (Wrap or Grid).
    ///
    /// If no facets are configured, delegates to `compile`.
    #[must_use]
    pub fn compile_faceted(&self, expr: &GrammarExpr, data: &[Value]) -> SceneGraph {
        let Some(facet) = &expr.facets else {
            return self.compile(expr, data);
        };

        let columns = match &facet.layout {
            FacetLayout::Wrap { columns } => *columns,
            FacetLayout::Grid { .. } => 3,
        };
        let columns = columns.max(1);

        let groups = partition_by_field(data, &facet.variable);
        if groups.is_empty() {
            return self.compile(expr, data);
        }

        let mut graph = SceneGraph::new();
        let axes = crate::math::Axes::default();
        let panel_w = axes.width + 40.0;
        let panel_h = axes.height + 60.0;

        for (idx, (key, group_data)) in groups.iter().enumerate() {
            let col = idx % columns;
            let row = idx / columns;
            #[expect(clippy::cast_precision_loss, reason = "panel offset: f64 sufficient")]
            let offset_x = col as f64 * panel_w;
            #[expect(clippy::cast_precision_loss, reason = "panel offset: f64 sufficient")]
            let offset_y = row as f64 * panel_h;

            let key_str = key.as_ref();
            let mut facet_expr = expr.clone();
            facet_expr.facets = None;
            facet_expr.title = Some(key_str.to_string());

            let sub_scene = self.compile(&facet_expr, group_data);
            let flattened = sub_scene.flatten();

            let mut prims: Vec<Primitive> = Vec::new();
            for (_, prim) in &flattened {
                let mut p = (*prim).clone();
                offset_primitive(&mut p, offset_x, offset_y);
                prims.push(p);
            }

            let panel_node = SceneNode::new(format!("facet-{key_str}")).with_primitives(prims);
            graph.add_to_root(panel_node);

            let label_node =
                SceneNode::new(format!("facet-label-{key_str}")).with_primitive(Primitive::Text {
                    x: offset_x + panel_w / 2.0,
                    y: offset_y - 5.0,
                    content: key_str.to_string(),
                    font_size: 12.0,
                    color: Color::rgba(0.3, 0.3, 0.3, 1.0),
                    anchor: AnchorPoint::BottomCenter,
                    bold: true,
                    italic: false,
                    data_id: None,
                });
            graph.add_to_root(label_node);
        }
        graph
    }
}
