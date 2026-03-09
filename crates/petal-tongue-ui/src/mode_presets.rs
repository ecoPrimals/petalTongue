// SPDX-License-Identifier: AGPL-3.0-only
//! Mode presets — named bundles of motor commands.
//!
//! A "mode" is a clinical / developer / presentation configuration that
//! sets panel visibility, zoom, layout, and other UI state through the
//! efferent motor channel. Each mode is a `Vec<MotorCommand>`.

use petal_tongue_core::{MotorCommand, PanelId};

/// Return the motor commands for a named mode.
///
/// Unknown mode names return an empty list (no-op).
#[must_use]
pub fn commands_for_mode(mode: &str) -> Vec<MotorCommand> {
    match mode {
        "clinical" => clinical_mode(),
        "developer" => developer_mode(),
        "presentation" => presentation_mode(),
        "full" => full_mode(),
        _ => Vec::new(),
    }
}

/// Clinical mode: graph + data channels only, no dev tooling.
fn clinical_mode() -> Vec<MotorCommand> {
    vec![
        MotorCommand::SetPanelVisibility {
            panel: PanelId::LeftSidebar,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::AudioPanel,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TrustDashboard,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::Proprioception,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::GraphStats,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: true,
        },
        MotorCommand::SetAwakening { enabled: false },
        MotorCommand::FitToView,
    ]
}

/// Developer mode: everything visible, full tooling.
fn developer_mode() -> Vec<MotorCommand> {
    vec![
        MotorCommand::SetPanelVisibility {
            panel: PanelId::LeftSidebar,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::AudioPanel,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TrustDashboard,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::Proprioception,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::GraphStats,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: true,
        },
    ]
}

/// Presentation mode: clean, graph-centered, minimal chrome.
fn presentation_mode() -> Vec<MotorCommand> {
    vec![
        MotorCommand::SetPanelVisibility {
            panel: PanelId::LeftSidebar,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::AudioPanel,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TrustDashboard,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::Proprioception,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::GraphStats,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: false,
        },
        MotorCommand::SetAwakening { enabled: false },
        MotorCommand::FitToView,
    ]
}

/// Full mode: restore all panels to default (backward compatible).
fn full_mode() -> Vec<MotorCommand> {
    developer_mode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clinical_disables_sidebars() {
        let cmds = commands_for_mode("clinical");
        assert!(!cmds.is_empty());
        let has_fit = cmds.iter().any(|c| matches!(c, MotorCommand::FitToView));
        assert!(has_fit, "clinical mode should include FitToView");
    }

    #[test]
    fn unknown_mode_is_noop() {
        let cmds = commands_for_mode("nonexistent");
        assert!(cmds.is_empty());
    }

    #[test]
    fn all_modes_produce_commands() {
        for mode in &["clinical", "developer", "presentation", "full"] {
            let cmds = commands_for_mode(mode);
            assert!(!cmds.is_empty(), "{mode} should produce commands");
        }
    }
}
