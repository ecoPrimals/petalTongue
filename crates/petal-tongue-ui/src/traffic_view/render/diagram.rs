// SPDX-License-Identifier: AGPL-3.0-or-later

//! Sankey-style traffic diagram painting and hit testing.

use std::collections::HashMap;

use crate::traffic_view::helpers::{
    bezier_control_points, calculate_flow_width, primal_lane_layout,
};
use crate::traffic_view::types::TrafficIntent;
use crate::traffic_view::view::TrafficView;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};

pub fn render_traffic_diagram(view: &TrafficView, ui: &mut egui::Ui) -> Vec<TrafficIntent> {
    let mut intents = Vec::new();
    let available_size = ui.available_size();
    let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click());

    let rect = response.rect;
    painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 20, 25));

    if view.flows().is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No traffic data to display",
            egui::FontId::proportional(16.0),
            Color32::GRAY,
        );
        return intents;
    }

    let mut primal_ids: Vec<_> = view.primals_map().keys().cloned().collect();
    if primal_ids.is_empty() {
        for flow in view.flows() {
            if !primal_ids.contains(&flow.from) {
                primal_ids.push(flow.from.clone());
            }
            if !primal_ids.contains(&flow.to) {
                primal_ids.push(flow.to.clone());
            }
        }
    }
    primal_ids.sort();

    if primal_ids.is_empty() {
        return intents;
    }

    let node_width = 120.0;
    let margin = 20.0;

    let lane_layout = primal_lane_layout(
        primal_ids.len(),
        rect.min.x,
        rect.min.y,
        rect.max.x,
        rect.max.y,
        margin,
        node_width,
    );

    let primal_positions: HashMap<_, _> = primal_ids
        .iter()
        .enumerate()
        .filter_map(|(i, id)| {
            lane_layout.get(i).map(|&(y, left_x, right_x)| {
                (id.clone(), (Pos2::new(left_x, y), Pos2::new(right_x, y)))
            })
        })
        .collect();

    let max_vol = view.max_volume_impl();

    for flow in view.flows() {
        if let (Some((_from_left, from_right)), Some((to_left, _to_right))) = (
            primal_positions.get(&flow.from),
            primal_positions.get(&flow.to),
        ) {
            let width = calculate_flow_width(
                &flow.metrics,
                max_vol,
                view.min_flow_width(),
                view.max_flow_width(),
            );

            let start = *from_right;
            let end = *to_left;

            let (ctrl1_arr, ctrl2_arr) = bezier_control_points(start.x, start.y, end.x, end.y);
            let ctrl1 = Pos2::new(ctrl1_arr[0], ctrl1_arr[1]);
            let ctrl2 = Pos2::new(ctrl2_arr[0], ctrl2_arr[1]);

            draw_bezier_flow(
                &painter,
                start,
                ctrl1,
                ctrl2,
                end,
                width,
                to_color32(flow.color),
            );

            let click_rect = Rect::from_center_size(
                Pos2::new(f32::midpoint(start.x, end.x), f32::midpoint(start.y, end.y)),
                Vec2::splat(30.0),
            );
            if response.clicked()
                && let Some(pointer_pos) = response.interact_pointer_pos()
                && click_rect.contains(pointer_pos)
            {
                intents.push(TrafficIntent::SelectFlow {
                    from: flow.from.clone(),
                    to: flow.to.clone(),
                });
            }
        }
    }

    for (i, primal_id) in primal_ids.iter().enumerate() {
        if let Some(&(y, left_x, right_x)) = lane_layout.get(i) {
            let left_pos = Pos2::new(left_x, y);
            let right_pos = Pos2::new(right_x, y);
            draw_primal_node(&painter, left_pos, primal_id, node_width);
            draw_primal_node(&painter, right_pos, primal_id, node_width);
        }
    }

    intents
}

fn to_color32(rgba: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
}

fn draw_bezier_flow(
    painter: &egui::Painter,
    start: Pos2,
    ctrl1: Pos2,
    ctrl2: Pos2,
    end: Pos2,
    width: f32,
    color: Color32,
) {
    let segments = 20;
    let mut points = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let x =
            (3.0 * mt * t2).mul_add(ctrl2.x, mt3 * start.x + 3.0 * mt2 * t * ctrl1.x) + t3 * end.x;
        let y =
            (3.0 * mt * t2).mul_add(ctrl2.y, mt3 * start.y + 3.0 * mt2 * t * ctrl1.y) + t3 * end.y;

        points.push(Pos2::new(x, y));
    }

    for i in 0..segments {
        painter.line_segment([points[i], points[i + 1]], Stroke::new(width, color));
    }
}

fn draw_primal_node(painter: &egui::Painter, pos: Pos2, primal_id: &str, width: f32) {
    let height = 30.0;
    let rect = Rect::from_center_size(pos, Vec2::new(width, height));

    painter.rect_filled(rect, 5.0, Color32::from_rgb(40, 40, 50));
    painter.rect_stroke(
        rect,
        5.0,
        Stroke::new(1.0, Color32::from_rgb(100, 100, 120)),
    );

    painter.text(
        pos,
        egui::Align2::CENTER_CENTER,
        primal_id,
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
}
