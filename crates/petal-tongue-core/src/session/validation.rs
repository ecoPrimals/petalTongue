// SPDX-License-Identifier: AGPL-3.0-only
//! Session validation - version checks and format validation.

use super::SessionError;
use super::SessionState;

/// Validate that a loaded session has a compatible version
pub(super) fn validate_version(session: &SessionState) -> Result<(), SessionError> {
    if session.version > SessionState::VERSION {
        return Err(SessionError::VersionMismatch {
            found: session.version,
            expected: SessionState::VERSION,
        });
    }
    Ok(())
}
