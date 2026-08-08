// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP route handlers, static-file fallback, and shared response utilities.

mod api;
mod manifest;
mod static_content;
mod topology;

pub(super) use api::{content_stats_handler, primals_handler, snapshot_handler, status_handler};
#[allow(unused_imports)]
pub(super) use static_content::resolve_docroot_path;
pub(super) use static_content::{
    docroot_fallback, events_sse_handler, health_handler, index_handler, liveness_handler,
    readiness_handler,
};
#[allow(unused_imports)]
pub(super) use topology::VizQuery;
pub(super) use topology::{
    ecosystem_handler, gate_mesh_handler, live_topology_handler, mesh_peers_handler,
    physical_topology_handler, primal_health_handler, sporeprint_handler, topology_layers_handler,
    viz_handler,
};

pub use static_content::{build_response, is_ipynb, is_notebook_mime};
