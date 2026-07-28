// SPDX-License-Identifier: AGPL-3.0-or-later

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::AppError;
use crate::web_mode;

/// Discover composition mount points from environment.
///
/// Discovery: `PETALTONGUE_COMPOSITIONS` env (comma-separated `name=path` pairs).
/// Example: `PETALTONGUE_COMPOSITIONS=footprint=/opt/ecoPrimals/compositions/footprint/dist/client`
///
/// Falls back to scanning `COMPOSITIONS_DIR` (default: adjacent `compositions/` or
/// `/opt/ecoPrimals/compositions/`).
pub fn discover_compositions() -> Vec<web_mode::CompositionMount> {
    if let Ok(val) = std::env::var("PETALTONGUE_COMPOSITIONS") {
        return val
            .split(',')
            .filter_map(|entry| {
                let (name, path) = entry.split_once('=')?;
                Some(web_mode::CompositionMount {
                    name: name.trim().to_owned(),
                    path: std::path::PathBuf::from(path.trim()),
                })
            })
            .collect();
    }

    let compositions_dir = std::env::var("PETALTONGUE_COMPOSITIONS_DIR").map_or_else(
        |_| std::path::PathBuf::from("/opt/ecoPrimals/compositions"),
        std::path::PathBuf::from,
    );

    if !compositions_dir.is_dir() {
        return Vec::new();
    }

    std::fs::read_dir(&compositions_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let name = path.file_name()?.to_string_lossy().into_owned();
            let dist_client = path.join("dist").join("client");
            if dist_client.is_dir() {
                Some(web_mode::CompositionMount {
                    name,
                    path: dist_client,
                })
            } else if path.join("index.html").exists() {
                Some(web_mode::CompositionMount { name, path })
            } else {
                None
            }
        })
        .collect()
}

/// Initialize structured logging with proper filtering
pub fn init_tracing(level: &str, format: &str) -> Result<(), AppError> {
    // Parse log level
    let env_filter = tracing_subscriber::EnvFilter::try_new(level)
        .map_err(|e| AppError::TracingInit(format!("parse log level: {e}")))?;

    let init_result = match format {
        "json" => tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().with_target(true))
            .try_init(),
        "compact" => tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().compact())
            .try_init(),
        _ => tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .try_init(),
    };
    init_result.map_err(|e| AppError::TracingInit(format!("{format} subscriber: {e}")))?;

    Ok(())
}
