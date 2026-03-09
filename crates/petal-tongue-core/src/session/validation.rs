// SPDX-License-Identifier: AGPL-3.0-only

use super::SessionError;

pub(super) trait SessionStateLike {
    fn version(&self) -> u32;
}

pub(super) fn validate_version_trait<T: SessionStateLike>(session: &T) -> Result<(), SessionError> {
    const EXPECTED_VERSION: u32 = 1;
    if session.version() > EXPECTED_VERSION {
        return Err(SessionError::VersionMismatch {
            found: session.version(),
            expected: EXPECTED_VERSION,
        });
    }
    Ok(())
}
