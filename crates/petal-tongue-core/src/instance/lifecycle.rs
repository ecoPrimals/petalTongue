// SPDX-License-Identifier: AGPL-3.0-only
//! Instance lifecycle helpers - process liveness, directory resolution.

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::InstanceError;

/// Get the current Unix timestamp
pub(super) fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| {
            // SAFETY: System clock went backwards (extremely rare).
            tracing::warn!("System clock went backwards during timestamp generation");
            Duration::from_secs(0)
        })
        .as_secs()
}

/// Check if a process exists
pub(super) fn process_exists(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // rustix 0.38+ Signal::from_raw(0) returns None because 0 is not a
        // real signal. Use /proc on Linux for process existence check instead.
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }

    #[cfg(not(unix))]
    {
        std::path::Path::new(&format!("/proc/{}", pid)).exists()
    }
}

/// Get the base directory for petalTongue data
pub(super) fn get_base_dir() -> Result<PathBuf, InstanceError> {
    crate::platform_dirs::data_dir()
        .map(|dir| dir.join(crate::constants::APP_DIR_NAME))
        .map_err(|e| {
            InstanceError::DirectoryError(format!("Could not determine data directory: {e}"))
        })
}

/// Get the socket directory
#[expect(clippy::unnecessary_wraps)]
pub(super) fn get_socket_dir() -> Result<PathBuf, InstanceError> {
    if let Ok(uid) = std::env::var("UID") {
        let run_dir = PathBuf::from(format!(
            "/run/user/{uid}/{}",
            crate::constants::APP_DIR_NAME
        ));
        if run_dir.parent().is_some_and(std::path::Path::exists) {
            return Ok(run_dir);
        }
    }

    Ok(PathBuf::from(crate::constants::LEGACY_TMP_PREFIX).join(crate::constants::APP_DIR_NAME))
}

/// Get the path to the instance registry file
pub(super) fn get_registry_path() -> Result<PathBuf, InstanceError> {
    Ok(get_base_dir()?.join("instances.ron"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_timestamp_nonzero() {
        let ts = current_timestamp();
        assert!(ts > 0);
    }

    #[test]
    fn process_exists_current_process() {
        let pid = std::process::id();
        assert!(process_exists(pid));
    }

    #[test]
    fn process_exists_invalid_pid() {
        assert!(!process_exists(99_999_999));
    }

    #[test]
    fn get_base_dir_returns_path() {
        let dir = get_base_dir().expect("base dir");
        assert!(
            dir.to_string_lossy()
                .contains(crate::constants::APP_DIR_NAME)
        );
    }

    #[test]
    fn get_socket_dir_returns_path() {
        let dir = get_socket_dir().expect("socket dir");
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn get_registry_path_includes_instances_ron() {
        let path = get_registry_path().expect("registry path");
        assert!(path.to_string_lossy().ends_with("instances.ron"));
    }
}
