// SPDX-License-Identifier: AGPL-3.0-or-later
//! Generic primal discovery helpers (DRY socket path resolution). loamSpine/sweetGrass pattern.

use std::path::PathBuf;

/// Env var name for primal socket, e.g. `socket_env_var("barracuda")` → `"BARRACUDA_SOCKET"`.
#[must_use]
pub fn socket_env_var(primal: &str) -> String {
    format!("{}_SOCKET", primal.to_uppercase())
}

/// Env var name for primal address/URL, e.g. `address_env_var("biomeos")` → `"BIOMEOS_URL"`.
#[must_use]
pub fn address_env_var(primal: &str) -> String {
    format!("{}_URL", primal.to_uppercase())
}

fn path_exists_as_file(p: &std::path::Path) -> bool {
    p.exists() && p.is_file()
}

/// Resolves primal socket: env override → `$XDG_RUNTIME_DIR/biomeos/{primal}.sock`
/// → `/tmp/biomeos/{primal}.sock`. Returns first path that exists as a file.
#[must_use]
pub fn resolve_primal_socket(primal: &str) -> Option<PathBuf> {
    resolve_primal_socket_with_env(primal, |k| std::env::var(k).ok())
}

/// Same as `resolve_primal_socket` but uses injectable env reader (DI for tests).
#[must_use]
pub fn resolve_primal_socket_with_env(
    primal: &str,
    env_reader: impl Fn(&str) -> Option<String>,
) -> Option<PathBuf> {
    let socket_var = socket_env_var(primal);
    if let Some(path) = env_reader(&socket_var) {
        let p = PathBuf::from(path);
        if path_exists_as_file(&p) {
            return Some(p);
        }
    }

    let xdg_path = env_reader("XDG_RUNTIME_DIR").map(|d| {
        PathBuf::from(d)
            .join("biomeos")
            .join(format!("{primal}.sock"))
    });
    if let Some(ref p) = xdg_path
        && path_exists_as_file(p)
    {
        return Some(p.clone());
    }

    let tmp_path = PathBuf::from("/tmp")
        .join("biomeos")
        .join(format!("{primal}.sock"));
    if path_exists_as_file(&tmp_path) {
        return Some(tmp_path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_env_var_format() {
        assert_eq!(socket_env_var("barracuda"), "BARRACUDA_SOCKET");
        assert_eq!(socket_env_var("biomeos"), "BIOMEOS_SOCKET");
    }

    #[test]
    fn address_env_var_format() {
        assert_eq!(address_env_var("biomeos"), "BIOMEOS_URL");
        assert_eq!(address_env_var("songbird"), "SONGBIRD_URL");
    }

    #[test]
    fn resolve_prefers_env_var_when_exists() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("custom.sock");
        std::fs::File::create(&sock).unwrap();
        let env = |k: &str| (k == "BARRACUDA_SOCKET").then(|| sock.to_string_lossy().into_owned());
        assert_eq!(resolve_primal_socket_with_env("barracuda", env), Some(sock));
    }

    #[test]
    fn resolve_skips_env_var_when_path_missing() {
        let env = |k: &str| (k == "BARRACUDA_SOCKET").then(|| "/nonexistent/path.sock".to_string());
        assert!(resolve_primal_socket_with_env("barracuda", env).is_none());
    }

    fn xdg_env(dir: &std::path::Path) -> impl Fn(&str) -> Option<String> {
        let xdg = dir.to_string_lossy().into_owned();
        move |k: &str| {
            if k == "XDG_RUNTIME_DIR" {
                Some(xdg.clone())
            } else {
                None
            }
        }
    }

    #[test]
    fn resolve_uses_xdg_biomeos_layout() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos_dir = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        let sock = biomeos_dir.join("barracuda.sock");
        std::fs::File::create(&sock).unwrap();
        assert_eq!(
            resolve_primal_socket_with_env("barracuda", xdg_env(dir.path())),
            Some(sock)
        );
    }

    #[test]
    fn resolve_returns_none_when_nothing_exists() {
        let env = |_k: &str| None;
        assert!(resolve_primal_socket_with_env("barracuda", env).is_none());
    }
}
