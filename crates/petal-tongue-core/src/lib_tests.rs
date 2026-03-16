// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
mod tests {
    use crate::{PetalTongue, PetalTongueConfig, PrimalHealth, PrimalLifecycle, PrimalState};

    #[test]
    fn petal_tongue_new() {
        let config = PetalTongueConfig::default();
        let pt = PetalTongue::new(config.clone());
        assert_eq!(pt.config().common.name, config.common.name);
    }

    #[test]
    fn petal_tongue_config() {
        let config = PetalTongueConfig::default();
        let pt = PetalTongue::new(config);
        let cfg = pt.config();
        assert_eq!(cfg.refresh_interval_secs, 5);
    }

    #[test]
    fn petal_tongue_state_created() {
        let config = PetalTongueConfig::default();
        let pt = PetalTongue::new(config);
        assert_eq!(pt.state(), PrimalState::Created);
    }

    #[test]
    fn petal_tongue_health_status_unhealthy_when_created() {
        let config = PetalTongueConfig::default();
        let pt = PetalTongue::new(config);
        let status = pt.health_status();
        assert!(!status.is_healthy());
        let status_debug = format!("{status:?}");
        assert!(status_debug.contains("state"));
    }

    #[tokio::test]
    async fn petal_tongue_start_stop() {
        let config = PetalTongueConfig::default();
        let mut pt = PetalTongue::new(config);
        pt.start().await.unwrap();
        assert_eq!(pt.state(), PrimalState::Running);
        assert!(pt.health_status().is_healthy());
        pt.stop().await.unwrap();
        assert_eq!(pt.state(), PrimalState::Stopped);
    }

    #[tokio::test]
    async fn petal_tongue_health_check_running() {
        let config = PetalTongueConfig::default();
        let mut pt = PetalTongue::new(config);
        pt.start().await.unwrap();
        let report = pt.health_check().await.unwrap();
        assert_eq!(report.name, "petalTongue");
        assert!(report.status.is_healthy());
    }

    #[tokio::test]
    async fn petal_tongue_health_check_created() {
        let config = PetalTongueConfig::default();
        let pt = PetalTongue::new(config);
        let report = pt.health_check().await.unwrap();
        assert_eq!(report.name, "petalTongue");
        assert!(!report.status.is_healthy());
    }
}
