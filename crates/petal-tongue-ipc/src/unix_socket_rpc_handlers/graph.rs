// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers for `visualization.render.graph` and graph rendering.
//!
//! Builds a `SceneGraph` from the `GraphEngine` topology and compiles it
//! through the real `SvgCompiler` / `TerminalCompiler` / HTML wrapper.

use super::RpcHandlers;
use crate::json_rpc::{JsonRpcResponse, error_codes};
use petal_tongue_scene::modality::{
    ModalityCompiler, ModalityOutput, SvgCompiler, TerminalCompiler,
};
use petal_tongue_scene::primitive::{Color, LineCap, LineJoin, Primitive, StrokeStyle};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};
use serde_json::{Value, json};

/// Build a `SceneGraph` from graph engine nodes and edges.
#[expect(
    clippy::significant_drop_tightening,
    reason = "nodes/edges borrow from graph for iteration"
)]
fn graph_to_scene(handlers: &RpcHandlers) -> (SceneGraph, usize, usize) {
    let graph = handlers
        .graph
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let nodes = graph.nodes();
    let edges = graph.edges();
    let node_count = nodes.len();
    let edge_count = edges.len();

    let mut scene = SceneGraph::new();

    for node in nodes {
        let mut scene_node = SceneNode::new(format!("node-{}", node.info.id));
        scene_node.primitives.push(Primitive::Point {
            x: f64::from(node.position.x),
            y: f64::from(node.position.y),
            radius: 8.0,
            fill: Some(Color::rgba(0.3, 0.6, 1.0, 1.0)),
            stroke: Some(StrokeStyle {
                color: Color::rgba(0.2, 0.4, 0.8, 1.0),
                width: 1.5,
                cap: LineCap::Round,
                join: LineJoin::Round,
            }),
            data_id: Some(node.info.id.to_string()),
        });
        scene_node.primitives.push(Primitive::Text {
            x: f64::from(node.position.x) + 12.0,
            y: f64::from(node.position.y) - 6.0,
            content: node.info.name.clone(),
            font_size: 11.0,
            color: Color::rgba(0.85, 0.85, 0.85, 1.0),
            anchor: petal_tongue_scene::primitive::AnchorPoint::CenterLeft,
            bold: false,
            italic: false,
            data_id: None,
        });
        scene.add_to_root(scene_node);
    }

    for (idx, edge) in edges.iter().enumerate() {
        let from_pos = nodes
            .iter()
            .find(|n| n.info.id == edge.from)
            .map(|n| (f64::from(n.position.x), f64::from(n.position.y)));
        let to_pos = nodes
            .iter()
            .find(|n| n.info.id == edge.to)
            .map(|n| (f64::from(n.position.x), f64::from(n.position.y)));

        if let (Some((x1, y1)), Some((x2, y2))) = (from_pos, to_pos) {
            let mut edge_node = SceneNode::new(format!("edge-{idx}"));
            edge_node.primitives.push(Primitive::Line {
                points: vec![(x1, y1).into(), (x2, y2).into()],
                stroke: StrokeStyle {
                    color: Color::rgba(0.5, 0.5, 0.5, 0.6),
                    width: 1.0,
                    cap: LineCap::Butt,
                    join: LineJoin::Miter,
                },
                closed: false,
                data_id: None,
            });
            scene.add_to_root(edge_node);
        }
    }

    (scene, node_count, edge_count)
}

/// Internal: render graph data (used by ui.render for `content_type` "graph")
#[expect(
    clippy::unused_async,
    reason = "async trait requirement for RPC handler"
)]
pub async fn render_graph_data(
    _handlers: &RpcHandlers,
    data: Value,
) -> Result<(), std::convert::Infallible> {
    tracing::debug!("Rendering graph data: {:?}", data);
    Ok(())
}

/// Handle `visualization.render.graph`: render graph to specified format.
///
/// Builds a real `SceneGraph` from the graph engine and compiles through
/// `SvgCompiler` or `TerminalCompiler`.
#[expect(
    clippy::unused_async,
    reason = "async trait requirement for RPC handler"
)]
pub async fn render_graph(handlers: &RpcHandlers, params: Value, id: Value) -> JsonRpcResponse {
    let format = params["format"].as_str().unwrap_or("svg");
    let (scene, node_count, edge_count) = graph_to_scene(handlers);

    match format {
        "svg" => {
            let compiler = SvgCompiler;
            match compiler.compile(&scene) {
                ModalityOutput::Svg(bytes) => JsonRpcResponse::success(
                    id,
                    json!({
                        "format": "svg",
                        "data": String::from_utf8_lossy(bytes.as_ref()),
                        "metadata": { "nodes": node_count, "edges": edge_count },
                    }),
                ),
                _ => JsonRpcResponse::error(
                    id,
                    error_codes::INTERNAL_ERROR,
                    "SVG compilation failed",
                ),
            }
        }
        "html" => {
            let compiler = SvgCompiler;
            match compiler.compile(&scene) {
                ModalityOutput::Svg(bytes) => {
                    let svg = String::from_utf8_lossy(bytes.as_ref());
                    let html = petal_tongue_ui_core::wrap_svg_in_html(&svg);
                    let html_str = String::from_utf8_lossy(&html).into_owned();
                    JsonRpcResponse::success(
                        id,
                        json!({
                            "format": "html",
                            "data": html_str,
                            "metadata": { "nodes": node_count, "edges": edge_count },
                        }),
                    )
                }
                _ => JsonRpcResponse::error(
                    id,
                    error_codes::INTERNAL_ERROR,
                    "HTML compilation failed (SVG stage)",
                ),
            }
        }
        "terminal" => {
            let compiler = TerminalCompiler::new(120, 40);
            match compiler.compile(&scene) {
                ModalityOutput::TerminalCells(cells) => {
                    let grid: Vec<String> = cells.iter().map(|row| row.iter().collect()).collect();
                    JsonRpcResponse::success(
                        id,
                        json!({
                            "format": "terminal",
                            "data": grid.join("\n"),
                            "metadata": { "nodes": node_count, "edges": edge_count },
                        }),
                    )
                }
                _ => JsonRpcResponse::error(
                    id,
                    error_codes::INTERNAL_ERROR,
                    "Terminal compilation failed",
                ),
            }
        }
        _ => JsonRpcResponse::error(
            id,
            error_codes::INVALID_PARAMS,
            format!("Unsupported format: {format}. Supported: svg, html, terminal"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unix_socket_rpc_handlers::RpcHandlers;
    use petal_tongue_core::graph_engine::GraphEngine;
    use petal_tongue_core::test_fixtures::primals;
    use petal_tongue_core::{LayoutAlgorithm, TopologyEdge};
    use serde_json::json;
    use std::sync::{Arc, RwLock};

    fn test_handlers() -> RpcHandlers {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let viz_state = Arc::new(RwLock::new(
            crate::visualization_handler::VisualizationState::new(),
        ));
        RpcHandlers::new(graph, "test".to_string(), viz_state)
    }

    #[tokio::test]
    async fn render_graph_svg_empty_graph() {
        let h = test_handlers();
        let resp = render_graph(&h, json!({"format": "svg"}), json!(1)).await;
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["format"], "svg");
        assert!(r["data"].as_str().unwrap().contains("svg"));
        assert_eq!(r["metadata"]["nodes"], 0);
        assert_eq!(r["metadata"]["edges"], 0);
    }

    #[tokio::test]
    async fn render_graph_html_format() {
        let h = test_handlers();
        let resp = render_graph(&h, json!({"format": "html"}), json!(1)).await;
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["format"], "html");
        let data = r["data"].as_str().expect("html data");
        assert!(data.starts_with("<!DOCTYPE html>"));
        assert!(data.contains("<svg"));
        assert_eq!(r["metadata"]["nodes"], 0);
    }

    #[tokio::test]
    async fn render_graph_terminal_format() {
        let h = test_handlers();
        let resp = render_graph(&h, json!({"format": "terminal"}), json!(1)).await;
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["format"], "terminal");
        assert!(r["data"].as_str().is_some());
    }

    #[tokio::test]
    async fn render_graph_default_format_is_svg() {
        let h = test_handlers();
        let resp = render_graph(&h, json!({}), json!(1)).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.expect("result")["format"], "svg");
    }

    #[tokio::test]
    async fn render_graph_unsupported_format_returns_error() {
        let h = test_handlers();
        let resp = render_graph(&h, json!({"format": "pdf"}), json!(1)).await;
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[tokio::test]
    async fn render_graph_with_nodes_produces_svg() {
        let mut graph = GraphEngine::new();
        graph.add_node(primals::test_primal("n1"));
        graph.add_node(primals::test_primal("n2"));
        graph.add_edge(TopologyEdge {
            from: "n1".into(),
            to: "n2".into(),
            edge_type: "test".to_string(),
            label: None,
            capability: None,
            metrics: None,
        });
        graph.set_layout(LayoutAlgorithm::Circular);
        graph.layout(1);

        let h = RpcHandlers::new(
            Arc::new(RwLock::new(graph)),
            "test".to_string(),
            Arc::new(RwLock::new(
                crate::visualization_handler::VisualizationState::new(),
            )),
        );
        let resp = render_graph(&h, json!({"format": "svg"}), json!(1)).await;
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["metadata"]["nodes"], 2);
        assert_eq!(r["metadata"]["edges"], 1);
    }
}
