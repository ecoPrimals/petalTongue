// SPDX-License-Identifier: AGPL-3.0-or-later
//! Environment detection for [`RenderingCapabilities`](super::types::RenderingCapabilities).

use super::types::{
    DeviceType, InputMethod, PerformanceTier, RenderingCapabilities, RenderingModality,
};

impl RenderingCapabilities {
    /// Detect capabilities from environment
    #[must_use]
    pub fn detect() -> Self {
        let screen_size = Self::detect_screen_size();
        let has_mouse = Self::has_mouse();
        let has_touch = Self::has_touch();

        let device_type = DeviceType::detect(Some(screen_size), has_mouse, has_touch);
        let ui_complexity = device_type.recommended_complexity();

        let mut modalities = Vec::new();
        let mut input_methods = Vec::new();

        // Visual modality
        let (w, h) = screen_size;
        modalities.push(RenderingModality::Visual2D {
            resolution: (w, h),
            color: true,
        });

        // Input methods
        if has_mouse {
            input_methods.push(InputMethod::Mouse);
        }
        if has_touch {
            input_methods.push(InputMethod::Touch);
        }
        if Self::has_keyboard() {
            input_methods.push(InputMethod::Keyboard);
        }

        // Performance tier (basic heuristic)
        let performance_tier = match device_type {
            DeviceType::Desktop | DeviceType::TV => PerformanceTier::High,
            DeviceType::Tablet | DeviceType::Phone | DeviceType::Unknown => PerformanceTier::Medium,
            DeviceType::Watch | DeviceType::CLI => PerformanceTier::Low,
        };

        Self {
            device_type,
            modalities,
            screen_size: Some(screen_size),
            input_methods,
            performance_tier,
            ui_complexity,
        }
    }

    /// Detect screen size from environment
    ///
    /// Tries, in order:
    /// 1. Linux framebuffer: `/sys/class/graphics/fb0/virtual_size`
    /// 2. Terminal dimensions (when no graphical display)
    /// 3. `DISPLAY/WAYLAND_DISPLAY` presence → sensible desktop defaults
    /// 4. Fallback: (1280, 720) when detection fails
    fn detect_screen_size() -> (f32, f32) {
        use terminal_size::{Height, Width};

        // 1. Linux: Read framebuffer virtual size
        #[cfg(target_os = "linux")]
        if let Some((w, h)) = Self::read_fb0_virtual_size() {
            return (w, h);
        }

        // 2. Graphical environment detected → use defaults (actual size from winit at runtime)
        let has_graphical = std::env::var("DISPLAY")
            .map(|d| !d.is_empty())
            .unwrap_or(false)
            || std::env::var("WAYLAND_DISPLAY")
                .map(|w| !w.is_empty())
                .unwrap_or(false);

        if has_graphical {
            return (1400.0, 900.0);
        }

        // 3. CLI/terminal: Use terminal dimensions (convert cols/rows to pixel estimate)
        if let Some((Width(cols), Height(rows))) = terminal_size::terminal_size() {
            let w = f32::from(cols) * 8.0;
            let h = f32::from(rows) * 16.0;
            return (w, h);
        }

        // 4. Fallback when all detection fails
        (1280.0, 720.0)
    }

    /// Read Linux framebuffer virtual size from sysfs
    #[cfg(target_os = "linux")]
    fn read_fb0_virtual_size() -> Option<(f32, f32)> {
        let path = "/sys/class/graphics/fb0/virtual_size";
        let s = std::fs::read_to_string(path).ok()?;
        let s = s.trim();
        let mut parts = s.split(',');
        let w: f32 = parts.next()?.trim().parse().ok()?;
        let h: f32 = parts.next()?.trim().parse().ok()?;
        Some((w, h))
    }

    /// Check if mouse is available
    fn has_mouse() -> bool {
        // Heuristic: If we have a display server, assume mouse
        std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    /// Check if touch is available
    ///
    /// On Linux: checks for touch input devices and graphical session type.
    fn has_touch() -> bool {
        // 1. Linux: Check for touch devices in /dev/input/
        #[cfg(target_os = "linux")]
        if Self::has_touch_input_devices() {
            return true;
        }

        // 2. Wayland/touch-friendly session (often indicates touch-capable device)
        if std::env::var("WAYLAND_DISPLAY")
            .map(|w| !w.is_empty())
            .unwrap_or(false)
        {
            // Wayland is common on touch devices (tablets, convertibles)
            // Conservative: don't assume touch from Wayland alone
        }

        if std::env::var("XDG_SESSION_TYPE")
            .map(|s| s.to_lowercase() == "wayland")
            .unwrap_or(false)
        {
            // Wayland session - could be touch, but we already checked devices
        }

        false
    }

    /// Check for touch input devices on Linux
    #[cfg(target_os = "linux")]
    fn has_touch_input_devices() -> bool {
        let Ok(entries) = std::fs::read_dir("/sys/class/input") else {
            return false;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("event") {
                let device_name = entry.path().join("device/name");
                if let Ok(name_content) = std::fs::read_to_string(device_name) {
                    let name_lower = name_content.to_lowercase();
                    if name_lower.contains("touch")
                        || name_lower.contains("touchscreen")
                        || name_lower.contains("wacom")
                        || name_lower.contains("digitizer")
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if keyboard is available
    const fn has_keyboard() -> bool {
        // Assume keyboard is always available if we have input
        true
    }

    /// Check if this device supports a specific modality
    pub fn supports_modality(&self, check: impl Fn(&RenderingModality) -> bool) -> bool {
        self.modalities.iter().any(check)
    }

    /// Check if this device supports visual rendering
    #[must_use]
    pub fn has_visual(&self) -> bool {
        self.supports_modality(|m| matches!(m, RenderingModality::Visual2D { .. }))
    }

    /// Check if this device supports audio
    #[must_use]
    pub fn has_audio(&self) -> bool {
        self.supports_modality(|m| matches!(m, RenderingModality::Audio { .. }))
    }

    /// Check if this device supports haptics
    #[must_use]
    pub fn has_haptic(&self) -> bool {
        self.supports_modality(|m| matches!(m, RenderingModality::Haptic { .. }))
    }

    /// Get visual resolution, if available
    #[must_use]
    pub fn visual_resolution(&self) -> Option<(f32, f32)> {
        for modality in &self.modalities {
            if let RenderingModality::Visual2D { resolution, .. } = modality {
                return Some(*resolution);
            }
        }
        None
    }
}
