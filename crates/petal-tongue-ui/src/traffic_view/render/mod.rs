// SPDX-License-Identifier: AGPL-3.0-or-later

//! egui subcomponents for [`super::TrafficView`](crate::traffic_view::TrafficView).

mod diagram;
mod metrics_panel;
mod toolbar;

pub use diagram::render_traffic_diagram;
pub use metrics_panel::render_metrics_panel;
pub use toolbar::render as render_toolbar;
