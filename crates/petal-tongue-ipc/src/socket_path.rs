// SPDX-License-Identifier: AGPL-3.0-only
//! Socket path resolution for ecoPrimals ecosystem
//!
//! Provides capability-based socket path resolution following the biomeOS convention:
//! `/run/user/<uid>/<primal>-<family>.sock`
//!
//! This enables zero-configuration discovery and inter-primal communication.

use anyhow::Result;
use petal_tongue_core::constants::APP_DIR_NAME;
use std::env;
use std::path::PathBuf;

/// Get the socket path for this petalTongue instance
///
/// Path format: `/run/user/<uid>/petaltongue-<family>-<node>.sock`
///
/// # Environment Variables (Priority Order)
///
/// 1. **`PETALTONGUE_SOCKET`**: Explicit socket path override (highest priority)
/// 2. **`FAMILY_ID`**: Family identifier (default: "nat0")
/// 3. **`PETALTONGUE_NODE_ID`**: Node identifier for multi-instance (default: "default")
/// 4. **`XDG_RUNTIME_DIR`**: Runtime directory (standard, falls back to `/run/user/<uid>`)
///
/// # biomeOS Socket Standard Compliance
///
/// This function implements the biomeOS socket standardization:
/// - **Explicit Override**: `PETALTONGUE_SOCKET` for custom paths
/// - **XDG Compliant**: Uses `/run/user/<uid>/` for secure sockets
/// - **Multi-Instance**: Supports `NODE_ID` for multiple instances per family
/// - **Graceful Fallback**: Falls back to `/tmp/` if XDG runtime unavailable
/// - **Parent Directory Creation**: Ensures socket parent directory exists
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
/// // Default (nat0 family, default node)
/// let path = socket_path::get_petaltongue_socket_path().unwrap();
/// // Returns: /run/user/1000/petaltongue-nat0-default.sock
///
/// // Custom family and node
/// // SAFETY: Doctest-only env var manipulation
/// unsafe {
///     std::env::set_var("FAMILY_ID", "staging");
///     std::env::set_var("PETALTONGUE_NODE_ID", "node1");
/// }
/// let path = socket_path::get_petaltongue_socket_path().unwrap();
/// // Returns: /run/user/1000/petaltongue-staging-node1.sock
///
/// // Explicit override
/// unsafe {
///     std::env::set_var("PETALTONGUE_SOCKET", "/tmp/custom.sock");
/// }
/// let path = socket_path::get_petaltongue_socket_path().unwrap();
/// // Returns: /tmp/custom.sock
/// ```
pub fn get_petaltongue_socket_path() -> Result<PathBuf> {
    // Priority 1: Explicit override (biomeOS standard)
    if let Ok(socket_path) = env::var("PETALTONGUE_SOCKET") {
        let path = PathBuf::from(socket_path);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        return Ok(path);
    }

    // Priority 2: XDG runtime directory (standard)
    let family_id = get_family_id();
    let node_id = get_node_id();

    match get_runtime_dir() {
        Ok(runtime_dir) => {
            let path = runtime_dir.join(format!("{APP_DIR_NAME}-{family_id}-{node_id}.sock"));

            // Ensure runtime directory exists
            std::fs::create_dir_all(&runtime_dir)?;

            Ok(path)
        }
        Err(_) => {
            // Priority 3: Fallback to /tmp (last resort)
            let path = PathBuf::from(format!("/tmp/{APP_DIR_NAME}-{family_id}-{node_id}.sock"));

            // Ensure /tmp exists (it should, but be defensive)
            std::fs::create_dir_all("/tmp")?;

            Ok(path)
        }
    }
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

/// Get the node ID for this instance
///
/// Returns value from `PETALTONGUE_NODE_ID` environment variable, or "default" as default.
///
/// # biomeOS Multi-Instance Support
///
/// Node IDs enable running multiple petalTongue instances in the same family:
/// - `petaltongue-nat0-node1.sock`
/// - `petaltongue-nat0-node2.sock`
///
/// This is critical for atomic deployments where multiple instances
/// may need to run on the same machine.
pub fn get_node_id() -> String {
    env::var("PETALTONGUE_NODE_ID").unwrap_or_else(|_| "default".to_string())
}

/// Get the runtime directory for socket placement
///
/// Returns:
/// 1. `XDG_RUNTIME_DIR` if set (standard)
/// 2. `/run/user/<uid>` as fallback
///
/// # biomeOS Socket Standard
///
/// Returns an error if neither XDG nor `/run/user/<uid>` are available.
/// Callers should fall back to `/tmp/` in this case (see `get_petaltongue_socket_path`).
///
/// # TRUE PRIMAL: Capability-Based
///
/// Uses standard Unix conventions (XDG Base Directory Specification)
/// rather than hardcoded paths.
pub fn get_runtime_dir() -> Result<PathBuf> {
    // Try XDG_RUNTIME_DIR first (standard)
    if let Ok(dir) = env::var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() || path.parent().is_some_and(std::path::Path::exists) {
            return Ok(path);
        }
    }

    // Fallback to /run/user/<uid>
    // Use `id -u` command to get UID in a portable way
    let uid = get_current_uid()?;
    let runtime_dir = PathBuf::from(format!("/run/user/{uid}"));

    // Check if directory exists (don't require it, caller will create)
    if runtime_dir.exists() || runtime_dir.parent().is_some_and(std::path::Path::exists) {
        Ok(runtime_dir)
    } else {
        anyhow::bail!(
            "Runtime directory does not exist: {}. \
            Will fall back to /tmp/",
            runtime_dir.display()
        )
    }
}

/// Discover another primal's socket path (capability-based)
///
/// # TRUE PRIMAL: Runtime Discovery
///
/// This function discovers OTHER primals at runtime, without hardcoding.
/// It follows the biomeOS convention but remains agnostic to specific primals.
///
/// # biomeOS Socket Standard
///
/// Tries in order:
/// 1. `<PRIMAL>_SOCKET` env var (if set)
/// 2. `/run/user/<uid>/<primal>-<family>-<node>.sock` (XDG)
/// 3. `/tmp/<primal>-<family>-<node>.sock` (fallback)
///
/// # Arguments
///
/// * `primal_name` - Name of the primal to discover (e.g., "beardog", "songbird")
/// * `family_id` - Optional family ID (defaults to current FAMILY_ID)
/// * `node_id` - Optional node ID (defaults to "default")
///
/// # Examples
///
/// ```
/// use petal_tongue_ipc::socket_path;
///
/// // Discover beardog in same family
/// let beardog = socket_path::discover_primal_socket("beardog", None, None).unwrap();
/// // Returns: /run/user/1000/beardog-nat0-default.sock
///
/// // Discover in specific family and node
/// let songbird = socket_path::discover_primal_socket("songbird", Some("staging"), Some("node1")).unwrap();
/// // Returns: /run/user/1000/songbird-staging-node1.sock
/// ```
pub fn discover_primal_socket(
    primal_name: &str,
    family_id: Option<&str>,
    node_id: Option<&str>,
) -> Result<PathBuf> {
    // Priority 1: Check for explicit override env var
    let env_var = format!("{}_SOCKET", primal_name.to_uppercase());
    if let Ok(socket_path) = env::var(&env_var) {
        return Ok(PathBuf::from(socket_path));
    }

    let family = match family_id {
        Some(f) => f.to_string(),
        None => get_family_id(),
    };

    let node = match node_id {
        Some(n) => n.to_string(),
        None => "default".to_string(),
    };

    // Priority 2: XDG runtime directory
    if let Ok(runtime_dir) = get_runtime_dir() {
        return Ok(runtime_dir.join(format!("{primal_name}-{family}-{node}.sock")));
    }

    // Priority 3: Fallback to /tmp
    Ok(PathBuf::from(format!(
        "/tmp/{primal_name}-{family}-{node}.sock"
    )))
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
        .map_err(|e| anyhow::anyhow!("Failed to run 'id -u': {e}"))?;

    if !output.status.success() {
        anyhow::bail!("'id -u' command failed");
    }

    let uid_str = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 from 'id -u': {e}"))?;

    uid_str
        .trim()
        .parse::<u32>()
        .map_err(|e| anyhow::anyhow!("Failed to parse UID: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Mutex for serializing tests that modify environment variables
    // This prevents race conditions in parallel test execution
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_default_family_id() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // SAFETY: Test-only environment variable modification, serialized by mutex
        unsafe {
            env::remove_var("FAMILY_ID");
        }
        assert_eq!(get_family_id(), "nat0");
    }

    #[test]
    fn test_custom_family_id() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // SAFETY: Test-only environment variable modification, serialized by mutex
        unsafe {
            env::set_var("FAMILY_ID", "test-family");
        }
        let result = get_family_id();
        unsafe {
            env::remove_var("FAMILY_ID");
        }
        assert_eq!(result, "test-family");
    }

    #[test]
    fn test_petaltongue_socket_path_format() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // SAFETY: Test-only environment variable modification, serialized by mutex
        unsafe {
            env::remove_var("FAMILY_ID");
            env::remove_var("PETALTONGUE_NODE_ID");
            env::remove_var("PETALTONGUE_SOCKET");
            env::remove_var("XDG_RUNTIME_DIR");
        }

        // This test uses fallback path since we cleared XDG_RUNTIME_DIR
        if let Ok(path) = get_petaltongue_socket_path() {
            let path_str = path.to_string_lossy();
            // Should include node ID: petaltongue-nat0-default.sock
            assert!(
                path_str.contains("petaltongue-nat0-default.sock"),
                "Expected path to contain 'petaltongue-nat0-default.sock', got: {path_str}"
            );
        }
    }

    #[test]
    fn test_discover_primal_socket() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // SAFETY: Test-only environment variable modification, serialized by mutex
        unsafe {
            env::remove_var("FAMILY_ID");
            env::remove_var("BEARDOG_SOCKET");
            env::remove_var("XDG_RUNTIME_DIR");
        }

        if let Ok(path) = discover_primal_socket("beardog", None, None) {
            let path_str = path.to_string_lossy();
            assert!(path_str.contains("beardog-nat0-default.sock"));
        }
    }

    #[test]
    fn test_discover_primal_socket_custom_family() {
        // This test uses explicit parameters, no env vars needed
        if let Ok(path) = discover_primal_socket("songbird", Some("staging"), Some("node1")) {
            let path_str = path.to_string_lossy();
            assert!(path_str.contains("songbird-staging-node1.sock"));
        }
    }

    #[test]
    fn test_petaltongue_socket_override() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // SAFETY: Test-only environment variable modification, serialized by mutex
        // Clear any potentially interfering vars first
        unsafe {
            env::remove_var("XDG_RUNTIME_DIR");
            env::set_var("PETALTONGUE_SOCKET", "/tmp/custom-petaltongue.sock");
        }

        let path = get_petaltongue_socket_path().unwrap();

        unsafe {
            env::remove_var("PETALTONGUE_SOCKET");
        }

        assert_eq!(
            path,
            PathBuf::from("/tmp/custom-petaltongue.sock"),
            "PETALTONGUE_SOCKET override should take priority"
        );
    }

    #[test]
    fn test_node_id() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // SAFETY: Test-only environment variable modification, serialized by mutex
        unsafe {
            env::remove_var("PETALTONGUE_NODE_ID");
        }
        assert_eq!(get_node_id(), "default");

        unsafe {
            env::set_var("PETALTONGUE_NODE_ID", "node1");
        }
        assert_eq!(get_node_id(), "node1");

        unsafe {
            env::remove_var("PETALTONGUE_NODE_ID");
        }
    }

    #[test]
    fn test_primal_socket_env_override() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // SAFETY: Test-only environment variable modification, serialized by mutex
        unsafe {
            env::set_var("SONGBIRD_SOCKET", "/tmp/custom-songbird.sock");
        }

        let path = discover_primal_socket("songbird", None, None).unwrap();

        unsafe {
            env::remove_var("SONGBIRD_SOCKET");
        }

        assert_eq!(path, PathBuf::from("/tmp/custom-songbird.sock"));
    }

    #[test]
    fn test_runtime_dir_from_xdg() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let test_dir = "/tmp/test-runtime";
        // SAFETY: Test-only environment variable modification, serialized by mutex
        unsafe {
            env::set_var("XDG_RUNTIME_DIR", test_dir);
        }

        let runtime_dir = get_runtime_dir().unwrap();

        unsafe {
            env::remove_var("XDG_RUNTIME_DIR");
        }

        assert_eq!(runtime_dir, PathBuf::from(test_dir));
    }
}
