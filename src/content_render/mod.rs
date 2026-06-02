// SPDX-License-Identifier: AGPL-3.0-or-later
//! Content rendering pipeline — markdown + TOML front matter to DocumentNode.
//!
//! Parses raw markdown with TOML `+++` front matter into a typed
//! `DocumentNode` tree that can be compiled to any output modality
//! (HTML, description, braille, audio).
//!
//! Pipeline: content source → split front matter → compile markdown → resolve shortcodes → modality output.

mod site;

pub use site::{build_nav_tree, load_entity_registry};

use petal_tongue_scene::document::{DocumentNode, EntityRef, Inline, ListItem, PageMeta};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
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

    if let Some(end_pos) = after_first.find("\n+++") {
        let toml_content = &after_first[..end_pos];
        let body_start = end_pos + 4; // skip "\n+++"
        let body = after_first[body_start..]
            .strip_prefix('\n')
            .unwrap_or(&after_first[body_start..]);
        (Some(toml_content), body)
    } else {
        (None, input)
    }
}

/// Parse TOML front matter into a `PageMeta`.
pub fn parse_front_matter(toml_str: &str) -> PageMeta {
    let table: toml::Table = toml::from_str(toml_str).unwrap_or_default();

    let title = table
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
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
        .and_then(|v| v.as_integer())
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
            extra.insert(key.clone(), val.clone());
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

/// Compile markdown body text into a `Vec<DocumentNode>`.
pub fn compile_markdown(markdown: &str) -> Vec<DocumentNode> {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(markdown, opts);
    let mut nodes: Vec<DocumentNode> = Vec::new();
    let mut inline_buf: Vec<Inline> = Vec::new();
    let mut stack: Vec<StackFrame> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    stack.push(StackFrame::Heading(level as u8));
                    inline_buf.clear();
                }
                Tag::Paragraph => {
                    inline_buf.clear();
                }
                Tag::CodeBlock(kind) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(l) => {
                            let s = l.to_string();
                            if s.is_empty() { None } else { Some(s) }
                        }
                        pulldown_cmark::CodeBlockKind::Indented => None,
                    };
                    stack.push(StackFrame::CodeBlock(lang, String::new()));
                }
                Tag::List(start) => {
                    stack.push(StackFrame::List {
                        ordered: start.is_some(),
                        start,
                        items: Vec::new(),
                    });
                }
                Tag::Item => {
                    stack.push(StackFrame::ListItem(Vec::new()));
                }
                Tag::BlockQuote(_) => {
                    stack.push(StackFrame::BlockQuote(Vec::new()));
                }
                Tag::Table(_) => {
                    stack.push(StackFrame::Table {
                        headers: Vec::new(),
                        rows: Vec::new(),
                        in_head: false,
                    });
                }
                Tag::TableHead => {
                    if let Some(StackFrame::Table { in_head, .. }) = stack.last_mut() {
                        *in_head = true;
                    }
                }
                Tag::TableRow => {
                    inline_buf.clear();
                }
                Tag::TableCell => {
                    inline_buf.clear();
                }
                Tag::Emphasis => {
                    stack.push(StackFrame::Emphasis(Vec::new()));
                }
                Tag::Strong => {
                    stack.push(StackFrame::Strong(Vec::new()));
                }
                Tag::Strikethrough => {
                    stack.push(StackFrame::Strikethrough(Vec::new()));
                }
                Tag::Link {
                    dest_url, title, ..
                } => {
                    stack.push(StackFrame::Link {
                        href: dest_url.to_string(),
                        title: if title.is_empty() {
                            None
                        } else {
                            Some(title.to_string())
                        },
                        inlines: Vec::new(),
                    });
                }
                Tag::Image {
                    dest_url, title, ..
                } => {
                    push_inline(
                        &mut stack,
                        &mut inline_buf,
                        Inline::Image {
                            alt: String::new(),
                            src: dest_url.to_string(),
                            title: if title.is_empty() {
                                None
                            } else {
                                Some(title.to_string())
                            },
                        },
                    );
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    if let Some(StackFrame::Heading(level)) = stack.pop() {
                        let text: String = inline_buf.iter().map(inline_to_text).collect();
                        let id = slugify(&text);
                        nodes.push(DocumentNode::Heading {
                            level,
                            inlines: std::mem::take(&mut inline_buf),
                            id,
                        });
                    }
                }
                TagEnd::Paragraph => {
                    let inlines = std::mem::take(&mut inline_buf);
                    if !inlines.is_empty() {
                        let target = current_block_target(&mut stack, &mut nodes);
                        target.push(DocumentNode::Paragraph { inlines });
                    }
                }
                TagEnd::CodeBlock => {
                    if let Some(StackFrame::CodeBlock(lang, content)) = stack.pop() {
                        let target = current_block_target(&mut stack, &mut nodes);
                        target.push(DocumentNode::CodeBlock {
                            language: lang,
                            content,
                        });
                    }
                }
                TagEnd::List(_) => {
                    if let Some(StackFrame::List {
                        ordered,
                        start,
                        items,
                    }) = stack.pop()
                    {
                        let target = current_block_target(&mut stack, &mut nodes);
                        target.push(DocumentNode::List {
                            ordered,
                            start: start.map(|s| s as u64),
                            items,
                        });
                    }
                }
                TagEnd::Item => {
                    if let Some(StackFrame::ListItem(content)) = stack.pop() {
                        if let Some(StackFrame::List { items, .. }) = stack.last_mut() {
                            items.push(ListItem {
                                checked: None,
                                content,
                            });
                        }
                    }
                }
                TagEnd::BlockQuote(_) => {
                    if let Some(StackFrame::BlockQuote(children)) = stack.pop() {
                        let target = current_block_target(&mut stack, &mut nodes);
                        target.push(DocumentNode::BlockQuote { children });
                    }
                }
                TagEnd::Table => {
                    if let Some(StackFrame::Table { headers, rows, .. }) = stack.pop() {
                        let target = current_block_target(&mut stack, &mut nodes);
                        target.push(DocumentNode::Table { headers, rows });
                    }
                }
                TagEnd::TableHead => {
                    if let Some(StackFrame::Table { in_head, .. }) = stack.last_mut() {
                        *in_head = false;
                    }
                }
                TagEnd::TableRow => {}
                TagEnd::TableCell => {
                    let cell_inlines = std::mem::take(&mut inline_buf);
                    if let Some(StackFrame::Table {
                        headers,
                        rows,
                        in_head,
                    }) = stack.last_mut()
                    {
                        if *in_head {
                            headers.push(cell_inlines);
                        } else {
                            if rows.is_empty()
                                || rows.last().map_or(false, |r| r.len() >= headers.len())
                            {
                                rows.push(Vec::new());
                            }
                            if let Some(row) = rows.last_mut() {
                                row.push(cell_inlines);
                            }
                        }
                    }
                }
                TagEnd::Emphasis => {
                    if let Some(StackFrame::Emphasis(inlines)) = stack.pop() {
                        push_inline(&mut stack, &mut inline_buf, Inline::Italic(inlines));
                    }
                }
                TagEnd::Strong => {
                    if let Some(StackFrame::Strong(inlines)) = stack.pop() {
                        push_inline(&mut stack, &mut inline_buf, Inline::Bold(inlines));
                    }
                }
                TagEnd::Link => {
                    if let Some(StackFrame::Link {
                        href,
                        title,
                        inlines,
                    }) = stack.pop()
                    {
                        push_inline(
                            &mut stack,
                            &mut inline_buf,
                            Inline::Link {
                                text: inlines,
                                href,
                                title,
                            },
                        );
                    }
                }
                TagEnd::Strikethrough => {
                    if let Some(StackFrame::Strikethrough(inlines)) = stack.pop() {
                        push_inline(&mut stack, &mut inline_buf, Inline::Strikethrough(inlines));
                    }
                }
                TagEnd::Image => {}
                _ => {}
            },
            Event::Text(text) => match stack.last_mut() {
                Some(StackFrame::CodeBlock(_, content)) => {
                    content.push_str(&text);
                }
                Some(StackFrame::Emphasis(inlines)) => {
                    inlines.push(Inline::Text(text.to_string()));
                }
                Some(StackFrame::Strong(inlines)) => {
                    inlines.push(Inline::Text(text.to_string()));
                }
                Some(StackFrame::Strikethrough(inlines)) => {
                    inlines.push(Inline::Text(text.to_string()));
                }
                Some(StackFrame::Link { inlines, .. }) => {
                    inlines.push(Inline::Text(text.to_string()));
                }
                _ => {
                    inline_buf.push(Inline::Text(text.to_string()));
                }
            },
            Event::Code(code) => {
                push_inline(&mut stack, &mut inline_buf, Inline::Code(code.to_string()));
            }
            Event::SoftBreak | Event::HardBreak => {
                push_inline(&mut stack, &mut inline_buf, Inline::LineBreak);
            }
            Event::Rule => {
                let target = current_block_target(&mut stack, &mut nodes);
                target.push(DocumentNode::ThematicBreak);
            }
            Event::Html(html) => {
                let target = current_block_target(&mut stack, &mut nodes);
                target.push(DocumentNode::RawHtml(html.to_string()));
            }
            _ => {}
        }
    }

    nodes
}

/// Parse a full markdown document (front matter + body) into a `DocumentNode::Page`.
pub fn parse_document(input: &str, path: &str) -> DocumentNode {
    let (front_matter, body) = split_front_matter(input);
    let mut meta = front_matter.map(parse_front_matter).unwrap_or_default();
    meta.path = path.to_string();

    let body_nodes = compile_markdown(body);

    DocumentNode::Page {
        meta,
        body: body_nodes,
    }
}

/// Resolve entity shortcodes (`{{ entity(name="...") }}`) in inline text.
pub fn resolve_shortcodes(
    nodes: &mut Vec<DocumentNode>,
    registry: &HashMap<String, petal_tongue_scene::document::EntityRegistryEntry>,
) {
    for node in nodes.iter_mut() {
        match node {
            DocumentNode::Page { body, .. } => resolve_shortcodes(body, registry),
            DocumentNode::Paragraph { inlines } => resolve_inlines(inlines, registry),
            DocumentNode::Heading { inlines, .. } => resolve_inlines(inlines, registry),
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

fn expand_entity_shortcodes(
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

// --- Internal helpers ---

#[derive(Debug)]
enum StackFrame {
    Heading(u8),
    CodeBlock(Option<String>, String),
    List {
        ordered: bool,
        start: Option<u64>,
        items: Vec<ListItem>,
    },
    ListItem(Vec<DocumentNode>),
    BlockQuote(Vec<DocumentNode>),
    Table {
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
        in_head: bool,
    },
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Link {
        href: String,
        title: Option<String>,
        inlines: Vec<Inline>,
    },
}

fn current_block_target<'a>(
    stack: &'a mut Vec<StackFrame>,
    root: &'a mut Vec<DocumentNode>,
) -> &'a mut Vec<DocumentNode> {
    for frame in stack.iter_mut().rev() {
        match frame {
            StackFrame::ListItem(content) => return content,
            StackFrame::BlockQuote(children) => return children,
            _ => {}
        }
    }
    root
}

fn push_inline(stack: &mut [StackFrame], inline_buf: &mut Vec<Inline>, inline: Inline) {
    match stack.last_mut() {
        Some(StackFrame::Emphasis(inlines)) => inlines.push(inline),
        Some(StackFrame::Strong(inlines)) => inlines.push(inline),
        Some(StackFrame::Link { inlines, .. }) => inlines.push(inline),
        _ => inline_buf.push(inline),
    }
}

fn inline_to_text(inline: &Inline) -> String {
    match inline {
        Inline::Text(t) => t.clone(),
        Inline::Bold(inlines) | Inline::Italic(inlines) | Inline::Strikethrough(inlines) => {
            inlines.iter().map(inline_to_text).collect()
        }
        Inline::Code(c) => c.clone(),
        Inline::Link { text, .. } => text.iter().map(inline_to_text).collect(),
        Inline::Image { alt, .. } => alt.clone(),
        Inline::Entity(e) => format!("{} {}", e.emoji, e.display),
        Inline::LineBreak => " ".to_string(),
    }
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_front_matter_basic() {
        let input = "+++\ntitle = \"Hello\"\n+++\n\n# Body";
        let (fm, body) = split_front_matter(input);
        assert_eq!(fm.unwrap(), "title = \"Hello\"");
        assert!(body.starts_with("# Body") || body.starts_with('\n'));
    }

    #[test]
    fn split_front_matter_none() {
        let input = "# Just markdown\n\nNo front matter.";
        let (fm, body) = split_front_matter(input);
        assert!(fm.is_none());
        assert_eq!(body, input);
    }

    #[test]
    fn parse_front_matter_basic() {
        let toml = "title = \"Test Page\"\ndescription = \"A test\"\nweight = 5\n\n[taxonomies]\nprimals = [\"beardog\", \"songbird\"]";
        let meta = parse_front_matter(toml);
        assert_eq!(meta.title, "Test Page");
        assert_eq!(meta.description.as_deref(), Some("A test"));
        assert_eq!(meta.weight, Some(5));
        assert_eq!(
            meta.taxonomies.get("primals").unwrap(),
            &["beardog", "songbird"]
        );
    }

    #[test]
    fn compile_markdown_headings() {
        let md = "# Hello World\n\n## Second\n\nParagraph text.";
        let nodes = compile_markdown(md);
        assert!(nodes.len() >= 3);
        match &nodes[0] {
            DocumentNode::Heading { level, id, .. } => {
                assert_eq!(*level, 1);
                assert_eq!(id, "hello-world");
            }
            _ => panic!("expected heading"),
        }
    }

    #[test]
    fn compile_markdown_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let nodes = compile_markdown(md);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            DocumentNode::CodeBlock { language, content } => {
                assert_eq!(language.as_deref(), Some("rust"));
                assert!(content.contains("fn main()"));
            }
            _ => panic!("expected code block"),
        }
    }

    #[test]
    fn parse_document_full() {
        let input = "+++\ntitle = \"My Page\"\n+++\n\n# Hello\n\nWorld.";
        let doc = parse_document(input, "/test/page/");
        match doc {
            DocumentNode::Page { meta, body } => {
                assert_eq!(meta.title, "My Page");
                assert_eq!(meta.path, "/test/page/");
                assert!(!body.is_empty());
            }
            _ => panic!("expected page"),
        }
    }

    #[test]
    fn shortcode_expansion() {
        let mut registry = HashMap::new();
        registry.insert(
            "beardog".to_string(),
            petal_tongue_scene::document::EntityRegistryEntry {
                display: "BearDog".into(),
                emoji: "🐻🐕".into(),
                kind: "primal".into(),
                description: Some("Crypto identity".into()),
                page: None,
                repo: None,
                domain: None,
                loc: None,
                loc_display: None,
                tests: None,
                tests_display: None,
                files: None,
                crates: None,
            },
        );

        let text = "See {{ entity(name=\"beardog\") }} for details.";
        let result = expand_entity_shortcodes(text, &registry);
        assert_eq!(result.len(), 3); // "See " + Entity + " for details."
        match &result[1] {
            Inline::Entity(e) => {
                assert_eq!(e.key, "beardog");
                assert_eq!(e.display, "BearDog");
                assert_eq!(e.href, None);
            }
            _ => panic!("expected entity ref"),
        }
    }

    #[test]
    fn slugify_works() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(
            slugify("The Five Properties — Adapted"),
            "the-five-properties-adapted"
        );
    }
}
