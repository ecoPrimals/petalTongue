// SPDX-License-Identifier: AGPL-3.0-or-later
//! Document modality compilers — DocumentNode tree to output formats.
//!
//! These compilers produce multi-modal output from document content,
//! mirroring the existing scene-graph compilers but for rich text content.

use std::fmt::Write;

use bytes::Bytes;

use crate::document::{DocumentNode, EntityRef, Inline, ListItem, NavSection, PageMeta};

use super::ModalityOutput;

/// Compile a DocumentNode tree to HTML for sighted users.
pub fn compile_to_html(doc: &DocumentNode) -> ModalityOutput {
    let mut buf = String::with_capacity(4096);
    render_html_node(doc, &mut buf);
    ModalityOutput::Svg(Bytes::from(buf))
}

/// Compile a DocumentNode tree to accessible text description (screen readers).
pub fn compile_to_description(doc: &DocumentNode) -> ModalityOutput {
    let mut buf = String::with_capacity(2048);
    render_description_node(doc, &mut buf, 0);
    ModalityOutput::Description(Bytes::from(buf))
}

// --- HTML rendering ---

fn render_html_node(node: &DocumentNode, buf: &mut String) {
    match node {
        DocumentNode::Page { meta, body } => {
            render_html_page(meta, body, buf);
        }
        DocumentNode::Heading { level, inlines, id } => {
            let _ = write!(buf, "<h{level} id=\"{id}\">");
            render_html_inlines(inlines, buf);
            let _ = writeln!(buf, "</h{level}>");
        }
        DocumentNode::Paragraph { inlines } => {
            buf.push_str("<p>");
            render_html_inlines(inlines, buf);
            buf.push_str("</p>\n");
        }
        DocumentNode::CodeBlock { language, content } => {
            if let Some(lang) = language {
                let _ = write!(buf, "<pre><code class=\"language-{lang}\">");
            } else {
                buf.push_str("<pre><code>");
            }
            buf.push_str(&html_escape(content));
            buf.push_str("</code></pre>\n");
        }
        DocumentNode::BlockQuote { children } => {
            buf.push_str("<blockquote>\n");
            for child in children {
                render_html_node(child, buf);
            }
            buf.push_str("</blockquote>\n");
        }
        DocumentNode::List {
            ordered,
            start,
            items,
        } => {
            if *ordered {
                if let Some(s) = start {
                    let _ = writeln!(buf, "<ol start=\"{s}\">");
                } else {
                    buf.push_str("<ol>\n");
                }
            } else {
                buf.push_str("<ul>\n");
            }
            for item in items {
                render_html_list_item(item, buf);
            }
            if *ordered {
                buf.push_str("</ol>\n");
            } else {
                buf.push_str("</ul>\n");
            }
        }
        DocumentNode::Table { headers, rows } => render_html_table(headers, rows, buf),
        DocumentNode::ThematicBreak => {
            buf.push_str("<hr />\n");
        }
        DocumentNode::EntityReference(entity) => {
            render_html_entity(entity, buf);
        }
        DocumentNode::EntityMetrics {
            key,
            loc_display,
            tests_display,
            files,
            crates,
        } => {
            render_html_entity_metrics(
                key,
                loc_display,
                tests_display,
                files.as_ref(),
                crates.as_ref(),
                buf,
            );
        }
        DocumentNode::NavTree { sections } => {
            render_html_nav(sections, buf);
        }
        DocumentNode::RawHtml(html) => {
            buf.push_str(html);
        }
    }
}

fn render_html_table(headers: &[Vec<Inline>], rows: &[Vec<Vec<Inline>>], buf: &mut String) {
    buf.push_str("<table>\n<thead><tr>\n");
    for header in headers {
        buf.push_str("<th>");
        render_html_inlines(header, buf);
        buf.push_str("</th>");
    }
    buf.push_str("\n</tr></thead>\n<tbody>\n");
    for row in rows {
        buf.push_str("<tr>");
        for cell in row {
            buf.push_str("<td>");
            render_html_inlines(cell, buf);
            buf.push_str("</td>");
        }
        buf.push_str("</tr>\n");
    }
    buf.push_str("</tbody></table>\n");
}

fn render_html_entity_metrics(
    key: &str,
    loc_display: &str,
    tests_display: &str,
    files: Option<&u64>,
    crates: Option<&u64>,
    buf: &mut String,
) {
    let _ = write!(buf, "<div class=\"entity-metrics\" data-entity=\"{key}\">");
    let _ = write!(buf, "<span class=\"loc\">{loc_display}</span>");
    let _ = write!(buf, " <span class=\"tests\">{tests_display}</span>");
    if let Some(f) = files {
        let _ = write!(buf, " <span class=\"files\">{f} files</span>");
    }
    if let Some(c) = crates {
        let _ = write!(buf, " <span class=\"crates\">{c} crates</span>");
    }
    buf.push_str("</div>\n");
}

fn render_html_page(meta: &PageMeta, body: &[DocumentNode], buf: &mut String) {
    buf.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    let _ = writeln!(buf, "<title>{}</title>", html_escape(&meta.title));
    if let Some(desc) = &meta.description {
        let _ = writeln!(
            buf,
            "<meta name=\"description\" content=\"{}\" />",
            html_escape(desc)
        );
    }
    buf.push_str("</head>\n<body>\n<article>\n");
    let _ = writeln!(buf, "<h1>{}</h1>", html_escape(&meta.title));
    for node in body {
        render_html_node(node, buf);
    }
    buf.push_str("</article>\n</body>\n</html>");
}

fn render_html_list_item(item: &ListItem, buf: &mut String) {
    if let Some(checked) = item.checked {
        let check = if checked { "checked" } else { "" };
        let _ = write!(buf, "<li><input type=\"checkbox\" disabled {check} />");
    } else {
        buf.push_str("<li>");
    }
    for child in &item.content {
        render_html_node(child, buf);
    }
    buf.push_str("</li>\n");
}

fn render_html_inlines(inlines: &[Inline], buf: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Text(t) => buf.push_str(&html_escape(t)),
            Inline::Bold(inner) => {
                buf.push_str("<strong>");
                render_html_inlines(inner, buf);
                buf.push_str("</strong>");
            }
            Inline::Italic(inner) => {
                buf.push_str("<em>");
                render_html_inlines(inner, buf);
                buf.push_str("</em>");
            }
            Inline::Strikethrough(inner) => {
                buf.push_str("<del>");
                render_html_inlines(inner, buf);
                buf.push_str("</del>");
            }
            Inline::Code(c) => {
                let _ = write!(buf, "<code>{}</code>", html_escape(c));
            }
            Inline::Image { alt, src, title } => {
                let _ = write!(buf, "<img src=\"{src}\" alt=\"{}\"", html_escape(alt));
                if let Some(t) = title {
                    let _ = write!(buf, " title=\"{t}\"");
                }
                buf.push_str(" />");
            }
            Inline::Link { text, href, title } => {
                let _ = write!(buf, "<a href=\"{href}\"");
                if let Some(t) = title {
                    let _ = write!(buf, " title=\"{t}\"");
                }
                buf.push('>');
                render_html_inlines(text, buf);
                buf.push_str("</a>");
            }
            Inline::Entity(entity) => {
                render_html_entity(entity, buf);
            }
            Inline::LineBreak => {
                buf.push_str("<br />");
            }
        }
    }
}

fn render_html_entity(entity: &EntityRef, buf: &mut String) {
    if let Some(href) = &entity.href {
        let _ = write!(
            buf,
            "<a class=\"entity-ref\" href=\"{href}\" data-entity=\"{}\">{} {}</a>",
            entity.key, entity.emoji, entity.display
        );
    } else {
        let _ = write!(
            buf,
            "<span class=\"entity-ref\" data-entity=\"{}\">{} {}</span>",
            entity.key, entity.emoji, entity.display
        );
    }
}

fn render_html_nav(sections: &[NavSection], buf: &mut String) {
    buf.push_str("<nav class=\"site-nav\">\n");
    for section in sections {
        let active = if section.active {
            " class=\"active\""
        } else {
            ""
        };
        let _ = writeln!(buf, "<details{active} open>");
        let _ = writeln!(
            buf,
            "<summary><a href=\"{}\">{}</a></summary>",
            section.path,
            html_escape(&section.title)
        );
        buf.push_str("<ul>\n");
        for page in &section.pages {
            let current = if page.current {
                " class=\"current\""
            } else {
                ""
            };
            let _ = writeln!(
                buf,
                "<li{current}><a href=\"{}\">{}</a></li>",
                page.path,
                html_escape(&page.title)
            );
        }
        buf.push_str("</ul>\n</details>\n");
    }
    buf.push_str("</nav>\n");
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// --- Description rendering (screen reader / accessibility) ---

fn render_description_node(node: &DocumentNode, buf: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);
    match node {
        DocumentNode::Page { meta, body } => {
            let _ = writeln!(buf, "Document: {}", meta.title);
            if let Some(desc) = &meta.description {
                let _ = writeln!(buf, "Description: {desc}");
            }
            let _ = writeln!(buf);
            for child in body {
                render_description_node(child, buf, depth);
            }
        }
        DocumentNode::Heading { level, inlines, .. } => {
            let marker = "#".repeat(*level as usize);
            let _ = write!(buf, "{indent}{marker} ");
            render_description_inlines(inlines, buf);
            let _ = writeln!(buf);
        }
        DocumentNode::Paragraph { inlines } => {
            let _ = write!(buf, "{indent}");
            render_description_inlines(inlines, buf);
            let _ = writeln!(buf);
        }
        DocumentNode::CodeBlock { language, content } => {
            let lang_label = language.as_deref().unwrap_or("code");
            let _ = writeln!(buf, "{indent}[Code block: {lang_label}]");
            for line in content.lines() {
                let _ = writeln!(buf, "{indent}  {line}");
            }
        }
        DocumentNode::BlockQuote { children } => {
            let _ = writeln!(buf, "{indent}[Quote:]");
            for child in children {
                render_description_node(child, buf, depth + 1);
            }
        }
        DocumentNode::List { ordered, items, .. } => {
            render_description_list(&indent, *ordered, items, buf, depth);
        }
        DocumentNode::Table { headers, rows } => {
            let _ = writeln!(
                buf,
                "{indent}[Table: {} columns, {} rows]",
                headers.len(),
                rows.len()
            );
            let _ = write!(buf, "{indent}  Headers: ");
            for (i, h) in headers.iter().enumerate() {
                if i > 0 {
                    buf.push_str(" | ");
                }
                render_description_inlines(h, buf);
            }
            let _ = writeln!(buf);
        }
        DocumentNode::ThematicBreak => {
            let _ = writeln!(buf, "{indent}---");
        }
        DocumentNode::EntityReference(entity) => {
            let _ = writeln!(
                buf,
                "{indent}[Entity: {} {} - {}]",
                entity.emoji,
                entity.display,
                entity.description.as_deref().unwrap_or("no description")
            );
        }
        DocumentNode::EntityMetrics {
            key,
            loc_display,
            tests_display,
            ..
        } => {
            let _ = writeln!(
                buf,
                "{indent}[Metrics for {key}: {loc_display}, {tests_display}]"
            );
        }
        DocumentNode::NavTree { sections } => {
            let _ = writeln!(buf, "{indent}[Navigation: {} sections]", sections.len());
            for section in sections {
                let _ = writeln!(
                    buf,
                    "{indent}  Section: {} ({} pages)",
                    section.title,
                    section.pages.len()
                );
            }
        }
        DocumentNode::RawHtml(_) => {
            let _ = writeln!(buf, "{indent}[Embedded content]");
        }
    }
}

fn render_description_list(
    indent: &str,
    ordered: bool,
    items: &[ListItem],
    buf: &mut String,
    depth: usize,
) {
    let kind = if ordered { "Ordered list" } else { "List" };
    let _ = writeln!(buf, "{indent}[{kind}, {count} items]", count = items.len());
    for (i, item) in items.iter().enumerate() {
        if ordered {
            let _ = write!(buf, "{indent}  {}. ", i + 1);
        } else {
            let _ = write!(buf, "{indent}  - ");
        }
        for child in &item.content {
            render_description_node(child, buf, depth + 2);
        }
    }
}

fn render_description_inlines(inlines: &[Inline], buf: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Text(t) => buf.push_str(t),
            Inline::Bold(inner) | Inline::Italic(inner) | Inline::Strikethrough(inner) => {
                render_description_inlines(inner, buf);
            }
            Inline::Code(c) => {
                let _ = write!(buf, "`{c}`");
            }
            Inline::Link { text, href, .. } => {
                render_description_inlines(text, buf);
                let _ = write!(buf, " (link: {href})");
            }
            Inline::Image { alt, src, .. } => {
                let _ = write!(buf, "[Image: {alt} ({src})]");
            }
            Inline::Entity(entity) => {
                let _ = write!(buf, "{} {}", entity.emoji, entity.display);
            }
            Inline::LineBreak => buf.push(' '),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentNode, Inline, PageMeta};

    #[test]
    fn html_heading_renders() {
        let node = DocumentNode::Heading {
            level: 2,
            inlines: vec![Inline::Text("Test Heading".into())],
            id: "test-heading".into(),
        };
        let output = compile_to_html(&node);
        match output {
            ModalityOutput::Svg(bytes) => {
                let html = String::from_utf8(bytes.to_vec()).unwrap();
                assert!(html.contains("<h2 id=\"test-heading\">Test Heading</h2>"));
            }
            _ => panic!("expected Svg (HTML) output"),
        }
    }

    #[test]
    fn html_page_renders() {
        let doc = DocumentNode::Page {
            meta: PageMeta {
                title: "Hello World".into(),
                description: Some("A test page".into()),
                ..PageMeta::default()
            },
            body: vec![DocumentNode::Paragraph {
                inlines: vec![Inline::Text("Body text.".into())],
            }],
        };
        let output = compile_to_html(&doc);
        match output {
            ModalityOutput::Svg(bytes) => {
                let html = String::from_utf8(bytes.to_vec()).unwrap();
                assert!(html.contains("<title>Hello World</title>"));
                assert!(html.contains("<p>Body text.</p>"));
            }
            _ => panic!("expected Svg (HTML) output"),
        }
    }

    #[test]
    fn description_renders_heading() {
        let node = DocumentNode::Heading {
            level: 1,
            inlines: vec![Inline::Text("Main Title".into())],
            id: "main-title".into(),
        };
        let output = compile_to_description(&node);
        match output {
            ModalityOutput::Description(bytes) => {
                let text = String::from_utf8(bytes.to_vec()).unwrap();
                assert!(text.contains("# Main Title"));
            }
            _ => panic!("expected Description output"),
        }
    }

    #[test]
    fn description_renders_entity() {
        let node = DocumentNode::EntityReference(EntityRef {
            key: "beardog".into(),
            display: "BearDog".into(),
            emoji: "🐻🐕".into(),
            href: Some("/primals/beardog/".into()),
            description: Some("Crypto identity primal".into()),
        });
        let output = compile_to_description(&node);
        match output {
            ModalityOutput::Description(bytes) => {
                let text = String::from_utf8(bytes.to_vec()).unwrap();
                assert!(text.contains("BearDog"));
                assert!(text.contains("Crypto identity primal"));
            }
            _ => panic!("expected Description output"),
        }
    }

    #[test]
    fn html_entity_link_renders() {
        let node = DocumentNode::Paragraph {
            inlines: vec![Inline::Entity(EntityRef {
                key: "songbird".into(),
                display: "Songbird".into(),
                emoji: "🐦".into(),
                href: Some("/primals/songbird/".into()),
                description: None,
            })],
        };
        let output = compile_to_html(&node);
        match output {
            ModalityOutput::Svg(bytes) => {
                let html = String::from_utf8(bytes.to_vec()).unwrap();
                assert!(html.contains("class=\"entity-ref\""));
                assert!(html.contains("href=\"/primals/songbird/\""));
                assert!(html.contains("🐦 Songbird"));
            }
            _ => panic!("expected Svg (HTML) output"),
        }
    }
}
