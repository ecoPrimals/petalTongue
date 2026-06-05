// SPDX-License-Identifier: AGPL-3.0-or-later
//! Jupyter notebook (`.ipynb`) → HTML renderer.
//!
//! Pure Rust implementation using `serde_json` for nbformat parsing and
//! `pulldown-cmark` for markdown cell rendering.  Supports:
//!
//! - `metadata.title` → `<title>` + `<h1>` page header
//! - `strip_sources` config → hides code input cells (shows outputs only)
//! - Code cells rendered as `<pre><code>` with language annotation
//! - Rich outputs: HTML passthrough, plain text, base64 images

use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;

/// Notebook rendering configuration.
#[derive(Debug, Clone, Default)]
pub struct NotebookRenderConfig {
    /// When `true`, source code cells are hidden (only outputs shown).
    pub strip_sources: bool,
}

/// Render an `.ipynb` notebook to a complete HTML document.
///
/// Returns `None` if the JSON is not valid nbformat (v4).
pub fn render_notebook(ipynb_json: &[u8], config: &NotebookRenderConfig) -> Option<String> {
    let nb: Notebook = serde_json::from_slice(ipynb_json).ok()?;

    if nb.nbformat < 4 {
        return None;
    }

    let title = nb.title().unwrap_or("Notebook");
    let lang = nb.language().unwrap_or("python");

    let mut html_out = String::with_capacity(4096);
    write_document_head(&mut html_out, title);

    for cell in &nb.cells {
        match cell.kind.as_str() {
            "markdown" => {
                render_markdown_cell(&mut html_out, &cell.source);
            }
            "code" => {
                if !config.strip_sources {
                    render_code_source(&mut html_out, &cell.source, lang);
                }
                render_outputs(&mut html_out, &cell.outputs);
            }
            "raw" => {
                html_out.push_str("<div class=\"nb-raw\"><pre>");
                push_escaped(&mut html_out, &join_source(&cell.source));
                html_out.push_str("</pre></div>\n");
            }
            _ => {}
        }
    }

    write_document_tail(&mut html_out);
    Some(html_out)
}

// ---------------------------------------------------------------------------
// nbformat v4 structures (minimal, forward-compatible)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Notebook {
    nbformat: u32,
    #[serde(default)]
    metadata: NotebookMetadata,
    #[serde(default)]
    cells: Vec<Cell>,
}

impl Notebook {
    fn title(&self) -> Option<&str> {
        self.metadata.title.as_deref()
    }

    fn language(&self) -> Option<&str> {
        self.metadata
            .kernelspec
            .as_ref()
            .and_then(|k| k.language.as_deref())
            .or_else(|| {
                self.metadata
                    .language_info
                    .as_ref()
                    .and_then(|l| l.name.as_deref())
            })
    }
}

#[derive(Deserialize, Default)]
struct NotebookMetadata {
    title: Option<String>,
    kernelspec: Option<KernelSpec>,
    language_info: Option<LanguageInfo>,
}

#[derive(Deserialize)]
struct KernelSpec {
    language: Option<String>,
}

#[derive(Deserialize)]
struct LanguageInfo {
    name: Option<String>,
}

#[derive(Deserialize)]
struct Cell {
    #[serde(rename = "cell_type")]
    kind: String,
    #[serde(default)]
    source: Vec<String>,
    #[serde(default)]
    outputs: Vec<Output>,
}

#[derive(Deserialize)]
struct Output {
    #[serde(rename = "output_type")]
    kind: Option<String>,
    #[serde(default)]
    text: Vec<String>,
    #[serde(default)]
    data: Option<OutputData>,
}

#[derive(Deserialize)]
struct OutputData {
    #[serde(rename = "text/html")]
    text_html: Option<Vec<String>>,
    #[serde(rename = "text/plain")]
    text_plain: Option<Vec<String>>,
    #[serde(rename = "image/png")]
    image_png: Option<String>,
    #[serde(rename = "image/jpeg")]
    image_jpeg: Option<String>,
    #[serde(rename = "image/svg+xml")]
    image_svg: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

fn join_source(source: &[String]) -> String {
    source.join("")
}

fn render_markdown_cell(out: &mut String, source: &[String]) {
    let md = join_source(source);
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(&md, opts);
    out.push_str("<div class=\"nb-md\">");
    html::push_html(out, parser);
    out.push_str("</div>\n");
}

fn render_code_source(out: &mut String, source: &[String], lang: &str) {
    let code = join_source(source);
    if code.trim().is_empty() {
        return;
    }
    out.push_str("<div class=\"nb-code\"><pre><code class=\"language-");
    push_escaped(out, lang);
    out.push_str("\">");
    push_escaped(out, &code);
    out.push_str("</code></pre></div>\n");
}

fn render_outputs(out: &mut String, outputs: &[Output]) {
    for output in outputs {
        match (output.kind.as_deref(), output.data.as_ref()) {
            (Some("stream"), _) if !output.text.is_empty() => {
                out.push_str("<div class=\"nb-output nb-stream\"><pre>");
                push_escaped(out, &join_source(&output.text));
                out.push_str("</pre></div>\n");
            }
            (Some("execute_result" | "display_data"), Some(data)) => {
                render_rich_output(out, data);
            }
            (Some("error"), _) => {
                out.push_str("<div class=\"nb-output nb-error\"><pre>");
                push_escaped(out, &join_source(&output.text));
                out.push_str("</pre></div>\n");
            }
            _ => {}
        }
    }
}

fn render_rich_output(out: &mut String, data: &OutputData) {
    if let Some(ref html_lines) = data.text_html {
        out.push_str("<div class=\"nb-output nb-html\">");
        out.push_str(&join_source(html_lines));
        out.push_str("</div>\n");
        return;
    }
    if let Some(ref svg_lines) = data.image_svg {
        out.push_str("<div class=\"nb-output nb-svg\">");
        out.push_str(&join_source(svg_lines));
        out.push_str("</div>\n");
        return;
    }
    if let Some(ref b64) = data.image_png {
        out.push_str("<div class=\"nb-output\"><img src=\"data:image/png;base64,");
        out.push_str(b64.trim());
        out.push_str("\" /></div>\n");
        return;
    }
    if let Some(ref b64) = data.image_jpeg {
        out.push_str("<div class=\"nb-output\"><img src=\"data:image/jpeg;base64,");
        out.push_str(b64.trim());
        out.push_str("\" /></div>\n");
        return;
    }
    if let Some(ref text_lines) = data.text_plain {
        out.push_str("<div class=\"nb-output nb-text\"><pre>");
        push_escaped(out, &join_source(text_lines));
        out.push_str("</pre></div>\n");
    }
}

fn push_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

// ---------------------------------------------------------------------------
// HTML document template
// ---------------------------------------------------------------------------

fn write_document_head(out: &mut String, title: &str) {
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("<meta charset=\"utf-8\" />\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n");
    out.push_str("<meta name=\"generator\" content=\"petalTongue\" />\n");
    out.push_str("<title>");
    push_escaped(out, title);
    out.push_str("</title>\n");
    out.push_str(NOTEBOOK_CSS);
    out.push_str("</head>\n<body>\n<article class=\"notebook\">\n");
    out.push_str("<header><h1>");
    push_escaped(out, title);
    out.push_str("</h1></header>\n");
}

fn write_document_tail(out: &mut String) {
    out.push_str("</article>\n</body>\n</html>\n");
}

const NOTEBOOK_CSS: &str = r"<style>
:root { --bg: #fff; --fg: #1a1a2e; --code-bg: #f5f5f5; --border: #e0e0e0;
        --out-bg: #fafafa; --err-bg: #fff0f0; --accent: #4a6fa5; }
@media (prefers-color-scheme: dark) {
  :root { --bg: #1a1a2e; --fg: #e0e0e0; --code-bg: #252540; --border: #3a3a5e;
          --out-bg: #20203a; --err-bg: #3a2020; --accent: #7ba0d4; }
}
*, *::before, *::after { box-sizing: border-box; }
body { font-family: system-ui, -apple-system, sans-serif; color: var(--fg);
       background: var(--bg); max-width: 52rem; margin: 0 auto; padding: 1rem 2rem;
       line-height: 1.6; }
article.notebook > header { border-bottom: 2px solid var(--accent); margin-bottom: 1.5rem;
       padding-bottom: 0.5rem; }
article.notebook > header h1 { margin: 0 0 0.25rem; font-size: 1.75rem; }
.nb-md { margin: 1rem 0; }
.nb-md h1, .nb-md h2, .nb-md h3 { margin-top: 1.25rem; }
.nb-md img { max-width: 100%; }
.nb-md table { border-collapse: collapse; margin: 0.5rem 0; }
.nb-md th, .nb-md td { border: 1px solid var(--border); padding: 0.35rem 0.75rem; }
.nb-code { background: var(--code-bg); border: 1px solid var(--border);
           border-radius: 4px; margin: 0.75rem 0; overflow-x: auto; }
.nb-code pre { margin: 0; padding: 0.75rem 1rem; }
.nb-code code { font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 0.9rem; }
.nb-output { background: var(--out-bg); border-left: 3px solid var(--accent);
             margin: 0.25rem 0 0.75rem 0; padding: 0.5rem 1rem; overflow-x: auto; }
.nb-output pre { margin: 0; white-space: pre-wrap; font-family: 'JetBrains Mono', monospace;
                 font-size: 0.85rem; }
.nb-error { border-left-color: #c0392b; background: var(--err-bg); }
.nb-raw pre { background: var(--code-bg); padding: 0.75rem 1rem; border-radius: 4px; }
</style>
";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "test code uses unwrap/expect for brevity"
    )]

    use super::*;

    fn minimal_notebook(title: Option<&str>) -> serde_json::Value {
        let mut metadata = serde_json::json!({});
        if let Some(t) = title {
            metadata["title"] = serde_json::json!(t);
        }
        serde_json::json!({
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": metadata,
            "cells": [
                {
                    "cell_type": "markdown",
                    "source": ["# Hello\n", "World"],
                    "metadata": {}
                },
                {
                    "cell_type": "code",
                    "source": ["print('hi')"],
                    "metadata": {},
                    "outputs": [
                        {
                            "output_type": "stream",
                            "name": "stdout",
                            "text": ["hi\n"]
                        }
                    ]
                }
            ]
        })
    }

    #[test]
    fn renders_minimal_notebook() {
        let json = serde_json::to_vec(&minimal_notebook(Some("Test Title"))).unwrap();
        let config = NotebookRenderConfig::default();
        let html = render_notebook(&json, &config).expect("should render");

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Title</title>"));
        assert!(html.contains("<h1>Test Title</h1>"));
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("print(&#39;hi&#39;)") || html.contains("print('hi')"));
        assert!(html.contains("hi\n") || html.contains("hi"));
    }

    #[test]
    fn strip_sources_hides_code() {
        let json = serde_json::to_vec(&minimal_notebook(None)).unwrap();
        let config = NotebookRenderConfig {
            strip_sources: true,
        };
        let html = render_notebook(&json, &config).expect("should render");

        assert!(
            !html.contains("class=\"nb-code\""),
            "code source div should be hidden"
        );
        assert!(html.contains("nb-stream"), "outputs should still appear");
    }

    #[test]
    fn untitled_notebook_uses_fallback() {
        let json = serde_json::to_vec(&minimal_notebook(None)).unwrap();
        let config = NotebookRenderConfig::default();
        let html = render_notebook(&json, &config).expect("should render");

        assert!(html.contains("<title>Notebook</title>"));
    }

    #[test]
    fn rejects_old_nbformat() {
        let nb = serde_json::json!({ "nbformat": 3, "cells": [] });
        let json = serde_json::to_vec(&nb).unwrap();
        assert!(render_notebook(&json, &NotebookRenderConfig::default()).is_none());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(render_notebook(b"not json", &NotebookRenderConfig::default()).is_none());
    }

    #[test]
    fn renders_execute_result_html() {
        let nb = serde_json::json!({
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": {},
            "cells": [{
                "cell_type": "code",
                "source": ["df.head()"],
                "metadata": {},
                "outputs": [{
                    "output_type": "execute_result",
                    "data": {
                        "text/html": ["<table><tr><td>1</td></tr></table>"],
                        "text/plain": ["fallback"]
                    },
                    "metadata": {},
                    "execution_count": 1
                }]
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("<table><tr><td>1</td></tr></table>"));
        assert!(
            !html.contains("fallback"),
            "HTML output takes priority over text/plain"
        );
    }

    #[test]
    fn renders_image_output() {
        let nb = serde_json::json!({
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": {},
            "cells": [{
                "cell_type": "code",
                "source": [],
                "metadata": {},
                "outputs": [{
                    "output_type": "display_data",
                    "data": { "image/png": "iVBORw0KGgo=" },
                    "metadata": {}
                }]
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("data:image/png;base64,iVBORw0KGgo="));
    }

    #[test]
    fn renders_error_output() {
        let nb = serde_json::json!({
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": {},
            "cells": [{
                "cell_type": "code",
                "source": ["1/0"],
                "metadata": {},
                "outputs": [{
                    "output_type": "error",
                    "ename": "ZeroDivisionError",
                    "evalue": "division by zero",
                    "text": ["ZeroDivisionError: division by zero"]
                }]
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("nb-error"));
        assert!(html.contains("ZeroDivisionError"));
    }

    #[test]
    fn renders_raw_cell() {
        let nb = serde_json::json!({
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": {},
            "cells": [{
                "cell_type": "raw",
                "source": ["verbatim <content>"],
                "metadata": {}
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("nb-raw"));
        assert!(html.contains("&lt;content&gt;"));
    }

    #[test]
    fn escapes_title_html() {
        let nb = serde_json::json!({
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": { "title": "<script>alert('xss')</script>" },
            "cells": []
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(!html.contains("<script>"), "title must be escaped");
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn detects_language_from_kernelspec() {
        let nb = serde_json::json!({
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": {
                "kernelspec": { "language": "rust", "display_name": "Rust" }
            },
            "cells": [{
                "cell_type": "code",
                "source": ["fn main() {}"],
                "metadata": {},
                "outputs": []
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("language-rust"));
    }

    #[test]
    fn dark_mode_css_present() {
        let nb = serde_json::json!({
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": {},
            "cells": []
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn empty_code_cell_skipped() {
        let nb = serde_json::json!({
            "nbformat": 4,
            "nbformat_minor": 5,
            "metadata": {},
            "cells": [{
                "cell_type": "code",
                "source": ["  \n  "],
                "metadata": {},
                "outputs": []
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(
            !html.contains("class=\"nb-code\""),
            "empty code cells should not render"
        );
    }

    #[test]
    fn render_jpeg_output() {
        let nb = serde_json::json!({
            "nbformat": 4, "nbformat_minor": 5, "metadata": {},
            "cells": [{
                "cell_type": "code", "source": ["plot()"], "metadata": {},
                "outputs": [{
                    "output_type": "display_data",
                    "data": { "image/jpeg": "AAAA" }
                }]
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("data:image/jpeg;base64,AAAA"));
    }

    #[test]
    fn render_svg_output() {
        let nb = serde_json::json!({
            "nbformat": 4, "nbformat_minor": 5, "metadata": {},
            "cells": [{
                "cell_type": "code", "source": ["svg()"], "metadata": {},
                "outputs": [{
                    "output_type": "display_data",
                    "data": { "image/svg+xml": ["<svg>", "<circle/>", "</svg>"] }
                }]
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("<svg>"));
    }

    #[test]
    fn render_text_plain_only_output() {
        let nb = serde_json::json!({
            "nbformat": 4, "nbformat_minor": 5, "metadata": {},
            "cells": [{
                "cell_type": "code", "source": ["print(42)"], "metadata": {},
                "outputs": [{
                    "output_type": "execute_result",
                    "data": { "text/plain": ["42"] }
                }]
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("42"));
    }

    #[test]
    fn language_from_language_info() {
        let nb = serde_json::json!({
            "nbformat": 4, "nbformat_minor": 5,
            "metadata": { "language_info": { "name": "julia" } },
            "cells": [{
                "cell_type": "code", "source": ["1+1"], "metadata": {},
                "outputs": []
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(
            html.contains("julia"),
            "should annotate with language_info.name"
        );
    }

    #[test]
    fn unknown_cell_type_ignored() {
        let nb = serde_json::json!({
            "nbformat": 4, "nbformat_minor": 5, "metadata": {},
            "cells": [{
                "cell_type": "widget",
                "source": ["ignored"],
                "metadata": {},
                "outputs": []
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(!html.contains("ignored"));
    }

    #[test]
    fn raw_cell_rendered() {
        let nb = serde_json::json!({
            "nbformat": 4, "nbformat_minor": 5, "metadata": {},
            "cells": [{
                "cell_type": "raw",
                "source": ["raw <content>"],
                "metadata": {},
                "outputs": []
            }]
        });
        let json = serde_json::to_vec(&nb).unwrap();
        let html = render_notebook(&json, &NotebookRenderConfig::default()).unwrap();
        assert!(html.contains("nb-raw"));
        assert!(html.contains("raw &lt;content&gt;"));
    }

    #[test]
    fn invalid_json_returns_none() {
        assert!(render_notebook(b"not json", &NotebookRenderConfig::default()).is_none());
    }

    #[test]
    fn nbformat_3_rejected() {
        let nb = serde_json::json!({
            "nbformat": 3, "nbformat_minor": 0, "metadata": {}, "cells": []
        });
        let json = serde_json::to_vec(&nb).unwrap();
        assert!(render_notebook(&json, &NotebookRenderConfig::default()).is_none());
    }
}
