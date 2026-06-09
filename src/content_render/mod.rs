// SPDX-License-Identifier: AGPL-3.0-or-later
//! Content rendering pipeline — markdown + TOML front matter to DocumentNode.
//!
//! Parses raw markdown with TOML `+++` front matter into a typed
//! `DocumentNode` tree that can be compiled to any output modality
//! (HTML, description, braille, audio).
//!
//! Pipeline: content source → split front matter → compile markdown → resolve shortcodes → modality output.

mod front_matter;
mod markdown;
mod shortcodes;
mod site;

pub use front_matter::{parse_front_matter, split_front_matter};
pub use markdown::compile_markdown;
pub use shortcodes::resolve_shortcodes;
pub use site::{build_nav_tree, load_entity_registry};

use petal_tongue_scene::document::DocumentNode;

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

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_scene::document::Inline;
    use shortcodes::expand_entity_shortcodes;
    use std::collections::HashMap;

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
            "beardog".to_owned(),
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
        assert_eq!(markdown::slugify("Hello World"), "hello-world");
        assert_eq!(
            markdown::slugify("The Five Properties — Adapted"),
            "the-five-properties-adapted"
        );
    }

    #[test]
    fn compile_markdown_table() {
        let md = "| Col A | Col B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |";
        let nodes = compile_markdown(md);
        let table_found = nodes
            .iter()
            .any(|n| matches!(n, DocumentNode::Table { .. }));
        assert!(table_found, "expected a Table node, got {nodes:?}");
        if let DocumentNode::Table { headers, rows } = &nodes[0] {
            assert_eq!(headers.len(), 2);
            assert_eq!(rows.len(), 2);
        }
    }

    #[test]
    fn compile_markdown_ordered_list() {
        let md = "1. First\n2. Second\n3. Third";
        let nodes = compile_markdown(md);
        let list_found = nodes
            .iter()
            .any(|n| matches!(n, DocumentNode::List { ordered: true, .. }));
        assert!(list_found, "expected ordered list, got {nodes:?}");
        if let DocumentNode::List { items, ordered, .. } = &nodes[0] {
            assert!(*ordered);
            assert_eq!(items.len(), 3);
        }
    }

    #[test]
    fn compile_markdown_unordered_list() {
        let md = "- Alpha\n- Beta\n- Gamma";
        let nodes = compile_markdown(md);
        let list_found = nodes
            .iter()
            .any(|n| matches!(n, DocumentNode::List { ordered: false, .. }));
        assert!(list_found, "expected unordered list, got {nodes:?}");
    }

    #[test]
    fn compile_markdown_blockquote() {
        let md = "> This is a quote\n>\n> Second paragraph";
        let nodes = compile_markdown(md);
        let bq_found = nodes
            .iter()
            .any(|n| matches!(n, DocumentNode::BlockQuote { .. }));
        assert!(bq_found, "expected blockquote, got {nodes:?}");
    }

    #[test]
    fn compile_markdown_link() {
        let md = "Visit [example](https://example.com) now.";
        let nodes = compile_markdown(md);
        assert!(!nodes.is_empty());
        if let DocumentNode::Paragraph { inlines } = &nodes[0] {
            let has_link = inlines.iter().any(|i| matches!(i, Inline::Link { .. }));
            assert!(has_link, "expected link inline in {inlines:?}");
        }
    }

    #[test]
    fn compile_markdown_image() {
        let md = "![alt text](image.png \"Title\")";
        let nodes = compile_markdown(md);
        assert!(!nodes.is_empty());
        if let DocumentNode::Paragraph { inlines } = &nodes[0] {
            let has_image = inlines.iter().any(|i| matches!(i, Inline::Image { .. }));
            assert!(has_image, "expected image inline in {inlines:?}");
        }
    }

    #[test]
    fn compile_markdown_emphasis_and_strong() {
        let md = "This has *italic* and **bold** text.";
        let nodes = compile_markdown(md);
        if let DocumentNode::Paragraph { inlines } = &nodes[0] {
            let has_italic = inlines.iter().any(|i| matches!(i, Inline::Italic(_)));
            let has_bold = inlines.iter().any(|i| matches!(i, Inline::Bold(_)));
            assert!(has_italic, "expected italic in {inlines:?}");
            assert!(has_bold, "expected bold in {inlines:?}");
        }
    }

    #[test]
    fn compile_markdown_strikethrough() {
        let md = "This is ~~deleted~~ text.";
        let nodes = compile_markdown(md);
        if let DocumentNode::Paragraph { inlines } = &nodes[0] {
            let has_strike = inlines
                .iter()
                .any(|i| matches!(i, Inline::Strikethrough(_)));
            assert!(has_strike, "expected strikethrough in {inlines:?}");
        }
    }

    #[test]
    fn compile_markdown_horizontal_rule() {
        let md = "Above\n\n---\n\nBelow";
        let nodes = compile_markdown(md);
        let hr_found = nodes
            .iter()
            .any(|n| matches!(n, DocumentNode::ThematicBreak));
        assert!(hr_found, "expected thematic break in {nodes:?}");
    }

    #[test]
    fn compile_markdown_inline_code() {
        let md = "Use `cargo test` to run.";
        let nodes = compile_markdown(md);
        if let DocumentNode::Paragraph { inlines } = &nodes[0] {
            let has_code = inlines.iter().any(|i| matches!(i, Inline::Code(_)));
            assert!(has_code, "expected inline code in {inlines:?}");
        }
    }

    #[test]
    fn parse_front_matter_with_date() {
        let toml = "title = \"Dated\"\ndate = 2026-06-05";
        let meta = parse_front_matter(toml);
        assert_eq!(meta.title, "Dated");
        assert!(meta.date.is_some());
    }

    #[test]
    fn parse_front_matter_with_extra() {
        let toml = "title = \"Extra\"\n\n[extra]\nfoo = \"bar\"\ncount = 42";
        let meta = parse_front_matter(toml);
        assert_eq!(meta.extra.get("foo").and_then(|v| v.as_str()), Some("bar"));
        assert_eq!(
            meta.extra.get("count").and_then(serde_json::Value::as_i64),
            Some(42)
        );
    }

    #[test]
    fn parse_front_matter_invalid_toml() {
        let meta = parse_front_matter("not valid { toml }");
        assert_eq!(meta.title, "");
        assert!(meta.taxonomies.is_empty());
    }

    #[test]
    fn split_front_matter_unclosed() {
        let input = "+++\ntitle = \"Open\"\nNo closing delimiter.";
        let (fm, body) = split_front_matter(input);
        assert!(fm.is_none());
        assert_eq!(body, input);
    }

    #[test]
    fn shortcode_unknown_entity() {
        let registry = HashMap::new();
        let text = "See {{ entity(name=\"unknown\") }} here.";
        let result = expand_entity_shortcodes(text, &registry);
        let has_fallback = result
            .iter()
            .any(|i| matches!(i, Inline::Text(t) if t.contains("unknown")));
        assert!(
            has_fallback,
            "expected unknown entity fallback text in {result:?}"
        );
    }

    #[test]
    fn resolve_shortcodes_in_page_body() {
        let mut registry = HashMap::new();
        registry.insert(
            "test".to_owned(),
            petal_tongue_scene::document::EntityRegistryEntry {
                display: "Test".into(),
                emoji: "🧪".into(),
                kind: "tool".into(),
                description: None,
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

        let doc = parse_document(
            "+++\ntitle = \"SC\"\n+++\n\nSee {{ entity(name=\"test\") }}.",
            "/sc/",
        );
        let mut nodes = vec![doc];
        resolve_shortcodes(&mut nodes, &registry);
        if let DocumentNode::Page { body, .. } = &nodes[0] {
            let has_entity = body.iter().any(|n| {
                if let DocumentNode::Paragraph { inlines } = n {
                    inlines.iter().any(|i| matches!(i, Inline::Entity(_)))
                } else {
                    false
                }
            });
            assert!(has_entity, "expected entity after shortcode resolution");
        } else {
            panic!("expected Page node");
        }
    }
}
