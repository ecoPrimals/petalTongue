// SPDX-License-Identifier: AGPL-3.0-or-later

/// Load the ecosystem manifest TOML from the workspace root.
///
/// Discovery order:
/// 1. `ECOSYSTEM_MANIFEST_PATH` env (explicit override)
/// 2. Adjacent `ecosystem_manifest.toml` (standard location)
pub(super) fn load_ecosystem_manifest() -> toml::Table {
    let path = std::env::var("ECOSYSTEM_MANIFEST_PATH").map_or_else(
        |_| {
            let mut p = std::env::current_dir().unwrap_or_default();
            p.push("ecosystem_manifest.toml");
            p
        },
        std::path::PathBuf::from,
    );
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.parse::<toml::Table>().ok())
        .unwrap_or_default()
}
