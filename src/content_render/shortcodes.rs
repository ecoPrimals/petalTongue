// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entity shortcode resolution (`{{ entity(name="...") }}`).

use petal_tongue_scene::document::{DocumentNode, EntityRef, Inline};
use std::collections::HashMap;

/// Resolve entity shortcodes in inline text across a `DocumentNode` tree.
pub fn resolve_shortcodes(
    nodes: &mut [DocumentNode],
    registry: &HashMap<String, petal_tongue_scene::document::EntityRegistryEntry>,
) {
    for node in nodes.iter_mut() {
        match node {
            DocumentNode::Page { body, .. } => resolve_shortcodes(body, registry),
            DocumentNode::Paragraph { inlines } | DocumentNode::Heading { inlines, .. } => {
                resolve_inlines(inlines, registry);
            }
            DocumentNode::BlockQuote { children } => resolve_shortcodes(children, registry),
            DocumentNode::List { items, .. } => {
                for item in items {
                    resolve_shortcodes(&mut item.content, registry);
                }
            }
            DocumentNode::Table { headers, rows } => {
                for cell in headers.iter_mut().chain(rows.iter_mut().flatten()) {
                    resolve_inlines(cell, registry);
                }
            }
            _ => {}
        }
    }
}

fn resolve_inlines(
    inlines: &mut Vec<Inline>,
    registry: &HashMap<String, petal_tongue_scene::document::EntityRegistryEntry>,
) {
    let mut resolved = Vec::with_capacity(inlines.len());
    for inline in inlines.drain(..) {
        match inline {
            Inline::Text(ref text) if text.contains("{{ entity(") => {
                resolved.extend(expand_entity_shortcodes(text, registry));
            }
            other => resolved.push(other),
        }
    }
    *inlines = resolved;
}

pub fn expand_entity_shortcodes(
    text: &str,
    registry: &HashMap<String, petal_tongue_scene::document::EntityRegistryEntry>,
) -> Vec<Inline> {
    let mut result = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("{{ entity(name=\"") {
        if start > 0 {
            result.push(Inline::Text(remaining[..start].to_string()));
        }
        let after_prefix = &remaining[start + 16..]; // skip `{{ entity(name="`
        if let Some(end_quote) = after_prefix.find("\") }}") {
            let key = &after_prefix[..end_quote];
            if let Some(entry) = registry.get(key) {
                let href = entry.page.clone().unwrap_or_default();
                result.push(Inline::Entity(EntityRef {
                    key: key.to_string(),
                    display: entry.display.clone(),
                    emoji: entry.emoji.clone(),
                    href: if href.is_empty() { None } else { Some(href) },
                    description: entry.description.clone(),
                }));
            } else {
                result.push(Inline::Text(format!("{} {key}", "⚠️")));
            }
            remaining = &after_prefix[end_quote + 5..]; // skip `") }}`
        } else {
            result.push(Inline::Text(remaining.to_string()));
            remaining = "";
        }
    }

    if !remaining.is_empty() {
        result.push(Inline::Text(remaining.to_string()));
    }
    result
}
