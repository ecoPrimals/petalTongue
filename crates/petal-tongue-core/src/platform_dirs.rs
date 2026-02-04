//! Pure Rust platform directory resolution
//!
//! Zero dependencies, maximum portability, TRUE PRIMAL compliant.
//!
//! This module provides cross-platform directory resolution using ONLY
//! Rust stdlib (std::env + std::path). No external dependencies, works
//! with ANY toolchain (MSVC, MinGW, musl, ARM64, etc.).
//!
//! # Design
//!
//! - **Zero Hardcoding**: Uses environment variables and platform cfg
//! - **Self-Knowledge Only**: Just needs to know the OS type
//! - **Graceful Degradation**: Built-in fallbacks for missing env vars
//! - **Pure Rust**: Only stdlib, no crates, no C dependencies

use std::path::PathBuf;

/// Error type for directory resolution
#[derive(Debug)]
pub struct DirError {
    message: String,
}

impl DirError {
    fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for DirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Directory error: {}", self.message)
    }
}

impl std::error::Error for DirError {}

/// Get platform-specific data directory
///
/// Returns the appropriate directory for application data based on platform:
/// - **Linux**: `$XDG_DATA_HOME` or `$HOME/.local/share`
/// - **macOS**: `$HOME/Library/Application Support`
/// - **Windows**: `%APPDATA%` or `%USERPROFILE%\AppData\Roaming`
///
/// # Examples
///
/// ```no_run
/// use petal_tongue_core::platform_dirs;
///
/// let data = platform_dirs::data_dir().expect("Should get data dir");
/// let app_data = data.join("petaltongue");
/// ```
///
/// # Errors
///
/// Returns an error if required environment variables are not set
/// (e.g., no HOME on Linux/macOS, no APPDATA/USERPROFILE on Windows)
pub fn data_dir() -> Result<PathBuf, DirError> {
    #[cfg(target_os = "linux")]
    {
        // XDG Base Directory Specification
        // https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data));
        }

        // Fallback to ~/.local/share
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local").join("share"));
        }

        Err(DirError::new(
            "No XDG_DATA_HOME or HOME environment variable found",
        ))
    }

    #[cfg(target_os = "macos")]
    {
        // macOS uses ~/Library/Application Support for app data
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support"));
        }

        Err(DirError::new("No HOME environment variable found"))
    }

    #[cfg(target_os = "windows")]
    {
        // Try APPDATA first (standard roaming application data)
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Ok(PathBuf::from(appdata));
        }

        // Fallback to constructing from USERPROFILE
        if let Ok(profile) = std::env::var("USERPROFILE") {
            return Ok(PathBuf::from(profile).join("AppData").join("Roaming"));
        }

        Err(DirError::new(
            "No APPDATA or USERPROFILE environment variable found",
        ))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        // For unknown platforms, try HOME as a best-effort fallback
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local").join("share"));
        }

        Err(DirError::new(
            "Unsupported platform and no HOME environment variable found",
        ))
    }
}

/// Get platform-specific config directory
///
/// Returns the appropriate directory for application configuration based on platform:
/// - **Linux**: `$XDG_CONFIG_HOME` or `$HOME/.config`
/// - **macOS**: `$HOME/Library/Application Support` (same as data_dir on macOS)
/// - **Windows**: Same as `data_dir()` (config and data are not separated on Windows)
///
/// # Examples
///
/// ```no_run
/// use petal_tongue_core::platform_dirs;
///
/// let config = platform_dirs::config_dir().expect("Should get config dir");
/// let app_config = config.join("petaltongue");
/// ```
///
/// # Errors
///
/// Returns an error if required environment variables are not set
pub fn config_dir() -> Result<PathBuf, DirError> {
    #[cfg(target_os = "linux")]
    {
        // XDG Base Directory Specification
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(xdg_config));
        }

        // Fallback to ~/.config
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join(".config"));
        }

        Err(DirError::new(
            "No XDG_CONFIG_HOME or HOME environment variable found",
        ))
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, config and data both go to ~/Library/Application Support
        data_dir()
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, config and data both go to %APPDATA%
        data_dir()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        // Fallback for unknown platforms
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join(".config"));
        }

        Err(DirError::new(
            "Unsupported platform and no HOME environment variable found",
        ))
    }
}

/// Get platform-specific runtime directory
///
/// Returns the appropriate directory for runtime files (sockets, PIDs) based on platform:
/// - **Linux**: `$XDG_RUNTIME_DIR` or `/run/user/$UID`
/// - **macOS**: `/tmp`
/// - **Windows**: `%TEMP%`
pub fn runtime_dir() -> Result<PathBuf, DirError> {
    #[cfg(target_os = "linux")]
    {
        // XDG Base Directory Specification - runtime dir
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            return Ok(PathBuf::from(xdg_runtime));
        }

        // Fallback: /run/user/$UID
        if let Ok(uid) = std::env::var("UID") {
            return Ok(PathBuf::from(format!("/run/user/{}", uid)));
        }

        // Last resort: /tmp
        return Ok(PathBuf::from("/tmp"));
    }

    #[cfg(target_os = "macos")]
    {
        // macOS doesn't have XDG_RUNTIME_DIR, use /tmp
        Ok(PathBuf::from("/tmp"))
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: use TEMP
        if let Ok(temp) = std::env::var("TEMP") {
            return Ok(PathBuf::from(temp));
        }
        Ok(PathBuf::from("C:\\Windows\\Temp"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err(DirError::new(
        "Runtime directory not available on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_dir_returns_path() {
        // Should always return a path on supported platforms
        let dir = data_dir().expect("Should get data dir on this platform");
        assert!(dir.as_os_str().len() > 0, "Data dir should not be empty");
    }

    #[test]
    fn test_config_dir_returns_path() {
        // Should always return a path on supported platforms
        let dir = config_dir().expect("Should get config dir on this platform");
        assert!(dir.as_os_str().len() > 0, "Config dir should not be empty");
    }

    #[test]
    fn test_data_dir_is_absolute() {
        let dir = data_dir().expect("Should get data dir");
        assert!(dir.is_absolute(), "Data dir should be absolute path");
    }

    #[test]
    fn test_config_dir_is_absolute() {
        let dir = config_dir().expect("Should get config dir");
        assert!(dir.is_absolute(), "Config dir should be absolute path");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_respects_xdg() {
        // If XDG vars are set, they should be used
        // SAFETY: Test-only environment variable manipulation
        unsafe {
            std::env::set_var("XDG_DATA_HOME", "/custom/data");
        }
        let data = data_dir().expect("Should get data dir");
        assert_eq!(data, PathBuf::from("/custom/data"));

        // SAFETY: Test-only environment variable manipulation
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", "/custom/config");
        }
        let config = config_dir().expect("Should get config dir");
        assert_eq!(config, PathBuf::from("/custom/config"));

        // Cleanup
        // SAFETY: Test-only environment variable manipulation
        unsafe {
            std::env::remove_var("XDG_DATA_HOME");
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_respects_appdata() {
        // If APPDATA is set, it should be used
        std::env::set_var("APPDATA", "C:\\Custom\\AppData");
        let data = data_dir().expect("Should get data dir");
        assert_eq!(data, PathBuf::from("C:\\Custom\\AppData"));

        // Cleanup
        std::env::remove_var("APPDATA");
    }
}
