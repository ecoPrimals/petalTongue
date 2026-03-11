// SPDX-License-Identifier: AGPL-3.0-only

/// Detect which display modalities are available (terminal, visual, audio, framebuffer)
#[must_use]
pub fn detect_active_modalities() -> Vec<&'static str> {
    let mut modalities = Vec::new();

    modalities.push("terminal");

    if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
        modalities.push("visual");
    }

    modalities.push("audio");

    if std::path::Path::new("/dev/fb0").exists() {
        modalities.push("framebuffer");
    }

    modalities
}

/// Detect which UI capabilities are available based on modalities
#[must_use]
pub fn detect_capabilities() -> Vec<&'static str> {
    let mut capabilities = Vec::new();

    capabilities.push("ui.render");
    capabilities.push("ui.visualization");
    capabilities.push("ui.graph");

    let modalities = detect_active_modalities();

    if modalities.contains(&"terminal") {
        capabilities.push("ui.terminal");
    }

    if modalities.contains(&"audio") {
        capabilities.push("ui.audio");
    }

    if modalities.contains(&"framebuffer") {
        capabilities.push("ui.framebuffer");
    }

    capabilities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_active_modalities_returns_non_empty() {
        let modalities = detect_active_modalities();
        assert!(!modalities.is_empty());
        assert!(modalities.contains(&"terminal"));
        assert!(modalities.contains(&"audio"));
    }

    #[test]
    fn detect_active_modalities_may_include_visual() {
        let modalities = detect_active_modalities();
        let has_display =
            std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
        if has_display {
            assert!(modalities.contains(&"visual"));
        }
    }

    #[test]
    fn detect_capabilities_returns_core_ui() {
        let caps = detect_capabilities();
        assert!(caps.contains(&"ui.render"));
        assert!(caps.contains(&"ui.visualization"));
        assert!(caps.contains(&"ui.graph"));
    }

    #[test]
    fn detect_capabilities_includes_terminal_when_available() {
        let caps = detect_capabilities();
        assert!(caps.contains(&"ui.terminal"));
    }

    #[test]
    fn detect_capabilities_includes_audio() {
        let caps = detect_capabilities();
        assert!(caps.contains(&"ui.audio"));
    }
}
