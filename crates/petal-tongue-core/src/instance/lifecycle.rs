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
        use nix::sys::signal::kill;
        use nix::unistd::Pid;

        #[expect(clippy::cast_possible_wrap)]
        match kill(Pid::from_raw(pid as i32), None) {
            Ok(()) | Err(nix::errno::Errno::EPERM) => true,
            Err(nix::errno::Errno::ESRCH | _) => false,
        }
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
