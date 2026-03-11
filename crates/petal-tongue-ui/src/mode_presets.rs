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
        "research" => research_mode(),
        "patient-facing" => patient_facing_mode(),
        _ => Vec::new(),
    }
}

/// Clinical mode: dashboard + trust + graph stats. Clean, health-focused.
fn clinical_mode() -> Vec<MotorCommand> {
    vec![
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::LeftSidebar,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::AudioPanel,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TrustDashboard,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::Proprioception,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::GraphStats,
            visible: true,
        },
        MotorCommand::SetAwakening { enabled: false },
        MotorCommand::FitToView,
    ]
}

/// Developer mode: everything visible — the power-user view.
fn developer_mode() -> Vec<MotorCommand> {
    vec![
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: true,
        },
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
    ]
}

/// Presentation mode: graph canvas only, minimal chrome for projection.
fn presentation_mode() -> Vec<MotorCommand> {
    vec![
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: false,
        },
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
        MotorCommand::SetAwakening { enabled: false },
        MotorCommand::FitToView,
    ]
}

/// Full mode: synonym for developer (backward compatible).
fn full_mode() -> Vec<MotorCommand> {
    developer_mode()
}

/// Research mode: data-analysis focus — proprioception, metrics, stats, trust, no audio or graph builder.
fn research_mode() -> Vec<MotorCommand> {
    vec![
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::LeftSidebar,
            visible: false,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::SystemDashboard,
            visible: true,
        },
        MotorCommand::SetPanelVisibility {
            panel: PanelId::AudioPanel,
            visible: false,
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
    ]
}

/// Patient-facing mode: minimal top menu, graph canvas only.
fn patient_facing_mode() -> Vec<MotorCommand> {
    vec![
        MotorCommand::SetPanelVisibility {
            panel: PanelId::TopMenu,
            visible: true,
        },
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
        MotorCommand::SetAwakening { enabled: false },
        MotorCommand::FitToView,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel_visible(cmds: &[MotorCommand], panel: &PanelId) -> Option<bool> {
        cmds.iter().find_map(|c| match c {
            MotorCommand::SetPanelVisibility { panel: p, visible } if p == panel => Some(*visible),
            _ => None,
        })
    }

    #[test]
    fn clinical_shows_dashboard_hides_audio() {
        let cmds = commands_for_mode("clinical");
        assert_eq!(panel_visible(&cmds, &PanelId::SystemDashboard), Some(true));
        assert_eq!(panel_visible(&cmds, &PanelId::TrustDashboard), Some(true));
        assert_eq!(panel_visible(&cmds, &PanelId::AudioPanel), Some(false));
        assert_eq!(panel_visible(&cmds, &PanelId::Proprioception), Some(false));
        assert!(cmds.iter().any(|c| matches!(c, MotorCommand::FitToView)));
    }

    #[test]
    fn developer_enables_everything() {
        let cmds = commands_for_mode("developer");
        assert_eq!(panel_visible(&cmds, &PanelId::Proprioception), Some(true));
        assert_eq!(panel_visible(&cmds, &PanelId::AudioPanel), Some(true));
        assert_eq!(panel_visible(&cmds, &PanelId::LeftSidebar), Some(true));
    }

    #[test]
    fn research_differs_from_developer() {
        let research = commands_for_mode("research");
        let developer = commands_for_mode("developer");
        assert_eq!(
            panel_visible(&research, &PanelId::Proprioception),
            Some(true)
        );
        assert_eq!(panel_visible(&research, &PanelId::AudioPanel), Some(false));
        assert_eq!(panel_visible(&developer, &PanelId::AudioPanel), Some(true));
    }

    #[test]
    fn presentation_hides_all_chrome() {
        let cmds = commands_for_mode("presentation");
        assert_eq!(panel_visible(&cmds, &PanelId::TopMenu), Some(false));
        assert_eq!(panel_visible(&cmds, &PanelId::LeftSidebar), Some(false));
        assert_eq!(panel_visible(&cmds, &PanelId::GraphStats), Some(false));
        assert!(cmds.iter().any(|c| matches!(c, MotorCommand::FitToView)));
    }

    #[test]
    fn patient_facing_shows_top_menu_only() {
        let cmds = commands_for_mode("patient-facing");
        assert_eq!(panel_visible(&cmds, &PanelId::TopMenu), Some(true));
        assert_eq!(panel_visible(&cmds, &PanelId::LeftSidebar), Some(false));
        assert_eq!(panel_visible(&cmds, &PanelId::AudioPanel), Some(false));
        assert_eq!(panel_visible(&cmds, &PanelId::Proprioception), Some(false));
    }

    #[test]
    fn full_is_synonym_for_developer() {
        let full = commands_for_mode("full");
        let dev = commands_for_mode("developer");
        assert_eq!(full.len(), dev.len());
    }

    #[test]
    fn unknown_mode_is_noop() {
        let cmds = commands_for_mode("nonexistent");
        assert!(cmds.is_empty());
    }

    #[test]
    fn all_modes_produce_commands() {
        for mode in &[
            "clinical",
            "developer",
            "presentation",
            "full",
            "research",
            "patient-facing",
        ] {
            let cmds = commands_for_mode(mode);
            assert!(!cmds.is_empty(), "{mode} should produce commands");
        }
    }
}
