// SPDX-License-Identifier: AGPL-3.0-or-later
//! CLI mode - System status and information
//!
//! Pure Rust! ✅
//! Fully concurrent, no blocking operations

mod gather;
mod output;
mod types;

#[cfg(test)]
mod tests;

use crate::error::AppError;
use std::sync::Arc;

/// Show system status
///
/// Fully concurrent - gathers system info in parallel
pub async fn status(
    verbose: bool,
    format: &str,
    data_service: Arc<crate::data_service::DataService>,
) -> Result<(), AppError> {
    let status = gather::gather_status(verbose, &data_service).await?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&status)
                .map_err(|e| AppError::Other(format!("Failed to serialize status to JSON: {e}")))?;
            println!("{json}");
        }
        _ => {
            output::print_status_text(&status);
        }
    }

    Ok(())
}
