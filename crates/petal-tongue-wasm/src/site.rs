// SPDX-License-Identifier: AGPL-3.0-or-later
//! Static site builder WASM entry points — Zola replacement path.

use wasm_bindgen::prelude::*;

use petal_tongue_scene::document::SiteContent;
use petal_tongue_scene::site_builder::{SiteBuilder, SiteLayout};

/// Build a complete static site from `SiteContent` JSON.
///
/// Returns a JSON object mapping output paths to HTML content strings:
/// ```json
/// {
///   "files": {
///     "index.html": "<!DOCTYPE html>...",
///     "about/index.html": "<!DOCTYPE html>...",
///     "search-index.json": "[...]"
///   },
///   "file_count": 3,
///   "total_bytes": 12345
/// }
/// ```
///
/// # Arguments
/// * `content_json` — JSON string of `SiteContent` (pages, nav, `search_index`, `entity_registry`)
/// * `layout_json` — JSON string of `SiteLayout` (`site_title`, css, footer), or empty for defaults
#[must_use]
#[wasm_bindgen]
pub fn build_site(content_json: &str, layout_json: &str) -> String {
    let content: SiteContent = match serde_json::from_str(content_json) {
        Ok(c) => c,
        Err(e) => return format!("{{\"error\": \"invalid content: {e}\"}}"),
    };

    let layout: SiteLayout = if layout_json.is_empty() {
        SiteLayout::default()
    } else {
        match serde_json::from_str(layout_json) {
            Ok(l) => l,
            Err(e) => return format!("{{\"error\": \"invalid layout: {e}\"}}"),
        }
    };

    let builder = SiteBuilder::new(layout);
    match builder.build(&content) {
        Ok(site) => {
            let files: std::collections::HashMap<&str, &str> = site
                .files
                .iter()
                .filter_map(|(path, bytes)| {
                    std::str::from_utf8(bytes).ok().map(|s| (path.as_str(), s))
                })
                .collect();

            serde_json::json!({
                "files": files,
                "file_count": site.file_count(),
                "total_bytes": site.total_bytes()
            })
            .to_string()
        }
        Err(e) => format!("{{\"error\": \"{e}\"}}"),
    }
}

/// Render a single page with site layout composition.
///
/// Takes a `DocumentNode::Page` JSON and optional nav/layout config,
/// returns the rendered HTML with full site chrome (header, nav, footer, CSS).
///
/// # Arguments
/// * `page_json` — JSON string of a `DocumentNode::Page`
/// * `nav_json` — JSON array of `NavSection` objects (or empty for no nav)
/// * `layout_json` — JSON string of `SiteLayout` (or empty for defaults)
#[must_use]
#[wasm_bindgen]
pub fn render_page_with_layout(page_json: &str, nav_json: &str, layout_json: &str) -> String {
    let page: petal_tongue_scene::document::DocumentNode = match serde_json::from_str(page_json) {
        Ok(p) => p,
        Err(e) => return format!("Error: invalid page: {e}"),
    };

    let nav: Vec<petal_tongue_scene::document::NavSection> = if nav_json.is_empty() {
        Vec::new()
    } else {
        match serde_json::from_str(nav_json) {
            Ok(n) => n,
            Err(e) => return format!("Error: invalid nav: {e}"),
        }
    };

    let layout: SiteLayout = if layout_json.is_empty() {
        SiteLayout::default()
    } else {
        match serde_json::from_str(layout_json) {
            Ok(l) => l,
            Err(e) => return format!("Error: invalid layout: {e}"),
        }
    };

    let builder = SiteBuilder::new(layout);
    builder.render_page(&page, &nav)
}
