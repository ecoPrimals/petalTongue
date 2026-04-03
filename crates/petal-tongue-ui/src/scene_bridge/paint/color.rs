// SPDX-License-Identifier: AGPL-3.0-or-later
//! Color and stroke conversion for scene primitives.

use egui::{Color32, Stroke};
use petal_tongue_scene::primitive::Color;

/// Apply accumulated opacity to a `Color32`.
pub fn apply_opacity(c: Color32, opacity: f32) -> Color32 {
    if opacity >= 1.0 {
        return c;
    }
    let a = (f32::from(c.a()) * opacity).round() as u8;
    Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), a)
}

pub fn to_color32(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}

pub fn to_egui_stroke(s: &petal_tongue_scene::primitive::StrokeStyle) -> Stroke {
    Stroke::new(s.width, to_color32(s.color))
}
