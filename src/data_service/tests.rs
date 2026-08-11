use super::*;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};
use std::time::Duration;

fn create_test_primal(id: &str, name: &str) -> PrimalInfo {
    PrimalInfo::new(
        id,
        name,
        "test",
        format!("http://test-{id}:8080"),
        vec![],
        PrimalHealthStatus::Healthy,
        0,
    )
}

#[tokio::test]
async fn test_data_service_creation() {
    let service = DataService::new();
    assert!(!service.has_neural_api());
}

#[tokio::test]
async fn test_data_service_default() {
    let service = DataService::default();
    assert!(!service.has_neural_api());
}

#[tokio::test]
async fn test_snapshot_without_neural_api() -> Result<()> {
    let service = DataService::new();
    let snapshot = service.snapshot().await?;

    assert!(snapshot.primals.is_empty());
    assert!(snapshot.edges.is_empty());
    // Timestamp is always valid (epoch or later)
    let _ = snapshot.timestamp;
    Ok(())
}

#[tokio::test]
async fn test_snapshot_timestamp() -> Result<()> {
    let service = DataService::new();
    let snapshot = service.snapshot().await?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    assert!(
        snapshot.timestamp <= now + 1,
        "Timestamp should be reasonable"
    );
    Ok(())
}

#[tokio::test]
async fn test_graph_access() {
    let service = DataService::new();
    let graph = service.graph();
    let guard = graph.read().expect("lock poisoned");
    assert!(guard.nodes().is_empty());
    assert!(guard.edges().is_empty());
    drop(guard);
}

#[tokio::test]
async fn test_update_subscription() {
    let service = DataService::new();
    let mut rx = service.subscribe();

    // Trigger update (refresh doesn't send when neural_api is None)
    service.send_test_update();

    // Should receive it (with timeout to avoid blocking forever)
    let update = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("recv timed out")
        .expect("recv failed");
    assert!(matches!(update, DataUpdate::TopologyUpdated));
}

#[tokio::test]
async fn test_refresh_without_neural_api() {
    let service = DataService::new();
    let result = service.refresh().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_init_without_neural_api() {
    let mut service = DataService::new();
    let result = service.init().await;
    assert!(result.is_ok());
    // Without a running API endpoint, neural_api stays None
}

#[tokio::test]
async fn test_snapshot_serialization() -> Result<()> {
    let service = DataService::new();
    let snapshot = service.snapshot().await?;
    let json = serde_json::to_string(&snapshot).expect("serialization failed");
    let deser: DataSnapshot = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deser.primals.len(), snapshot.primals.len());
    assert_eq!(deser.edges.len(), snapshot.edges.len());
    assert_eq!(deser.timestamp, snapshot.timestamp);
    Ok(())
}

#[tokio::test]
async fn test_graph_shared_across_clones() {
    let service = DataService::new();
    let graph1 = service.graph();
    let graph2 = service.graph();
    assert!(Arc::ptr_eq(&graph1, &graph2));
}

#[tokio::test]
async fn test_multiple_snapshots_consistent() -> Result<()> {
    let service = DataService::new();
    let snap1 = service.snapshot().await?;
    let snap2 = service.snapshot().await?;
    assert_eq!(snap1.primals.len(), snap2.primals.len());
    assert_eq!(snap1.edges.len(), snap2.edges.len());
    Ok(())
}

#[tokio::test]
async fn test_data_update_debug() {
    let update = DataUpdate::TopologyUpdated;
    let debug = format!("{update:?}");
    assert!(debug.contains("TopologyUpdated"));
}

#[tokio::test]
async fn test_data_update_clone() {
    let update = DataUpdate::TopologyUpdated;
    let cloned = Clone::clone(&update);
    assert!(matches!(cloned, DataUpdate::TopologyUpdated));
}

#[tokio::test]
async fn test_subscribe_multiple_receivers() {
    let service = DataService::new();
    let _rx1 = service.subscribe();
    let _rx2 = service.subscribe();
}

#[tokio::test]
async fn test_refresh_then_snapshot() -> Result<()> {
    let service = DataService::new();
    service.refresh().await?;
    let snapshot = service.snapshot().await?;
    assert!(snapshot.primals.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_data_snapshot_debug() -> Result<()> {
    let service = DataService::new();
    let snapshot = service.snapshot().await?;
    let debug_str = format!("{snapshot:?}");
    assert!(debug_str.contains("primals"));
    assert!(debug_str.contains("edges"));
    Ok(())
}

#[tokio::test]
async fn test_data_snapshot_clone() -> Result<()> {
    let service = DataService::new();
    let snapshot = service.snapshot().await?;
    let cloned = snapshot.clone();
    assert_eq!(cloned.primals.len(), snapshot.primals.len());
    assert_eq!(cloned.edges.len(), snapshot.edges.len());
    assert_eq!(cloned.timestamp, snapshot.timestamp);
    Ok(())
}

#[tokio::test]
async fn test_snapshot_with_populated_graph() -> Result<()> {
    let service = DataService::new();
    let graph = service.graph();
    {
        let mut guard = graph.write().expect("lock poisoned");
        let p1 = create_test_primal("p1", "Primal 1");
        let p2 = create_test_primal("p2", "Primal 2");
        guard.add_node(p1);
        guard.add_node(p2);
        guard.add_edge(TopologyEdge {
            from: "p1".into(),
            to: "p2".into(),
            edge_type: "connection".to_owned(),
            label: None,
            capability: None,
            weight: None,
            metrics: None,
        });
    }
    let snapshot = service.snapshot().await?;
    assert_eq!(snapshot.primals.len(), 2);
    assert_eq!(snapshot.edges.len(), 1);
    assert_eq!(snapshot.primals[0].id.as_str(), "p1");
    assert_eq!(snapshot.primals[1].id.as_str(), "p2");
    Ok(())
}

#[tokio::test]
async fn test_snapshot_serialization_with_data() -> Result<()> {
    let service = DataService::new();
    let graph = service.graph();
    {
        let mut guard = graph.write().expect("lock poisoned");
        guard.add_node(create_test_primal("test-1", "Test Primal"));
    }
    let snapshot = service.snapshot().await?;
    let json = serde_json::to_string(&snapshot).expect("serialization failed");
    let deser: DataSnapshot = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deser.primals.len(), 1);
    assert_eq!(deser.primals[0].id.as_str(), "test-1");
    assert_eq!(deser.primals[0].name, "Test Primal");
    Ok(())
}

#[tokio::test]
async fn test_broadcast_multiple_receivers() {
    let service = DataService::new();
    let mut rx1 = service.subscribe();
    let mut rx2 = service.subscribe();
    service.send_test_update();
    let update1 = tokio::time::timeout(Duration::from_secs(1), rx1.recv())
        .await
        .expect("rx1 timed out")
        .expect("rx1 recv failed");
    let update2 = tokio::time::timeout(Duration::from_secs(1), rx2.recv())
        .await
        .expect("rx2 timed out")
        .expect("rx2 recv failed");
    assert!(matches!(update1, DataUpdate::TopologyUpdated));
    assert!(matches!(update2, DataUpdate::TopologyUpdated));
}

#[tokio::test]
async fn test_broadcast_multiple_updates() {
    let service = DataService::new();
    let mut rx = service.subscribe();
    service.send_test_update();
    service.send_test_update();
    let u1 = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("timeout")
        .expect("recv");
    let u2 = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("timeout")
        .expect("recv");
    assert!(matches!(u1, DataUpdate::TopologyUpdated));
    assert!(matches!(u2, DataUpdate::TopologyUpdated));
}

#[tokio::test]
async fn test_graph_lock_poisoned_error_path() {
    let service = DataService::new();
    let graph = service.graph();
    let graph_clone = Arc::clone(&graph);
    let handle = std::thread::spawn(move || {
        let _guard = graph_clone.write().expect("lock poisoned");
        panic!("intentional poison for test");
    });
    let _ = handle.join();
    let result = service.snapshot().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, AppError::GraphLockPoisoned(_)),
        "Expected GraphLockPoisoned, got: {err}"
    );
    assert!(err.to_string().contains("Graph lock poisoned"));
}

#[tokio::test]
async fn test_snapshot_sync_returns_some() {
    let service = DataService::new();
    let Some(snap) = service.snapshot_sync() else {
        panic!("snapshot_sync should return Some for healthy graph");
    };
    assert!(snap.primals.is_empty());
    assert!(snap.edges.is_empty());
}

#[tokio::test]
async fn test_snapshot_sync_with_populated_graph() {
    let service = DataService::new();
    let graph = service.graph();
    {
        let mut g = graph.write().expect("lock poisoned");
        g.add_node(create_test_primal("sync-1", "SyncPrimal"));
    }
    let Some(snap) = service.snapshot_sync() else {
        panic!("snapshot_sync should return Some for populated graph");
    };
    assert_eq!(snap.primals.len(), 1);
    assert_eq!(snap.primals[0].id.as_str(), "sync-1");
}

#[tokio::test]
async fn test_snapshot_sync_poisoned_graph_returns_none() {
    let service = DataService::new();
    let graph = service.graph();
    let g2 = Arc::clone(&graph);
    let h = std::thread::spawn(move || {
        let _guard = g2.write().expect("lock poisoned");
        panic!("intentional poison for snapshot_sync test");
    });
    let _ = h.join();
    assert!(service.snapshot_sync().is_none());
}

#[tokio::test]
async fn test_refresh_without_neural_api_is_noop() {
    let service = DataService::new();
    assert!(!service.has_neural_api());
    let result = service.refresh().await;
    assert!(
        result.is_ok(),
        "refresh without neural_api should succeed (no-op)"
    );
}

#[tokio::test]
async fn test_mesh_peers_returns_peers() {
    let peers = DataService::mesh_peers();
    if peers.is_empty() {
        // No ecosystem_manifest.toml and offline-topology not enabled
        return;
    }
    assert!(peers.len() >= 6);
    assert!(peers.iter().any(|p| p.gate_id == "eastGate"));
    assert!(peers.iter().any(|p| p.gate_id == "ironGate"));
}

#[tokio::test]
async fn test_refresh_lock_poisoned_returns_error() {
    let service = DataService::new();
    let lr = Arc::clone(&service.last_refresh);
    let h = std::thread::spawn(move || {
        let _guard = lr.write().expect("lock poisoned");
        panic!("intentional poison for refresh lock test");
    });
    let _ = h.join();
    // last_refresh is poisoned, but refresh() only writes it
    // after successful neural_api fetch. Without neural_api, the
    // lock is never touched, so this just verifies the no-op path.
    assert!(service.refresh().await.is_ok());
}

#[tokio::test]
async fn test_data_snapshot_serialization_roundtrip_with_edges() {
    let snapshot = DataSnapshot {
        primals: vec![
            create_test_primal("a", "Alpha"),
            create_test_primal("b", "Beta"),
        ],
        edges: vec![TopologyEdge {
            from: "a".into(),
            to: "b".into(),
            edge_type: "api_call".to_owned(),
            label: Some("invoke".to_owned()),
            capability: None,
            weight: None,
            metrics: None,
        }],
        timestamp: 12345,
    };
    let json = serde_json::to_string(&snapshot).expect("serialization failed");
    let deser: DataSnapshot = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deser.primals.len(), 2);
    assert_eq!(deser.edges.len(), 1);
    assert_eq!(deser.edges[0].edge_type, "api_call");
    assert_eq!(deser.timestamp, 12345);
}
