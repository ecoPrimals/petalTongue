// SPDX-License-Identifier: AGPL-3.0-only
//! Shared types for 2D graph visualization.

use egui::Pos2;
use petal_tongue_core::PrimalId;

/// Edge being drafted (during drag-to-connect)
#[derive(Debug, Clone)]
pub(crate) struct EdgeDraft {
    /// Source node ID
    pub from: PrimalId,
    /// Current cursor position
    pub current_pos: Pos2,
}
