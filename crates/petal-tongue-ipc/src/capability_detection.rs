// SPDX-License-Identifier: AGPL-3.0-or-later
//! Runtime capability detection using `CapabilityTaxonomy`.
//!
//! Detects which display modalities and UI capabilities are available on the
//! current host, returning typed `CapabilityTaxonomy` values rather than
//! hardcoded strings.

use petal_tongue_core::capability_taxonomy::CapabilityTaxonomy;

/// Detect which display modalities are available (terminal, visual, audio, framebuffer).
#[must_use]
pub fn detect_active_modalities() -> Vec<CapabilityTaxonomy> {
    let mut modalities = Vec::new();

    modalities.push(CapabilityTaxonomy::UITerminal);

    if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
        modalities.push(CapabilityTaxonomy::UIVisualization);
    }

    modalities.push(CapabilityTaxonomy::UIAudio);

    if std::path::Path::new("/dev/fb0").exists() {
        modalities.push(CapabilityTaxonomy::UIFramebuffer);
    }

    modalities
}

/// Detect which UI capabilities are available based on modalities.
#[must_use]
pub fn detect_capabilities() -> Vec<CapabilityTaxonomy> {
    let mut capabilities = vec![
        CapabilityTaxonomy::UIRender,
        CapabilityTaxonomy::UIVisualization,
        CapabilityTaxonomy::UIGraph,
        CapabilityTaxonomy::VisualizationRender,
        CapabilityTaxonomy::VisualizationRenderStream,
        CapabilityTaxonomy::VisualizationInteract,
        CapabilityTaxonomy::IpcJsonRpc,
        CapabilityTaxonomy::IpcUnixSocket,
    ];

    let modalities = detect_active_modalities();

    if modalities.contains(&CapabilityTaxonomy::UITerminal) {
        capabilities.push(CapabilityTaxonomy::UITerminal);
    }

    if modalities.contains(&CapabilityTaxonomy::UIAudio) {
        capabilities.push(CapabilityTaxonomy::UIAudio);
    }

    if modalities.contains(&CapabilityTaxonomy::UIFramebuffer) {
        capabilities.push(CapabilityTaxonomy::UIFramebuffer);
    }

    capabilities.push(CapabilityTaxonomy::UIInputKeyboard);
    capabilities.push(CapabilityTaxonomy::UIInputMouse);

    capabilities
}

/// Return capability strings for IPC responses (backward-compatible).
#[must_use]
pub fn detect_capability_strings() -> Vec<&'static str> {
    detect_capabilities()
        .iter()
        .map(CapabilityTaxonomy::as_str)
        .collect()
}

/// Return modality strings for IPC responses (backward-compatible).
#[must_use]
pub fn detect_modality_strings() -> Vec<&'static str> {
    detect_active_modalities()
        .iter()
        .map(CapabilityTaxonomy::as_str)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_active_modalities_returns_non_empty() {
        let modalities = detect_active_modalities();
        assert!(!modalities.is_empty());
        assert!(modalities.contains(&CapabilityTaxonomy::UITerminal));
        assert!(modalities.contains(&CapabilityTaxonomy::UIAudio));
    }

    #[test]
    fn detect_active_modalities_may_include_visual() {
        let modalities = detect_active_modalities();
        let has_display =
            std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
        if has_display {
            assert!(modalities.contains(&CapabilityTaxonomy::UIVisualization));
        }
    }

    #[test]
    fn detect_capabilities_returns_core_ui() {
        let caps = detect_capabilities();
        assert!(caps.contains(&CapabilityTaxonomy::UIRender));
        assert!(caps.contains(&CapabilityTaxonomy::UIVisualization));
        assert!(caps.contains(&CapabilityTaxonomy::UIGraph));
    }

    #[test]
    fn detect_capabilities_includes_terminal_when_available() {
        let caps = detect_capabilities();
        assert!(caps.contains(&CapabilityTaxonomy::UITerminal));
    }

    #[test]
    fn detect_capabilities_includes_audio() {
        let caps = detect_capabilities();
        assert!(caps.contains(&CapabilityTaxonomy::UIAudio));
    }

    #[test]
    fn detect_capabilities_includes_visualization() {
        let caps = detect_capabilities();
        assert!(caps.contains(&CapabilityTaxonomy::VisualizationRender));
        assert!(caps.contains(&CapabilityTaxonomy::VisualizationRenderStream));
        assert!(caps.contains(&CapabilityTaxonomy::VisualizationInteract));
    }

    #[test]
    fn detect_capabilities_includes_ipc() {
        let caps = detect_capabilities();
        assert!(caps.contains(&CapabilityTaxonomy::IpcJsonRpc));
        assert!(caps.contains(&CapabilityTaxonomy::IpcUnixSocket));
    }

    #[test]
    fn backward_compatible_strings() {
        let strs = detect_capability_strings();
        assert!(strs.contains(&"ui.render"));
        assert!(strs.contains(&"ui.graph"));
    }

    #[test]
    fn test_detect_modality_strings() {
        let strs = super::detect_modality_strings();
        assert!(!strs.is_empty());
        assert!(strs.contains(&"ui.terminal"));
        assert!(strs.contains(&"ui.audio"));
    }

    #[test]
    fn detect_capabilities_includes_input() {
        let caps = detect_capabilities();
        assert!(caps.contains(&CapabilityTaxonomy::UIInputKeyboard));
        assert!(caps.contains(&CapabilityTaxonomy::UIInputMouse));
    }

    #[test]
    fn detect_capability_strings_matches_detect_capabilities() {
        let caps = detect_capabilities();
        let strs = detect_capability_strings();
        assert_eq!(caps.len(), strs.len());
        for cap in &caps {
            assert!(strs.contains(&cap.as_str()));
        }
    }

    #[test]
    fn test_detect_modality_strings_matches_detect_active_modalities() {
        let modalities = detect_active_modalities();
        let strs = super::detect_modality_strings();
        assert_eq!(modalities.len(), strs.len());
        for m in &modalities {
            assert!(strs.contains(&m.as_str()));
        }
    }

    #[test]
    fn detect_active_modalities_may_include_framebuffer() {
        let modalities = detect_active_modalities();
        if std::path::Path::new("/dev/fb0").exists() {
            assert!(modalities.contains(&CapabilityTaxonomy::UIFramebuffer));
        }
    }
}
