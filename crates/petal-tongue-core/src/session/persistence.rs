// SPDX-License-Identifier: AGPL-3.0-only
//! Session persistence - save/load to disk with atomic writes.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::instance::InstanceId;

use super::validation;
use super::SessionError;
use super::SessionState;

/// Get current Unix timestamp
pub(super) fn current_timestamp() -> u64 {
    if let Ok(d) = SystemTime::now().duration_since(UNIX_EPOCH) {
        d.as_secs()
    } else {
        tracing::warn!("System time is before Unix epoch, using 0 as fallback");
        0
    }
}

/// Get the session file path for an instance
pub(super) fn get_session_path(instance_id: &InstanceId) -> Result<PathBuf, SessionError> {
    let base_dir = get_base_dir()?;
    let sessions_dir = base_dir.join("sessions");

    fs::create_dir_all(&sessions_dir)
        .map_err(|e| SessionError::IoError(format!("Failed to create sessions directory: {e}")))?;

    Ok(sessions_dir.join(format!("{}.ron", instance_id.as_str())))
}

/// Get the base directory for petalTongue data
fn get_base_dir() -> Result<PathBuf, SessionError> {
    crate::platform_dirs::data_dir()
        .map(|dir| dir.join("petaltongue"))
        .map_err(|e| {
            SessionError::DirectoryError(format!("Could not determine data directory: {e}"))
        })
}

/// Save session to disk (atomic write)
pub(super) fn save_session(session: &SessionState, path: &Path) -> Result<(), SessionError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| SessionError::IoError(format!("Failed to create directory: {e}")))?;
    }

    let contents = ron::ser::to_string_pretty(session, ron::ser::PrettyConfig::default())
        .map_err(|e| SessionError::SerializeError(format!("Failed to serialize: {e}")))?;

    let temp_path = path.with_extension("ron.tmp");
    fs::write(&temp_path, contents)
        .map_err(|e| SessionError::IoError(format!("Failed to write temp file: {e}")))?;

    fs::rename(&temp_path, path)
        .map_err(|e| SessionError::IoError(format!("Failed to rename file: {e}")))?;

    tracing::debug!("Session saved to: {}", path.display());
    Ok(())
}

/// Load session from disk
pub(super) fn load_session(path: &Path) -> Result<SessionState, SessionError> {
    if !path.exists() {
        return Err(SessionError::NotFound(path.display().to_string()));
    }

    let contents = fs::read_to_string(path)
        .map_err(|e| SessionError::IoError(format!("Failed to read file: {e}")))?;

    let session: SessionState = ron::from_str(&contents)
        .map_err(|e| SessionError::ParseError(format!("Failed to parse: {e}")))?;

    validation::validate_version(&session)?;

    tracing::debug!("Session loaded from: {}", path.display());
    Ok(session)
}
