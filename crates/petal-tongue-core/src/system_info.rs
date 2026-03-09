// SPDX-License-Identifier: AGPL-3.0-only
//! System Information Utilities
//!
//! Safe wrappers around system calls for discovering runtime information.
//!
//! # Philosophy
//!
//! - **Encapsulate unsafe**: Wrap FFI in safe, well-documented APIs
//! - **No assumptions**: Discover at runtime, don't hardcode
//! - **Cross-platform**: Abstract platform differences

use std::path::PathBuf;

/// Get the current user's UID (User ID)
///
/// Uses `rustix::process::getuid()` - 100% safe Rust, no unsafe code.
/// The function is always safe because `getuid()` cannot fail
/// and returns the process's effective user ID.
///
/// # Platform Support
///
/// - **Linux**: ✅ Supported
/// - **macOS**: ✅ Supported  
/// - **Windows**: ❌ Not applicable (Windows uses SIDs, not UIDs)
///
/// # Examples
///
/// ```
/// use petal_tongue_core::system_info::get_current_uid;
///
/// let uid = get_current_uid();
/// println!("Running as UID: {}", uid);
/// ```
///
/// # TRUE PRIMAL Principles
///
/// - **Self-Knowledge**: Discover own UID at runtime
/// - **No Hardcoding**: Never assume a specific UID
/// - **Capability-Based**: Use for socket paths, not authentication
#[must_use]
pub fn get_current_uid() -> u32 {
    // rustix::process::getuid() is a safe wrapper around the getuid syscall.
    // No unsafe code needed - this is pure Rust with the same functionality.
    //
    // Benefits of rustix:
    // - 100% safe Rust (no unsafe blocks)
    // - Type-safe wrappers for Unix syscalls
    // - Better error handling
    // - Zero-cost abstractions

    #[cfg(unix)]
    {
        rustix::process::getuid().as_raw()
    }

    #[cfg(not(unix))]
    {
        // On Windows, UID concept doesn't exist
        // Return a placeholder value (0 = "system")
        0
    }
}

/// Get the current effective user ID (EUID)
///
/// Uses `rustix::process::geteuid()` - 100% safe Rust.
/// For root check (e.g. framebuffer access): `get_current_euid() == 0`.
///
/// # Platform Support
///
/// - **Linux**: ✅ Supported
/// - **macOS**: ✅ Supported
/// - **Windows**: ❌ Returns 0 (not applicable)
#[must_use]
pub fn get_current_euid() -> u32 {
    #[cfg(unix)]
    {
        rustix::process::geteuid().as_raw()
    }

    #[cfg(not(unix))]
    {
        0
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
    if let Ok(path) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(path)
    } else {
        let uid = get_current_uid();
        PathBuf::from(format!("/run/user/{uid}"))
    }
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
            assert_eq!(path_str, expected, "Should construct /run/user/{{uid}}");
        }
    }

    #[test]
    fn test_runtime_dir_is_path() {
        let runtime_dir = get_user_runtime_dir();
        // Should be a valid path (doesn't need to exist)
        assert!(!runtime_dir.as_os_str().is_empty());
    }
}
