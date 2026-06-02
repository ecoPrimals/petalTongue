// SPDX-License-Identifier: AGPL-3.0-or-later
//! Site structure scanning — entity registry + navigation tree.
//!
//! Filesystem operations that scan a content directory to build
//! the entity registry and navigation sidebar for `content-direct`.

use std::collections::HashMap;

use petal_tongue_scene::document::{EntityRegistryEntry, NavPage, NavSection};

use super::split_front_matter;

/// Load entity registry from a content site's `config.toml` file.
///
/// Parses `[extra.entity_registry.<key>]` sections into the typed registry map.
pub fn load_entity_registry(config_path: &std::path::Path) -> HashMap<String, EntityRegistryEntry> {
    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(?config_path, error = %e, "Failed to read config.toml");
            return HashMap::new();
        }
    };

    let table: toml::Table = match toml::from_str(&content) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse config.toml");
            return HashMap::new();
        }
    };

    let Some(extra) = table.get("extra").and_then(|v| v.as_table()) else {
        return HashMap::new();
    };
    let Some(registry_table) = extra.get("entity_registry").and_then(|v| v.as_table()) else {
        return HashMap::new();
    };

    let mut registry = HashMap::with_capacity(registry_table.len());

    for (key, value) in registry_table {
        let Some(entry_table) = value.as_table() else {
            continue;
        };

        registry.insert(key.clone(), parse_entity_entry(key, entry_table));
    }

    registry
}

fn parse_entity_entry(key: &str, t: &toml::Table) -> EntityRegistryEntry {
    EntityRegistryEntry {
        display: t
            .get("display")
            .and_then(|v| v.as_str())
            .unwrap_or(key)
            .to_string(),
        emoji: t
            .get("emoji")
            .and_then(|v| v.as_str())
            .unwrap_or("")
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

/// Build a navigation tree from a content directory structure.
///
/// Reads `_index.md` front matter from each subdirectory to get section titles.
/// Returns sorted `NavSection` entries for sidebar rendering.
pub fn build_nav_tree(content_dir: &std::path::Path) -> Vec<NavSection> {
    let mut sections: Vec<(i32, NavSection)> = Vec::new();

    let Ok(entries) = std::fs::read_dir(content_dir) else {
        return Vec::new();
    };

    let mut dirs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();
    dirs.sort_by_key(std::fs::DirEntry::file_name);

    for entry in dirs {
        let dir_path = entry.path();
        let dir_name = entry.file_name().to_string_lossy().to_string();

        let index_path = dir_path.join("_index.md");
        let (title, weight) = read_section_meta(&index_path, &dir_name);

        let pages = scan_section_pages(&dir_path, &dir_name);

        sections.push((
            weight,
            NavSection {
                title,
                path: format!("/{dir_name}/"),
                pages,
                active: false,
            },
        ));
    }

    sections.sort_by_key(|(w, _)| *w);
    sections.into_iter().map(|(_, s)| s).collect()
}

fn read_section_meta(index_path: &std::path::Path, dir_name: &str) -> (String, i32) {
    if !index_path.is_file() {
        return (dir_name.to_string(), 999);
    }

    let Ok(content) = std::fs::read_to_string(index_path) else {
        return (dir_name.to_string(), 999);
    };

    let (fm, _) = split_front_matter(&content);
    let Some(toml_str) = fm else {
        return (dir_name.to_string(), 999);
    };

    let tbl: toml::Table = toml::from_str(toml_str).unwrap_or_default();

    let title = tbl
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(dir_name)
        .to_string();

    let weight = tbl
        .get("weight")
        .and_then(toml::Value::as_integer)
        .map_or(999, |w| w as i32);

    (title, weight)
}

fn scan_section_pages(dir_path: &std::path::Path, dir_name: &str) -> Vec<NavPage> {
    let Ok(files) = std::fs::read_dir(dir_path) else {
        return Vec::new();
    };

    let mut file_entries: Vec<_> = files
        .flatten()
        .filter(|f| {
            let p = f.path();
            p.is_file() && p.extension().is_some_and(|e| e == "md") && f.file_name() != "_index.md"
        })
        .collect();
    file_entries.sort_by_key(std::fs::DirEntry::file_name);

    file_entries
        .into_iter()
        .map(|file_entry| {
            let file_path = file_entry.path();
            let stem = file_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let page_title = std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|content| {
                    let (fm, _) = split_front_matter(&content);
                    fm.and_then(|toml_str| {
                        let tbl: toml::Table = toml::from_str(toml_str).ok()?;
                        tbl.get("title").and_then(|v| v.as_str()).map(String::from)
                    })
                })
                .unwrap_or_else(|| stem.clone());

            NavPage {
                title: page_title,
                path: format!("/{dir_name}/{stem}/"),
                current: false,
            }
        })
        .collect()
}
