// SPDX-License-Identifier: AGPL-3.0-or-later

use super::renderer_trait::AdaptiveRenderer;
use super::types::{
    DeviceType, PerformanceTier, RenderingCapabilities, RenderingModality, UIComplexity,
};

#[test]
fn test_device_type_detection() {
    // Desktop
    let desktop = DeviceType::detect(Some((1920.0, 1080.0)), true, false);
    assert_eq!(desktop, DeviceType::Desktop);

    // Phone
    let phone = DeviceType::detect(Some((428.0, 926.0)), false, true);
    assert_eq!(phone, DeviceType::Phone);

    // Watch
    let watch = DeviceType::detect(Some((368.0, 448.0)), false, true);
    assert_eq!(watch, DeviceType::Watch);

    // CLI
    let cli = DeviceType::detect(None, false, false);
    assert_eq!(cli, DeviceType::CLI);
}

#[test]
fn test_ui_complexity() {
    assert_eq!(
        DeviceType::Desktop.recommended_complexity(),
        UIComplexity::Full
    );
    assert_eq!(
        DeviceType::Phone.recommended_complexity(),
        UIComplexity::Minimal
    );
    assert_eq!(
        DeviceType::Watch.recommended_complexity(),
        UIComplexity::Essential
    );
}

#[test]
fn test_rendering_capabilities_detection() {
    let caps = RenderingCapabilities::detect();

    assert!(!caps.modalities.is_empty());

    assert!(matches!(
        caps.ui_complexity,
        UIComplexity::Full
            | UIComplexity::Simplified
            | UIComplexity::Minimal
            | UIComplexity::Essential
    ));
}

#[test]
fn test_device_type_tablet() {
    let tablet = DeviceType::detect(Some((1024.0, 768.0)), false, true);
    assert_eq!(tablet, DeviceType::Tablet);
}

#[test]
fn test_device_type_tv() {
    let tv = DeviceType::detect(Some((1920.0, 1200.0)), false, false);
    assert_eq!(tv, DeviceType::TV);
}

#[test]
fn test_device_type_unknown_no_mouse() {
    let unknown = DeviceType::detect(Some((1920.0, 1080.0)), false, false);
    assert_eq!(unknown, DeviceType::Unknown);
}

#[test]
fn test_device_type_phone_touch_no_mouse() {
    let phone = DeviceType::detect(Some((375.0, 667.0)), false, true);
    assert_eq!(phone, DeviceType::Phone);
}

#[test]
fn test_device_type_watch_small_screen() {
    let watch = DeviceType::detect(Some((184.0, 224.0)), false, true);
    assert_eq!(watch, DeviceType::Watch);
}

#[test]
fn test_device_type_display() {
    assert_eq!(format!("{}", DeviceType::Desktop), "Desktop");
    assert_eq!(format!("{}", DeviceType::Phone), "Phone");
    assert_eq!(format!("{}", DeviceType::Watch), "Watch");
    assert_eq!(format!("{}", DeviceType::Tablet), "Tablet");
    assert_eq!(format!("{}", DeviceType::TV), "TV");
    assert_eq!(format!("{}", DeviceType::CLI), "CLI");
    assert_eq!(format!("{}", DeviceType::Unknown), "Unknown");
}

#[test]
fn test_ui_complexity_tablet() {
    assert_eq!(
        DeviceType::Tablet.recommended_complexity(),
        UIComplexity::Simplified
    );
}

#[test]
fn test_ui_complexity_tv() {
    assert_eq!(DeviceType::TV.recommended_complexity(), UIComplexity::Full);
}

#[test]
fn test_ui_complexity_unknown() {
    assert_eq!(
        DeviceType::Unknown.recommended_complexity(),
        UIComplexity::Minimal
    );
}

#[test]
fn test_rendering_capabilities_has_visual() {
    let caps = RenderingCapabilities::detect();
    assert!(caps.has_visual());
}

#[test]
fn test_rendering_capabilities_visual_resolution() {
    let caps = RenderingCapabilities::detect();
    let res = caps.visual_resolution();
    assert!(res.is_some());
    let (w, h) = res.unwrap();
    assert!(w > 0.0 && h > 0.0);
}

#[test]
fn test_rendering_capabilities_supports_modality() {
    let caps = RenderingCapabilities::detect();
    assert!(caps.supports_modality(|m| matches!(m, RenderingModality::Visual2D { .. })));
}

#[test]
fn test_rendering_capabilities_no_audio_by_default() {
    let caps = RenderingCapabilities::detect();
    assert!(!caps.has_audio());
}

#[test]
fn test_rendering_capabilities_no_haptic_by_default() {
    let caps = RenderingCapabilities::detect();
    assert!(!caps.has_haptic());
}

#[test]
fn test_rendering_modality_visual2d() {
    let caps = RenderingCapabilities {
        device_type: DeviceType::Desktop,
        modalities: vec![RenderingModality::Visual2D {
            resolution: (800.0, 600.0),
            color: true,
        }],
        screen_size: Some((800.0, 600.0)),
        input_methods: vec![],
        performance_tier: PerformanceTier::High,
        ui_complexity: UIComplexity::Full,
    };
    assert!(caps.has_visual());
    assert_eq!(caps.visual_resolution(), Some((800.0, 600.0)));
}

#[test]
fn test_adaptive_renderer_trait_default_priority() {
    struct TestRenderer;
    impl AdaptiveRenderer for TestRenderer {
        fn supports(&self, _caps: &RenderingCapabilities) -> bool {
            true
        }
    }
    let r = TestRenderer;
    let caps = RenderingCapabilities::detect();
    assert!(r.supports(&caps));
    assert_eq!(r.priority(), 0);
}
