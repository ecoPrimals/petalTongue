// SPDX-License-Identifier: AGPL-3.0-or-later
//! TOML `+++` front matter extraction and parsing.

use petal_tongue_scene::document::PageMeta;
use std::collections::HashMap;

/// Parse TOML `+++` front matter from a markdown document.
///
/// Returns `(front_matter_toml, body_markdown)`.
/// If no front matter delimiters are found, the entire input is treated as body.
pub fn split_front_matter(input: &str) -> (Option<&str>, &str) {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("+++") {
        return (None, input);
    }

    let after_first = &trimmed[3..];
    let after_first = after_first.strip_prefix('\n').unwrap_or(after_first);

    after_first.find("\n+++").map_or((None, input), |end_pos| {
        let toml_content = &after_first[..end_pos];
        let body_start = end_pos + 4; // skip "\n+++"
        let body = after_first[body_start..]
            .strip_prefix('\n')
            .unwrap_or_else(|| &after_first[body_start..]);
        (Some(toml_content), body)
    })
}

/// Parse TOML front matter into a `PageMeta`.
pub fn parse_front_matter(toml_str: &str) -> PageMeta {
    let table: toml::Table = toml::from_str(toml_str).unwrap_or_default();

    let title = table
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let description = table
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);

    let date = table
        .get("date")
        .map(|v| v.to_string().trim_matches('"').to_string());

    let weight = table
        .get("weight")
        .and_then(toml::Value::as_integer)
        .map(|w| w as i32);

    let mut taxonomies = HashMap::new();
    if let Some(tax_table) = table.get("taxonomies").and_then(|v| v.as_table()) {
        for (key, val) in tax_table {
            if let Some(arr) = val.as_array() {
                let terms: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                taxonomies.insert(key.clone(), terms);
            }
        }
    }

    let mut extra = HashMap::new();
    if let Some(extra_table) = table.get("extra").and_then(|v| v.as_table()) {
        for (key, val) in extra_table {
            let json_val = serde_json::to_value(val).unwrap_or(serde_json::Value::Null);
            extra.insert(key.clone(), json_val);
        }
    }

    PageMeta {
        title,
        description,
        date,
        weight,
        section: None,
        path: String::new(),
        taxonomies,
        extra,
    }
}
