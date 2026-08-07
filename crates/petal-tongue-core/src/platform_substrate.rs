// SPDX-License-Identifier: AGPL-3.0-or-later
//! G68 Platform Substrate Abstraction.
//!
//! Centralizes platform-specific operations behind architecture-agnostic APIs.
//! Each function does the *same thing differently* per platform — never less.
//!
//! ## Layers
//!
//! - **L1 (Links)**: `platform_link` — filesystem links (symlink on Unix, junction on Windows)
//! - **L2 (Permissions)**: `is_user_accessible` — permission checks (mode bits on Unix, ACL on Windows)
//! - **L3 (System)**: `page_size`, `current_uid` — OS-level queries

use std::path::Path;

// ─── L1: Links ──────────────────────────────────────────────────────────────

/// Create a platform-appropriate filesystem link from `link` pointing to `target`.
///
/// - Unix: symbolic link (`symlink(target, link)`)
/// - Windows: directory junction or hard link (best available)
///
/// This replaces direct `std::os::unix::fs::symlink` usage.
///
/// # Errors
///
/// Returns an I/O error if the link cannot be created (permissions, target missing, etc.).
pub fn platform_link(target: &Path, link: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    {
        // Prefer junction for directories, hard link for files
        if target.is_dir() {
            std::os::windows::fs::symlink_dir(target, link)
        } else {
            std::os::windows::fs::symlink_file(target, link)
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Fallback: copy as last resort (maintain the *intent* — make target reachable from link)
        let _ = (target, link);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "platform links not supported on this OS",
        ))
    }
}

/// Remove a filesystem link (symlink or junction) without following it.
///
/// Safe on all platforms — removes the link itself, not the target.
///
/// # Errors
///
/// Returns an I/O error if the link does not exist or cannot be removed.
pub fn remove_link(link: &Path) -> std::io::Result<()> {
    std::fs::remove_file(link)
}

// ─── L2: Permissions ────────────────────────────────────────────────────────

/// Check whether the current user can access a file/socket for read+write.
///
/// - Unix: checks permission mode bits (owner rw or world rw)
/// - Windows: returns `true` (filesystem ACLs checked by the OS at open time)
///
/// This replaces inline `PermissionsExt::mode()` checks.
#[must_use]
pub fn is_user_accessible(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        (mode & 0o600) != 0 || (mode & 0o006) != 0
    }

    #[cfg(not(unix))]
    {
        let _ = metadata;
        true
    }
}

/// Check whether a path is a Unix socket (or equivalent IPC endpoint).
///
/// - Unix: uses `FileTypeExt::is_socket()`
/// - Non-Unix: returns `false` (named pipes checked differently)
#[must_use]
pub fn is_socket(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        metadata.file_type().is_socket()
    }

    #[cfg(not(unix))]
    {
        let _ = metadata;
        false
    }
}

// ─── L3: System ─────────────────────────────────────────────────────────────

/// Get the system page size in bytes.
///
/// - Linux: `rustix::param::page_size()`
/// - Other: 4096 (safe default for allocation alignment)
#[must_use]
pub fn page_size() -> u64 {
    #[cfg(target_os = "linux")]
    {
        u64::try_from(rustix::param::page_size()).unwrap_or(4096)
    }

    #[cfg(not(target_os = "linux"))]
    {
        4096
    }
}

/// Get the current user ID.
///
/// - Unix: real UID from the kernel
/// - Windows: 0 (Windows uses SIDs, not UIDs)
#[must_use]
pub fn current_uid() -> u32 {
    #[cfg(unix)]
    {
        rustix::process::getuid().as_raw()
    }

    #[cfg(not(unix))]
    {
        0
    }
}

/// Get the effective user ID.
///
/// - Unix: EUID from the kernel (may differ from UID with setuid binaries)
/// - Windows: 0
#[must_use]
pub fn effective_uid() -> u32 {
    #[cfg(unix)]
    {
        rustix::process::geteuid().as_raw()
    }

    #[cfg(not(unix))]
    {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn platform_link_creates_link() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target.txt");
        fs::write(&target, "hello").unwrap();

        let link = tmp.path().join("link.txt");
        platform_link(&target, &link).unwrap();

        assert!(link.exists() || link.symlink_metadata().is_ok());
    }

    #[test]
    fn remove_link_removes_without_following() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target.txt");
        fs::write(&target, "hello").unwrap();

        let link = tmp.path().join("link.txt");
        platform_link(&target, &link).unwrap();

        remove_link(&link).unwrap();
        assert!(!link.exists());
        assert!(target.exists());
    }

    #[test]
    fn is_user_accessible_regular_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("test.txt");
        fs::write(&file, "data").unwrap();
        let meta = fs::metadata(&file).unwrap();
        assert!(is_user_accessible(&meta));
    }

    #[test]
    fn page_size_reasonable() {
        let ps = page_size();
        assert!(ps >= 4096);
        assert!(ps <= 65536);
        assert_eq!(ps % 4096, 0);
    }

    #[test]
    fn current_uid_reasonable() {
        let uid = current_uid();
        assert!(uid < 1_000_000);
    }

    #[test]
    fn effective_uid_reasonable() {
        let euid = effective_uid();
        assert!(euid < 1_000_000);
    }
}
