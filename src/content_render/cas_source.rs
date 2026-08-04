// SPDX-License-Identifier: AGPL-3.0-or-later
//! `CasSource` — content-addressed storage source implementing `ContentSource`.
//!
//! Resolves site content from nestGate CAS, supporting two access modes:
//!
//! 1. **Shared filesystem** — reads blobs directly from the CAS storage directory
//!    (same as `coord_handlers.rs` uses for coordination data).
//! 2. **JSON-RPC** — calls `cas.get` on a nestGate endpoint for cross-gate access.
//!
//! The CAS manifest (`site-manifest.json`) maps page paths to content hashes:
//!
//! ```json
//! {
//!   "version": 1,
//!   "pages": [
//!     { "path": "/", "hash": "sha256:abc123...", "title": "Home" },
//!     { "path": "/about", "hash": "sha256:def456...", "title": "About" }
//!   ],
//!   "nav": [...],
//!   "config_hash": "sha256:..."
//! }
//! ```
//!
//! # Architecture (sporePrint Pipeline)
//!
//! ```text
//! nestGate CAS (filesystem or JSON-RPC)
//!     → CasManifest (site-manifest.json)
//!     → resolve hashes → markdown content
//!     → parse_document → SiteContent
//!     → SiteBuilder → StaticSite
//! ```

use std::path::{Path, PathBuf};

use petal_tongue_scene::document::{NavSection, SearchEntry, SiteContent};
use petal_tongue_scene::site_builder::{ContentError, ContentSource};
use serde::{Deserialize, Serialize};

use super::{parse_document, resolve_shortcodes};

/// CAS manifest that indexes all pages in a site by content hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasManifest {
    pub version: u32,
    pub pages: Vec<CasPageEntry>,
    #[serde(default)]
    pub nav: Vec<NavSection>,
    pub config_hash: Option<String>,
}

/// A page entry in the CAS manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasPageEntry {
    pub path: String,
    pub hash: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub weight: Option<i32>,
    #[serde(default)]
    pub section: Option<String>,
}

/// Access mode for CAS blob resolution.
#[derive(Debug, Clone)]
pub enum CasAccess {
    /// Shared filesystem — read blobs directly from storage path.
    Filesystem(PathBuf),
}

/// Resolves site content from nestGate content-addressed storage.
///
/// Reads a `site-manifest.json` that maps page paths to content hashes,
/// then resolves each hash to its markdown content via the CAS backend.
pub struct CasSource {
    access: CasAccess,
    manifest_path: PathBuf,
}

impl CasSource {
    /// Create a CAS source from a filesystem storage path.
    ///
    /// Expects `site-manifest.json` at `storage_base/datasets/{family}/_site/site-manifest.json`.
    #[must_use]
    pub const fn filesystem(storage_base: PathBuf, manifest_path: PathBuf) -> Self {
        Self {
            access: CasAccess::Filesystem(storage_base),
            manifest_path,
        }
    }

    /// Create from environment variables (same discovery as `coord_handlers`).
    ///
    /// Uses `COORD_STORAGE_PATH` or XDG/system defaults.
    #[must_use]
    pub fn from_env() -> Self {
        let storage_base = resolve_storage_base();
        let family = petal_tongue_ipc::socket_path::get_family_id();
        let manifest_path = storage_base
            .join("datasets")
            .join(&family)
            .join("_site")
            .join("site-manifest.json");
        Self {
            access: CasAccess::Filesystem(storage_base),
            manifest_path,
        }
    }

    fn load_manifest(&self) -> Result<CasManifest, ContentError> {
        let content = std::fs::read_to_string(&self.manifest_path).map_err(|e| {
            ContentError::NotFound(format!(
                "CAS site manifest not found at {}: {e}",
                self.manifest_path.display()
            ))
        })?;
        serde_json::from_str(&content)
            .map_err(|e| ContentError::InvalidContent(format!("invalid CAS manifest: {e}")))
    }

    fn resolve_blob(&self, hash: &str) -> Result<String, ContentError> {
        match &self.access {
            CasAccess::Filesystem(base) => {
                let blob_path = cas_blob_path(base, hash);
                std::fs::read_to_string(&blob_path).map_err(|e| {
                    ContentError::LoadFailed(format!(
                        "CAS blob {hash} not found at {}: {e}",
                        blob_path.display()
                    ))
                })
            }
        }
    }

    fn resolve_config(
        &self,
        hash: &str,
    ) -> std::collections::HashMap<String, petal_tongue_scene::document::EntityRegistryEntry> {
        let Ok(content) = self.resolve_blob(hash) else {
            return std::collections::HashMap::new();
        };
        let Ok(table): Result<toml::Table, _> = toml::from_str(&content) else {
            return std::collections::HashMap::new();
        };
        parse_entity_registry_from_table(&table)
    }
}

impl ContentSource for CasSource {
    fn load(&self) -> Result<SiteContent, ContentError> {
        let manifest = self.load_manifest()?;

        let mut pages = Vec::with_capacity(manifest.pages.len());
        for entry in &manifest.pages {
            let markdown = self.resolve_blob(&entry.hash)?;
            let page = parse_document(&markdown, &entry.path);
            pages.push(page);
        }

        let entity_registry = manifest
            .config_hash
            .as_deref()
            .map_or_else(std::collections::HashMap::new, |h| self.resolve_config(h));

        resolve_shortcodes(&mut pages, &entity_registry);

        let search_index = build_cas_search_index(&manifest);

        Ok(SiteContent {
            pages,
            nav: manifest.nav,
            search_index,
            entity_registry,
        })
    }

    #[expect(
        clippy::unnecessary_literal_bound,
        reason = "trait ContentSource defines return as &str"
    )]
    fn source_id(&self) -> &str {
        "cas"
    }
}

/// Resolve the CAS/coordination storage base directory.
///
/// Discovery order (capability-based, zero primal-specific knowledge):
/// 1. `COORD_STORAGE_PATH` env (explicit configuration — highest priority)
/// 2. `$XDG_DATA_HOME/ecoPrimals/coord-storage` (XDG standard)
/// 3. `$HOME/.local/share/ecoPrimals/coord-storage` (XDG fallback)
/// 4. `/var/lib/ecoPrimals/coord-storage` (system-wide default)
pub fn resolve_storage_base() -> PathBuf {
    if let Ok(base) = std::env::var("COORD_STORAGE_PATH") {
        return PathBuf::from(base);
    }

    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let eco_path = PathBuf::from(xdg).join("ecoPrimals/coord-storage");
        if eco_path.exists() {
            return eco_path;
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let eco_path = PathBuf::from(&home).join(".local/share/ecoPrimals/coord-storage");
        if eco_path.exists() {
            return eco_path;
        }
    }

    PathBuf::from("/var/lib/ecoPrimals/coord-storage")
}

fn cas_blob_path(storage_base: &Path, hash: &str) -> PathBuf {
    let clean_hash = hash
        .strip_prefix("sha256:")
        .or_else(|| hash.strip_prefix("blake3:"))
        .unwrap_or(hash);
    storage_base.join("blobs").join(clean_hash)
}

fn build_cas_search_index(manifest: &CasManifest) -> Vec<SearchEntry> {
    manifest
        .pages
        .iter()
        .map(|entry| SearchEntry {
            title: entry.title.clone(),
            path: entry.path.clone(),
            description: entry.description.clone(),
            body_preview: entry.description.clone().unwrap_or_default(),
        })
        .collect()
}

/// Parse entity registry from a TOML table (extracted for reuse).
pub fn parse_entity_registry_from_table(
    table: &toml::Table,
) -> std::collections::HashMap<String, petal_tongue_scene::document::EntityRegistryEntry> {
    let Some(extra) = table.get("extra").and_then(|v| v.as_table()) else {
        return std::collections::HashMap::new();
    };
    let Some(registry_table) = extra.get("entity_registry").and_then(|v| v.as_table()) else {
        return std::collections::HashMap::new();
    };

    let mut registry = std::collections::HashMap::with_capacity(registry_table.len());
    for (key, value) in registry_table {
        let Some(entry_table) = value.as_table() else {
            continue;
        };
        registry.insert(key.clone(), parse_entity_entry(key, entry_table));
    }
    registry
}

fn parse_entity_entry(
    key: &str,
    t: &toml::Table,
) -> petal_tongue_scene::document::EntityRegistryEntry {
    petal_tongue_scene::document::EntityRegistryEntry {
        display: t
            .get("display")
            .and_then(|v| v.as_str())
            .unwrap_or(key)
            .to_string(),
        emoji: t
            .get("emoji")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        kind: t
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        description: t
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from),
        page: t.get("page").and_then(|v| v.as_str()).map(String::from),
        repo: t.get("repo").and_then(|v| v.as_str()).map(String::from),
        domain: t.get("domain").and_then(|v| v.as_str()).map(String::from),
        loc: t
            .get("loc")
            .and_then(toml::Value::as_integer)
            .and_then(|n| u64::try_from(n).ok()),
        loc_display: t
            .get("loc_display")
            .and_then(|v| v.as_str())
            .map(String::from),
        tests: t
            .get("tests")
            .and_then(toml::Value::as_integer)
            .and_then(|n| u64::try_from(n).ok()),
        tests_display: t
            .get("tests_display")
            .and_then(|v| v.as_str())
            .map(String::from),
        files: t
            .get("files")
            .and_then(toml::Value::as_integer)
            .and_then(|n| u64::try_from(n).ok()),
        crates: t
            .get("crates")
            .and_then(toml::Value::as_integer)
            .and_then(|n| u64::try_from(n).ok()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_cas(dir: &Path) -> PathBuf {
        let blobs = dir.join("blobs");
        std::fs::create_dir_all(&blobs).unwrap();

        let home_content = "+++\ntitle = \"Home\"\ndescription = \"Welcome page\"\n+++\n\n# Welcome\n\nThis is the home page.";
        let about_content = "+++\ntitle = \"About\"\n+++\n\n# About\n\nLearn more about us.";

        let home_hash = "abc123";
        let about_hash = "def456";

        std::fs::write(blobs.join(home_hash), home_content).unwrap();
        std::fs::write(blobs.join(about_hash), about_content).unwrap();

        let site_dir = dir.join("datasets").join("default").join("_site");
        std::fs::create_dir_all(&site_dir).unwrap();

        let manifest = CasManifest {
            version: 1,
            pages: vec![
                CasPageEntry {
                    path: "/".into(),
                    hash: format!("sha256:{home_hash}"),
                    title: "Home".into(),
                    description: Some("Welcome page".into()),
                    weight: None,
                    section: None,
                },
                CasPageEntry {
                    path: "/about".into(),
                    hash: format!("sha256:{about_hash}"),
                    title: "About".into(),
                    description: None,
                    weight: None,
                    section: None,
                },
            ],
            nav: vec![NavSection {
                title: "Main".into(),
                path: "/".into(),
                pages: vec![
                    petal_tongue_scene::document::NavPage {
                        title: "Home".into(),
                        path: "/".into(),
                        current: true,
                    },
                    petal_tongue_scene::document::NavPage {
                        title: "About".into(),
                        path: "/about".into(),
                        current: false,
                    },
                ],
                active: true,
            }],
            config_hash: None,
        };

        let manifest_path = site_dir.join("site-manifest.json");
        let mut f = std::fs::File::create(&manifest_path).unwrap();
        serde_json::to_writer_pretty(&mut f, &manifest).unwrap();
        writeln!(f).unwrap();

        manifest_path
    }

    #[test]
    fn cas_source_loads_from_filesystem() {
        let tmp = TempDir::new().unwrap();
        let manifest_path = create_test_cas(tmp.path());

        let source = CasSource::filesystem(tmp.path().to_path_buf(), manifest_path);
        let content = source.load().unwrap();

        assert_eq!(content.pages.len(), 2);
        assert!(!content.nav.is_empty());
        assert_eq!(content.search_index.len(), 2);
    }

    #[test]
    fn cas_source_builds_full_site() {
        let tmp = TempDir::new().unwrap();
        let manifest_path = create_test_cas(tmp.path());

        let source = CasSource::filesystem(tmp.path().to_path_buf(), manifest_path);
        let content = source.load().unwrap();

        let layout = petal_tongue_scene::site_builder::SiteLayout::default();
        let builder = petal_tongue_scene::site_builder::SiteBuilder::new(layout);
        let site = builder.build(&content).unwrap();

        assert!(site.files.contains_key("index.html"));
        assert!(site.files.contains_key("about/index.html"));

        let home = String::from_utf8(site.files["index.html"].to_vec()).unwrap();
        assert!(home.contains("Welcome"));
    }

    #[test]
    fn cas_source_missing_manifest() {
        let source = CasSource::filesystem(
            PathBuf::from("/nonexistent"),
            PathBuf::from("/nonexistent/manifest.json"),
        );
        let result = source.load();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn cas_source_missing_blob() {
        let tmp = TempDir::new().unwrap();
        let site_dir = tmp.path().join("datasets").join("default").join("_site");
        std::fs::create_dir_all(&site_dir).unwrap();
        std::fs::create_dir_all(tmp.path().join("blobs")).unwrap();

        let manifest = CasManifest {
            version: 1,
            pages: vec![CasPageEntry {
                path: "/missing".into(),
                hash: "sha256:nonexistent_hash".into(),
                title: "Missing".into(),
                description: None,
                weight: None,
                section: None,
            }],
            nav: vec![],
            config_hash: None,
        };

        let manifest_path = site_dir.join("site-manifest.json");
        let f = std::fs::File::create(&manifest_path).unwrap();
        serde_json::to_writer(&f, &manifest).unwrap();

        let source = CasSource::filesystem(tmp.path().to_path_buf(), manifest_path);
        let result = source.load();
        assert!(result.is_err());
    }

    #[test]
    fn cas_blob_path_strips_prefix() {
        let base = PathBuf::from("/storage");
        assert_eq!(
            cas_blob_path(&base, "sha256:abc123"),
            PathBuf::from("/storage/blobs/abc123")
        );
        assert_eq!(
            cas_blob_path(&base, "blake3:def456"),
            PathBuf::from("/storage/blobs/def456")
        );
        assert_eq!(
            cas_blob_path(&base, "raw_hash"),
            PathBuf::from("/storage/blobs/raw_hash")
        );
    }

    #[test]
    fn cas_manifest_roundtrip() {
        let manifest = CasManifest {
            version: 1,
            pages: vec![CasPageEntry {
                path: "/test".into(),
                hash: "sha256:test123".into(),
                title: "Test".into(),
                description: Some("A test page".into()),
                weight: Some(1),
                section: Some("docs".into()),
            }],
            nav: vec![],
            config_hash: Some("sha256:config_hash".into()),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: CasManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.pages.len(), 1);
        assert_eq!(parsed.pages[0].title, "Test");
        assert_eq!(parsed.config_hash.as_deref(), Some("sha256:config_hash"));
    }
}
