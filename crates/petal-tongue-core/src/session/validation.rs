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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSessionState {
        version: u32,
    }

    impl SessionStateLike for MockSessionState {
        fn version(&self) -> u32 {
            self.version
        }
    }

    #[test]
    fn test_validate_version_ok() {
        let session = MockSessionState { version: 1 };
        assert!(validate_version_trait(&session).is_ok());
    }

    #[test]
    fn test_validate_version_zero_ok() {
        let session = MockSessionState { version: 0 };
        assert!(validate_version_trait(&session).is_ok());
    }

    #[test]
    fn test_validate_version_mismatch() {
        let session = MockSessionState { version: 2 };
        let result = validate_version_trait(&session);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(SessionError::VersionMismatch {
                found: 2,
                expected: 1
            })
        ));
    }
}
