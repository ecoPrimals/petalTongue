// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unit tests for proprioception panel and helpers.

mod proprioception_tests {
    use super::super::*;
    use petal_tongue_core::{
        ProprioceptionData,
        proprioception::{HealthData, HealthStatus},
    };

    #[test]
    fn test_new_panel() {
        let panel = ProprioceptionPanel::new();
        assert!(panel.data.is_none());
        assert!(!panel.fetching);
    }

    #[test]
    fn test_panel_with_healthy_data() {
        let mut data = ProprioceptionData::empty("test");
        data.health.percentage = 95.0;
        data.confidence = 90.0;

        let mut panel = ProprioceptionPanel::new();
        panel.data = Some(data);

        assert!(panel.data.is_some());
        assert!(panel.data.as_ref().unwrap().is_healthy());
    }

    #[test]
    fn test_motor_history_recording() {
        let mut panel = ProprioceptionPanel::new();
        panel.record_motor_command("SetMode(clinical)");
        panel.record_motor_command("FitToView");
        assert_eq!(panel.motor_history.len(), 2);
    }

    #[test]
    fn test_motor_history_max_entries() {
        let mut panel = ProprioceptionPanel::new();
        for i in 0..25 {
            panel.record_motor_command(&format!("Command {i}"));
        }
        assert_eq!(panel.motor_history.len(), 20);
    }

    #[test]
    fn test_current_mode() {
        let mut panel = ProprioceptionPanel::new();
        assert_eq!(panel.current_mode, "default");
        panel.set_current_mode("clinical");
        assert_eq!(panel.current_mode, "clinical");
    }

    #[test]
    fn test_session_domain() {
        let mut panel = ProprioceptionPanel::new();
        assert!(panel.session_domain.is_none());
        panel.set_session_domain(Some("health".to_string()));
        assert_eq!(panel.session_domain.as_deref(), Some("health"));
    }

    #[test]
    fn test_merge_local_channels_empty_no_data() {
        let mut panel = ProprioceptionPanel::new();
        panel.merge_local_channels(vec![], vec![]);
        assert!(panel.data.is_none());
    }

    #[test]
    fn test_merge_local_channels_creates_data_when_none() {
        use petal_tongue_core::ChannelSnapshot;
        use petal_tongue_core::channel::{ChannelDirection, ChannelModality};

        let mut panel = ProprioceptionPanel::new();
        let afferent = vec![ChannelSnapshot {
            id: "ch1".to_string(),
            direction: ChannelDirection::Afferent,
            modality: ChannelModality::Ipc,
            active: true,
            signals_in: 10,
            signals_out: 5,
            throughput: 0.5,
            node_count: 2,
        }];
        panel.merge_local_channels(afferent, vec![]);

        assert!(panel.data.is_some());
        let data = panel.data.as_ref().unwrap();
        assert_eq!(data.afferent_channels.len(), 1);
        assert_eq!(data.afferent_channels[0].id, "ch1");
        assert!(data.efferent_channels.is_empty());
    }

    #[test]
    fn test_merge_local_channels_updates_existing() {
        use petal_tongue_core::ChannelSnapshot;
        use petal_tongue_core::channel::{ChannelDirection, ChannelModality};

        let mut panel = ProprioceptionPanel::new();
        let mut data = ProprioceptionData::empty("test");
        data.afferent_channels.push(ChannelSnapshot {
            id: "ch1".to_string(),
            direction: ChannelDirection::Afferent,
            modality: ChannelModality::Ipc,
            active: false,
            signals_in: 0,
            signals_out: 0,
            throughput: 0.0,
            node_count: 0,
        });
        panel.data = Some(data);

        panel.merge_local_channels(
            vec![ChannelSnapshot {
                id: "ch1".to_string(),
                direction: ChannelDirection::Afferent,
                modality: ChannelModality::Ipc,
                active: true,
                signals_in: 100,
                signals_out: 50,
                throughput: 0.8,
                node_count: 5,
            }],
            vec![],
        );

        let data = panel.data.as_ref().unwrap();
        assert_eq!(data.afferent_channels.len(), 1);
        assert!(data.afferent_channels[0].active);
        assert_eq!(data.afferent_channels[0].signals_in, 100);
    }

    #[test]
    fn test_panel_default() {
        let panel = ProprioceptionPanel::default();
        assert!(panel.data.is_none());
        assert_eq!(panel.current_mode, "default");
    }

    #[test]
    fn test_format_age_seconds() {
        assert_eq!(format_age_seconds(0), "0s ago");
        assert_eq!(format_age_seconds(30), "30s ago");
        assert_eq!(format_age_seconds(59), "59s ago");
        assert_eq!(format_age_seconds(60), "1m ago");
        assert_eq!(format_age_seconds(120), "2m ago");
        assert_eq!(format_age_seconds(90), "1m ago");
    }

    #[test]
    fn test_confidence_bar_color() {
        let green = egui::Color32::from_rgb(34, 197, 94);
        let yellow = egui::Color32::from_rgb(234, 179, 8);
        let red = egui::Color32::from_rgb(239, 68, 68);
        assert_eq!(confidence_bar_color(80.0), green);
        assert_eq!(confidence_bar_color(100.0), green);
        assert_eq!(confidence_bar_color(50.0), yellow);
        assert_eq!(confidence_bar_color(79.9), yellow);
        assert_eq!(confidence_bar_color(49.9), red);
        assert_eq!(confidence_bar_color(0.0), red);
    }

    #[test]
    fn test_evaluative_status_text() {
        assert_eq!(
            evaluative_status_text(true, true),
            "System is healthy and confident"
        );
        assert_eq!(
            evaluative_status_text(true, false),
            "System is healthy but low confidence"
        );
        assert_eq!(
            evaluative_status_text(false, true),
            "System is confident but degraded"
        );
        assert_eq!(
            evaluative_status_text(false, false),
            "System requires attention"
        );
    }

    #[test]
    fn render_shared_health_headless() {
        let health = HealthData {
            percentage: 85.0,
            status: HealthStatus::Healthy,
        };
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_shared_health(ui, &health);
            });
        });
    }

    #[test]
    fn render_shared_health_degraded() {
        let health = HealthData {
            percentage: 50.0,
            status: HealthStatus::Degraded,
        };
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_shared_health(ui, &health);
            });
        });
    }

    #[test]
    fn render_shared_same_dave_headless() {
        let mut data = ProprioceptionData::empty("test");
        data.confidence = 90.0;
        data.sensory.active_sockets = 5;
        data.self_awareness.knows_about = 3;
        data.self_awareness.has_security = true;
        data.self_awareness.has_discovery = true;
        data.self_awareness.has_compute = false;
        data.self_awareness.can_coordinate = true;
        data.motor.can_deploy = true;
        data.motor.can_execute_graphs = true;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_shared_same_dave(ui, &data);
            });
        });
    }

    #[test]
    fn render_shared_same_dave_all_capabilities_false() {
        let data = ProprioceptionData::empty("test");
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_shared_same_dave(ui, &data);
            });
        });
    }

    #[test]
    fn proprioception_panel_render_no_data() {
        let panel = ProprioceptionPanel::new();
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                panel.render(ui);
            });
        });
    }

    #[test]
    fn proprioception_panel_render_with_data() {
        let mut panel = ProprioceptionPanel::new();
        let mut data = ProprioceptionData::empty("test");
        data.health.percentage = 95.0;
        data.health.status = HealthStatus::Healthy;
        data.confidence = 88.0;
        data.sensory.active_sockets = 3;
        data.self_awareness.knows_about = 2;
        data.motor.can_deploy = true;
        panel.data = Some(data);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                panel.render(ui);
            });
        });
    }

    #[test]
    fn proprioception_panel_render_with_motor_history() {
        let mut panel = ProprioceptionPanel::new();
        panel.data = Some(ProprioceptionData::empty("test"));
        panel.record_motor_command("Deploy");
        panel.record_motor_command("FitToView");
        panel.set_current_mode("clinical");
        panel.set_session_domain(Some("health".to_string()));

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                panel.render(ui);
            });
        });
    }

    #[test]
    fn proprioception_panel_render_with_channels() {
        use petal_tongue_core::ChannelSnapshot;
        use petal_tongue_core::channel::{ChannelDirection, ChannelModality};

        let mut panel = ProprioceptionPanel::new();
        let mut data = ProprioceptionData::empty("test");
        data.afferent_channels.push(ChannelSnapshot {
            id: "keyboard-afferent".to_string(),
            direction: ChannelDirection::Afferent,
            modality: ChannelModality::Ipc,
            active: true,
            signals_in: 10,
            signals_out: 8,
            throughput: 0.8,
            node_count: 2,
        });
        data.efferent_channels.push(ChannelSnapshot {
            id: "visual-efferent".to_string(),
            direction: ChannelDirection::Efferent,
            modality: ChannelModality::Ipc,
            active: true,
            signals_in: 5,
            signals_out: 5,
            throughput: 1.0,
            node_count: 1,
        });
        panel.data = Some(data);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                panel.render(ui);
            });
        });
    }

    #[test]
    fn proprioception_panel_render_with_inactive_channel() {
        use petal_tongue_core::ChannelSnapshot;
        use petal_tongue_core::channel::{ChannelDirection, ChannelModality};

        let mut panel = ProprioceptionPanel::new();
        let mut data = ProprioceptionData::empty("test");
        data.afferent_channels.push(ChannelSnapshot {
            id: "ch-inactive".to_string(),
            direction: ChannelDirection::Afferent,
            modality: ChannelModality::Ipc,
            active: false,
            signals_in: 0,
            signals_out: 0,
            throughput: 0.0,
            node_count: 0,
        });
        panel.data = Some(data);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                panel.render(ui);
            });
        });
    }

    #[test]
    fn proprioception_panel_render_with_stale_data() {
        use chrono::Duration;

        let mut panel = ProprioceptionPanel::new();
        let mut data = ProprioceptionData::empty("test");
        data.timestamp = chrono::Utc::now() - Duration::seconds(120);
        panel.data = Some(data);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                panel.render(ui);
            });
        });
    }
}
