// SPDX-License-Identifier: AGPL-3.0-only
//! Universal UI trait definition
//!
//! Defines the platform-agnostic interface that all UI implementations must satisfy.

use anyhow::Result;
use bytes::Bytes;
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

/// Universal UI interface (platform-agnostic)
///
/// This trait defines the core interface that all UI implementations must satisfy.
/// It enables petalTongue to render visualizations on any platform, in any format,
/// without external dependencies.
///
/// # Philosophy
///
/// External GUI frameworks (like egui) are enhancements, not dependencies.
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
        anyhow::bail!("Interactive mode not supported for {}", self.mode_name())
    }

    /// Get recommended export format for this UI
    fn recommended_format(&self) -> ExportFormat {
        ExportFormat::Svg // Default to SVG (universal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_export_format_all_mime_types() {
        assert_eq!(ExportFormat::Text.mime_type(), "text/plain");
        assert_eq!(ExportFormat::Dot.mime_type(), "text/vnd.graphviz");
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
