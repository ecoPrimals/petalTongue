// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use super::DevicePanel;
use super::test_fixtures::{make_device, make_device_assigned};
use crate::biomeos_integration::{DeviceStatus, DeviceType};
use crate::ui_events::{UIEvent, UIEventHandler};
use egui::Color32;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_device_panel_device_removed_clears_selection() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler.clone());

    panel
        .refresh(vec![make_device(
            "gpu-0",
            "GPU",
            DeviceType::GPU,
            DeviceStatus::Online,
            0.5,
        )])
        .await;
    panel.selected = Some("gpu-0".to_string());

    event_handler
        .write()
        .await
        .handle_event(UIEvent::DeviceRemoved("gpu-0".to_string()))
        .await;
    panel.process_events().await;

    assert!(panel.selected.is_none());
    assert!(panel.devices.is_empty());
}

#[tokio::test]
async fn test_device_panel_device_assigned_event() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler.clone());

    panel
        .refresh(vec![make_device(
            "cpu-0",
            "CPU",
            DeviceType::CPU,
            DeviceStatus::Online,
            0.0,
        )])
        .await;

    event_handler
        .write()
        .await
        .handle_event(UIEvent::DeviceAssigned(
            "cpu-0".to_string(),
            "primal-x".to_string(),
        ))
        .await;
    panel.process_events().await;

    assert_eq!(panel.devices[0].assigned_to, Some("primal-x".to_string()));
}

#[tokio::test]
async fn test_device_panel_device_unassigned_event() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler.clone());

    panel
        .refresh(vec![make_device_assigned(
            "cpu-0",
            "CPU",
            DeviceType::CPU,
            DeviceStatus::Online,
            0.0,
            "primal-x",
        )])
        .await;

    event_handler
        .write()
        .await
        .handle_event(UIEvent::DeviceUnassigned(
            "cpu-0".to_string(),
            "primal-x".to_string(),
        ))
        .await;
    panel.process_events().await;

    assert!(panel.devices[0].assigned_to.is_none());
}

#[tokio::test]
async fn test_device_panel_device_usage_changed_event() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler.clone());

    panel
        .refresh(vec![make_device(
            "gpu-0",
            "GPU",
            DeviceType::GPU,
            DeviceStatus::Online,
            0.3,
        )])
        .await;

    event_handler
        .write()
        .await
        .handle_event(UIEvent::DeviceUsageChanged("gpu-0".to_string(), 0.95))
        .await;
    panel.process_events().await;

    assert!((panel.devices[0].resource_usage - 0.95).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_device_panel_search_by_id() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler);

    panel
        .refresh(vec![make_device(
            "gpu-nvidia-0",
            "Graphics",
            DeviceType::GPU,
            DeviceStatus::Online,
            0.5,
        )])
        .await;

    panel.search_query = "nvidia".to_string();
    assert_eq!(super::list_view::filtered_devices(&panel).len(), 1);
    assert_eq!(
        super::list_view::filtered_devices(&panel)[0].id,
        "gpu-nvidia-0"
    );
}

#[test]
fn test_compute_device_stats() {
    let devices = vec![
        make_device_assigned("d1", "D1", DeviceType::GPU, DeviceStatus::Online, 0.5, "p1"),
        make_device("d2", "D2", DeviceType::CPU, DeviceStatus::Offline, 0.0),
        make_device("d3", "D3", DeviceType::Storage, DeviceStatus::Online, 0.3),
    ];
    let (total, online, assigned) = DevicePanel::compute_device_stats(&devices);
    assert_eq!(total, 3);
    assert_eq!(online, 2);
    assert_eq!(assigned, 1);
}

#[test]
fn test_compute_device_stats_empty() {
    let (total, online, assigned) = DevicePanel::compute_device_stats(&[]);
    assert_eq!(total, 0);
    assert_eq!(online, 0);
    assert_eq!(assigned, 0);
}

#[test]
fn test_usage_bar_color() {
    assert_eq!(DevicePanel::usage_bar_color(0.0), Color32::GREEN);
    assert_eq!(DevicePanel::usage_bar_color(0.7), Color32::GREEN);
    assert_eq!(DevicePanel::usage_bar_color(0.71), Color32::YELLOW);
    assert_eq!(DevicePanel::usage_bar_color(0.9), Color32::YELLOW);
    assert_eq!(DevicePanel::usage_bar_color(0.91), Color32::RED);
    assert_eq!(DevicePanel::usage_bar_color(1.0), Color32::RED);
}

#[tokio::test]
async fn test_device_panel_ui_headless_empty() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            panel.ui(ui);
        });
    });
}

#[tokio::test]
async fn test_device_panel_ui_headless_with_devices() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler);
    panel
        .refresh(vec![
            make_device("gpu-0", "GPU", DeviceType::GPU, DeviceStatus::Online, 0.5),
            make_device_assigned(
                "cpu-0",
                "CPU",
                DeviceType::CPU,
                DeviceStatus::Busy,
                0.9,
                "primal-1",
            ),
        ])
        .await;

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            panel.ui(ui);
        });
    });
}

#[tokio::test]
async fn test_device_panel_ui_headless_all_statuses() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler);
    panel
        .refresh(vec![
            make_device("d1", "Online", DeviceType::GPU, DeviceStatus::Online, 0.3),
            make_device("d2", "Offline", DeviceType::CPU, DeviceStatus::Offline, 0.0),
            make_device("d3", "Error", DeviceType::Storage, DeviceStatus::Error, 1.0),
        ])
        .await;

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            panel.ui(ui);
        });
    });
}
