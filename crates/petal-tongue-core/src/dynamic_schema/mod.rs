// SPDX-License-Identifier: AGPL-3.0-or-later
//! Dynamic schema system for live-evolving data structures
//!
//! This module provides schema-agnostic data handling that enables petalTongue
//! to adapt to changing JSON schemas without recompilation.
//!
//! # Philosophy
//!
//! **Code should not know the future.** Data structures evolve over time:
//! - New fields are added
//! - Old fields are deprecated
//! - Types change (string → enum, number → object)
//!
//! Traditional approach (BRITTLE):
//! ```rust,ignore
//! #[derive(Deserialize)]
//! struct Primal {
//!     id: String,
//!     name: String,
//!     // ❌ What if a new field "tier" is added tomorrow?
//!     // ❌ Requires recompilation!
//! }
//! ```
//!
//! Dynamic approach (ADAPTIVE):
//! ```rust,ignore
//! let primal = DynamicData::from_json(json)?;
//! // ✅ Captures ALL fields (known and unknown)
//! // ✅ No recompilation needed
//! // ✅ Can migrate between versions
//! ```

mod migrations;
mod types;

#[cfg(test)]
mod tests;

pub use migrations::{MigrationRegistry, SchemaMigration, SchemaMigrationImpl, V1ToV2Migration};
pub use types::{DynamicData, DynamicValue, SchemaVersion};
