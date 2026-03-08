// SPDX-License-Identifier: AGPL-3.0-only
//! TUI Widgets
//!
//! Reusable widgets for the TUI.
//! Pure Rust, zero unsafe code.

pub mod footer;
pub mod header;
pub mod status;

pub use footer::Footer;
pub use header::Header;
pub use status::StatusBar;
