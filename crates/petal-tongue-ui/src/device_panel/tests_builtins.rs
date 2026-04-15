// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use super::test_fixtures::{make_device, make_device_assigned};
use super::{DeviceFilter, DevicePanel, device_icon};
use crate::biomeos_integration::{DeviceStatus, DeviceType};
use crate::ui_events::{UIEvent, UIEventHandler};
use tokio::sync::RwLock;

#[tokio::test]
async fn test_device_panel_creation() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let panel = DevicePanel::new(event_handler);

    assert_eq!(panel.devices.len(), 0);
    assert_eq!(panel.filter, DeviceFilter::All);
    assert!(panel.search_query.is_empty());
}

#[tokio::test]
async fn test_device_panel_refresh() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler);

    let devices = vec![
        make_device(
            "gpu-0",
            "Test GPU",
            DeviceType::GPU,
            DeviceStatus::Online,
            0.5,
        ),
        make_device(
            "cpu-0",
            "Test CPU",
            DeviceType::CPU,
            DeviceStatus::Online,
            0.3,
        ),
    ];

    panel.refresh(devices).await;

    assert_eq!(panel.devices.len(), 2);
}

#[tokio::test]
async fn test_device_panel_event_processing() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler.clone());

    panel
        .refresh(vec![make_device(
            "gpu-0",
            "Test GPU",
            DeviceType::GPU,
            DeviceStatus::Online,
            0.5,
        )])
        .await;

    event_handler
        .write()
        .await
        .handle_event(UIEvent::DeviceStatusChanged(
            "gpu-0".to_string(),
            DeviceStatus::Busy,
        ))
        .await;

    panel.process_events().await;

    assert_eq!(panel.devices[0].status, DeviceStatus::Busy);
}

#[tokio::test]
async fn test_device_panel_filtering() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler);

    let devices = vec![
        make_device_assigned(
            "gpu-0",
            "Test GPU",
            DeviceType::GPU,
            DeviceStatus::Online,
            0.5,
            "primal-1",
        ),
        make_device(
            "cpu-0",
            "Test CPU",
            DeviceType::CPU,
            DeviceStatus::Online,
            0.3,
        ),
    ];

    panel.refresh(devices).await;

    panel.filter = DeviceFilter::All;
    assert_eq!(super::list_view::filtered_devices(&panel).len(), 2);

    panel.filter = DeviceFilter::Available;
    assert_eq!(super::list_view::filtered_devices(&panel).len(), 1);
    assert_eq!(super::list_view::filtered_devices(&panel)[0].id, "cpu-0");

    panel.filter = DeviceFilter::Assigned;
    assert_eq!(super::list_view::filtered_devices(&panel).len(), 1);
    assert_eq!(super::list_view::filtered_devices(&panel)[0].id, "gpu-0");
}

#[tokio::test]
async fn test_device_panel_search() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler);

    let devices = vec![
        make_device(
            "gpu-0",
            "NVIDIA GPU",
            DeviceType::GPU,
            DeviceStatus::Online,
            0.5,
        ),
        make_device(
            "cpu-0",
            "AMD CPU",
            DeviceType::CPU,
            DeviceStatus::Online,
            0.3,
        ),
    ];

    panel.refresh(devices).await;

    panel.search_query = "nvidia".to_string();
    assert_eq!(super::list_view::filtered_devices(&panel).len(), 1);
    assert_eq!(
        super::list_view::filtered_devices(&panel)[0].name,
        "NVIDIA GPU"
    );

    panel.search_query = "cpu".to_string();
    assert_eq!(super::list_view::filtered_devices(&panel).len(), 1);
    assert_eq!(
        super::list_view::filtered_devices(&panel)[0].name,
        "AMD CPU"
    );

    panel.search_query = String::new();
    assert_eq!(super::list_view::filtered_devices(&panel).len(), 2);
}

#[test]
fn test_device_icon() {
    assert_eq!(device_icon(DeviceType::GPU), "🎮");
    assert_eq!(device_icon(DeviceType::CPU), "🧠");
    assert_eq!(device_icon(DeviceType::Storage), "💾");
    assert_eq!(device_icon(DeviceType::Network), "🌐");
    assert_eq!(device_icon(DeviceType::Memory), "🔲");
    assert_eq!(device_icon(DeviceType::Other), "❓");
}

#[tokio::test]
async fn test_device_panel_selected_device() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut panel = DevicePanel::new(event_handler);

    let devices = vec![make_device(
        "dev-1",
        "Device 1",
        DeviceType::GPU,
        DeviceStatus::Online,
        0.5,
    )];
    panel.refresh(devices).await;
    assert!(panel.selected_device().is_none());

    panel.selected = Some("dev-1".to_string());
    let selected = panel.selected_device().expect("selected");
    assert_eq!(selected.id, "dev-1");
}

#[tokio::test]
async fn test_device_panel_device_filter_variants() {
    assert_eq!(DeviceFilter::All, DeviceFilter::All);
    assert_ne!(DeviceFilter::All, DeviceFilter::Available);
    assert_ne!(DeviceFilter::Available, DeviceFilter::Assigned);
}
