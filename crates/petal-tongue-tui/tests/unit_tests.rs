//! Unit Tests for petalTongue Rich TUI
//!
//! Comprehensive unit testing for all TUI components.

use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use petal_tongue_tui::state::{LogLevel, LogMessage, TUIState, View};
use std::time::Duration;

mod common;

/// Test suite for TUI state management
mod state_tests {
    use super::*;

    #[tokio::test]
    async fn test_view_navigation() {
        let state = TUIState::new();

        // Test initial state
        assert_eq!(state.get_view().await, View::Dashboard);

        // Test all view switches
        for view in View::all() {
            state.set_view(view).await;
            assert_eq!(state.get_view().await, view);
            // Selection should reset on view change
            assert_eq!(state.get_selected_index().await, 0);
        }
    }

    #[tokio::test]
    async fn test_selection_navigation() {
        let state = TUIState::new();

        // Test with 10 items
        let max = 10;

        // Move down through all items
        for i in 0..max {
            assert_eq!(state.get_selected_index().await, i);
            state.select_next(max).await;
        }

        // Should wrap to 0
        assert_eq!(state.get_selected_index().await, 0);

        // Move up (should wrap to last)
        state.select_previous(max).await;
        assert_eq!(state.get_selected_index().await, max - 1);
    }

    #[tokio::test]
    async fn test_log_management() {
        let state = TUIState::new();

        // Add logs
        for i in 0..50 {
            state
                .add_log(LogMessage {
                    timestamp: Utc::now(),
                    source: Some(format!("primal-{}", i)),
                    level: LogLevel::Info,
                    message: format!("Test message {}", i),
                })
                .await;
        }

        let logs = state.get_logs().await;
        assert_eq!(logs.len(), 50);

        // Test log levels
        state
            .add_log(LogMessage {
                timestamp: Utc::now(),
                source: None,
                level: LogLevel::Error,
                message: "Error message".to_string(),
            })
            .await;

        let logs = state.get_logs().await;
        assert!(logs.last().unwrap().level == LogLevel::Error);
    }

    #[tokio::test]
    async fn test_log_ring_buffer_overflow() {
        let state = TUIState::new();

        // Add more than buffer size (1000)
        for i in 0..1500 {
            state
                .add_log(LogMessage {
                    timestamp: Utc::now(),
                    source: None,
                    level: LogLevel::Debug,
                    message: format!("Log {}", i),
                })
                .await;
        }

        // Should only keep last 1000
        let logs = state.get_logs().await;
        assert_eq!(logs.len(), 1000);

        // Should have latest logs
        assert!(logs.last().unwrap().message.contains("1499"));
        assert!(logs.first().unwrap().message.contains("500"));
    }

    #[tokio::test]
    async fn test_capability_management() {
        let state = TUIState::new();

        // Register capabilities
        state.register_capability(
            "songbird".to_string(),
            vec![
                "discovery".to_string(),
                "events".to_string(),
                "topology".to_string(),
            ],
        );

        state.register_capability(
            "toadstool".to_string(),
            vec!["compute".to_string(), "gpu".to_string()],
        );

        // Test capability checks
        assert!(state.has_capability("songbird", "discovery"));
        assert!(state.has_capability("songbird", "topology"));
        assert!(!state.has_capability("songbird", "compute"));

        assert!(state.has_capability("toadstool", "gpu"));
        assert!(!state.has_capability("toadstool", "discovery"));

        assert!(!state.has_capability("beardog", "auth"));

        // Test getting capabilities
        let songbird_caps = state.get_capabilities("songbird").unwrap();
        assert_eq!(songbird_caps.len(), 3);
        assert!(songbird_caps.contains(&"discovery".to_string()));
    }

    #[tokio::test]
    async fn test_standalone_mode_detection() {
        let state = TUIState::new();

        assert!(!state.is_standalone().await);

        state.set_standalone_mode(true).await;
        assert!(state.is_standalone().await);

        state.set_standalone_mode(false).await;
        assert!(!state.is_standalone().await);
    }

    #[tokio::test]
    async fn test_concurrent_state_access() {
        let state = TUIState::new();
        let state_clone = state.clone();

        // Spawn concurrent tasks
        let handle1 = tokio::spawn(async move {
            for i in 0..100 {
                state_clone
                    .add_log(LogMessage {
                        timestamp: Utc::now(),
                        source: Some("task1".to_string()),
                        level: LogLevel::Info,
                        message: format!("Message {}", i),
                    })
                    .await;
            }
        });

        let state_clone2 = state.clone();
        let handle2 = tokio::spawn(async move {
            for i in 0..100 {
                state_clone2
                    .add_log(LogMessage {
                        timestamp: Utc::now(),
                        source: Some("task2".to_string()),
                        level: LogLevel::Debug,
                        message: format!("Message {}", i),
                    })
                    .await;
            }
        });

        handle1.await.unwrap();
        handle2.await.unwrap();

        // Should have 200 logs total
        let logs = state.get_logs().await;
        assert_eq!(logs.len(), 200);
    }

    #[tokio::test]
    async fn test_statistics() {
        let state = TUIState::new();

        state.set_view(View::Topology).await;
        state.register_capability("songbird".to_string(), vec!["discovery".to_string()]);
        state.register_capability("toadstool".to_string(), vec!["compute".to_string()]);

        let stats = state.stats().await;
        assert_eq!(stats.view, View::Topology);
        assert_eq!(stats.registered_capabilities, 2);
        assert_eq!(stats.log_count, 0);
    }
}

/// Test suite for event handling
mod event_tests {
    use super::*;

    #[test]
    fn test_event_handler_creation() {
        // Event handler creation requires tick_rate parameter
        let _handler = petal_tongue_tui::events::EventHandler::new(Duration::from_millis(250));
        // If we reach here without panic, creation succeeded
    }
}

/// Test suite for View enum
mod view_tests {
    use super::*;

    #[test]
    fn test_view_shortcuts() {
        assert_eq!(View::Dashboard.shortcut(), '1');
        assert_eq!(View::Topology.shortcut(), '2');
        assert_eq!(View::Devices.shortcut(), '3');
        assert_eq!(View::Primals.shortcut(), '4');
        assert_eq!(View::Logs.shortcut(), '5');
        assert_eq!(View::NeuralAPI.shortcut(), '6');
        assert_eq!(View::Nucleus.shortcut(), '7');
        assert_eq!(View::LiveSpore.shortcut(), '8');
    }

    #[test]
    fn test_view_names() {
        assert_eq!(View::Dashboard.name(), "Dashboard");
        assert_eq!(View::Topology.name(), "Topology");
        assert_eq!(View::NeuralAPI.name(), "neuralAPI");
        assert_eq!(View::Nucleus.name(), "NUCLEUS");
        assert_eq!(View::LiveSpore.name(), "LiveSpore");
    }

    #[test]
    fn test_view_all() {
        let all_views = View::all();
        assert_eq!(all_views.len(), 8);
        assert!(all_views.contains(&View::Dashboard));
        assert!(all_views.contains(&View::LiveSpore));
    }
}

/// Test suite for LogLevel
mod log_level_tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        // Ensure levels can be compared
        assert_eq!(LogLevel::Error, LogLevel::Error);
        assert_ne!(LogLevel::Error, LogLevel::Warn);
    }

    #[test]
    fn test_log_level_clone() {
        let level = LogLevel::Info;
        let cloned = level;
        assert_eq!(level, cloned);
    }
}
