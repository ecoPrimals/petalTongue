// SPDX-License-Identifier: AGPL-3.0-or-later
//! `FilesystemSource` — Zola-compatible directory scanner implementing `ContentSource`.
//!
//! Reads a content directory structured as:
//! ```text
//! content/
//! ├── config.toml         (entity registry + site metadata)
//! ├── _index.md           (homepage)
//! ├── primals/
//! │   ├── _index.md       (section index — title, weight)
//! │   ├── beardog.md
//! │   └── songbird.md
//! └── architecture/
//!     ├── _index.md
//!     └── tower.md
//! ```
//!
//! Front matter format: TOML `+++` delimiters (Zola-compatible).

use std::path::{Path, PathBuf};

use petal_tongue_scene::document::{DocumentNode, NavSection, SearchEntry, SiteContent};
use petal_tongue_scene::site_builder::{ContentError, ContentSource};

use super::{build_nav_tree, load_entity_registry, parse_document, resolve_shortcodes};

/// Reads markdown content from a filesystem directory tree.
///
/// Compatible with Zola-style content directories: TOML `+++` front matter,
/// `_index.md` section indexes, `config.toml` entity registry.
pub struct FilesystemSource {
    content_dir: PathBuf,
    config_path: Option<PathBuf>,
}

impl FilesystemSource {
    /// Create a new filesystem source rooted at the given content directory.
    ///
    /// If `config_path` is `None`, looks for `config.toml` in the content directory.
    #[must_use]
    pub const fn new(content_dir: PathBuf, config_path: Option<PathBuf>) -> Self {
        Self {
            content_dir,
            config_path,
        }
    }

    fn effective_config_path(&self) -> PathBuf {
        self.config_path
            .clone()
            .unwrap_or_else(|| self.content_dir.join("config.toml"))
    }

    fn scan_pages(&self) -> Result<Vec<DocumentNode>, ContentError> {
        let mut pages = Vec::new();

        let root_index = self.content_dir.join("_index.md");
        if root_index.is_file() {
            let page = load_page_file(&root_index, "/")?;
            pages.push(page);
        }

        scan_directory(&self.content_dir, "", &mut pages)?;
        Ok(pages)
    }
}

impl ContentSource for FilesystemSource {
    fn load(&self) -> Result<SiteContent, ContentError> {
        let entity_registry = load_entity_registry(&self.effective_config_path());
        let nav = build_nav_tree(&self.content_dir);
        let mut pages = self.scan_pages()?;

        resolve_shortcodes(&mut pages, &entity_registry);

        let search_index = build_search_index(&pages);

        Ok(SiteContent {
            pages,
            nav,
            search_index,
            entity_registry,
        })
    }

    #[expect(
        clippy::unnecessary_literal_bound,
        reason = "trait ContentSource defines return as &str"
    )]
    fn source_id(&self) -> &str {
        "filesystem"
    }
}

fn scan_directory(
    dir: &Path,
    prefix: &str,
    pages: &mut Vec<DocumentNode>,
) -> Result<(), ContentError> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| ContentError::LoadFailed(format!("cannot read {}: {e}", dir.display())))?;

    let mut items: Vec<_> = entries.flatten().collect();
    items.sort_by_key(std::fs::DirEntry::file_name);

    for entry in items {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if path.is_dir() {
            let section_prefix = if prefix.is_empty() {
                format!("/{name}")
            } else {
                format!("{prefix}/{name}")
            };
            scan_directory(&path, &section_prefix, pages)?;
        } else if path.extension().is_some_and(|e| e == "md") && name != "_index.md" {
            let stem = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let page_path = if prefix.is_empty() {
                format!("/{stem}")
            } else {
                format!("{prefix}/{stem}")
            };
            let page = load_page_file(&path, &page_path)?;
            pages.push(page);
        }
    }
    Ok(())
}

fn load_page_file(path: &Path, url_path: &str) -> Result<DocumentNode, ContentError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ContentError::LoadFailed(format!("cannot read {}: {e}", path.display())))?;
    Ok(parse_document(&content, url_path))
}

fn build_search_index(pages: &[DocumentNode]) -> Vec<SearchEntry> {
    pages
        .iter()
        .filter_map(|page| {
            if let DocumentNode::Page { meta, body } = page {
                let body_preview = extract_text_preview(body, 200);
                Some(SearchEntry {
                    title: meta.title.clone(),
                    path: meta.path.clone(),
                    description: meta.description.clone(),
                    body_preview,
                })
            } else {
                None
            }
        })
        .collect()
}

fn extract_text_preview(nodes: &[DocumentNode], max_chars: usize) -> String {
    let mut text = String::new();
    collect_text(nodes, &mut text, max_chars);
    text
}

fn collect_text(nodes: &[DocumentNode], buf: &mut String, max: usize) {
    for node in nodes {
        if buf.len() >= max {
            break;
        }
        match node {
            DocumentNode::Paragraph { inlines } | DocumentNode::Heading { inlines, .. } => {
                collect_inlines(inlines, buf, max);
            }
            DocumentNode::BlockQuote { children } | DocumentNode::Page { body: children, .. } => {
                collect_text(children, buf, max);
            }
            DocumentNode::List { items, .. } => {
                for item in items {
                    collect_text(&item.content, buf, max);
                }
            }
            _ => {}
        }
    }
}

fn collect_inlines(inlines: &[petal_tongue_scene::document::Inline], buf: &mut String, max: usize) {
    use petal_tongue_scene::document::Inline;

    for inline in inlines {
        if buf.len() >= max {
            break;
        }
        match inline {
            Inline::Text(t) | Inline::Code(t) => {
                buf.push_str(t);
                buf.push(' ');
            }
            Inline::Bold(children) | Inline::Italic(children) | Inline::Strikethrough(children) => {
                collect_inlines(children, buf, max);
            }
            Inline::Link { text, .. } => {
                collect_inlines(text, buf, max);
            }
            _ => {}
        }
    }
}

/// Build nav tree with an active section marker based on the current page path.
#[allow(
    dead_code,
    reason = "public API for sporePrint page rendering with active navigation"
)]
pub fn nav_with_active(nav: &[NavSection], current_path: &str) -> Vec<NavSection> {
    nav.iter()
        .map(|section| {
            let active = current_path.starts_with(&section.path);
            NavSection {
                title: section.title.clone(),
                path: section.path.clone(),
                pages: section
                    .pages
                    .iter()
                    .map(|p| petal_tongue_scene::document::NavPage {
                        title: p.title.clone(),
                        path: p.path.clone(),
                        current: p.path == current_path,
                    })
                    .collect(),
                active,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_content(dir: &Path) {
        std::fs::write(
            dir.join("_index.md"),
            "+++\ntitle = \"Home\"\n+++\n\nWelcome to the site.",
        )
        .unwrap();

        let section = dir.join("docs");
        std::fs::create_dir_all(&section).unwrap();
        std::fs::write(
            section.join("_index.md"),
            "+++\ntitle = \"Documentation\"\nweight = 1\n+++\n",
        )
        .unwrap();
        std::fs::write(
            section.join("getting-started.md"),
            "+++\ntitle = \"Getting Started\"\ndescription = \"How to begin\"\n+++\n\n# Getting Started\n\nInstall with `cargo install`.",
        )
        .unwrap();
        std::fs::write(
            section.join("architecture.md"),
            "+++\ntitle = \"Architecture\"\n+++\n\n# Architecture\n\nPure Rust, modular crates.",
        )
        .unwrap();

        let mut config = std::fs::File::create(dir.join("config.toml")).unwrap();
        writeln!(
            config,
            "[extra.entity_registry.petaltongue]\ndisplay = \"petalTongue\"\nemoji = \"\"\nkind = \"primal\"\ndescription = \"Universal UI\""
        )
        .unwrap();
    }

    #[test]
    fn filesystem_source_loads_content() {
        let tmp = TempDir::new().unwrap();
        create_test_content(tmp.path());

        let source = FilesystemSource::new(tmp.path().to_path_buf(), None);
        let content = source.load().unwrap();

        assert_eq!(content.pages.len(), 3);
        assert!(!content.nav.is_empty());
        assert!(!content.search_index.is_empty());
        assert!(content.entity_registry.contains_key("petaltongue"));
    }

    #[test]
    fn filesystem_source_search_index() {
        let tmp = TempDir::new().unwrap();
        create_test_content(tmp.path());

        let source = FilesystemSource::new(tmp.path().to_path_buf(), None);
        let content = source.load().unwrap();

        let gs_entry = content
            .search_index
            .iter()
            .find(|e| e.title == "Getting Started");
        assert!(gs_entry.is_some());
        let entry = gs_entry.unwrap();
        assert_eq!(entry.description.as_deref(), Some("How to begin"));
        assert!(entry.body_preview.contains("cargo install"));
    }

    #[test]
    fn filesystem_source_builds_full_site() {
        let tmp = TempDir::new().unwrap();
        create_test_content(tmp.path());

        let source = FilesystemSource::new(tmp.path().to_path_buf(), None);
        let content = source.load().unwrap();

        let layout = petal_tongue_scene::site_builder::SiteLayout::default();
        let builder = petal_tongue_scene::site_builder::SiteBuilder::new(layout);
        let site = builder.build(&content).unwrap();

        assert!(site.files.contains_key("index.html"));
        assert!(site.files.contains_key("docs/getting-started/index.html"));
        assert!(site.files.contains_key("docs/architecture/index.html"));
        assert!(site.files.contains_key("search-index.json"));

        let home = String::from_utf8(site.files["index.html"].to_vec()).unwrap();
        assert!(home.contains("Welcome to the site"));
    }

    #[test]
    fn filesystem_source_missing_dir() {
        let source = FilesystemSource::new(PathBuf::from("/nonexistent/path"), None);
        let result = source.load();
        assert!(result.is_err());
    }

    #[test]
    fn nav_with_active_marks_current() {
        let nav = vec![NavSection {
            title: "Docs".into(),
            path: "/docs/".into(),
            pages: vec![
                petal_tongue_scene::document::NavPage {
                    title: "Intro".into(),
                    path: "/docs/intro/".into(),
                    current: false,
                },
                petal_tongue_scene::document::NavPage {
                    title: "API".into(),
                    path: "/docs/api/".into(),
                    current: false,
                },
            ],
            active: false,
        }];

        let result = nav_with_active(&nav, "/docs/api/");
        assert!(result[0].active);
        assert!(!result[0].pages[0].current);
        assert!(result[0].pages[1].current);
    }
}
