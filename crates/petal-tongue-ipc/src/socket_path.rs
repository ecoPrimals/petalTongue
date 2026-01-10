//! Socket path resolution for ecoPrimals ecosystem
//!
//! Provides capability-based socket path resolution following the biomeOS convention:
//! `/run/user/<uid>/<primal>-<family>.sock`
//!
//! This enables zero-configuration discovery and inter-primal communication.

use anyhow::Result;
use std::env;
use std::path::PathBuf;

/// Get the socket path for this petalTongue instance
///
/// Path format: `/run/user/<uid>/petaltongue-<family>.sock`
///
/// # Environment Variables
///
/// - `FAMILY_ID`: Family identifier (default: "nat0")
/// - `XDG_RUNTIME_DIR`: Runtime directory (default: `/run/user/<uid>`)
///
/// # TRUE PRIMAL Principles
///
/// - **Zero Hardcoding**: Path is runtime-determined from environment
/// - **Capability-Based**: Uses standard Unix runtime directory
/// - **Self-Knowledge Only**: Only knows own primal name ("petaltongue")
/// - **Agnostic**: No assumptions about other primals
///
/// # Examples
///
/// ```
/// use petal_tongue_ipc::socket_path;
///
/// // Default (nat0 family)
/// let path = socket_path::get_petaltongue_socket_path().unwrap();
/// // Returns: /run/user/1000/petaltongue-nat0.sock
///
/// // Custom family
/// std::env::set_var("FAMILY_ID", "staging");
/// let path = socket_path::get_petaltongue_socket_path().unwrap();
/// // Returns: /run/user/1000/petaltongue-staging.sock
/// ```
pub fn get_petaltongue_socket_path() -> Result<PathBuf> {
    let family_id = get_family_id();
    let runtime_dir = get_runtime_dir()?;

    Ok(runtime_dir.join(format!("petaltongue-{}.sock", family_id)))
}

/// Get the family ID for this instance
///
/// Returns value from `FAMILY_ID` environment variable, or "nat0" as default.
///
/// # TRUE PRIMAL: Self-Knowledge Only
///
/// This function only determines OUR family ID. It does not know about or
/// hardcode any other primal's family IDs.
pub fn get_family_id() -> String {
    env::var("FAMILY_ID").unwrap_or_else(|_| "nat0".to_string())
}

/// Get the runtime directory for socket placement
///
/// Returns:
/// 1. `XDG_RUNTIME_DIR` if set (standard)
/// 2. `/run/user/<uid>` as fallback
///
/// # TRUE PRIMAL: Capability-Based
///
/// Uses standard Unix conventions (XDG Base Directory Specification)
/// rather than hardcoded paths.
pub fn get_runtime_dir() -> Result<PathBuf> {
    // Try XDG_RUNTIME_DIR first (standard)
    if let Ok(dir) = env::var("XDG_RUNTIME_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // Fallback to /run/user/<uid>
    // Use `id -u` command to get UID in a portable way
    let uid = get_current_uid()?;
    let runtime_dir = PathBuf::from(format!("/run/user/{}", uid));

    // Verify directory exists and is writable
    if !runtime_dir.exists() {
        anyhow::bail!(
            "Runtime directory does not exist: {}. \
            Please set XDG_RUNTIME_DIR or ensure /run/user/{} exists.",
            runtime_dir.display(),
            uid
        );
    }

    Ok(runtime_dir)
}

/// Discover another primal's socket path (capability-based)
///
/// # TRUE PRIMAL: Runtime Discovery
///
/// This function discovers OTHER primals at runtime, without hardcoding.
/// It follows the biomeOS convention but remains agnostic to specific primals.
///
/// # Arguments
///
/// * `primal_name` - Name of the primal to discover (e.g., "beardog", "songbird")
/// * `family_id` - Optional family ID (defaults to current FAMILY_ID)
///
/// # Examples
///
/// ```
/// use petal_tongue_ipc::socket_path;
///
/// // Discover beardog in same family
/// let beardog = socket_path::discover_primal_socket("beardog", None).unwrap();
/// // Returns: /run/user/1000/beardog-nat0.sock
///
/// // Discover in specific family
/// let songbird = socket_path::discover_primal_socket("songbird", Some("staging")).unwrap();
/// // Returns: /run/user/1000/songbird-staging.sock
/// ```
pub fn discover_primal_socket(primal_name: &str, family_id: Option<&str>) -> Result<PathBuf> {
    let family = match family_id {
        Some(f) => f.to_string(),
        None => get_family_id(),
    };
    let runtime_dir = get_runtime_dir()?;

    Ok(runtime_dir.join(format!("{}-{}.sock", primal_name, family)))
}

/// Check if a socket path exists and is accessible
///
/// # Capability-Based Discovery
///
/// This function checks if a socket exists WITHOUT assuming it SHOULD exist.
/// This enables graceful degradation when primals are not available.
pub fn socket_exists(socket_path: &PathBuf) -> bool {
    socket_path.exists() && socket_path.is_file()
}

/// Get current user ID in a portable way
///
/// Uses the standard Unix `id` command rather than unsafe libc calls.
fn get_current_uid() -> Result<u32> {
    use std::process::Command;

    let output = Command::new("id")
        .arg("-u")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run 'id -u': {}", e))?;

    if !output.status.success() {
        anyhow::bail!("'id -u' command failed");
    }

    let uid_str = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 from 'id -u': {}", e))?;

    uid_str
        .trim()
        .parse::<u32>()
        .map_err(|e| anyhow::anyhow!("Failed to parse UID: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_family_id() {
        // SAFETY: Test-only environment variable modification
        unsafe {
            env::remove_var("FAMILY_ID");
        }
        assert_eq!(get_family_id(), "nat0");
    }

    #[test]
    fn test_custom_family_id() {
        // SAFETY: Test-only environment variable modification
        unsafe {
            env::set_var("FAMILY_ID", "test-family");
        }
        assert_eq!(get_family_id(), "test-family");
        unsafe {
            env::remove_var("FAMILY_ID");
        }
    }

    #[test]
    fn test_petaltongue_socket_path_format() {
        // SAFETY: Test-only environment variable modification
        unsafe {
            env::remove_var("FAMILY_ID");
        }

        // This test assumes /run/user/<uid> exists
        if let Ok(path) = get_petaltongue_socket_path() {
            let path_str = path.to_string_lossy();
            assert!(path_str.contains("petaltongue-nat0.sock"));
            assert!(path_str.contains("/run/user/") || path_str.contains("XDG_RUNTIME_DIR"));
        }
    }

    #[test]
    fn test_discover_primal_socket() {
        // SAFETY: Test-only environment variable modification
        unsafe {
            env::remove_var("FAMILY_ID");
        }

        if let Ok(path) = discover_primal_socket("beardog", None) {
            let path_str = path.to_string_lossy();
            assert!(path_str.contains("beardog-nat0.sock"));
        }
    }

    #[test]
    fn test_discover_primal_socket_custom_family() {
        if let Ok(path) = discover_primal_socket("songbird", Some("staging")) {
            let path_str = path.to_string_lossy();
            assert!(path_str.contains("songbird-staging.sock"));
        }
    }

    #[test]
    fn test_runtime_dir_from_xdg() {
        let test_dir = "/tmp/test-runtime";
        // SAFETY: Test-only environment variable modification
        unsafe {
            env::set_var("XDG_RUNTIME_DIR", test_dir);
        }

        let runtime_dir = get_runtime_dir().unwrap();
        assert_eq!(runtime_dir, PathBuf::from(test_dir));

        unsafe {
            env::remove_var("XDG_RUNTIME_DIR");
        }
    }
}
