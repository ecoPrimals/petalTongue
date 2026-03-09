// SPDX-License-Identifier: AGPL-3.0-only
//! System Dashboard - Unit tests

use std::time::{Duration, Instant};

use super::state::SystemDashboard;

#[test]
fn test_dashboard_creation() {
    let dashboard = SystemDashboard::default();
    assert_eq!(dashboard.max_history, 30);
}

#[test]
fn test_refresh_updates_metrics() {
    let mut dashboard = SystemDashboard::default();
    // Force refresh interval to zero so no sleep is needed
    dashboard.refresh_interval = Duration::ZERO;
    dashboard.last_refresh = Instant::now() - Duration::from_secs(10);
    let initial_count = dashboard.cpu_history.len();

    dashboard.refresh(None);

    assert!(dashboard.cpu_history.len() > initial_count);
}

#[test]
fn test_audio_toggle() {
    let mut dashboard = SystemDashboard::default();
    assert!(!dashboard.is_audio_enabled());
    dashboard.set_audio_enabled(true);
    assert!(dashboard.is_audio_enabled());
    dashboard.set_audio_enabled(false);
    assert!(!dashboard.is_audio_enabled());
}

#[test]
fn test_audio_volume_clamping() {
    let mut dashboard = SystemDashboard::default();
    dashboard.set_audio_volume(0.5);
    assert_eq!(dashboard.modality_prefs_mut().audio_volume, 0.5);
    dashboard.set_audio_volume(2.0);
    assert_eq!(dashboard.modality_prefs_mut().audio_volume, 1.0);
    dashboard.set_audio_volume(-0.5);
    assert_eq!(dashboard.modality_prefs_mut().audio_volume, 0.0);
}

#[test]
fn test_modality_prefs_mut() {
    let mut dashboard = SystemDashboard::default();
    let prefs = dashboard.modality_prefs_mut();
    prefs.audio_enabled = true;
    prefs.audio_volume = 0.8;
    assert!(dashboard.is_audio_enabled());
    assert_eq!(dashboard.modality_prefs_mut().audio_volume, 0.8);
}

#[test]
fn test_dashboard_default() {
    let dashboard = SystemDashboard::default();
    assert_eq!(dashboard.max_history, 30);
    assert!(!dashboard.is_audio_enabled());
}

#[test]
fn test_refresh_with_empty_cpus() {
    let mut dashboard = SystemDashboard::default();
    dashboard.refresh_interval = Duration::ZERO;
    dashboard.last_refresh = Instant::now() - Duration::from_secs(10);
    // Refresh with no audio - exercises the refresh path
    dashboard.refresh(None);
    // Should not panic; cpu_history may or may not have data
    assert!(dashboard.cpu_history.len() <= dashboard.max_history);
}
