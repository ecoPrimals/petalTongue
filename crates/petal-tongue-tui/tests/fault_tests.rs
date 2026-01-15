//! Fault Injection Tests for petalTongue Rich TUI
//!
//! Tests error handling and graceful degradation

use chrono::Utc;
use petal_tongue_tui::state::{LogLevel, LogMessage, TUIState, View};

mod common;
use common::{create_test_edge, create_test_primal, create_test_primal_with_health};
use petal_tongue_core::PrimalHealthStatus;

/// Test handling of empty primal list
#[tokio::test]
async fn test_fault_empty_primals() {
    let state = TUIState::new();

    // Update with empty list
    state.update_primals(vec![]).await;

    // Should handle gracefully
    let primals = state.get_primals().await;
    assert_eq!(primals.len(), 0);

    let stats = state.stats().await;
    assert_eq!(stats.primal_count, 0);
}

/// Test handling of empty topology
#[tokio::test]
async fn test_fault_empty_topology() {
    let state = TUIState::new();

    // Update with empty topology
    state.update_topology(vec![]).await;

    // Should handle gracefully
    let topology = state.get_topology().await;
    assert_eq!(topology.len(), 0);
}

/// Test handling of invalid log messages (empty content)
#[tokio::test]
async fn test_fault_empty_logs() {
    let state = TUIState::new();

    // Add log with empty message
    state
        .add_log(LogMessage {
            timestamp: Utc::now(),
            source: None,
            level: LogLevel::Info,
            message: String::new(),
        })
        .await;

    // Should store empty message
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "");
}

/// Test handling of unknown primal health status
#[tokio::test]
async fn test_fault_unknown_health_status() {
    let state = TUIState::new();

    let primals = vec![create_test_primal_with_health(
        "unknown-primal",
        "unknown-1",
        PrimalHealthStatus::Unknown,
    )];

    state.update_primals(primals).await;

    // Should handle unknown status
    let retrieved_primals = state.get_primals().await;
    assert_eq!(retrieved_primals.len(), 1);
    assert!(matches!(
        retrieved_primals[0].health,
        PrimalHealthStatus::Unknown
    ));
}

/// Test handling of critical health status
#[tokio::test]
async fn test_fault_critical_health_status() {
    let state = TUIState::new();

    let primals = vec![create_test_primal_with_health(
        "failing-primal",
        "failing-1",
        PrimalHealthStatus::Critical,
    )];

    state.update_primals(primals).await;

    // Should handle critical status
    let retrieved_primals = state.get_primals().await;
    assert_eq!(retrieved_primals.len(), 1);
    assert!(matches!(
        retrieved_primals[0].health,
        PrimalHealthStatus::Critical
    ));
}

/// Test handling of invalid topology (non-existent nodes)
#[tokio::test]
async fn test_fault_invalid_topology_references() {
    let state = TUIState::new();

    // Add primals
    let primals = vec![create_test_primal("primal-1", "primal-1")];

    state.update_primals(primals).await;

    // Add topology with non-existent nodes
    let topology = vec![
        create_test_edge("primal-1", "non-existent-primal", "discovery"),
        create_test_edge("another-missing", "primal-1", "compute"),
    ];

    state.update_topology(topology).await;

    // Should store topology even if references are invalid
    // (rendering layer will handle gracefully)
    let retrieved_topology = state.get_topology().await;
    assert_eq!(retrieved_topology.len(), 2);
}

/// Test handling of selection with zero items
#[tokio::test]
async fn test_fault_selection_zero_items() {
    let state = TUIState::new();

    // Navigate with 0 max items
    state.select_next(0).await;
    state.select_previous(0).await;

    // Should not crash, selection should stay at 0
    assert_eq!(state.get_selected_index().await, 0);
}

/// Test handling of very large log messages
#[tokio::test]
async fn test_fault_large_log_messages() {
    let state = TUIState::new();

    // Create very large log message (10MB string)
    let large_message = "X".repeat(10 * 1024 * 1024);

    state
        .add_log(LogMessage {
            timestamp: Utc::now(),
            source: Some("large-source".to_string()),
            level: LogLevel::Warn,
            message: large_message.clone(),
        })
        .await;

    // Should store large message
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message.len(), 10 * 1024 * 1024);
}

/// Test handling of rapid state changes
#[tokio::test]
async fn test_fault_rapid_state_flip() {
    let state = TUIState::new();

    // Rapidly flip standalone mode
    for _ in 0..1000 {
        state.set_standalone_mode(true).await;
        state.set_standalone_mode(false).await;
    }

    // Should end in consistent state
    assert!(!state.is_standalone().await);
}

/// Test handling of missing capabilities
#[tokio::test]
async fn test_fault_missing_capabilities() {
    let state = TUIState::new();

    // Check capabilities for non-existent primal
    assert!(!state.has_capability("non-existent", "discovery"));

    // Get capabilities for non-existent primal
    assert!(state.get_capabilities("non-existent").is_none());
}

/// Test handling of empty capability list
#[tokio::test]
async fn test_fault_empty_capability_list() {
    let state = TUIState::new();

    // Register primal with no capabilities
    state.register_capability("empty-primal".to_string(), vec![]);

    // Should handle empty list
    let caps = state.get_capabilities("empty-primal").unwrap();
    assert_eq!(caps.len(), 0);

    // Should return false for any capability
    assert!(!state.has_capability("empty-primal", "anything"));
}

/// Test handling of duplicate primals
#[tokio::test]
async fn test_fault_duplicate_primals() {
    let state = TUIState::new();

    // Add primals with duplicate IDs
    let primals = vec![
        create_test_primal("songbird", "duplicate-id"),
        create_test_primal("toadstool", "duplicate-id"),
    ];

    state.update_primals(primals).await;

    // Should store both (rendering layer handles duplicates)
    let retrieved_primals = state.get_primals().await;
    assert_eq!(retrieved_primals.len(), 2);
}

/// Test handling of circular topology
#[tokio::test]
async fn test_fault_circular_topology() {
    let state = TUIState::new();

    // Create circular topology
    let topology = vec![
        create_test_edge("primal-1", "primal-2", "discovery"),
        create_test_edge("primal-2", "primal-3", "discovery"),
        create_test_edge("primal-3", "primal-1", "discovery"),
    ];

    state.update_topology(topology).await;

    // Should store circular topology (rendering handles cycles)
    let retrieved_topology = state.get_topology().await;
    assert_eq!(retrieved_topology.len(), 3);
}

/// Test handling of log flooding (OOM protection)
#[tokio::test]
async fn test_fault_log_flooding_oom_protection() {
    let state = TUIState::new();

    // Flood with 10,000 logs
    for i in 0..10_000 {
        state
            .add_log(LogMessage {
                timestamp: Utc::now(),
                source: Some(format!("source-{}", i % 100)),
                level: LogLevel::Debug,
                message: format!("Flood log {}", i),
            })
            .await;
    }

    // Ring buffer should protect against OOM
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), 1000); // Max ring buffer size

    // Should have latest logs
    assert!(logs.last().unwrap().message.contains("9999"));
}

/// Test handling of concurrent primal updates and reads
#[tokio::test]
async fn test_fault_concurrent_primal_access() {
    let state = TUIState::new();

    // Writer task
    let write_state = state.clone();
    let writer = tokio::spawn(async move {
        for i in 0..100 {
            let primals = vec![create_test_primal(
                &format!("primal-{}", i),
                &format!("primal-{}", i),
            )];
            write_state.update_primals(primals).await;
        }
    });

    // Reader task
    let read_state = state.clone();
    let reader = tokio::spawn(async move {
        for _ in 0..100 {
            let _primals = read_state.get_primals().await;
        }
    });

    writer.await.unwrap();
    reader.await.unwrap();

    // System should be consistent
    let primals = state.get_primals().await;
    assert_eq!(primals.len(), 1);
}

/// Test handling of Unicode in log messages
#[tokio::test]
async fn test_fault_unicode_log_messages() {
    let state = TUIState::new();

    // Add logs with various Unicode
    let unicode_messages = vec![
        "Hello 世界",
        "🌸 petalTongue 🦀",
        "Ñoño español",
        "Привет мир",
        "مرحبا بالعالم",
        "🍄🐸🌸",
    ];

    for msg in &unicode_messages {
        state
            .add_log(LogMessage {
                timestamp: Utc::now(),
                source: Some("unicode-source".to_string()),
                level: LogLevel::Info,
                message: msg.to_string(),
            })
            .await;
    }

    // Should handle Unicode correctly
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), unicode_messages.len());

    for (i, log) in logs.iter().enumerate() {
        assert_eq!(log.message, unicode_messages[i]);
    }
}
