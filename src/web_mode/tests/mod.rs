// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code uses unwrap/expect for brevity"
)]

use super::*;

mod config_tests;
mod content_backend_tests;
mod content_direct_tests;
mod content_render_tests;
mod docroot_tests;
mod handler_tests;
mod router_tests;
mod startup_tests;
mod viz_registry_tests;

pub(super) fn test_config(bind: &str) -> WebConfig<'_> {
    WebConfig {
        bind,
        scenario: None,
        docroot: None,
        backend: "filesystem",
        workers: 4,
        strip_sources: false,
        cache_ttl_secs: 0,
        spa: false,
        allowed_origins: Vec::new(),
    }
}
