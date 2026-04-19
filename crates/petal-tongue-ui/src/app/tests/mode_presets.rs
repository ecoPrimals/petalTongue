// SPDX-License-Identifier: AGPL-3.0-or-later

#[test]
fn mode_presets_produce_commands() {
    let cmds = crate::mode_presets::commands_for_mode("clinical");
    assert!(!cmds.is_empty());
    let cmds = crate::mode_presets::commands_for_mode("developer");
    assert!(!cmds.is_empty());
}

#[test]
fn mode_presets_unknown_returns_empty() {
    let cmds = crate::mode_presets::commands_for_mode("nonexistent_mode_xyz");
    assert!(cmds.is_empty());
}

#[test]
fn mode_presets_developer() {
    let cmds = crate::mode_presets::commands_for_mode("developer");
    assert!(!cmds.is_empty());
}

#[test]
fn mode_presets_presentation() {
    let cmds = crate::mode_presets::commands_for_mode("presentation");
    assert!(!cmds.is_empty());
}
