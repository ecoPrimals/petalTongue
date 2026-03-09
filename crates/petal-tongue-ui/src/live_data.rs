// SPDX-License-Identifier: AGPL-3.0-only
//! Live Data Indicator Components
//!
//! Visual components that prove data is LIVE with timestamps, badges, and source labels

use crate::accessibility::LiveIndicator;
use egui::{Color32, Context, RichText, Ui};
use std::time::{Duration, Instant};

/// Live data badge - shows "LIVE" or "STALE" with color
pub struct LiveBadge {
    /// The live indicator
    indicator: LiveIndicator,
    /// Last render time (for animations)
    last_render: Instant,
}

impl LiveBadge {
    /// Create a new live badge
    #[must_use]
    pub fn new(source: String, update_interval: f64) -> Self {
        Self {
            indicator: LiveIndicator::new(source, update_interval),
            last_render: Instant::now(),
        }
    }

    /// Mark data as updated
    pub fn mark_updated(&mut self) {
        self.indicator.mark_updated();
    }

    /// Render the badge
    pub fn render(&mut self, ui: &mut Ui) {
        let age = self.indicator.age_seconds();
        let is_stale = self.indicator.is_stale();

        // Color based on freshness
        let (color, text) = if !self.indicator.is_live {
            (Color32::GRAY, "WAITING")
        } else if is_stale {
            (Color32::from_rgb(200, 140, 0), "STALE")
        } else if age < 1.0 {
            // Pulse animation for very fresh data
            let pulse = (self.last_render.elapsed().as_secs_f32() * 2.0).sin() * 0.3 + 0.7;
            let green = (180.0 * pulse) as u8;
            (Color32::from_rgb(0, green + 75, 50), "● LIVE")
        } else {
            (Color32::from_rgb(0, 200, 100), "LIVE")
        };

        ui.label(RichText::new(text).size(11.0).strong().color(color));

        self.last_render = Instant::now();
    }

    /// Render with timestamp
    pub fn render_with_timestamp(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.render(ui);
            ui.label(
                RichText::new(format!("  {}", self.indicator.age_string()))
                    .size(10.0)
                    .color(Color32::GRAY),
            );
        });
    }

    /// Render full info (badge + timestamp + source)
    pub fn render_full(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.render(ui);
            ui.label(
                RichText::new(format!(
                    "  {} • {}",
                    self.indicator.age_string(),
                    self.indicator.source
                ))
                .size(10.0)
                .color(Color32::GRAY),
            );
        });
    }
}

/// Live graph header - shows title, LIVE badge, and metadata
pub struct LiveGraphHeader {
    /// Graph title
    title: String,
    /// Live badge
    badge: LiveBadge,
    /// Show update frequency
    show_frequency: bool,
}

impl LiveGraphHeader {
    /// Create a new live graph header
    #[must_use]
    pub fn new(title: String, source: String, update_interval: f64) -> Self {
        Self {
            title,
            badge: LiveBadge::new(source, update_interval),
            show_frequency: true,
        }
    }

    /// Mark data as updated
    pub fn mark_updated(&mut self) {
        self.badge.mark_updated();
    }

    /// Render the header
    pub fn render(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new(&self.title).size(16.0));
            ui.add_space(10.0);
            self.badge.render(ui);

            if self.show_frequency {
                ui.label(
                    RichText::new(format!("  ⟳ {:.1}s", self.badge.indicator.update_interval))
                        .size(10.0)
                        .color(Color32::DARK_GRAY),
                );
            }
        });
    }

    /// Render compact version
    pub fn render_compact(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&self.title).size(14.0).strong());
            ui.add_space(5.0);
            self.badge.render(ui);
        });
    }
}

/// Live metric display - shows a single live value with indicator
pub struct LiveMetric {
    /// Metric label
    label: String,
    /// Current value
    value: String,
    /// Unit (optional)
    unit: Option<String>,
    /// Live badge
    badge: LiveBadge,
}

impl LiveMetric {
    /// Create a new live metric
    #[must_use]
    pub fn new(label: String, source: String, update_interval: f64) -> Self {
        Self {
            label,
            value: "0".to_string(),
            unit: None,
            badge: LiveBadge::new(source, update_interval),
        }
    }

    /// Update the value
    pub fn update(&mut self, value: String, unit: Option<String>) {
        self.value = value;
        self.unit = unit;
        self.badge.mark_updated();
    }

    /// Render the metric
    pub fn render(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&self.label).size(12.0).color(Color32::GRAY));
            ui.add_space(5.0);

            let value_text = if let Some(ref unit) = self.unit {
                format!("{}{}", self.value, unit)
            } else {
                self.value.clone()
            };

            ui.label(RichText::new(value_text).size(14.0).strong());
            ui.add_space(5.0);
            self.badge.render(ui);
        });
    }

    /// Render large (for dashboard)
    pub fn render_large(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new(&self.label).size(11.0).color(Color32::GRAY));
            ui.add_space(2.0);

            let value_text = if let Some(ref unit) = self.unit {
                format!("{}{}", self.value, unit)
            } else {
                self.value.clone()
            };

            ui.label(RichText::new(value_text).size(20.0).strong());
            ui.add_space(2.0);
            self.badge.render(ui);
        });
    }
}

/// Connection status indicator
pub struct ConnectionStatus {
    /// Is connected?
    pub connected: bool,
    /// Connection target (e.g., "biomeOS at localhost:3000")
    pub target: String,
    /// Last successful connection
    last_connection: Option<Instant>,
}

impl ConnectionStatus {
    /// Create a new connection status
    #[must_use]
    pub fn new(target: String) -> Self {
        Self {
            connected: false,
            target,
            last_connection: None,
        }
    }

    /// Mark as connected
    pub fn mark_connected(&mut self) {
        self.connected = true;
        self.last_connection = Some(Instant::now());
    }

    /// Mark as disconnected
    pub fn mark_disconnected(&mut self) {
        self.connected = false;
    }

    /// Render the status
    pub fn render(&self, ui: &mut Ui) {
        let (color, symbol, status_text) = if self.connected {
            (Color32::from_rgb(0, 200, 100), "●", "Connected")
        } else {
            (Color32::from_rgb(200, 50, 50), "○", "Disconnected")
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new(symbol).size(14.0).color(color));
            ui.label(RichText::new(status_text).size(12.0).color(color));
            ui.label(
                RichText::new(format!("• {}", self.target))
                    .size(10.0)
                    .color(Color32::GRAY),
            );
        });

        if let Some(last) = self.last_connection {
            let elapsed = last.elapsed();
            if elapsed < Duration::from_secs(60) {
                ui.label(
                    RichText::new(format!(
                        "  Last connected: {:.0}s ago",
                        elapsed.as_secs_f32()
                    ))
                    .size(9.0)
                    .color(Color32::DARK_GRAY),
                );
            }
        }
    }

    /// Render compact (just symbol and status)
    pub fn render_compact(&self, ui: &mut Ui) {
        let (color, symbol) = if self.connected {
            (Color32::from_rgb(0, 200, 100), "●")
        } else {
            (Color32::from_rgb(200, 50, 50), "○")
        };

        ui.label(RichText::new(symbol).size(12.0).color(color));
    }
}

/// Helper function to render a timestamp
pub fn render_timestamp(ui: &mut Ui, instant: Instant) {
    let age = instant.elapsed().as_secs_f32();
    let text = if age < 1.0 {
        "Just now".to_string()
    } else if age < 60.0 {
        format!("{age:.1}s ago")
    } else if age < 3600.0 {
        format!("{:.1}m ago", age / 60.0)
    } else {
        format!("{:.1}h ago", age / 3600.0)
    };

    ui.label(RichText::new(text).size(10.0).color(Color32::GRAY));
}

/// Request continuous repaint for live updates
pub fn request_live_updates(ctx: &Context) {
    // Request repaint on next frame for smooth live updates
    ctx.request_repaint_after(Duration::from_millis(100));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_badge_creation() {
        let badge = LiveBadge::new("test".to_string(), 1.0);
        assert!(!badge.indicator.is_live);
    }

    #[test]
    fn test_live_metric_update() {
        let mut metric = LiveMetric::new("CPU".to_string(), "sysinfo".to_string(), 1.0);
        metric.update("45.2".to_string(), Some("%".to_string()));
        assert_eq!(metric.value, "45.2");
        assert_eq!(metric.unit, Some("%".to_string()));
        assert!(metric.badge.indicator.is_live);
    }

    #[test]
    fn test_connection_status() {
        let mut status = ConnectionStatus::new("localhost:3000".to_string());
        assert!(!status.connected);

        status.mark_connected();
        assert!(status.connected);
        assert!(status.last_connection.is_some());

        status.mark_disconnected();
        assert!(!status.connected);
    }

    #[test]
    fn test_live_badge_mark_updated() {
        let mut badge = LiveBadge::new("test".to_string(), 1.0);
        badge.mark_updated();
        assert!(badge.indicator.is_live);
    }

    #[test]
    fn test_live_graph_header_creation() {
        let mut header = LiveGraphHeader::new("Test".to_string(), "source".to_string(), 1.0);
        header.mark_updated();
        // Just verify no panic
    }

    #[test]
    fn test_live_metric_without_unit() {
        let mut metric = LiveMetric::new("Test".to_string(), "src".to_string(), 1.0);
        metric.update("42".to_string(), None);
        assert_eq!(metric.value, "42");
        assert_eq!(metric.unit, None);
    }

    #[test]
    fn test_connection_status_target() {
        let status = ConnectionStatus::new("biomeOS:3000".to_string());
        assert_eq!(status.target, "biomeOS:3000");
    }
}
