// SPDX-License-Identifier: AGPL-3.0-or-later
//! Socket path resolution for ecoPrimals ecosystem
//!
//! Provides capability-based socket path resolution following the biomeOS convention:
//! `$XDG_RUNTIME_DIR/biomeos/<primal>.sock`
//!
//! Per `CAPABILITY_BASED_DISCOVERY_STANDARD.md` v1.1 and `IPC_COMPLIANCE_MATRIX.md` v1.2,
//! all primal sockets live under the shared `biomeos/` directory so that biomeOS socket
//! scanning can discover them via `readdir()`.

use crate::btsp::BtspPosture;
use crate::socket_path_error::SocketPathError;
use petal_tongue_core::capability_names::primal_names;
use std::env;
use std::path::{Path, PathBuf};

/// Shared biomeOS socket directory name.
const BIOMEOS_SOCK_DIR: &str = primal_names::BIOMEOS;

/// Get the socket path for this petalTongue instance.
///
/// Default path: `$XDG_RUNTIME_DIR/biomeos/petaltongue.sock`
///
/// # Environment Variables (Priority Order)
///
/// 1. **`PETALTONGUE_SOCKET`**: Explicit socket path override (highest priority)
/// 2. **`XDG_RUNTIME_DIR`**: Runtime directory (standard, falls back to `/run/user/<uid>`)
///
/// # biomeOS Socket Standard Compliance
///
/// Per `IPC_COMPLIANCE_MATRIX.md` v1.2 item PT-01, the canonical socket path is
/// `$XDG_RUNTIME_DIR/biomeos/petaltongue.sock`. This places petalTongue in the
/// shared `biomeos/` directory where biomeOS socket scanning can discover it.
///
/// - **Explicit Override**: `PETALTONGUE_SOCKET` for custom paths
/// - **XDG Compliant**: Uses `/run/user/<uid>/biomeos/` for secure sockets
/// - **Graceful Fallback**: Falls back to `/tmp/biomeos/` if XDG runtime unavailable
/// - **Parent Directory Creation**: Ensures socket parent directory exists
pub fn get_petaltongue_socket_path() -> Result<PathBuf, SocketPathError> {
    let posture = crate::btsp::validate_insecure_guard()?;
    get_petaltongue_socket_path_with_posture(&posture)
}

/// Resolve the petalTongue socket path using an explicit BTSP posture.
///
/// Same rules as [`get_petaltongue_socket_path`], but the filename under
/// `$XDG_RUNTIME_DIR/biomeos/` follows [`crate::btsp::socket_filename`].
pub fn get_petaltongue_socket_path_with_posture(
    posture: &BtspPosture,
) -> Result<PathBuf, SocketPathError> {
    if let Ok(socket_path) = env::var("PETALTONGUE_SOCKET") {
        let path = PathBuf::from(socket_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        return Ok(path);
    }

    let filename = crate::btsp::socket_filename(posture);

    if let Ok(runtime_dir) = get_runtime_dir() {
        let sock_dir = runtime_dir.join(BIOMEOS_SOCK_DIR);
        let path = sock_dir.join(&filename);
        std::fs::create_dir_all(&sock_dir)?;
        Ok(path)
    } else {
        let sock_dir = PathBuf::from("/tmp").join(BIOMEOS_SOCK_DIR);
        let path = sock_dir.join(&filename);
        std::fs::create_dir_all(&sock_dir)?;
        Ok(path)
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
#[must_use]
pub fn get_family_id() -> String {
    env::var("FAMILY_ID").unwrap_or_else(|_| "nat0".to_string())
}

/// Get the node ID for this instance.
///
/// Returns value from `PETALTONGUE_NODE_ID` environment variable, or "default" as default.
/// Used for registration and identity purposes (not embedded in the default socket filename).
#[must_use]
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
pub fn get_runtime_dir() -> Result<PathBuf, SocketPathError> {
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
        Err(SocketPathError::RuntimeDirNotFound { path: runtime_dir })
    }
}

/// Discover another primal's socket path (capability-based).
///
/// Follows the biomeOS convention: `$XDG_RUNTIME_DIR/biomeos/<primal>.sock`.
///
/// # Discovery Chain
///
/// 1. `<PRIMAL>_SOCKET` env var (explicit override)
/// 2. `$XDG_RUNTIME_DIR/biomeos/<primal>.sock` (standard)
/// 3. `/tmp/biomeos/<primal>.sock` (fallback)
///
/// The `family_id` and `node_id` parameters are accepted for API compatibility
/// but are not embedded in the default socket filename (per ecosystem standard).
pub fn discover_primal_socket(
    primal_name: &str,
    _family_id: Option<&str>,
    _node_id: Option<&str>,
) -> Result<PathBuf, SocketPathError> {
    let env_var = format!("{}_SOCKET", primal_name.to_uppercase());
    if let Ok(socket_path) = env::var(&env_var) {
        return Ok(PathBuf::from(socket_path));
    }

    if let Ok(runtime_dir) = get_runtime_dir() {
        return Ok(runtime_dir
            .join(BIOMEOS_SOCK_DIR)
            .join(format!("{primal_name}.sock")));
    }

    Ok(PathBuf::from(format!(
        "/tmp/{BIOMEOS_SOCK_DIR}/{primal_name}.sock"
    )))
}

/// Check if a socket path exists and is accessible
///
/// # Capability-Based Discovery
///
/// This function checks if a socket exists WITHOUT assuming it SHOULD exist.
/// This enables graceful degradation when primals are not available.
#[must_use]
pub fn socket_exists(socket_path: &Path) -> bool {
    socket_path.exists() && socket_path.is_file()
}

/// Get current user ID in a portable way
///
/// Uses the standard Unix `id` command rather than unsafe libc calls.
fn get_current_uid() -> Result<u32, SocketPathError> {
    use std::process::Command;

    let output = Command::new("id")
        .arg("-u")
        .output()
        .map_err(|e| SocketPathError::GetUid(format!("Failed to run 'id -u': {e}")))?;

    if !output.status.success() {
        return Err(SocketPathError::GetUid(
            "'id -u' command failed".to_string(),
        ));
    }

    let uid_str = String::from_utf8(output.stdout)
        .map_err(|e| SocketPathError::GetUid(format!("Invalid UTF-8 from 'id -u': {e}")))?;

    uid_str
        .trim()
        .parse::<u32>()
        .map_err(|e| SocketPathError::GetUid(format!("Failed to parse UID: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::test_fixtures::env_test_helpers;

    #[test]
    fn test_default_family_id() {
        env_test_helpers::with_env_var_removed("FAMILY_ID", || assert_eq!(get_family_id(), "nat0"));
    }

    #[test]
    fn test_custom_family_id() {
        env_test_helpers::with_env_var("FAMILY_ID", "test-family", || {
            assert_eq!(get_family_id(), "test-family");
        });
    }

    #[test]
    fn test_petaltongue_socket_path_format() {
        env_test_helpers::with_env_vars(
            &[
                ("FAMILY_ID", None),
                ("PETALTONGUE_NODE_ID", None),
                ("PETALTONGUE_SOCKET", None),
                ("XDG_RUNTIME_DIR", None),
                ("BIOMEOS_INSECURE", None),
            ],
            || {
                if let Ok(path) = get_petaltongue_socket_path() {
                    let path_str = path.to_string_lossy();
                    assert!(
                        path_str.ends_with("biomeos/petaltongue.sock"),
                        "Expected path ending with 'biomeos/petaltongue.sock', got: {path_str}"
                    );
                }
            },
        );
    }

    #[test]
    fn test_petaltongue_socket_path_family_scoped() {
        env_test_helpers::with_env_vars(
            &[
                ("FAMILY_ID", Some("acme")),
                ("PETALTONGUE_SOCKET", None),
                ("XDG_RUNTIME_DIR", None),
                ("BIOMEOS_INSECURE", None),
            ],
            || {
                if let Ok(path) = get_petaltongue_socket_path() {
                    let path_str = path.to_string_lossy();
                    assert!(
                        path_str.ends_with("biomeos/petaltongue-acme.sock"),
                        "Expected family-scoped socket, got: {path_str}"
                    );
                }
            },
        );
    }

    #[test]
    fn test_btsp_guard_conflicts_with_socket_path() {
        env_test_helpers::with_env_vars(
            &[
                ("FAMILY_ID", Some("prod")),
                ("BIOMEOS_INSECURE", Some("1")),
                ("PETALTONGUE_SOCKET", None),
                ("XDG_RUNTIME_DIR", None),
            ],
            || {
                let err = get_petaltongue_socket_path().expect_err("conflicting posture");
                assert!(
                    err.to_string().contains("BTSP guard"),
                    "Expected BTSP error, got: {err}"
                );
            },
        );
    }

    #[test]
    fn test_discover_primal_socket() {
        env_test_helpers::with_env_vars(
            &[
                ("FAMILY_ID", None),
                ("BEARDOG_SOCKET", None),
                ("XDG_RUNTIME_DIR", None),
            ],
            || {
                if let Ok(path) = discover_primal_socket("beardog", None, None) {
                    let path_str = path.to_string_lossy();
                    assert!(
                        path_str.ends_with("biomeos/beardog.sock"),
                        "Expected biomeos/beardog.sock, got: {path_str}"
                    );
                }
            },
        );
    }

    #[test]
    fn test_discover_primal_socket_ignores_family_node() {
        if let Ok(path) = discover_primal_socket("songbird", Some("staging"), Some("node1")) {
            let path_str = path.to_string_lossy();
            assert!(
                path_str.ends_with("biomeos/songbird.sock"),
                "Standard path should not embed family/node: {path_str}"
            );
        }
    }

    #[test]
    fn test_petaltongue_socket_override() {
        env_test_helpers::with_env_vars(
            &[
                ("XDG_RUNTIME_DIR", None),
                ("PETALTONGUE_SOCKET", Some("/tmp/custom-petaltongue.sock")),
            ],
            || {
                let path = get_petaltongue_socket_path().unwrap();
                assert_eq!(
                    path,
                    PathBuf::from("/tmp/custom-petaltongue.sock"),
                    "PETALTONGUE_SOCKET override should take priority"
                );
            },
        );
    }

    #[test]
    fn test_node_id() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_NODE_ID", || {
            assert_eq!(get_node_id(), "default");
        });
        env_test_helpers::with_env_var("PETALTONGUE_NODE_ID", "node1", || {
            assert_eq!(get_node_id(), "node1");
        });
    }

    #[test]
    fn test_primal_socket_env_override() {
        env_test_helpers::with_env_var("MYSERVICE_SOCKET", "/tmp/custom-myservice.sock", || {
            let path = discover_primal_socket("myservice", None, None).unwrap();
            assert_eq!(path, PathBuf::from("/tmp/custom-myservice.sock"));
        });
    }

    #[test]
    fn test_runtime_dir_from_xdg() {
        let test_dir = "/tmp/test-runtime";
        env_test_helpers::with_env_var("XDG_RUNTIME_DIR", test_dir, || {
            let runtime_dir = get_runtime_dir().unwrap();
            assert_eq!(runtime_dir, PathBuf::from(test_dir));
        });
    }

    #[test]
    fn test_get_runtime_dir_xdg_parent_exists() {
        // XDG_RUNTIME_DIR can point to non-existent dir if parent exists
        env_test_helpers::with_env_var("XDG_RUNTIME_DIR", "/tmp/xdg-runtime-test", || {
            let result = get_runtime_dir();
            assert!(result.is_ok(), "parent /tmp exists");
        });
    }
}
