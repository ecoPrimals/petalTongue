// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use super::*;
use crate::biomeos_integration::Health;
use crate::ui_events::UIEvent;
use egui::Color32;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_primal_panel_creation() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let panel = PrimalPanel::new(event_handler);

    assert_eq!(panel.primals.len(), 0);
    assert_eq!(panel.filter, PrimalFilter::All);
    assert!(panel.search_query.is_empty());
}

#[tokio::test]
async fn test_primal_panel_refresh() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler);

    let primals = vec![
        Primal {
            id: "primal-1".to_string(),
            name: "Test Primal 1".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
        Primal {
            id: "primal-2".to_string(),
            name: "Test Primal 2".to_string(),
            health: Health::Healthy,
            capabilities: vec!["storage".to_string()],
            load: 0.3,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
    ];

    panel.refresh(primals).await;

    assert_eq!(panel.primals.len(), 2);
}

#[tokio::test]
async fn test_primal_panel_event_processing() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler.clone());

    panel
        .refresh(vec![Primal {
            id: "primal-1".to_string(),
            name: "Test Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        }])
        .await;

    event_handler
        .write()
        .await
        .handle_event(UIEvent::PrimalHealthChanged(
            "primal-1".to_string(),
            Health::Degraded,
        ))
        .await;

    panel.process_events().await;

    assert_eq!(panel.primals[0].health, Health::Degraded);
}

#[tokio::test]
async fn test_primal_panel_filtering() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler);

    let primals = vec![
        Primal {
            id: "primal-1".to_string(),
            name: "Healthy Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
        Primal {
            id: "primal-2".to_string(),
            name: "Degraded Primal".to_string(),
            health: Health::Degraded,
            capabilities: vec!["storage".to_string()],
            load: 0.8,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
        Primal {
            id: "primal-3".to_string(),
            name: "Offline Primal".to_string(),
            health: Health::Offline,
            capabilities: vec!["network".to_string()],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
    ];

    panel.refresh(primals).await;

    panel.filter = PrimalFilter::All;
    assert_eq!(panel.filtered_primals().len(), 3);

    panel.filter = PrimalFilter::Healthy;
    assert_eq!(panel.filtered_primals().len(), 1);
    assert_eq!(panel.filtered_primals()[0].name, "Healthy Primal");

    panel.filter = PrimalFilter::Degraded;
    assert_eq!(panel.filtered_primals().len(), 1);
    assert_eq!(panel.filtered_primals()[0].name, "Degraded Primal");
}

#[tokio::test]
async fn test_primal_panel_search() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler);

    let primals = vec![
        Primal {
            id: "primal-1".to_string(),
            name: "Beardog".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.5,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
        Primal {
            id: "primal-2".to_string(),
            name: "Songbird".to_string(),
            health: Health::Healthy,
            capabilities: vec!["orchestration".to_string()],
            load: 0.3,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
    ];

    panel.refresh(primals).await;

    panel.search_query = "bear".to_string();
    assert_eq!(panel.filtered_primals().len(), 1);
    assert_eq!(panel.filtered_primals()[0].name, "Beardog");

    panel.search_query = "orchestration".to_string();
    assert_eq!(panel.filtered_primals().len(), 1);
    assert_eq!(panel.filtered_primals()[0].name, "Songbird");

    panel.search_query = String::new();
    assert_eq!(panel.filtered_primals().len(), 2);
}

#[tokio::test]
async fn test_primal_panel_selection() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler);

    let primals = vec![Primal {
        id: "primal-1".to_string(),
        name: "Test Primal".to_string(),
        health: Health::Healthy,
        capabilities: vec!["compute".to_string()],
        load: 0.5,
        assigned_devices: vec![],
        metadata: serde_json::json!({}),
    }];

    panel.refresh(primals).await;

    assert!(panel.selected_primal().is_none());

    panel.selected = Some("primal-1".to_string());
    assert!(panel.selected_primal().is_some());
    assert_eq!(
        panel.selected_primal().expect("selected").name,
        "Test Primal"
    );
}

#[tokio::test]
async fn test_primal_panel_search_by_id() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler);

    let primals = vec![
        Primal {
            id: "primal-abc-123".to_string(),
            name: "Alpha".to_string(),
            health: Health::Healthy,
            capabilities: vec![],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
        Primal {
            id: "primal-xyz-456".to_string(),
            name: "Beta".to_string(),
            health: Health::Healthy,
            capabilities: vec![],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
    ];
    panel.refresh(primals).await;

    panel.search_query = "xyz".to_string();
    let filtered = panel.filtered_primals();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "primal-xyz-456");
}

#[tokio::test]
async fn test_primal_panel_primal_removed_clears_selection() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler.clone());

    panel
        .refresh(vec![Primal {
            id: "p1".to_string(),
            name: "P1".to_string(),
            health: Health::Healthy,
            capabilities: vec![],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        }])
        .await;
    panel.selected = Some("p1".to_string());

    event_handler
        .write()
        .await
        .handle_event(UIEvent::PrimalRemoved("p1".to_string()))
        .await;
    panel.process_events().await;

    assert!(panel.selected.is_none());
    assert!(panel.primals.is_empty());
}

#[tokio::test]
async fn test_primal_panel_load_changed() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler.clone());

    panel
        .refresh(vec![Primal {
            id: "p1".to_string(),
            name: "P1".to_string(),
            health: Health::Healthy,
            capabilities: vec![],
            load: 0.3,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        }])
        .await;

    event_handler
        .write()
        .await
        .handle_event(UIEvent::PrimalLoadChanged("p1".to_string(), 0.9))
        .await;
    panel.process_events().await;

    assert!((panel.primals[0].load - 0.9).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_primal_panel_primal_discovered() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler.clone());

    event_handler
        .write()
        .await
        .handle_event(UIEvent::PrimalDiscovered(Primal {
            id: "new".to_string(),
            name: "New Primal".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        }))
        .await;
    panel.process_events().await;

    assert_eq!(panel.primals.len(), 1);
    assert_eq!(panel.primals[0].name, "New Primal");
}

#[tokio::test]
async fn test_compute_primal_stats() {
    let primals = vec![
        Primal {
            id: "h1".to_string(),
            name: "H1".to_string(),
            health: Health::Healthy,
            capabilities: vec![],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
        Primal {
            id: "h2".to_string(),
            name: "H2".to_string(),
            health: Health::Healthy,
            capabilities: vec![],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
        Primal {
            id: "d1".to_string(),
            name: "D1".to_string(),
            health: Health::Degraded,
            capabilities: vec![],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
        Primal {
            id: "o1".to_string(),
            name: "O1".to_string(),
            health: Health::Offline,
            capabilities: vec![],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
    ];
    let (total, healthy, degraded, error) = PrimalPanel::compute_primal_stats(&primals);
    assert_eq!(total, 4);
    assert_eq!(healthy, 2);
    assert_eq!(degraded, 1);
    assert_eq!(error, 1);
}

#[tokio::test]
async fn test_compute_primal_stats_empty() {
    let (total, healthy, degraded, error) = PrimalPanel::compute_primal_stats(&[]);
    assert_eq!(total, 0);
    assert_eq!(healthy, 0);
    assert_eq!(degraded, 0);
    assert_eq!(error, 0);
}

#[test]
fn test_load_bar_color() {
    assert_eq!(PrimalPanel::load_bar_color(0.0), Color32::GREEN);
    assert_eq!(PrimalPanel::load_bar_color(0.5), Color32::GREEN);
    assert_eq!(PrimalPanel::load_bar_color(0.7), Color32::GREEN);
    assert_eq!(PrimalPanel::load_bar_color(0.71), Color32::YELLOW);
    assert_eq!(PrimalPanel::load_bar_color(0.9), Color32::YELLOW);
    assert_eq!(PrimalPanel::load_bar_color(0.91), Color32::RED);
    assert_eq!(PrimalPanel::load_bar_color(1.0), Color32::RED);
}

#[test]
fn test_load_bar_color_rgb() {
    assert_eq!(load_bar_color_rgb(0.0), [0, 255, 0]);
    assert_eq!(load_bar_color_rgb(0.71), [255, 255, 0]);
    assert_eq!(load_bar_color_rgb(0.91), [255, 0, 0]);
}

#[test]
fn test_health_display_data() {
    let (text, rgb) = health_display_data(&Health::Healthy);
    assert_eq!(text, "● Healthy");
    assert_eq!(rgb, [0, 255, 0]);
    let (text, rgb) = health_display_data(&Health::Degraded);
    assert_eq!(text, "● Degraded");
    assert_eq!(rgb, [255, 255, 0]);
    let (text, rgb) = health_display_data(&Health::Offline);
    assert_eq!(text, "● Offline");
    assert_eq!(rgb, [255, 0, 0]);
}

#[tokio::test]
async fn test_primal_filter_all_includes_offline() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = PrimalPanel::new(event_handler);

    let primals = vec![
        Primal {
            id: "h".to_string(),
            name: "Healthy".to_string(),
            health: Health::Healthy,
            capabilities: vec![],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
        Primal {
            id: "o".to_string(),
            name: "Offline".to_string(),
            health: Health::Offline,
            capabilities: vec![],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        },
    ];
    panel.refresh(primals).await;
    panel.filter = PrimalFilter::All;

    assert_eq!(panel.filtered_primals().len(), 2);
}
