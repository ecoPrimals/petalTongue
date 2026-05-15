// SPDX-License-Identifier: AGPL-3.0-or-later

//! Parameter strip for interactive grammar exploration in expanded view.

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::data_binding::DataBindingCompiler;
use petal_tongue_scene::grammar::{CoordinateSystem, GeometryType, ScaleType};
use petal_tongue_scene::render_plan::RenderPlan;

/// Render a collapsible parameter strip for interactive exploration.
/// Returns an overridden `RenderPlan` if the user has tweaked parameters,
/// or `None` to use the default compiled plan.
pub(crate) fn render_parameter_strip(
    ui: &mut egui::Ui,
    compiled: &petal_tongue_ipc::CompiledBinding,
    key: &str,
) -> Option<RenderPlan> {
    let strip_id = egui::Id::new("param_strip").with(key);
    let mut expanded: bool = ui.data_mut(|d| d.get_temp(strip_id).unwrap_or(false));

    ui.horizontal(|ui| {
        if ui.small_button(if expanded { "Hide controls" } else { "Controls" }).clicked() {
            expanded = !expanded;
            ui.data_mut(|d| d.insert_temp(strip_id, expanded));
        }
    });

    if !expanded {
        return None;
    }

    let override_id = egui::Id::new("param_override").with(key);

    #[derive(Clone, Debug)]
    struct ParamOverride {
        geometry: GeometryType,
        coordinate: CoordinateSystem,
        x_scale: ScaleType,
        y_scale: ScaleType,
        dirty: bool,
    }

    let orig = &compiled.grammar;
    let default_x_scale = orig
        .scales
        .iter()
        .find(|s| s.variable == "x")
        .map_or(ScaleType::Linear, |s| s.scale_type);
    let default_y_scale = orig
        .scales
        .iter()
        .find(|s| s.variable == "y")
        .map_or(ScaleType::Linear, |s| s.scale_type);

    let mut params: ParamOverride = ui.data_mut(|d| {
        d.get_temp(override_id).unwrap_or(ParamOverride {
            geometry: orig.geometry,
            coordinate: orig.coordinate,
            x_scale: default_x_scale,
            y_scale: default_y_scale,
            dirty: false,
        })
    });

    let mut changed = false;

    egui::Frame::none()
        .fill(egui::Color32::from_gray(25))
        .rounding(3.0)
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Geometry:").size(10.0).color(egui::Color32::from_gray(160)));
                let geom_options = [
                    ("Point", GeometryType::Point),
                    ("Line", GeometryType::Line),
                    ("Bar", GeometryType::Bar),
                    ("Area", GeometryType::Area),
                    ("Tile", GeometryType::Tile),
                    ("Arc", GeometryType::Arc),
                ];
                let current_label = geom_options.iter()
                    .find(|(_, g)| *g == params.geometry)
                    .map_or("?", |(l, _)| l);
                egui::ComboBox::from_id_salt(egui::Id::new("param_geom").with(key))
                    .selected_text(current_label)
                    .width(70.0)
                    .show_ui(ui, |ui| {
                        for (label, geom) in &geom_options {
                            if ui.selectable_label(params.geometry == *geom, *label).clicked() {
                                params.geometry = *geom;
                                changed = true;
                            }
                        }
                    });

                ui.separator();
                ui.label(egui::RichText::new("Coord:").size(10.0).color(egui::Color32::from_gray(160)));
                let coord_options = [
                    ("Cartesian", CoordinateSystem::Cartesian),
                    ("Polar", CoordinateSystem::Polar),
                ];
                let current_coord = coord_options.iter()
                    .find(|(_, c)| *c == params.coordinate)
                    .map_or("?", |(l, _)| l);
                egui::ComboBox::from_id_salt(egui::Id::new("param_coord").with(key))
                    .selected_text(current_coord)
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        for (label, coord) in &coord_options {
                            if ui.selectable_label(params.coordinate == *coord, *label).clicked() {
                                params.coordinate = *coord;
                                changed = true;
                            }
                        }
                    });

                ui.separator();
                ui.label(egui::RichText::new("X scale:").size(10.0).color(egui::Color32::from_gray(160)));
                let scale_options = [
                    ("Linear", ScaleType::Linear),
                    ("Log", ScaleType::Log),
                    ("Sqrt", ScaleType::Sqrt),
                    ("Categorical", ScaleType::Categorical),
                ];
                let current_xs = scale_options.iter()
                    .find(|(_, s)| *s == params.x_scale)
                    .map_or("?", |(l, _)| l);
                egui::ComboBox::from_id_salt(egui::Id::new("param_xs").with(key))
                    .selected_text(current_xs)
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        for (label, scale) in &scale_options {
                            if ui.selectable_label(params.x_scale == *scale, *label).clicked() {
                                params.x_scale = *scale;
                                changed = true;
                            }
                        }
                    });

                ui.separator();
                ui.label(egui::RichText::new("Y scale:").size(10.0).color(egui::Color32::from_gray(160)));
                let current_ys = scale_options.iter()
                    .find(|(_, s)| *s == params.y_scale)
                    .map_or("?", |(l, _)| l);
                egui::ComboBox::from_id_salt(egui::Id::new("param_ys").with(key))
                    .selected_text(current_ys)
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        for (label, scale) in &scale_options {
                            if ui.selectable_label(params.y_scale == *scale, *label).clicked() {
                                params.y_scale = *scale;
                                changed = true;
                            }
                        }
                    });

                if ui.small_button("Reset").clicked() {
                    params = ParamOverride {
                        geometry: orig.geometry,
                        coordinate: orig.coordinate,
                        x_scale: default_x_scale,
                        y_scale: default_y_scale,
                        dirty: false,
                    };
                    changed = true;
                }
            });
        });

    if changed {
        params.dirty = true;
    }

    ui.data_mut(|d| d.insert_temp(override_id, params.clone()));

    if !params.dirty {
        return None;
    }

    if let Some(ref source_binding) = compiled.source_binding {
        let domain = compiled.grammar.domain.as_deref();
        let (mut grammar, data) = DataBindingCompiler::compile(source_binding, domain);
        grammar.geometry = params.geometry;
        grammar.coordinate = params.coordinate;
        for scale in &mut grammar.scales {
            if scale.variable == "x" {
                scale.scale_type = params.x_scale;
            } else if scale.variable == "y" {
                scale.scale_type = params.y_scale;
            }
        }
        let compiler = GrammarCompiler::new();
        let scene = compiler.compile(&grammar, &data);
        Some(RenderPlan::new(scene, grammar))
    } else {
        None
    }
}
