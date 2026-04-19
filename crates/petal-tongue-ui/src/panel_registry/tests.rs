// SPDX-License-Identifier: AGPL-3.0-or-later
//! Test doubles and panel registry unit tests.

#[cfg(test)]
#[allow(clippy::module_inception, clippy::redundant_pub_crate)]
pub(crate) mod panel_test_support {
    use crate::scenario::CustomPanelConfig;

    use super::super::factory::{PanelFactory, PanelInstanceImpl};
    use super::super::types::{PanelError, PanelInstance, Result};

    pub struct MockPanel;

    impl PanelInstance for MockPanel {
        fn render(&mut self, ui: &mut egui::Ui) {
            ui.label("Mock Panel");
        }

        fn title(&self) -> &'static str {
            "Mock"
        }
    }

    pub struct MockPanelFactory;

    impl PanelFactory for MockPanelFactory {
        fn panel_type(&self) -> &'static str {
            "mock_panel"
        }

        fn create(&self, _config: &CustomPanelConfig) -> Result<PanelInstanceImpl> {
            Ok(PanelInstanceImpl::TestMock(MockPanel))
        }
    }

    pub struct FailingPanelFactory;

    impl PanelFactory for FailingPanelFactory {
        fn panel_type(&self) -> &'static str {
            "failing_panel"
        }

        fn create(&self, _config: &CustomPanelConfig) -> Result<PanelInstanceImpl> {
            Err(PanelError::CreationFailed("init failed".to_string()))
        }

        fn description(&self) -> &'static str {
            "Failing panel for tests"
        }
    }
}

#[cfg(test)]
mod panel_registry_tests {
    use super::panel_test_support::{FailingPanelFactory, MockPanelFactory};
    use crate::scenario::CustomPanelConfig;
    use std::sync::Arc;

    use super::super::factory::{PanelFactory, PanelFactoryImpl};
    use super::super::registry::PanelRegistry;
    use super::super::types::{PanelAction, PanelError, PanelInstance};

    #[test]
    fn test_panel_registration() {
        let mut registry = PanelRegistry::new();
        registry.register(Arc::new(PanelFactoryImpl::TestMock(MockPanelFactory)));

        assert!(registry.has_type("mock_panel"));
        assert!(!registry.has_type("unknown"));
    }

    #[test]
    fn test_panel_creation() {
        let mut registry = PanelRegistry::new();
        registry.register(Arc::new(PanelFactoryImpl::TestMock(MockPanelFactory)));

        let config = CustomPanelConfig {
            panel_type: "mock_panel".to_string(),
            title: "Test".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };

        let panel = registry
            .create(&config)
            .expect("mock_panel should be registered");
        assert_eq!(panel.title(), "Mock");
    }

    #[test]
    fn test_unknown_panel_type() {
        let registry = PanelRegistry::new();

        let config = CustomPanelConfig {
            panel_type: "unknown".to_string(),
            title: "Test".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };

        let result = registry.create(&config);
        match result {
            Err(PanelError::UnknownType(t)) => assert_eq!(t, "unknown"),
            Err(e) => panic!("expected UnknownType, got {:?}", e),
            Ok(_) => panic!("expected Err, got Ok"),
        }
    }

    #[test]
    fn test_available_types() {
        let mut registry = PanelRegistry::new();
        assert!(registry.available_types().is_empty());

        registry.register(Arc::new(PanelFactoryImpl::TestMock(MockPanelFactory)));
        let types = registry.available_types();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0], "mock_panel");
    }

    #[test]
    fn test_panel_registry_default() {
        let registry = PanelRegistry::default();
        assert!(registry.available_types().is_empty());
        assert!(!registry.has_type("mock_panel"));
    }

    #[test]
    fn test_register_overwrites_same_type() {
        let mut registry = PanelRegistry::new();
        registry.register(Arc::new(PanelFactoryImpl::TestMock(MockPanelFactory)));
        registry.register(Arc::new(PanelFactoryImpl::TestMock(MockPanelFactory)));
        let types = registry.available_types();
        assert_eq!(
            types.len(),
            1,
            "registering same type twice should overwrite"
        );
    }

    #[test]
    fn test_panel_action_variants() {
        assert_eq!(PanelAction::Continue, PanelAction::Continue);
        assert_eq!(PanelAction::Close, PanelAction::Close);
        assert_eq!(PanelAction::Restart, PanelAction::Restart);
        assert_ne!(PanelAction::Continue, PanelAction::Close);
    }

    #[test]
    fn test_panel_error_creation_failed() {
        let err = PanelError::CreationFailed("missing asset".to_string());
        assert!(matches!(err, PanelError::CreationFailed(_)));
        assert!(err.to_string().contains("creation failed"));
        assert!(err.to_string().contains("missing asset"));
    }

    #[test]
    fn test_panel_error_invalid_config() {
        let err = PanelError::InvalidConfig("missing title".to_string());
        assert!(matches!(err, PanelError::InvalidConfig(_)));
        assert!(err.to_string().contains("Invalid configuration"));
        assert!(err.to_string().contains("missing title"));
    }

    #[test]
    fn test_panel_factory_description_default() {
        assert_eq!(
            PanelFactoryImpl::TestMock(MockPanelFactory).description(),
            "Custom panel"
        );
    }

    #[test]
    fn test_panel_creation_failed() {
        let mut registry = PanelRegistry::new();
        registry.register(Arc::new(PanelFactoryImpl::TestFailing(FailingPanelFactory)));

        let config = CustomPanelConfig {
            panel_type: "failing_panel".to_string(),
            title: "Fail".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };

        let result = registry.create(&config);
        match result {
            Err(PanelError::CreationFailed(msg)) => assert!(msg.contains("init failed")),
            Err(e) => panic!("expected CreationFailed, got {e:?}"),
            Ok(_) => panic!("expected Err(CreationFailed), got Ok"),
        }
    }
}
