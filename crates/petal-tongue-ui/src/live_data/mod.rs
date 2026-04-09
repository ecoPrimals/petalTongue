// SPDX-License-Identifier: AGPL-3.0-or-later
//! Live Data Indicator Components
//!
//! Visual components that prove data is LIVE with timestamps, badges, and source labels

use crate::accessibility::LiveIndicator;
use crate::live_data_helpers::{
    badge_display_state, connection_status_display, format_age_for_display, format_metric_value,
};
use egui::{Color32, Context, RichText, Ui};
use petal_tongue_core::constants;
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
        let badge = badge_display_state(age, is_stale, self.indicator.is_live);

        // Pulse animation for very fresh data (age < 1)
        let color = if badge.label == "● LIVE" && age < 1.0 {
            let pulse = (self.last_render.elapsed().as_secs_f32() * 2.0)
                .sin()
                .mul_add(0.3, 0.7);
            let green = (180.0 * pulse) as u8;
            Color32::from_rgb(0, green + 75, 50)
        } else {
            let (r, g, b) = badge.color_rgb;
            Color32::from_rgb(r, g, b)
        };

        ui.label(RichText::new(badge.label).size(11.0).strong().color(color));

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
                let v = self.value.parse().unwrap_or(0.0);
                format_metric_value(v, unit)
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
    pub const fn new(target: String) -> Self {
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
    pub const fn mark_disconnected(&mut self) {
        self.connected = false;
    }

    /// Render the status
    pub fn render(&self, ui: &mut Ui) {
        let disconnected_secs = if self.connected {
            None
        } else {
            self.last_connection
                .map(|last| last.elapsed().as_secs_f64())
        };
        let display = connection_status_display(self.connected, disconnected_secs);
        let color = Color32::from_rgb(
            display.color_rgb.0,
            display.color_rgb.1,
            display.color_rgb.2,
        );

        ui.horizontal(|ui| {
            ui.label(RichText::new(display.symbol).size(14.0).color(color));
            ui.label(RichText::new(display.status_text).size(12.0).color(color));
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

/// Default connection target for display (e.g. "localhost:3000").
/// Uses `BIOMEOS_URL` or `PETALTONGUE_LIVE_TARGET`; fallback from constants.
#[must_use]
pub fn default_connection_target() -> String {
    constants::default_biomeos_connection_target()
}

/// Helper function to render a timestamp
pub fn render_timestamp(ui: &mut Ui, instant: Instant) {
    let age = instant.elapsed().as_secs_f64();
    let text = format_age_for_display(age);
    ui.label(RichText::new(text).size(10.0).color(Color32::GRAY));
}

/// Request continuous repaint for live updates
pub fn request_live_updates(ctx: &Context) {
    // Request repaint on next frame for smooth live updates
    ctx.request_repaint_after(Duration::from_millis(100));
}

#[cfg(test)]
mod tests;
