// SPDX-License-Identifier: AGPL-3.0-or-later
//! Niche Designer - Visual Niche Editor & Deployment UI
//!
//! Provides a visual interface for designing, validating, and deploying niches.
//! Supports templates, drag-and-drop primal assignment, requirement validation,
//! and AI suggestions.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ NicheDesigner                                               │
//! │  ├─ Template Selector (dropdown)                            │
//! │  ├─ Canvas (visual niche representation)                    │
//! │  │   ├─ Primal Slots (required/optional)                    │
//! │  │   └─ Drop zones for primals                              │
//! │  ├─ Validation Panel                                        │
//! │  │   ├─ Requirements check                                  │
//! │  │   ├─ Resource estimation                                 │
//! │  │   └─ Warnings/Errors                                     │
//! │  └─ Deploy Button                                           │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod rendering;
mod state;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use state::NicheDesigner;
pub use types::ValidationResult;
