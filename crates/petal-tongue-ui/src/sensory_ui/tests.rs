// SPDX-License-Identifier: AGPL-3.0-only
//! Unit tests for sensory UI.

use super::*;
use petal_tongue_core::PrimalInfo;
use petal_tongue_core::SensoryCapabilities;
use petal_tongue_core::sensory_capabilities::{
    AudioOutputCapability, KeyboardInputCapability, PointerInputCapability,
    UIComplexity as SensoryUIComplexity, VisualOutputCapability,
};

#[test]
fn test_with_capabilities_simple() {
    // Touch + small screen => Simple
    let caps = SensoryCapabilities {
        visual_outputs: vec![VisualOutputCapability::TwoD {
            resolution: (800, 480),
            refresh_rate: 60,
            color_depth: 8,
            size_mm: Some((120, 72)),
        }],
        touch_inputs: vec![],
        pointer_inputs: vec![PointerInputCapability::TwoD {
            precision: 2.0,
            has_wheel: false,
            has_pressure: false,
            button_count: 1,
        }],
        ..Default::default()
    };
    let manager = SensoryUIManager::with_capabilities(caps);
    assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Simple);
}

#[test]
fn test_with_capabilities_standard() {
    // Desktop mouse + keyboard => Standard
    let caps = SensoryCapabilities {
        visual_outputs: vec![VisualOutputCapability::TwoD {
            resolution: (1280, 720),
            refresh_rate: 60,
            color_depth: 8,
            size_mm: None,
        }],
        pointer_inputs: vec![PointerInputCapability::TwoD {
            precision: 1.0,
            has_wheel: true,
            has_pressure: false,
            button_count: 3,
        }],
        keyboard_inputs: vec![KeyboardInputCapability::Physical {
            layout: "QWERTY".to_string(),
            has_numpad: false,
            modifier_keys: 3,
        }],
        ..Default::default()
    };
    let manager = SensoryUIManager::with_capabilities(caps);
    assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Standard);
}

#[test]
fn test_with_capabilities_immersive() {
    // VR/AR + spatial audio + haptics => Immersive
    use petal_tongue_core::sensory_capabilities::{
        AudioOutputCapability as AOC, HapticOutputCapability,
    };
    let caps = SensoryCapabilities {
        visual_outputs: vec![VisualOutputCapability::ThreeD {
            resolution_per_eye: (2160, 1200),
            field_of_view: (110.0, 90.0),
            refresh_rate: 90,
            has_depth_tracking: true,
            has_hand_tracking: true,
        }],
        audio_outputs: vec![AOC::Spatial {
            channels: 6,
            sample_rate: 48000,
            has_head_tracking: true,
        }],
        haptic_outputs: vec![HapticOutputCapability::SimpleVibration {
            intensity_levels: 255,
        }],
        ..Default::default()
    };
    let manager = SensoryUIManager::with_capabilities(caps);
    assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Immersive);
}

#[test]
fn test_rediscover_no_change_within_5_seconds() {
    let caps = SensoryCapabilities {
        audio_outputs: vec![AudioOutputCapability::Stereo {
            sample_rate: 48000,
            bit_depth: 16,
        }],
        keyboard_inputs: vec![KeyboardInputCapability::Physical {
            layout: "QWERTY".to_string(),
            has_numpad: false,
            modifier_keys: 3,
        }],
        ..Default::default()
    };
    let mut manager = SensoryUIManager::with_capabilities(caps);
    let initial_complexity = manager.ui_complexity();
    // Rediscover immediately - should skip due to 5 second throttle
    let result = manager.rediscover();
    assert!(result.is_ok());
    assert_eq!(manager.ui_complexity(), initial_complexity);
}

#[test]
fn test_capabilities_description() {
    let caps = SensoryCapabilities {
        audio_outputs: vec![AudioOutputCapability::Stereo {
            sample_rate: 48000,
            bit_depth: 16,
        }],
        ..Default::default()
    };
    let manager = SensoryUIManager::with_capabilities(caps);
    let desc = manager.capabilities_description();
    assert!(!desc.is_empty());
}

#[test]
fn test_with_capabilities_minimal() {
    let caps = SensoryCapabilities {
        audio_outputs: vec![AudioOutputCapability::Stereo {
            sample_rate: 48000,
            bit_depth: 16,
        }],
        keyboard_inputs: vec![KeyboardInputCapability::Physical {
            layout: "QWERTY".to_string(),
            has_numpad: false,
            modifier_keys: 3,
        }],
        ..Default::default()
    };
    let manager = SensoryUIManager::with_capabilities(caps);
    assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Minimal);
    let desc = manager.capabilities_description();
    assert!(desc.contains("audio") || !desc.is_empty());
}

#[test]
fn test_with_capabilities_rich() {
    // High-res + precision pointer + keyboard => Rich
    let caps = SensoryCapabilities {
        visual_outputs: vec![VisualOutputCapability::TwoD {
            resolution: (1920, 1080),
            refresh_rate: 60,
            color_depth: 8,
            size_mm: None,
        }],
        pointer_inputs: vec![PointerInputCapability::TwoD {
            precision: 1.5,
            has_wheel: true,
            has_pressure: false,
            button_count: 3,
        }],
        keyboard_inputs: vec![KeyboardInputCapability::Physical {
            layout: "QWERTY".to_string(),
            has_numpad: true,
            modifier_keys: 4,
        }],
        ..Default::default()
    };
    let manager = SensoryUIManager::with_capabilities(caps);
    assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Rich);
}

/// Headless egui test: render_primal_list does not panic
#[test]
fn test_render_primal_list_headless() {
    use petal_tongue_core::{PrimalHealthStatus, PrimalId};

    let caps = SensoryCapabilities {
        audio_outputs: vec![AudioOutputCapability::Stereo {
            sample_rate: 48000,
            bit_depth: 16,
        }],
        ..Default::default()
    };
    let mut manager = SensoryUIManager::with_capabilities(caps);
    let primals = vec![PrimalInfo::new(
        PrimalId::from("p1"),
        "Test Primal",
        "Compute",
        "http://localhost:8080",
        vec!["compute".to_string()],
        PrimalHealthStatus::Healthy,
        0,
    )];

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_primal_list(ui, &primals);
        });
    });
}

/// Headless egui test: render_topology does not panic
#[test]
fn test_render_topology_headless() {
    use petal_tongue_core::{GraphEngine, PrimalHealthStatus, PrimalId};

    let caps = SensoryCapabilities {
        visual_outputs: vec![VisualOutputCapability::TwoD {
            resolution: (1280, 720),
            refresh_rate: 60,
            color_depth: 8,
            size_mm: None,
        }],
        pointer_inputs: vec![PointerInputCapability::TwoD {
            precision: 1.0,
            has_wheel: true,
            has_pressure: false,
            button_count: 3,
        }],
        keyboard_inputs: vec![KeyboardInputCapability::Physical {
            layout: "QWERTY".to_string(),
            has_numpad: false,
            modifier_keys: 3,
        }],
        ..Default::default()
    };
    let mut manager = SensoryUIManager::with_capabilities(caps);
    let mut graph = GraphEngine::new();
    graph.add_node(PrimalInfo::new(
        PrimalId::from("n1"),
        "Node1",
        "Compute",
        "http://localhost:8080",
        vec![],
        PrimalHealthStatus::Healthy,
        0,
    ));

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_topology(ui, &graph);
        });
    });
}

/// Headless egui test: render_metrics does not panic (with and without metrics)
#[test]
fn test_render_metrics_headless() {
    use petal_tongue_core::metrics::{NeuralApiMetrics, SystemResourceMetrics};

    let caps = SensoryCapabilities {
        audio_outputs: vec![AudioOutputCapability::Stereo {
            sample_rate: 48000,
            bit_depth: 16,
        }],
        ..Default::default()
    };
    let mut manager = SensoryUIManager::with_capabilities(caps);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_metrics(ui, None);
        });
    });

    let metrics = petal_tongue_core::SystemMetrics {
        timestamp: chrono::Utc::now(),
        system: SystemResourceMetrics {
            cpu_percent: 45.0,
            memory_used_mb: 512,
            memory_total_mb: 1024,
            memory_percent: 50.0,
            uptime_seconds: 3600,
        },
        neural_api: NeuralApiMetrics {
            family_id: "test".to_string(),
            active_primals: 2,
            graphs_available: 1,
            active_executions: 0,
        },
    };

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_metrics(ui, Some(&metrics));
        });
    });
}

/// Headless egui test: render_proprioception does not panic (with and without data)
#[test]
fn test_render_proprioception_headless() {
    use petal_tongue_core::proprioception::{HealthData, HealthStatus};

    let caps = SensoryCapabilities {
        audio_outputs: vec![AudioOutputCapability::Stereo {
            sample_rate: 48000,
            bit_depth: 16,
        }],
        ..Default::default()
    };
    let mut manager = SensoryUIManager::with_capabilities(caps);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_proprioception(ui, None);
        });
    });

    let mut proprio = petal_tongue_core::ProprioceptionData::empty("test");
    proprio.health = HealthData {
        percentage: 85.0,
        status: HealthStatus::Healthy,
    };
    proprio.confidence = 92.5;

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.render_proprioception(ui, Some(&proprio));
        });
    });
}
