// SPDX-License-Identifier: AGPL-3.0-or-later
//! Universal UI trait definition
//!
//! Defines the platform-agnostic interface that all UI implementations must satisfy.

use bytes::Bytes;

use crate::error::{Result, UiCoreError};
use std::path::Path;

/// Export format for visualizations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// SVG (Scalable Vector Graphics)
    Svg,
    /// PNG (Portable Network Graphics)
    Png,
    /// Plain text
    Text,
    /// JSON (JavaScript Object Notation)
    Json,
    /// DOT (Graphviz)
    Dot,
    /// HTML (SVG wrapped in a standalone HTML document) (PT-04)
    Html,
}

impl ExportFormat {
    /// Get file extension for this format
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Svg => "svg",
            Self::Png => "png",
            Self::Text => "txt",
            Self::Json => "json",
            Self::Dot => "dot",
            Self::Html => "html",
        }
    }

    /// Get MIME type for this format
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Svg => "image/svg+xml",
            Self::Png => "image/png",
            Self::Text => "text/plain",
            Self::Json => "application/json",
            Self::Dot => "text/vnd.graphviz",
            Self::Html => "text/html",
        }
    }
}

/// UI capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UICapability {
    /// Can render to string (SVG, text, etc.)
    RenderToString,
    /// Can render to bytes (PNG, binary, etc.)
    RenderToBytes,
    /// Supports interactive mode
    Interactive,
    /// Can export to files
    Export,
    /// Supports real-time updates
    RealTime,
}

/// Basic sanity check for standalone HTML export (PT-04).
///
/// Ensures non-empty output, a document preamble (`<!DOCTYPE` or `<html`), and a closing
/// `</html>` (ASCII case-insensitive).
#[must_use]
pub fn validate_standalone_html_export(html: &str) -> bool {
    let t = html.trim_start();
    if t.is_empty() {
        return false;
    }
    let preamble_ok = t
        .get(..9)
        .is_some_and(|s| s.eq_ignore_ascii_case("<!doctype"))
        || t.get(..5).is_some_and(|s| s.eq_ignore_ascii_case("<html"));
    if !preamble_ok {
        return false;
    }
    t.as_bytes()
        .windows(7)
        .any(|w| w.eq_ignore_ascii_case(b"</html>"))
}

/// Wrap SVG content in a standalone HTML document (PT-04).
///
/// Produces a minimal, responsive HTML page suitable for browser viewing.
/// Mirrors the IPC `compile_html` modality path so headless CLI achieves parity.
#[must_use]
pub fn wrap_svg_in_html(svg: &str) -> Vec<u8> {
    let html = format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\
         <meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>petalTongue Export</title>\
         <style>body{{margin:0;display:flex;justify-content:center;\
         align-items:center;min-height:100vh;background:#1a1a2e}}\
         svg{{max-width:100%;height:auto}}</style>\
         </head>\n<body>\n{svg}\n</body>\n</html>"
    );
    if !validate_standalone_html_export(&html) {
        tracing::warn!("HTML export validation failed after wrap_svg_in_html");
    }
    html.into_bytes()
}

/// Universal UI interface (platform-agnostic)
///
/// This trait defines the core interface that all UI implementations must satisfy.
/// It enables petalTongue to render visualizations on any platform, in any format,
/// without external dependencies.
///
/// # Philosophy
///
/// External display frameworks (like egui) are enhancements, not dependencies.
/// This trait ensures petalTongue can always render, regardless of platform.
///
/// # Examples
///
/// ```no_run
/// use petal_tongue_ui_core::{UniversalUI, ExportFormat, UICapability};
/// use std::path::Path;
///
/// fn render_topology(ui: &dyn UniversalUI) -> anyhow::Result<()> {
///     if ui.supports(UICapability::Interactive) {
///         // Interactive mode
///         // ui.run_interactive()?;
///     } else {
///         // Export mode
///         ui.export(Path::new("topology.svg"), ExportFormat::Svg)?;
///     }
///     Ok(())
/// }
/// ```
pub trait UniversalUI: Send + Sync {
    /// Get the name of this UI mode
    fn mode_name(&self) -> &str;

    /// Check if this UI supports a capability
    fn supports(&self, capability: UICapability) -> bool;

    /// Render to string (for text-based formats)
    ///
    /// Returns the rendered visualization as a string.
    /// Suitable for SVG, text, JSON, DOT formats.
    fn render_to_string(&self) -> Result<String>;

    /// Render to bytes (for binary formats)
    ///
    /// Returns the rendered visualization as bytes.
    /// Suitable for PNG, binary formats.
    fn render_to_bytes(&self) -> Result<Bytes>;

    /// Export to file
    ///
    /// Exports the visualization to a file in the specified format.
    /// The format is inferred from the export format parameter.
    fn export(&self, path: &Path, format: ExportFormat) -> Result<()> {
        let content = match format {
            ExportFormat::Png => self.render_to_bytes()?,
            ExportFormat::Html => Bytes::from(wrap_svg_in_html(&self.render_to_string()?)),
            _ => Bytes::from(self.render_to_string()?.into_bytes()),
        };

        std::fs::write(path, content.as_ref())?;
        tracing::info!("Exported to {} ({:?})", path.display(), format);
        Ok(())
    }

    /// Run interactive mode (if supported)
    ///
    /// Runs an interactive UI session.
    /// Returns an error if interactive mode is not supported.
    fn run_interactive(&mut self) -> Result<()> {
        Err(UiCoreError::InteractiveNotSupported(
            self.mode_name().to_string(),
        ))
    }

    /// Get recommended export format for this UI
    fn recommended_format(&self) -> ExportFormat {
        ExportFormat::Svg // Default to SVG (universal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SvgUI;
    use petal_tongue_core::GraphEngine;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_run_interactive_not_supported() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut ui = SvgUI::new(graph, 800, 600);
        let result = ui.run_interactive();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Interactive mode not supported")
        );
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Svg.extension(), "svg");
        assert_eq!(ExportFormat::Png.extension(), "png");
        assert_eq!(ExportFormat::Json.extension(), "json");
    }

    #[test]
    fn test_export_format_mime() {
        assert_eq!(ExportFormat::Svg.mime_type(), "image/svg+xml");
        assert_eq!(ExportFormat::Png.mime_type(), "image/png");
        assert_eq!(ExportFormat::Json.mime_type(), "application/json");
    }

    #[test]
    fn test_export_format_all_extensions() {
        assert_eq!(ExportFormat::Svg.extension(), "svg");
        assert_eq!(ExportFormat::Png.extension(), "png");
        assert_eq!(ExportFormat::Text.extension(), "txt");
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Dot.extension(), "dot");
        assert_eq!(ExportFormat::Html.extension(), "html");
    }

    #[test]
    fn test_export_format_all_mime_types() {
        assert_eq!(ExportFormat::Text.mime_type(), "text/plain");
        assert_eq!(ExportFormat::Dot.mime_type(), "text/vnd.graphviz");
        assert_eq!(ExportFormat::Html.mime_type(), "text/html");
    }

    #[test]
    fn test_wrap_svg_in_html() {
        let svg = "<svg><circle r=\"10\"/></svg>";
        let html = String::from_utf8(wrap_svg_in_html(svg)).unwrap();
        assert!(validate_standalone_html_export(&html));
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains(svg));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn test_validate_standalone_html_export() {
        assert!(!validate_standalone_html_export(""));
        assert!(!validate_standalone_html_export("   "));
        assert!(!validate_standalone_html_export("not html"));
        assert!(!validate_standalone_html_export("<!DOCTYPE html><p>x</p>"));
        assert!(validate_standalone_html_export(
            "<!DOCTYPE html><html><body></body></html>"
        ));
        assert!(validate_standalone_html_export(
            "<HTML><head></head><body></BODY></HTML>"
        ));
    }

    #[test]
    fn test_ui_capability_variants() {
        use super::UICapability;
        assert!(matches!(
            UICapability::RenderToString,
            UICapability::RenderToString
        ));
        assert!(matches!(
            UICapability::RenderToBytes,
            UICapability::RenderToBytes
        ));
        assert!(matches!(
            UICapability::Interactive,
            UICapability::Interactive
        ));
        assert!(matches!(UICapability::Export, UICapability::Export));
        assert!(matches!(UICapability::RealTime, UICapability::RealTime));
    }

    #[test]
    fn test_export_format_equality() {
        assert_eq!(ExportFormat::Svg, ExportFormat::Svg);
        assert_ne!(ExportFormat::Svg, ExportFormat::Png);
    }
}
