// SPDX-License-Identifier: AGPL-3.0-or-later
//! Static site builder — Zola replacement path (Sovereignty Evolution Tier 1).
//!
//! Compiles [`SiteContent`] into a complete static site by combining:
//! - Content from a [`ContentSource`] (nestGate CAS, filesystem, or in-memory)
//! - Layout composition via [`SiteLayout`]
//! - Document compilation via [`document_compiler`](crate::modality::document_compiler)
//!
//! # Architecture (Sovereignty Evolution)
//!
//! ```text
//! ContentSource (nestGate CAS / filesystem / WASM fetch)
//!     → SiteContent (pages + nav + search_index + entity_registry)
//!     → SiteBuilder (layout + compilation)
//!     → StaticSite (path → HTML/CSS/JSON output files)
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! let source = FilesystemSource::new("content/");
//! let layout = SiteLayout::default();
//! let builder = SiteBuilder::new(layout);
//! let site = builder.build(&source.load()?)?;
//! for (path, content) in &site.files {
//!     std::fs::write(format!("public/{path}"), content)?;
//! }
//! ```

use std::collections::HashMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::document::{DocumentNode, NavSection, SiteContent};
use crate::modality::ModalityOutput;
use crate::modality::document_compiler::{compile_to_description, compile_to_html};

/// Trait for content sources that provide site content.
///
/// Implementors include:
/// - `FilesystemSource` — reads .md files from a directory (like Zola)
/// - `CasSource` — fetches content-addressed blobs from nestGate
/// - `InMemorySource` — holds pre-loaded content (WASM, testing)
pub trait ContentSource: Send + Sync {
    /// Load all site content from this source.
    ///
    /// # Errors
    /// Returns error if content cannot be loaded (IO, network, CAS resolution).
    fn load(&self) -> Result<SiteContent, ContentError>;

    /// Human-readable source identifier for diagnostics.
    fn source_id(&self) -> &str;
}

/// Errors from content loading or site building.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentError {
    NotFound(String),
    LoadFailed(String),
    InvalidContent(String),
}

impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "content not found: {msg}"),
            Self::LoadFailed(msg) => write!(f, "content load failed: {msg}"),
            Self::InvalidContent(msg) => write!(f, "invalid content: {msg}"),
        }
    }
}

impl std::error::Error for ContentError {}

/// Layout configuration for site-wide chrome (header, nav, CSS, footer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteLayout {
    /// Site title displayed in header and `<title>` suffix.
    pub site_title: String,
    /// Optional site description for meta tags.
    pub description: Option<String>,
    /// CSS to inline in `<style>` (or link paths).
    pub css: SiteCss,
    /// Footer HTML content.
    pub footer_html: String,
    /// Base URL for absolute links (e.g. `https://sporeprint.primals.eco`).
    pub base_url: String,
    /// Whether to generate a search index JSON file.
    pub generate_search_index: bool,
}

/// CSS strategy for the site layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SiteCss {
    /// Inline CSS in a `<style>` block (zero external dependencies).
    Inline(String),
    /// Link to external stylesheet path(s).
    Links(Vec<String>),
}

impl Default for SiteLayout {
    fn default() -> Self {
        Self {
            site_title: "primals.eco".to_owned(),
            description: None,
            css: SiteCss::Inline(default_css().to_owned()),
            footer_html: default_footer().to_owned(),
            base_url: String::new(),
            generate_search_index: true,
        }
    }
}

/// A compiled static site — collection of output files keyed by path.
#[derive(Debug, Clone, Default)]
pub struct StaticSite {
    /// Output files: path (e.g. `index.html`, `primals/beardog/index.html`) → content bytes.
    pub files: HashMap<String, Bytes>,
}

impl StaticSite {
    /// Total number of output files.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Total bytes across all output files.
    #[must_use]
    pub fn total_bytes(&self) -> usize {
        self.files.values().map(Bytes::len).sum()
    }
}

/// Builds a static site from content and layout.
pub struct SiteBuilder {
    layout: SiteLayout,
}

impl SiteBuilder {
    /// Create a new site builder with the given layout.
    #[must_use]
    pub const fn new(layout: SiteLayout) -> Self {
        Self { layout }
    }

    /// Build the complete static site from loaded content.
    ///
    /// # Errors
    /// Returns error if any page fails to compile.
    pub fn build(&self, content: &SiteContent) -> Result<StaticSite, ContentError> {
        let mut site = StaticSite::default();

        for page in &content.pages {
            if let DocumentNode::Page { meta, .. } = page {
                let path = self.page_output_path(&meta.path);
                let html = self.render_page(page, &content.nav);
                site.files.insert(path, Bytes::from(html));
            }
        }

        if self.layout.generate_search_index {
            let index_json = serde_json::to_vec(&content.search_index)
                .map_err(|e| ContentError::InvalidContent(format!("search index: {e}")))?;
            site.files
                .insert("search-index.json".to_owned(), Bytes::from(index_json));
        }

        Ok(site)
    }

    /// Render a single page with full layout composition.
    #[must_use]
    pub fn render_page(&self, page: &DocumentNode, nav: &[NavSection]) -> String {
        let page_html = match compile_to_html(page) {
            ModalityOutput::Svg(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            _ => String::new(),
        };

        let page_desc = match compile_to_description(page) {
            ModalityOutput::Description(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            _ => String::new(),
        };

        let (title, description) = if let DocumentNode::Page { meta, .. } = page {
            (
                meta.title.clone(),
                meta.description.clone().unwrap_or_default(),
            )
        } else {
            (String::new(), String::new())
        };

        let full_title = if title.is_empty() {
            self.layout.site_title.clone()
        } else {
            format!("{title} — {}", self.layout.site_title)
        };

        let nav_html = self.render_nav(nav);
        let css_block = self.render_css();

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{full_title}</title>
<meta name="description" content="{description}" />
<meta name="generator" content="petalTongue {version}" />
{css_block}
</head>
<body>
<header class="site-header">
<a class="site-title" href="/">{site_title}</a>
</header>
<div class="layout">
{nav_html}
<main>
<article aria-label="{title}">
{page_html}
</article>
</main>
</div>
<footer class="site-footer">
{footer}
</footer>
<script type="application/ld+json">{{"@context":"https://schema.org","@type":"WebPage","name":"{title}","description":"{description}"}}</script>
<!-- Accessibility: full text description available -->
<!-- {desc_preview} -->
</body>
</html>"#,
            version = env!("CARGO_PKG_VERSION"),
            site_title = html_escape(&self.layout.site_title),
            footer = self.layout.footer_html,
            desc_preview = page_desc.chars().take(100).collect::<String>(),
        )
    }

    #[expect(
        clippy::unused_self,
        reason = "will use self for layout-specific nav config"
    )]
    fn render_nav(&self, nav: &[NavSection]) -> String {
        if nav.is_empty() {
            return String::new();
        }
        let node = DocumentNode::NavTree {
            sections: nav.to_vec(),
        };
        match compile_to_html(&node) {
            ModalityOutput::Svg(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            _ => String::new(),
        }
    }

    fn render_css(&self) -> String {
        match &self.layout.css {
            SiteCss::Inline(css) => format!("<style>\n{css}\n</style>"),
            SiteCss::Links(links) => links
                .iter()
                .map(|l| format!("<link rel=\"stylesheet\" href=\"{l}\" />"))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    #[expect(
        clippy::unused_self,
        reason = "will use self for base_url prefix logic"
    )]
    fn page_output_path(&self, page_path: &str) -> String {
        let clean = page_path.trim_start_matches('/');
        if clean.is_empty() || clean == "/" {
            "index.html".to_owned()
        } else if clean.ends_with('/') {
            format!("{clean}index.html")
        } else if clean
            .rsplit_once('.')
            .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("html"))
        {
            clean.to_owned()
        } else {
            format!("{clean}/index.html")
        }
    }
}

/// In-memory content source for testing and WASM usage.
pub struct InMemorySource {
    content: SiteContent,
    id: String,
}

impl InMemorySource {
    /// Create from pre-loaded content.
    #[must_use]
    pub fn new(content: SiteContent) -> Self {
        Self {
            content,
            id: "in-memory".to_owned(),
        }
    }
}

impl ContentSource for InMemorySource {
    fn load(&self) -> Result<SiteContent, ContentError> {
        Ok(self.content.clone())
    }

    fn source_id(&self) -> &str {
        &self.id
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[expect(
    clippy::needless_raw_string_hashes,
    reason = "CSS hex colors read better in r#\"...\"#"
)]
const fn default_css() -> &'static str {
    r#":root { --fg: #1a1a2e; --bg: #fafafa; --accent: #16697a; --border: #e0e0e0; }
@media (prefers-color-scheme: dark) { :root { --fg: #e0e0e0; --bg: #1a1a2e; --accent: #82c4c4; --border: #333; } }
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: system-ui, -apple-system, sans-serif; color: var(--fg); background: var(--bg); line-height: 1.6; }
.site-header { padding: 1rem 2rem; border-bottom: 1px solid var(--border); }
.site-title { color: var(--accent); text-decoration: none; font-weight: 700; font-size: 1.25rem; }
.layout { display: grid; grid-template-columns: 240px 1fr; gap: 2rem; max-width: 1200px; margin: 0 auto; padding: 2rem; }
@media (max-width: 768px) { .layout { grid-template-columns: 1fr; } }
nav.site-nav { font-size: 0.9rem; }
nav.site-nav ul { list-style: none; padding-left: 1rem; }
nav.site-nav a { color: var(--accent); text-decoration: none; }
nav.site-nav .current a { font-weight: 700; }
main { min-width: 0; }
article { max-width: 72ch; }
article h1, article h2, article h3 { margin-top: 1.5em; margin-bottom: 0.5em; }
article p { margin-bottom: 1em; }
article code { background: var(--border); padding: 0.1em 0.3em; border-radius: 3px; font-size: 0.9em; }
article pre { background: var(--border); padding: 1rem; border-radius: 6px; overflow-x: auto; margin: 1em 0; }
article pre code { background: none; padding: 0; }
article table { border-collapse: collapse; width: 100%; margin: 1em 0; }
article th, article td { border: 1px solid var(--border); padding: 0.5em; text-align: left; }
article blockquote { border-left: 3px solid var(--accent); padding-left: 1em; margin: 1em 0; opacity: 0.85; }
.entity-ref { color: var(--accent); font-weight: 500; }
.entity-metrics { font-size: 0.85rem; opacity: 0.7; margin: 0.25em 0; }
.site-footer { padding: 2rem; border-top: 1px solid var(--border); text-align: center; font-size: 0.85rem; opacity: 0.7; margin-top: 4rem; }"#
}

const fn default_footer() -> &'static str {
    r#"<p>Generated by <a href="https://git.primals.eco/ecoPrimals/petalTongue">petalTongue</a> — sovereign static site rendering</p>"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentNode, Inline, NavPage, NavSection, PageMeta, SearchEntry};

    fn sample_content() -> SiteContent {
        SiteContent {
            pages: vec![
                DocumentNode::Page {
                    meta: PageMeta {
                        title: "Home".into(),
                        description: Some("Welcome to primals.eco".into()),
                        path: "/".into(),
                        ..PageMeta::default()
                    },
                    body: vec![DocumentNode::Paragraph {
                        inlines: vec![Inline::Text("Welcome to the ecosystem.".into())],
                    }],
                },
                DocumentNode::Page {
                    meta: PageMeta {
                        title: "About".into(),
                        description: Some("About ecoPrimals".into()),
                        path: "/about".into(),
                        ..PageMeta::default()
                    },
                    body: vec![DocumentNode::Heading {
                        level: 2,
                        inlines: vec![Inline::Text("About Us".into())],
                        id: "about-us".into(),
                    }],
                },
            ],
            nav: vec![NavSection {
                title: "Pages".into(),
                path: "/".into(),
                active: true,
                pages: vec![
                    NavPage {
                        title: "Home".into(),
                        path: "/".into(),
                        current: true,
                    },
                    NavPage {
                        title: "About".into(),
                        path: "/about".into(),
                        current: false,
                    },
                ],
            }],
            search_index: vec![SearchEntry {
                title: "Home".into(),
                path: "/".into(),
                description: Some("Welcome page".into()),
                body_preview: "Welcome to the ecosystem.".into(),
            }],
            entity_registry: HashMap::new(),
        }
    }

    #[test]
    fn site_builder_produces_pages() {
        let content = sample_content();
        let layout = SiteLayout::default();
        let builder = SiteBuilder::new(layout);
        let site = builder.build(&content).expect("build should succeed");

        assert!(site.files.contains_key("index.html"));
        assert!(site.files.contains_key("about/index.html"));
        assert!(site.files.contains_key("search-index.json"));
        assert_eq!(site.file_count(), 3);
    }

    #[test]
    fn rendered_page_has_layout() {
        let content = sample_content();
        let layout = SiteLayout::default();
        let builder = SiteBuilder::new(layout);
        let site = builder.build(&content).unwrap();

        let home = String::from_utf8(site.files["index.html"].to_vec()).unwrap();
        assert!(home.contains("<!DOCTYPE html>"));
        assert!(home.contains("primals.eco"));
        assert!(home.contains("site-header"));
        assert!(home.contains("site-footer"));
        assert!(home.contains("Welcome to the ecosystem"));
        assert!(home.contains("petalTongue"));
    }

    #[test]
    fn rendered_page_has_nav() {
        let content = sample_content();
        let layout = SiteLayout::default();
        let builder = SiteBuilder::new(layout);
        let site = builder.build(&content).unwrap();

        let home = String::from_utf8(site.files["index.html"].to_vec()).unwrap();
        assert!(home.contains("site-nav"));
        assert!(home.contains("About"));
    }

    #[test]
    fn rendered_page_has_css() {
        let content = sample_content();
        let layout = SiteLayout::default();
        let builder = SiteBuilder::new(layout);
        let site = builder.build(&content).unwrap();

        let home = String::from_utf8(site.files["index.html"].to_vec()).unwrap();
        assert!(home.contains("<style>"));
        assert!(home.contains("--accent"));
    }

    #[test]
    fn in_memory_source_loads() {
        let content = sample_content();
        let source = InMemorySource::new(content.clone());
        assert_eq!(source.source_id(), "in-memory");
        let loaded = source.load().unwrap();
        assert_eq!(loaded.pages.len(), content.pages.len());
    }

    #[test]
    fn search_index_generated() {
        let content = sample_content();
        let layout = SiteLayout::default();
        let builder = SiteBuilder::new(layout);
        let site = builder.build(&content).unwrap();

        let index = String::from_utf8(site.files["search-index.json"].to_vec()).unwrap();
        let entries: Vec<SearchEntry> = serde_json::from_str(&index).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Home");
    }

    #[test]
    fn site_builder_without_search_index() {
        let content = sample_content();
        let layout = SiteLayout {
            generate_search_index: false,
            ..SiteLayout::default()
        };
        let builder = SiteBuilder::new(layout);
        let site = builder.build(&content).unwrap();
        assert!(!site.files.contains_key("search-index.json"));
        assert_eq!(site.file_count(), 2);
    }

    #[test]
    fn page_output_path_mapping() {
        let layout = SiteLayout::default();
        let builder = SiteBuilder::new(layout);
        assert_eq!(builder.page_output_path("/"), "index.html");
        assert_eq!(builder.page_output_path("/about"), "about/index.html");
        assert_eq!(
            builder.page_output_path("/primals/beardog"),
            "primals/beardog/index.html"
        );
        assert_eq!(builder.page_output_path("/docs/api.html"), "docs/api.html");
    }
}
