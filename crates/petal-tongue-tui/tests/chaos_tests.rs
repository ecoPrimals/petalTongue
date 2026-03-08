// SPDX-License-Identifier: AGPL-3.0-only
//! Chaos Tests for petalTongue Rich TUI
//!
//! Tests system behavior under chaotic conditions

use chrono::Utc;
use petal_tongue_tui::state::{LogLevel, LogMessage, TUIState, View};

mod common;
use common::{create_test_edge, create_test_primal};

/// Test rapid view switching under load
#[tokio::test]
async fn test_chaos_rapid_view_switching() {
    let state = TUIState::new();

    // Spawn task that rapidly switches views
    let switch_state = state.clone();
    let switcher = tokio::spawn(async move {
        for _ in 0..1000 {
            for view in View::all() {
                switch_state.set_view(view).await;
            }
        }
    });

    // Meanwhile, add logs concurrently
    let log_state = state.clone();
    let logger = tokio::spawn(async move {
        for i in 0..1000 {
            log_state
                .add_log(LogMessage {
                    timestamp: Utc::now(),
                    source: Some("chaos".to_string()),
                    level: LogLevel::Info,
                    message: format!("Chaos log {}", i),
                })
                .await;
        }
    });

    switcher.await.unwrap();
    logger.await.unwrap();

    // System should still be functional
    let stats = state.stats().await;
    assert_eq!(stats.log_count, 1000);
}

/// Test extreme concurrent access
#[tokio::test]
async fn test_chaos_extreme_concurrency() {
    let state = TUIState::new();

    // Spawn many concurrent tasks (100 tasks)
    let mut handles = vec![];

    for task_id in 0..100 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            // Each task does random operations
            for i in 0..10 {
                // Random view switch
                let view_idx = (task_id + i) % 8;
                let view = View::all()[view_idx];
                state_clone.set_view(view).await;

                // Add log
                state_clone
                    .add_log(LogMessage {
                        timestamp: Utc::now(),
                        source: Some(format!("task-{}", task_id)),
                        level: LogLevel::Debug,
                        message: format!("Task {} iteration {}", task_id, i),
                    })
                    .await;

                // Navigate selection
                state_clone.select_next(10).await;
                state_clone.select_previous(10).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // System should have 1000 logs
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), 1000);
}

/// Test rapid topology changes
#[tokio::test]
async fn test_chaos_topology_thrashing() {
    let state = TUIState::new();

    // Create initial primals
    let primals: Vec<_> = (0..10)
        .map(|i| create_test_primal(&format!("primal-{}", i), &format!("primal-{}", i)))
        .collect();

    state.update_primals(primals).await;

    // Rapidly update topology
    let topo_state = state.clone();
    let topology_updater = tokio::spawn(async move {
        for iteration in 0..100 {
            // Generate random topology
            let topology: Vec<_> = (0..10)
                .map(|i| {
                    create_test_edge(
                        &format!("primal-{}", i),
                        &format!("primal-{}", (i + 1) % 10),
                        if iteration % 2 == 0 {
                            "discovery"
                        } else {
                            "compute"
                        },
                    )
                })
                .collect();

            topo_state.update_topology(topology).await;
        }
    });

    // Meanwhile, read topology concurrently
    let read_state = state.clone();
    let topology_reader = tokio::spawn(async move {
        for _ in 0..100 {
            let _topo = read_state.get_topology().await;
            tokio::time::sleep(std::time::Duration::from_micros(10)).await;
        }
    });

    topology_updater.await.unwrap();
    topology_reader.await.unwrap();

    // System should still be functional
    let topology = state.get_topology().await;
    assert_eq!(topology.len(), 10);
}

/// Test log overflow stress
#[tokio::test]
async fn test_chaos_log_overflow_stress() {
    let state = TUIState::new();

    // Multiple tasks flood logs
    let mut handles = vec![];

    for task_id in 0..20 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for i in 0..500 {
                state_clone
                    .add_log(LogMessage {
                        timestamp: Utc::now(),
                        source: Some(format!("task-{}", task_id)),
                        level: if i % 5 == 0 {
                            LogLevel::Error
                        } else {
                            LogLevel::Info
                        },
                        message: format!("Task {} log {}", task_id, i),
                    })
                    .await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Should only keep last 1000 logs (ring buffer)
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), 1000);
}

/// Test capability registration chaos
#[tokio::test]
async fn test_chaos_capability_thrashing() {
    let state = TUIState::new();

    // Multiple tasks register capabilities concurrently
    let mut handles = vec![];

    for primal_id in 0..50 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for iteration in 0..20 {
                let capabilities = vec![
                    format!("capability-{}-{}", primal_id, iteration),
                    format!("capability-{}-{}", primal_id, iteration + 1),
                ];
                state_clone.register_capability(format!("primal-{}", primal_id), capabilities);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // System should have registered capabilities
    let stats = state.stats().await;
    assert_eq!(stats.registered_capabilities, 50);
}

/// Test mixed operations chaos
#[tokio::test]
async fn test_chaos_mixed_operations() {
    let state = TUIState::new();

    // Task 1: View switching
    let view_state = state.clone();
    let view_task = tokio::spawn(async move {
        for _ in 0..200 {
            for view in View::all() {
                view_state.set_view(view).await;
            }
        }
    });

    // Task 2: Log flooding
    let log_state = state.clone();
    let log_task = tokio::spawn(async move {
        for i in 0..2000 {
            log_state
                .add_log(LogMessage {
                    timestamp: Utc::now(),
                    source: Some("chaos".to_string()),
                    level: LogLevel::Info,
                    message: format!("Log {}", i),
                })
                .await;
        }
    });

    // Task 3: Primal updates
    let primal_state = state.clone();
    let primal_task = tokio::spawn(async move {
        for iteration in 0..100 {
            let primals: Vec<_> = (0..5)
                .map(|i| {
                    create_test_primal(
                        &format!("primal-{}", i),
                        &format!("primal-{}-{}", i, iteration),
                    )
                })
                .collect();
            primal_state.update_primals(primals).await;
        }
    });

    // Task 4: Selection navigation
    let nav_state = state.clone();
    let nav_task = tokio::spawn(async move {
        for _ in 0..1000 {
            nav_state.select_next(20).await;
            nav_state.select_previous(20).await;
        }
    });

    view_task.await.unwrap();
    log_task.await.unwrap();
    primal_task.await.unwrap();
    nav_task.await.unwrap();

    // System should still be functional
    let stats = state.stats().await;
    assert_eq!(stats.log_count, 1000); // Ring buffer limit
    assert_eq!(stats.primal_count, 5);
}
