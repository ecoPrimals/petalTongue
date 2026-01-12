//! End-to-End Tests for petalTongue Rich TUI
//!
//! Tests complete workflows and user interactions.

use petal_tongue_tui::state::{LogLevel, LogMessage, TUIState, View};
use chrono::Utc;

mod common;
use common::{create_test_primal, create_test_primal_with_caps, create_test_primal_with_health, create_test_edge};
use petal_tongue_core::PrimalHealthStatus;

/// Test complete user workflow
#[tokio::test]
async fn test_complete_dashboard_workflow() {
    let state = TUIState::new();
    
    // 1. Start at dashboard
    assert_eq!(state.get_view().await, View::Dashboard);
    
    // 2. Add some primals (simulating discovery)
    let primals = vec![
        create_test_primal_with_caps("songbird", "songbird-1", vec!["discovery".to_string(), "events".to_string()]),
        create_test_primal_with_caps("toadstool", "toadstool-1", vec!["compute".to_string(), "gpu".to_string()]),
    ];
    
    state.update_primals(primals).await;
    
    // 3. Add topology edges
    let topology = vec![
        create_test_edge("songbird-1", "toadstool-1", "discovery"),
    ];
    
    state.update_topology(topology).await;
    
    // 4. Add some logs
    for i in 0..5 {
        state.add_log(LogMessage {
            timestamp: Utc::now(),
            source: Some("songbird".to_string()),
            level: LogLevel::Info,
            message: format!("Discovery event {}", i),
        }).await;
    }
    
    // 5. Verify state
    let stats = state.stats().await;
    assert_eq!(stats.primal_count, 2);
    assert_eq!(stats.topology_edge_count, 1);
    assert_eq!(stats.log_count, 5);
    
    // 6. Navigate to different views
    state.set_view(View::Topology).await;
    assert_eq!(state.get_view().await, View::Topology);
    
    state.set_view(View::Primals).await;
    assert_eq!(state.get_view().await, View::Primals);
    
    // 7. Test selection in Primals view (2 items)
    assert_eq!(state.get_selected_index().await, 0);
    state.select_next(2).await;
    assert_eq!(state.get_selected_index().await, 1);
    
    // 8. Switch view (selection should reset)
    state.set_view(View::Logs).await;
    assert_eq!(state.get_selected_index().await, 0);
}

/// Test primal discovery and registration workflow
#[tokio::test]
async fn test_primal_discovery_workflow() {
    let state = TUIState::new();
    
    // Initially standalone
    state.set_standalone_mode(true).await;
    assert!(state.is_standalone().await);
    
    // Discover first primal
    let primals = vec![
        create_test_primal("songbird", "songbird-1"),
    ];
    
    state.update_primals(primals.clone()).await;
    
    // Register capabilities
    state.register_capability(
        "songbird".to_string(),
        vec!["discovery".to_string(), "events".to_string()],
    );
    
    // No longer standalone
    state.set_standalone_mode(false).await;
    assert!(!state.is_standalone().await);
    
    // Verify discovery
    assert_eq!(state.get_primals().await.len(), 1);
    assert!(state.has_capability("songbird", "discovery"));
    
    // Discover more primals
    let mut all_primals = primals;
    all_primals.push(create_test_primal("toadstool", "toadstool-1"));
    
    state.update_primals(all_primals).await;
    assert_eq!(state.get_primals().await.len(), 2);
}

/// Test topology update workflow
#[tokio::test]
async fn test_topology_update_workflow() {
    let state = TUIState::new();
    
    // Add primals
    let primals = vec![
        create_test_primal("songbird", "songbird-1"),
        create_test_primal("toadstool", "toadstool-1"),
        create_test_primal("nestgate", "nestgate-1"),
    ];
    
    state.update_primals(primals).await;
    
    // Build topology
    let topology = vec![
        create_test_edge("songbird-1", "toadstool-1", "discovery"),
        create_test_edge("songbird-1", "nestgate-1", "discovery"),
        create_test_edge("toadstool-1", "nestgate-1", "storage"),
    ];
    
    state.update_topology(topology.clone()).await;
    
    // Verify topology
    let retrieved_topology = state.get_topology().await;
    assert_eq!(retrieved_topology.len(), 3);
    
    // Update topology (primal goes down)
    let new_topology = vec![
        create_test_edge("songbird-1", "nestgate-1", "discovery"),
    ];
    
    state.update_topology(new_topology).await;
    assert_eq!(state.get_topology().await.len(), 1);
}

/// Test log streaming workflow
#[tokio::test]
async fn test_log_streaming_workflow() {
    let state = TUIState::new();
    
    // Simulate log streaming from multiple sources
    let sources = vec!["songbird", "toadstool", "nestgate", "beardog"];
    let levels = vec![LogLevel::Trace, LogLevel::Debug, LogLevel::Info, LogLevel::Warn, LogLevel::Error];
    
    // Stream logs
    for i in 0..100 {
        let source = sources[i % sources.len()];
        let level = levels[i % levels.len()];
        
        state.add_log(LogMessage {
            timestamp: Utc::now(),
            source: Some(source.to_string()),
            level,
            message: format!("Event {} from {}", i, source),
        }).await;
    }
    
    // Verify logs
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), 100);
    
    // Verify log sources are distributed
    let songbird_logs: Vec<_> = logs.iter()
        .filter(|log| log.source.as_ref().map(|s| s.as_str()) == Some("songbird"))
        .collect();
    assert!(!songbird_logs.is_empty());
    
    // Continue streaming (test ring buffer)
    for i in 100..1500 {
        let source = sources[i % sources.len()];
        let level = levels[i % levels.len()];
        
        state.add_log(LogMessage {
            timestamp: Utc::now(),
            source: Some(source.to_string()),
            level,
            message: format!("Event {} from {}", i, source),
        }).await;
    }
    
    // Should only keep last 1000
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), 1000);
}

/// Test view switching workflow
#[tokio::test]
async fn test_view_switching_workflow() {
    let state = TUIState::new();
    
    // Navigate through all views in order
    let views = View::all();
    
    for view in &views {
        state.set_view(*view).await;
        assert_eq!(state.get_view().await, *view);
        
        // Selection should reset on view change
        assert_eq!(state.get_selected_index().await, 0);
    }
    
    // Navigate backwards
    for view in views.iter().rev() {
        state.set_view(*view).await;
        assert_eq!(state.get_view().await, *view);
    }
}

/// Test multi-user concurrent workflow (simulating multiple terminals)
#[tokio::test]
async fn test_concurrent_multi_user_workflow() {
    let state = TUIState::new();
    
    // Simulate 3 concurrent users
    let user1_state = state.clone();
    let user2_state = state.clone();
    let user3_state = state.clone();
    
    let user1 = tokio::spawn(async move {
        // User 1: Views topology
        user1_state.set_view(View::Topology).await;
        for i in 0..50 {
            user1_state.add_log(LogMessage {
                timestamp: Utc::now(),
                source: Some("user1".to_string()),
                level: LogLevel::Info,
                message: format!("User 1 action {}", i),
            }).await;
        }
    });
    
    let user2 = tokio::spawn(async move {
        // User 2: Views logs
        user2_state.set_view(View::Logs).await;
        for i in 0..50 {
            user2_state.add_log(LogMessage {
                timestamp: Utc::now(),
                source: Some("user2".to_string()),
                level: LogLevel::Debug,
                message: format!("User 2 action {}", i),
            }).await;
        }
    });
    
    let user3 = tokio::spawn(async move {
        // User 3: Manages primals
        user3_state.set_view(View::Primals).await;
        for i in 0..50 {
            user3_state.add_log(LogMessage {
                timestamp: Utc::now(),
                source: Some("user3".to_string()),
                level: LogLevel::Warn,
                message: format!("User 3 action {}", i),
            }).await;
        }
    });
    
    user1.await.unwrap();
    user2.await.unwrap();
    user3.await.unwrap();
    
    // All logs should be present
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), 150);
}

/// Test degraded mode workflow (no primals available)
#[tokio::test]
async fn test_degraded_mode_workflow() {
    let state = TUIState::new();
    
    // Set standalone mode
    state.set_standalone_mode(true).await;
    assert!(state.is_standalone().await);
    
    // No primals
    assert_eq!(state.get_primals().await.len(), 0);
    
    // No topology
    assert_eq!(state.get_topology().await.len(), 0);
    
    // Can still navigate views
    state.set_view(View::Topology).await;
    assert_eq!(state.get_view().await, View::Topology);
    
    // Can still add logs (local logging)
    state.add_log(LogMessage {
        timestamp: Utc::now(),
        source: None,
        level: LogLevel::Info,
        message: "Standalone mode active".to_string(),
    }).await;
    
    assert_eq!(state.get_logs().await.len(), 1);
}
