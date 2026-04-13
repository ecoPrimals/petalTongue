// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use petal_tongue_core::constants;

#[tokio::test]
async fn test_discovered_display_creation() {
    let display = DiscoveredDisplayBackend::with_socket(constants::biomeos_legacy_socket());
    assert_eq!(display.name(), "Discovered Display (via biomeOS)");
    assert_eq!(display.dimensions(), (1920, 1080));
}

#[test]
fn test_discovered_display_capabilities() {
    let display = DiscoveredDisplayBackend::with_socket(constants::biomeos_legacy_socket());
    let caps = display.capabilities();
    assert!(!caps.requires_network); // Unix socket is local
    assert!(!caps.requires_gpu); // provider may use GPU server-side
    assert!(!caps.requires_root);
    assert!(!caps.requires_display_server); // Direct DRM
    assert!(caps.remote_capable);
    assert!(caps.supports_resize);
}

#[test]
fn test_socket_discovery() {
    let _display = DiscoveredDisplayBackend::new();
}

#[test]
fn test_expected_rgba8_buffer_size() {
    assert_eq!(expected_rgba8_buffer_size(1920, 1080), 1920 * 1080 * 4);
    assert_eq!(expected_rgba8_buffer_size(640, 480), 640 * 480 * 4);
    assert_eq!(expected_rgba8_buffer_size(100, 100), 40_000);
    assert_eq!(expected_rgba8_buffer_size(1, 1), 4);
}

#[test]
fn test_with_socket_custom_path() {
    let display = DiscoveredDisplayBackend::with_socket("/tmp/custom.sock");
    assert_eq!(display.dimensions(), (1920, 1080));
}

#[test]
fn test_is_available() {
    let available = DiscoveredDisplayBackend::is_available();
    let _ = available;
}

#[test]
fn test_name() {
    let display = DiscoveredDisplayBackend::with_socket("/tmp/test.sock");
    assert!(display.name().contains("Discovered"));
    assert!(display.name().contains("biomeOS"));
}

#[test]
fn test_capabilities_values() {
    let display = DiscoveredDisplayBackend::with_socket(constants::biomeos_legacy_socket());
    let caps = display.capabilities();
    assert_eq!(caps.max_fps, 60);
    assert_eq!(caps.latency_ms, 10);
    assert!(caps.supports_resize);
}

#[tokio::test]
async fn test_present_invalid_buffer_size() {
    let mut display = DiscoveredDisplayBackend::with_socket("/tmp/nonexistent.sock");
    let wrong_size_buffer = vec![0u8; 100];
    let result = display.present(&wrong_size_buffer).await;
    assert!(result.is_err());
    if let Err(e) = result {
        let msg = format!("{e:?}");
        assert!(
            msg.contains("InvalidBufferSize") || msg.contains("buffer") || msg.contains("expected"),
            "expected buffer size error, got: {msg}"
        );
    }
}

#[tokio::test]
async fn test_present_buffer_too_small() {
    let mut display = DiscoveredDisplayBackend::with_socket(constants::biomeos_legacy_socket());
    let expected = expected_rgba8_buffer_size(1920, 1080);
    let too_small = vec![0u8; expected - 1];
    let result = display.present(&too_small).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_present_buffer_too_large() {
    let mut display = DiscoveredDisplayBackend::with_socket(constants::biomeos_legacy_socket());
    let expected = expected_rgba8_buffer_size(1920, 1080);
    let too_large = vec![0u8; expected + 1000];
    let result = display.present(&too_large).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_shutdown_no_window() {
    let mut display = DiscoveredDisplayBackend::with_socket(constants::biomeos_legacy_socket());
    let result = display.shutdown().await;
    assert!(result.is_ok());
}

#[test]
fn test_discover_biomeos_socket_env() {
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
        "BIOMEOS_SOCKET",
        "/tmp/biomeos-test.sock",
        || {
            if let Ok(display) = DiscoveredDisplayBackend::new() {
                assert_eq!(display.dimensions(), (1920, 1080));
            }
        },
    );
}

#[test]
fn test_expected_rgba8_edge_cases() {
    assert_eq!(expected_rgba8_buffer_size(0, 0), 0);
    assert_eq!(expected_rgba8_buffer_size(1, 2), 8);
    assert_eq!(expected_rgba8_buffer_size(800, 600), 1_920_000);
}
