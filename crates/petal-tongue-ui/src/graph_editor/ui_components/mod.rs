// SPDX-License-Identifier: AGPL-3.0-or-later
//! UI Components for Collaborative Intelligence
//!
//! Provides egui widgets for displaying:
//! - Node status and progress
//! - AI reasoning (transparent decision-making)
//! - Conflict resolution (human vs AI modifications)

mod conflict;
mod display;
mod reasoning_display;
mod status_display;

#[cfg(test)]
mod display_tests;
#[cfg(test)]
mod widget_tests;

pub use conflict::{
    Conflict, ConflictResolution, ConflictResolutionChoice, ConflictType,
    conflict_ai_label_color_rgb, conflict_ai_label_text, conflict_header_text,
    conflict_user_label_color_rgb, conflict_user_label_text,
};
pub use display::{
    alternative_display, confidence_color_rgb, confidence_percent_text, error_header_text,
    error_recoverable_color_rgb, error_recoverable_display, format_confidence_display,
    format_data_source_item, format_rationale_item, node_status_display, pattern_display,
    progress_percent_text, resource_usage_display,
};
pub use reasoning_display::ReasoningDisplay;
pub use status_display::StatusDisplay;
