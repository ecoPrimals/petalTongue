// SPDX-License-Identifier: AGPL-3.0-or-later
//! Layout algorithm parsing — pure logic for motor commands.
//!
//! Extracted from app/mod.rs for testability. Converts string identifiers
//! (from scenarios, mode presets, IPC) to `LayoutAlgorithm` enum values.

use petal_tongue_core::LayoutAlgorithm;

/// Parse a layout algorithm name from string (e.g. from motor commands).
///
/// Unknown values default to `ForceDirected`.
#[must_use]
pub fn layout_from_str(algorithm: &str) -> LayoutAlgorithm {
    match algorithm {
        "Hierarchical" => LayoutAlgorithm::Hierarchical,
        "Circular" => LayoutAlgorithm::Circular,
        "Random" => LayoutAlgorithm::Random,
        _ => LayoutAlgorithm::ForceDirected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_from_str_hierarchical() {
        assert_eq!(
            layout_from_str("Hierarchical"),
            LayoutAlgorithm::Hierarchical
        );
    }

    #[test]
    fn layout_from_str_circular() {
        assert_eq!(layout_from_str("Circular"), LayoutAlgorithm::Circular);
    }

    #[test]
    fn layout_from_str_random() {
        assert_eq!(layout_from_str("Random"), LayoutAlgorithm::Random);
    }

    #[test]
    fn layout_from_str_unknown_defaults_force_directed() {
        assert_eq!(layout_from_str("unknown"), LayoutAlgorithm::ForceDirected);
        assert_eq!(layout_from_str(""), LayoutAlgorithm::ForceDirected);
    }
}
