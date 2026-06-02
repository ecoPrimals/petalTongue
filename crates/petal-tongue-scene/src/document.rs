// SPDX-License-Identifier: AGPL-3.0-or-later
//! Document content model — rich structured content for the Universal User Interface.
//!
//! This module defines a document-oriented scene representation parallel to
//! the Grammar of Graphics pipeline. Where `GrammarExpr` models data
//! visualizations, `DocumentNode` models rich text content (articles, pages,
//! navigation) that compiles to the same modality outputs:
//!
//! ```text
//! Markdown + TOML front matter
//!     -> FrontMatterParser (split +++ blocks)
//!     -> MarkdownCompiler (pulldown-cmark -> DocumentNode tree)
//!     -> ShortcodeExpander (entity references from registry)
//!     -> DocumentCompiler (DocumentNode -> ModalityOutput)
//!         -> HTML (visual)
//!         -> Description (screen reader)
//!         -> Braille (tactile display)
//!         -> Audio (spoken content with sonified navigation)
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata extracted from TOML `+++` front matter.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PageMeta {
    pub title: String,
    pub description: Option<String>,
    pub date: Option<String>,
    pub weight: Option<i32>,
    pub section: Option<String>,
    pub path: String,
    pub taxonomies: HashMap<String, Vec<String>>,
    pub extra: HashMap<String, toml::Value>,
}

/// A resolved entity reference for shortcode expansion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef {
    pub key: String,
    pub display: String,
    pub emoji: String,
    pub href: Option<String>,
    pub description: Option<String>,
}

/// Inline content elements within a paragraph or heading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Inline {
    Text(String),
    Bold(Vec<Self>),
    Italic(Vec<Self>),
    Strikethrough(Vec<Self>),
    Code(String),
    Link {
        text: Vec<Self>,
        href: String,
        title: Option<String>,
    },
    Image {
        alt: String,
        src: String,
        title: Option<String>,
    },
    Entity(EntityRef),
    LineBreak,
}

/// A structured document node — the intermediate representation for content.
///
/// Designed for multi-modal compilation: the same `DocumentNode` tree can
/// produce HTML for sighted users, structured text for screen readers,
/// braille patterns for tactile displays, or audio with sonified navigation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentNode {
    /// A complete page with metadata and body content.
    Page { meta: PageMeta, body: Vec<Self> },

    /// Heading (h1-h6) with anchor ID for navigation.
    Heading {
        level: u8,
        inlines: Vec<Inline>,
        id: String,
    },

    /// Paragraph of inline content.
    Paragraph { inlines: Vec<Inline> },

    /// Fenced code block with optional language annotation.
    CodeBlock {
        language: Option<String>,
        content: String,
    },

    /// Block quote (recursive content).
    BlockQuote { children: Vec<Self> },

    /// Ordered or unordered list.
    List {
        ordered: bool,
        start: Option<u64>,
        items: Vec<ListItem>,
    },

    /// Table with headers and rows.
    Table {
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },

    /// Horizontal rule / thematic break.
    ThematicBreak,

    /// Entity reference shortcode (resolved from registry).
    EntityReference(EntityRef),

    /// Entity metrics line (LOC/tests/files).
    EntityMetrics {
        key: String,
        loc_display: String,
        tests_display: String,
        files: Option<u64>,
        crates: Option<u64>,
    },

    /// Navigation tree (site structure for sidebar).
    NavTree { sections: Vec<NavSection> },

    /// Raw HTML passthrough (escape hatch for embedded content).
    RawHtml(String),
}

/// A list item, which may contain nested block content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub content: Vec<DocumentNode>,
}

/// A section in the navigation tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavSection {
    pub title: String,
    pub path: String,
    pub pages: Vec<NavPage>,
    pub active: bool,
}

/// A page entry in navigation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavPage {
    pub title: String,
    pub path: String,
    pub current: bool,
}

/// Search index entry for full-text search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchEntry {
    pub title: String,
    pub path: String,
    pub description: Option<String>,
    pub body_preview: String,
}

/// The complete site content model — loaded at startup from a content provider.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SiteContent {
    pub pages: Vec<DocumentNode>,
    pub nav: Vec<NavSection>,
    pub search_index: Vec<SearchEntry>,
    pub entity_registry: HashMap<String, EntityRegistryEntry>,
}

/// An entity in the registry — configurable content entity with display metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EntityRegistryEntry {
    pub display: String,
    pub emoji: String,
    pub kind: String,
    pub description: Option<String>,
    pub page: Option<String>,
    pub repo: Option<String>,
    pub domain: Option<String>,
    pub loc: Option<u64>,
    pub loc_display: Option<String>,
    pub tests: Option<u64>,
    pub tests_display: Option<String>,
    pub files: Option<u64>,
    pub crates: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_meta_defaults() {
        let meta = PageMeta::default();
        assert!(meta.title.is_empty());
        assert!(meta.taxonomies.is_empty());
    }

    #[test]
    fn document_node_serializes() {
        let node = DocumentNode::Heading {
            level: 1,
            inlines: vec![Inline::Text("Hello".into())],
            id: "hello".into(),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("Hello"));
    }

    #[test]
    fn entity_ref_round_trips() {
        let entity = EntityRef {
            key: "beardog".into(),
            display: "BearDog".into(),
            emoji: "🐻🐕".into(),
            href: Some("/primals/beardog/".into()),
            description: Some("Crypto identity".into()),
        };
        let json = serde_json::to_string(&entity).unwrap();
        let back: EntityRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back.key, "beardog");
        assert_eq!(back.emoji, "🐻🐕");
    }

    #[test]
    fn site_content_empty_default() {
        let site = SiteContent::default();
        assert!(site.pages.is_empty());
        assert!(site.entity_registry.is_empty());
    }
}
