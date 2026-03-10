// SPDX-License-Identifier: AGPL-3.0-only
//! Graph Editor Demo - Collaborative Intelligence
//!
//! Demonstrates the complete graph editor functionality:
//! - Interactive graph creation
//! - Node manipulation
//! - Template save/load
//! - Real-time streaming
//! - AI reasoning display

use anyhow::Result;
use petal_tongue_ui::graph_editor::{
    DependencyType, Graph, GraphEdge, GraphEditorService, GraphNode, StreamHandler,
};
use tracing::{Level, info};

#[tokio::main]
#[expect(clippy::too_many_lines, reason = "demo main is cohesive setup and run")]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🤝 Graph Editor Demo - Collaborative Intelligence");
    info!("================================================\n");

    // 1. Create a graph editor service
    let service = GraphEditorService::new();
    info!("✅ Created GraphEditorService");

    // 2. Create a stream handler for real-time updates
    let stream_handler = StreamHandler::new();
    let mut rx = stream_handler.subscribe();
    info!("✅ Created StreamHandler with subscription\n");

    // 3. Create an example graph
    info!("📊 Creating example graph...");
    let mut graph = Graph::new("demo-graph".to_string(), "Demo Graph".to_string());
    graph.description = "A demonstration of collaborative intelligence".to_string();

    // Add nodes
    let node1 = GraphNode::new("data-ingestion".to_string(), "DataSource".to_string())
        .with_position(100.0, 100.0)
        .with_description("Ingest data from external sources".to_string());

    let node2 = GraphNode::new("processing".to_string(), "Transform".to_string())
        .with_position(300.0, 100.0)
        .with_description("Process and transform data".to_string());

    let node3 = GraphNode::new("analysis".to_string(), "Analysis".to_string())
        .with_position(500.0, 100.0)
        .with_description("Analyze processed data".to_string());

    let node4 = GraphNode::new("output".to_string(), "Storage".to_string())
        .with_position(700.0, 100.0)
        .with_description("Store results".to_string());

    graph.add_node(node1)?;
    graph.add_node(node2)?;
    graph.add_node(node3)?;
    graph.add_node(node4)?;

    // Add edges (dependencies)
    let edge1 = GraphEdge::new(
        "edge-1".to_string(),
        "data-ingestion".to_string(),
        "processing".to_string(),
        DependencyType::Sequential,
    )
    .with_label("raw_data".to_string());

    let edge2 = GraphEdge::new(
        "edge-2".to_string(),
        "processing".to_string(),
        "analysis".to_string(),
        DependencyType::DataFlow,
    )
    .with_label("processed_data".to_string());

    let edge3 = GraphEdge::new(
        "edge-3".to_string(),
        "analysis".to_string(),
        "output".to_string(),
        DependencyType::Sequential,
    )
    .with_label("results".to_string());

    graph.add_edge(edge1)?;
    graph.add_edge(edge2)?;
    graph.add_edge(edge3)?;

    info!(
        "✅ Created graph with {} nodes and {} edges",
        graph.get_nodes().len(),
        graph.get_edges().len()
    );

    // Validate graph
    graph.validate()?;
    info!("✅ Graph validation passed");

    // Get topological sort (execution order)
    let execution_order = graph.topological_sort()?;
    info!("📋 Execution order: {:?}\n", execution_order);

    // 4. Save as template
    info!("💾 Saving graph as template...");
    let save_req = petal_tongue_ui::graph_editor::rpc_methods::SaveTemplateRequest {
        graph_id: "demo-graph".to_string(),
        name: "Data Pipeline".to_string(),
        description: "Standard data ingestion and processing pipeline".to_string(),
        tags: vec![
            "data".to_string(),
            "pipeline".to_string(),
            "etl".to_string(),
        ],
    };

    // First add the graph to the service
    use petal_tongue_ui::graph_editor::rpc_methods::AddNodeRequest;
    for node in graph.get_nodes() {
        service
            .add_node(AddNodeRequest {
                graph_id: "demo-graph".to_string(),
                node: node.clone(),
            })
            .await?;
    }

    let template_resp = service.save_template(save_req).await?;
    info!(
        "✅ Saved template: {} (ID: {})\n",
        "Data Pipeline", template_resp.template_id
    );

    // 5. Simulate graph execution with streaming updates
    info!("🚀 Simulating graph execution...");
    stream_handler
        .start_execution("demo-graph".to_string())
        .await?;

    // Spawn a task to listen for stream messages
    let _listener = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            match msg {
                petal_tongue_ui::graph_editor::StreamMessage::NodeStatus {
                    node_id,
                    status,
                    ..
                } => {
                    info!("  📡 Node '{}' status: {:?}", node_id, status);
                }
                petal_tongue_ui::graph_editor::StreamMessage::Progress {
                    node_id,
                    progress,
                    message,
                    ..
                } => {
                    info!(
                        "  📊 Node '{}' progress: {:.0}% - {}",
                        node_id,
                        progress * 100.0,
                        message
                    );
                }
                petal_tongue_ui::graph_editor::StreamMessage::Reasoning { reasoning, .. } => {
                    info!(
                        "  🤖 AI Decision: {} (confidence: {:.0}%)",
                        reasoning.decision,
                        reasoning.confidence * 100.0
                    );
                }
                _ => {}
            }
        }
    });

    // Simulate execution
    for node_id in &execution_order {
        // Start node
        stream_handler
            .send_node_status(
                "demo-graph".to_string(),
                node_id.clone(),
                petal_tongue_ui::graph_editor::streaming::NodeStatus::Running { progress: 0 },
            )
            .await?;

        // Progress updates
        for progress in [25, 50, 75] {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            stream_handler
                .send_progress(
                    "demo-graph".to_string(),
                    node_id.clone(),
                    progress as f32 / 100.0,
                    format!("Processing... {progress}%"),
                )
                .await?;
        }

        // Complete node
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        stream_handler
            .send_node_status(
                "demo-graph".to_string(),
                node_id.clone(),
                petal_tongue_ui::graph_editor::streaming::NodeStatus::Completed,
            )
            .await?;
    }

    // Send AI reasoning
    let reasoning = petal_tongue_ui::graph_editor::streaming::AIReasoning {
        decision: "Execute 'analysis' node next".to_string(),
        confidence: 0.87,
        rationale: vec![
            "All dependencies satisfied".to_string(),
            "Resources available (CPU: 40%)".to_string(),
            "Highest priority in queue".to_string(),
        ],
        alternatives: vec![petal_tongue_ui::graph_editor::streaming::Alternative {
            description: "Wait for more data".to_string(),
            confidence: 0.73,
            reason_not_chosen: "Current dataset sufficient".to_string(),
        }],
        data_sources: vec!["user_history".to_string(), "system_metrics".to_string()],
        patterns: vec![petal_tongue_ui::graph_editor::streaming::Pattern {
            description: "User prefers immediate processing".to_string(),
            source: "user_history".to_string(),
            relevance: 0.9,
        }],
    };

    stream_handler
        .send_reasoning("demo-graph".to_string(), reasoning)
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    stream_handler.stop_execution("demo-graph").await?;
    info!("✅ Execution complete\n");

    // 6. Show graph statistics
    let stats = graph.stats();
    info!("📊 Graph Statistics:");
    info!("  - Nodes: {}", stats.node_count);
    info!("  - Edges: {}", stats.edge_count);
    info!("  - Max Depth: {}", stats.max_depth);
    info!("  - Has Cycles: {}", stats.has_cycles);

    info!("\n🎉 Demo complete! Collaborative Intelligence is ready!");

    Ok(())
}
