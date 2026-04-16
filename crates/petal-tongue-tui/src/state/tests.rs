// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::Utc;

use crate::state::{LogLevel, LogMessage, SystemStatus, TUIState, View};

#[tokio::test]
async fn test_view_switching() {
    let state = TUIState::new();

    assert_eq!(state.get_view().await, View::Dashboard);

    state.set_view(View::Topology).await;
    assert_eq!(state.get_view().await, View::Topology);
}

#[tokio::test]
async fn test_view_shortcuts() {
    assert_eq!(View::Dashboard.shortcut(), '1');
    assert_eq!(View::Topology.shortcut(), '2');
    assert_eq!(View::NeuralAPI.shortcut(), '6');
}

#[tokio::test]
async fn test_log_ring_buffer() {
    let state = TUIState::new();

    // Add 1100 logs
    for i in 0..1100 {
        state
            .add_log(LogMessage {
                timestamp: Utc::now(),
                source: None,
                level: LogLevel::Info,
                message: format!("Log {i}"),
            })
            .await;
    }

    // Should keep only last 1000
    let logs = state.get_logs().await;
    assert_eq!(logs.len(), 1000);
    assert!(
        logs.last()
            .expect("logs not empty")
            .message
            .contains("1099")
    );
}

#[tokio::test]
async fn test_selection_wrapping() {
    let state = TUIState::new();

    // Select next with 5 items
    state.select_next(5).await;
    assert_eq!(state.get_selected_index().await, 1);

    // Select previous
    state.select_previous(5).await;
    assert_eq!(state.get_selected_index().await, 0);

    // Wrap to bottom
    state.select_previous(5).await;
    assert_eq!(state.get_selected_index().await, 4);

    // Wrap to top
    state.select_next(5).await;
    assert_eq!(state.get_selected_index().await, 0);
}

#[tokio::test]
async fn test_capability_registration() {
    let state = TUIState::new();

    state.register_capability(
        "songbird".to_string(),
        vec!["discovery".to_string(), "events".to_string()],
    );

    assert!(state.has_capability("songbird", "discovery"));
    assert!(state.has_capability("songbird", "events"));
    assert!(!state.has_capability("songbird", "compute"));
    assert!(!state.has_capability("toadstool", "compute"));

    let caps = state.get_capabilities("songbird").unwrap();
    assert_eq!(caps.len(), 2);
}

#[tokio::test]
async fn test_standalone_mode() {
    let state = TUIState::new();

    assert!(!state.is_standalone().await);

    state.set_standalone_mode(true).await;
    assert!(state.is_standalone().await);

    state.set_standalone_mode(false).await;
    assert!(!state.is_standalone().await);
}

#[tokio::test]
async fn test_stats() {
    let state = TUIState::new();

    state.set_view(View::Primals).await;
    state.register_capability("songbird".to_string(), vec!["discovery".to_string()]);

    let stats = state.stats().await;
    assert_eq!(stats.view, View::Primals);
    assert_eq!(stats.registered_capabilities, 1);
}

#[tokio::test]
async fn test_default_state() {
    let state = TUIState::default();
    assert_eq!(state.get_view().await, View::Dashboard);
    assert_eq!(state.get_selected_index().await, 0);
    assert_eq!(state.primal_count().await, 0);
}

#[tokio::test]
async fn test_system_status_default() {
    let status = SystemStatus::default();
    assert_eq!(status.active_primals, 0);
    assert_eq!(status.discovered_devices, 0);
}

#[tokio::test]
#[expect(clippy::cast_sign_loss, reason = "test timestamps are always positive")]
async fn test_get_status() {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let state = TUIState::new();
        state
            .update_primals(vec![petal_tongue_core::PrimalInfo::new(
                "p1",
                "primal1",
                "Test",
                "unix:///tmp/p1.sock",
                vec![],
                petal_tongue_core::PrimalHealthStatus::Healthy,
                chrono::Utc::now().timestamp().max(0) as u64,
            )])
            .await;
        let status = state.get_status().await;
        assert_eq!(status.active_primals, 1);
    })
    .await
    .expect("test timed out after 5s");
}

#[tokio::test]
async fn test_get_capabilities_unknown_primal() {
    let state = TUIState::new();
    assert!(state.get_capabilities("unknown").is_none());
}

#[tokio::test]
async fn test_topology_storage() {
    let state = TUIState::new();
    let edges = vec![petal_tongue_core::TopologyEdge {
        from: "a".into(),
        to: "b".into(),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    }];
    state.update_topology(edges).await;
    assert_eq!(state.topology_edge_count().await, 1);
    let loaded = state.get_topology().await;
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].edge_type, "test");
}

#[tokio::test]
async fn test_set_selected_index() {
    let state = TUIState::new();
    state.set_selected_index(5).await;
    assert_eq!(state.get_selected_index().await, 5);
}

#[tokio::test]
async fn test_select_previous_max_zero() {
    let state = TUIState::new();
    state.set_selected_index(0).await;
    state.select_previous(0).await;
    assert_eq!(state.get_selected_index().await, 0);
}

#[tokio::test]
async fn test_select_next_max_zero() {
    let state = TUIState::new();
    state.set_selected_index(0).await;
    state.select_next(0).await;
    assert_eq!(state.get_selected_index().await, 0);
}

#[tokio::test]
async fn test_tui_stats_fields() {
    let state = TUIState::new();
    state.set_view(View::Logs).await;
    state
        .add_log(LogMessage {
            timestamp: Utc::now(),
            source: None,
            level: LogLevel::Info,
            message: "test".to_string(),
        })
        .await;
    let stats = state.stats().await;
    assert_eq!(stats.view, View::Logs);
    assert_eq!(stats.log_count, 1);
    assert_eq!(stats.topology_edge_count, 0);
}

#[test]
fn test_view_all_variants() {
    let all = View::all();
    assert!(all.contains(&View::Dashboard));
    assert!(all.contains(&View::Topology));
    assert!(all.contains(&View::Devices));
    assert!(all.contains(&View::Primals));
    assert!(all.contains(&View::Logs));
    assert!(all.contains(&View::NeuralAPI));
    assert!(all.contains(&View::Nucleus));
    assert!(all.contains(&View::LiveSpore));
}

#[test]
fn test_log_message_fields() {
    let msg = LogMessage {
        timestamp: Utc::now(),
        source: Some("src".to_string()),
        level: LogLevel::Warn,
        message: "msg".to_string(),
    };
    assert_eq!(msg.source.as_deref(), Some("src"));
    assert_eq!(msg.level, LogLevel::Warn);
    assert_eq!(msg.message, "msg");
}
