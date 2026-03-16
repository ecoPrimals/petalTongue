// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]
//! petalTongue CLI - Manage petalTongue instances from the command line
//!
//! Library crate providing CLI instance management. Consumed by the
//! petalTongue `UniBin` via the `status` / `cli` mode.
//!
//! # Commands
//!
//! - `list` - List all running instances
//! - `show <id>` - Show details of an instance
//! - `raise <id>` - Bring an instance window to front
//! - `ping <id>` - Check if instance is responsive
//! - `gc` - Clean up dead instances from registry

mod commands;
mod error;
mod handlers;
mod output;
mod resolve;

// Re-export public API for crate consumers
pub use commands::{Cli, Commands};
pub use error::CliError;
pub use handlers::run;

#[cfg(test)]
pub use commands::parse_args;
#[cfg(test)]
pub use output::{
    format_ping_failure, format_ping_success, format_raise_success, format_show_output,
};
