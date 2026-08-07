// SPDX-License-Identifier: AGPL-3.0-or-later
//! System Information Utilities
//!
//! Safe wrappers around system calls for discovering runtime information.
//!
//! # Philosophy
//!
//! - **Encapsulate unsafe**: Wrap FFI in safe, well-documented APIs
//! - **No assumptions**: Discover at runtime, don't hardcode
//! - **Cross-platform**: Abstract platform differences
//! - **Pure Rust**: Use /proc parsing on Linux (no libc)

use std::path::PathBuf;

/// Get the current user's UID (User ID)
///
/// Delegates to [`crate::platform_substrate::current_uid`] — the canonical G68 abstraction.
///
/// # Platform Support
///
/// - **Linux/macOS**: ✅ Supported (kernel UID)
/// - **Windows**: Returns 0 (Windows uses SIDs, not UIDs)
#[must_use]
pub fn get_current_uid() -> u32 {
    crate::platform_substrate::current_uid()
}

/// Get the current effective user ID (EUID)
///
/// Delegates to [`crate::platform_substrate::effective_uid`] — the canonical G68 abstraction.
///
/// # Platform Support
///
/// - **Linux/macOS**: ✅ Supported
/// - **Windows**: Returns 0
#[must_use]
pub fn get_current_euid() -> u32 {
    crate::platform_substrate::effective_uid()
}

/// System information (hostname, OS, etc.) from /proc on Linux.
///
/// Pure Rust, no libc. On Linux reads from /proc. On other platforms
/// returns sensible defaults.
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// Hostname
    pub hostname: String,
    /// OS identifier (e.g. "Linux")
    pub os: String,
    /// Kernel version string if available
    pub kernel_version: Option<String>,
}

impl SystemInfo {
    /// Discover system info. On Linux uses /proc; elsewhere uses env/fallbacks.
    #[must_use]
    pub fn discover() -> Self {
        #[cfg(target_os = "linux")]
        {
            let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
                .map_or_else(|_| "unknown".to_owned(), |s| s.trim().to_string());
            let os = "Linux".to_owned();
            let kernel_version = std::fs::read_to_string("/proc/version")
                .ok()
                .map(|s| {
                    s.lines()
                        .next()
                        .unwrap_or_default()
                        .split_whitespace()
                        .nth(2)
                        .unwrap_or_default()
                        .to_string()
                })
                .filter(|v| !v.is_empty());
            Self {
                hostname,
                os,
                kernel_version,
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            let hostname = std::env::var("HOSTNAME")
                .or_else(|_| std::env::var("COMPUTERNAME"))
                .unwrap_or_else(|_| "unknown".to_owned());
            let os = std::env::consts::OS.to_string();
            Self {
                hostname,
                os,
                kernel_version: None,
            }
        }
    }
}

/// Get the standard runtime directory for the current user
///
/// Returns `$XDG_RUNTIME_DIR` if set, otherwise `/run/user/{uid}`.
///
/// This follows the XDG Base Directory Specification:
/// <https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html>
///
/// # Platform Support
///
/// - **Linux**: ✅ `/run/user/{uid}` (systemd standard)
/// - **macOS**: ⚠️  No standard, uses `XDG_RUNTIME_DIR` or `/tmp`
/// - **Windows**: ❌ Not applicable
///
/// # Examples
///
/// ```
/// use petal_tongue_core::system_info::get_user_runtime_dir;
///
/// let runtime_dir = get_user_runtime_dir();
/// let socket_path = runtime_dir.join("myapp.sock");
/// ```
///
/// # TRUE PRIMAL Principles
///
/// - **No Hardcoding**: Uses environment and UID discovery
/// - **Capability-Based**: Standard XDG directories
/// - **Graceful Fallback**: Constructs path if env var missing
/// - **biomeOS Compatible**: Follows inter-primal socket conventions
#[must_use]
pub fn get_user_runtime_dir() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR").map_or_else(
        |_| {
            let uid = get_current_uid();
            PathBuf::from(format!("/run/user/{uid}"))
        },
        PathBuf::from,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_uid() {
        let uid = get_current_uid();
        // UID should be a reasonable value (0 for root, >0 for users)
        // On Unix systems, UIDs are typically 0-65535, but can go higher
        assert!(uid < 1_000_000, "UID should be reasonable: {uid}");
    }

    #[test]
    fn test_get_user_runtime_dir() {
        let runtime_dir = get_user_runtime_dir();

        // Should either be XDG_RUNTIME_DIR or /run/user/{uid}
        let path_str = runtime_dir.to_string_lossy();

        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            assert_eq!(path_str, xdg, "Should use XDG_RUNTIME_DIR when set");
        } else {
            let uid = get_current_uid();
            let expected = format!("/run/user/{uid}");
            assert_eq!(path_str, expected, "Should construct /run/user/<uid>");
        }
    }

    #[test]
    fn test_runtime_dir_is_path() {
        let runtime_dir = get_user_runtime_dir();
        // Should be a valid path (doesn't need to exist)
        assert!(!runtime_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_system_info_discover() {
        let info = SystemInfo::discover();
        assert!(!info.hostname.is_empty());
        assert!(!info.os.is_empty());
    }
}
