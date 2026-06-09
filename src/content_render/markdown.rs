// SPDX-License-Identifier: AGPL-3.0-or-later
//! Markdown-to-`DocumentNode` compiler using `pulldown-cmark`.
//!
//! Handles the event-driven parsing of CommonMark + GFM extensions into a
//! typed `DocumentNode` tree.

use petal_tongue_scene::document::{DocumentNode, Inline, ListItem};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

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
            Event::Start(tag) => {
                handle_start(tag, &mut inline_buf, &mut stack);
            }
            Event::End(tag_end) => {
                handle_end(tag_end, &mut nodes, &mut inline_buf, &mut stack);
            }
            Event::Text(text) => {
                handle_text(&text, &mut inline_buf, &mut stack);
            }
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

pub fn inline_to_text(inline: &Inline) -> String {
    match inline {
        Inline::Text(t) => t.clone(),
        Inline::Bold(inlines) | Inline::Italic(inlines) | Inline::Strikethrough(inlines) => {
            inlines.iter().map(inline_to_text).collect()
        }
        Inline::Code(c) => c.clone(),
        Inline::Link { text, .. } => text.iter().map(inline_to_text).collect(),
        Inline::Image { alt, .. } => alt.clone(),
        Inline::Entity(e) => format!("{} {}", e.emoji, e.display),
        Inline::LineBreak => " ".to_owned(),
    }
}

pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// --- Internal types and helpers ---

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

fn handle_start(tag: Tag, inline_buf: &mut Vec<Inline>, stack: &mut Vec<StackFrame>) {
    match tag {
        Tag::Heading { level, .. } => {
            stack.push(StackFrame::Heading(level as u8));
            inline_buf.clear();
        }
        Tag::Paragraph | Tag::TableRow | Tag::TableCell => {
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
                stack,
                inline_buf,
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
    }
}

fn handle_end(
    tag_end: TagEnd,
    nodes: &mut Vec<DocumentNode>,
    inline_buf: &mut Vec<Inline>,
    stack: &mut Vec<StackFrame>,
) {
    match tag_end {
        TagEnd::Heading(_) => {
            if let Some(StackFrame::Heading(level)) = stack.pop() {
                let text: String = inline_buf.iter().map(inline_to_text).collect();
                let id = slugify(&text);
                nodes.push(DocumentNode::Heading {
                    level,
                    inlines: std::mem::take(inline_buf),
                    id,
                });
            }
        }
        TagEnd::Paragraph => {
            let inlines = std::mem::take(inline_buf);
            if !inlines.is_empty() {
                let target = current_block_target(stack, nodes);
                target.push(DocumentNode::Paragraph { inlines });
            }
        }
        TagEnd::CodeBlock => {
            if let Some(StackFrame::CodeBlock(lang, content)) = stack.pop() {
                let target = current_block_target(stack, nodes);
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
                let target = current_block_target(stack, nodes);
                target.push(DocumentNode::List {
                    ordered,
                    start,
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
                let target = current_block_target(stack, nodes);
                target.push(DocumentNode::BlockQuote { children });
            }
        }
        TagEnd::Table => {
            if let Some(StackFrame::Table { headers, rows, .. }) = stack.pop() {
                let target = current_block_target(stack, nodes);
                target.push(DocumentNode::Table { headers, rows });
            }
        }
        TagEnd::TableHead => {
            if let Some(StackFrame::Table { in_head, .. }) = stack.last_mut() {
                *in_head = false;
            }
        }
        TagEnd::TableCell => finish_table_cell(inline_buf, stack),
        TagEnd::Emphasis => finish_emphasis(stack, inline_buf),
        TagEnd::Strong => finish_strong(stack, inline_buf),
        TagEnd::Link => finish_link(stack, inline_buf),
        TagEnd::Strikethrough => finish_strikethrough(stack, inline_buf),
        _ => {}
    }
}

fn finish_table_cell(inline_buf: &mut Vec<Inline>, stack: &mut [StackFrame]) {
    let cell_inlines = std::mem::take(inline_buf);
    if let Some(StackFrame::Table {
        headers,
        rows,
        in_head,
    }) = stack.last_mut()
    {
        if *in_head {
            headers.push(cell_inlines);
        } else {
            if rows.is_empty() || rows.last().is_some_and(|r| r.len() >= headers.len()) {
                rows.push(Vec::new());
            }
            if let Some(row) = rows.last_mut() {
                row.push(cell_inlines);
            }
        }
    }
}

fn finish_emphasis(stack: &mut Vec<StackFrame>, inline_buf: &mut Vec<Inline>) {
    if let Some(StackFrame::Emphasis(inlines)) = stack.pop() {
        push_inline(stack, inline_buf, Inline::Italic(inlines));
    }
}

fn finish_strong(stack: &mut Vec<StackFrame>, inline_buf: &mut Vec<Inline>) {
    if let Some(StackFrame::Strong(inlines)) = stack.pop() {
        push_inline(stack, inline_buf, Inline::Bold(inlines));
    }
}

fn finish_link(stack: &mut Vec<StackFrame>, inline_buf: &mut Vec<Inline>) {
    if let Some(StackFrame::Link {
        href,
        title,
        inlines,
    }) = stack.pop()
    {
        push_inline(
            stack,
            inline_buf,
            Inline::Link {
                text: inlines,
                href,
                title,
            },
        );
    }
}

fn finish_strikethrough(stack: &mut Vec<StackFrame>, inline_buf: &mut Vec<Inline>) {
    if let Some(StackFrame::Strikethrough(inlines)) = stack.pop() {
        push_inline(stack, inline_buf, Inline::Strikethrough(inlines));
    }
}

fn handle_text(text: &str, inline_buf: &mut Vec<Inline>, stack: &mut [StackFrame]) {
    match stack.last_mut() {
        Some(StackFrame::CodeBlock(_, content)) => {
            content.push_str(text);
        }
        Some(
            StackFrame::Emphasis(inlines)
            | StackFrame::Strong(inlines)
            | StackFrame::Strikethrough(inlines)
            | StackFrame::Link { inlines, .. },
        ) => {
            inlines.push(Inline::Text(text.to_string()));
        }
        _ => {
            inline_buf.push(Inline::Text(text.to_string()));
        }
    }
}

fn current_block_target<'a>(
    stack: &'a mut [StackFrame],
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
        Some(
            StackFrame::Emphasis(inlines)
            | StackFrame::Strong(inlines)
            | StackFrame::Link { inlines, .. },
        ) => inlines.push(inline),
        _ => inline_buf.push(inline),
    }
}
