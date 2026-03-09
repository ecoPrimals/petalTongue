// SPDX-License-Identifier: AGPL-3.0-only
//! Niche Designer - Unit tests

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::biomeos_integration::Health;
use crate::ui_events::UIEventHandler;

use crate::biomeos_integration::{NicheTemplate, Primal};

use super::state::NicheDesigner;
use super::types::ValidationResult;

#[tokio::test]
async fn test_niche_designer_creation() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let designer = NicheDesigner::new(event_handler);

    assert_eq!(designer.templates.len(), 0);
    assert!(designer.selected_template.is_none());
    assert_eq!(designer.validation, ValidationResult::Valid);
}

#[tokio::test]
async fn test_niche_designer_refresh() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut designer = NicheDesigner::new(event_handler);

    let templates = vec![NicheTemplate {
        id: "template-1".to_string(),
        name: "Test Template".to_string(),
        description: "Test".to_string(),
        required_primals: vec!["compute".to_string()],
        optional_primals: vec![],
        metadata: serde_json::json!({}),
    }];

    let primals = vec![Primal {
        id: "primal-1".to_string(),
        name: "Test Primal".to_string(),
        health: Health::Healthy,
        capabilities: vec!["compute".to_string()],
        load: 0.5,
        assigned_devices: vec![],
        metadata: serde_json::json!({}),
    }];

    designer.refresh(templates, primals).await;

    assert_eq!(designer.templates.len(), 1);
    assert_eq!(designer.available_primals.len(), 1);
}

#[tokio::test]
async fn test_template_selection() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut designer = NicheDesigner::new(event_handler);

    let template = NicheTemplate {
        id: "template-1".to_string(),
        name: "Test Template".to_string(),
        description: "Test".to_string(),
        required_primals: vec!["compute".to_string()],
        optional_primals: vec![],
        metadata: serde_json::json!({}),
    };

    designer.select_template(template.clone());

    assert!(designer.selected_template.is_some());
    assert_eq!(designer.selected_template.unwrap().id, "template-1");
    assert_eq!(designer.assigned_primals.len(), 0);
}

#[tokio::test]
async fn test_primal_assignment() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut designer = NicheDesigner::new(event_handler);

    let template = NicheTemplate {
        id: "template-1".to_string(),
        name: "Test Template".to_string(),
        description: "Test".to_string(),
        required_primals: vec!["compute".to_string()],
        optional_primals: vec![],
        metadata: serde_json::json!({}),
    };

    let primal = Primal {
        id: "primal-1".to_string(),
        name: "Test Primal".to_string(),
        health: Health::Healthy,
        capabilities: vec!["compute".to_string()],
        load: 0.5,
        assigned_devices: vec![],
        metadata: serde_json::json!({}),
    };

    designer.refresh(vec![template.clone()], vec![primal]).await;
    designer.select_template(template);
    designer.assign_primal("compute".to_string(), "primal-1".to_string());

    assert_eq!(designer.assigned_primals.len(), 1);
    assert_eq!(
        designer.assigned_primals.get("compute"),
        Some(&"primal-1".to_string())
    );
}

#[tokio::test]
async fn test_validation_missing_requirements() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut designer = NicheDesigner::new(event_handler);

    let template = NicheTemplate {
        id: "template-1".to_string(),
        name: "Test Template".to_string(),
        description: "Test".to_string(),
        required_primals: vec!["compute".to_string(), "storage".to_string()],
        optional_primals: vec![],
        metadata: serde_json::json!({}),
    };

    designer.select_template(template);

    // No primals assigned - should fail validation
    match designer.validation {
        ValidationResult::MissingRequirements(ref missing) => {
            assert_eq!(missing.len(), 2);
            assert!(missing.contains(&"compute".to_string()));
            assert!(missing.contains(&"storage".to_string()));
        }
        _ => panic!("Expected MissingRequirements validation result"),
    }
}

#[tokio::test]
async fn test_validation_success() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut designer = NicheDesigner::new(event_handler);

    let template = NicheTemplate {
        id: "template-1".to_string(),
        name: "Test Template".to_string(),
        description: "Test".to_string(),
        required_primals: vec!["compute".to_string()],
        optional_primals: vec![],
        metadata: serde_json::json!({}),
    };

    let primal = Primal {
        id: "primal-1".to_string(),
        name: "Test Primal".to_string(),
        health: Health::Healthy,
        capabilities: vec!["compute".to_string()],
        load: 0.5,
        assigned_devices: vec![],
        metadata: serde_json::json!({}),
    };

    designer.refresh(vec![template.clone()], vec![primal]).await;
    designer.select_template(template);
    designer.assign_primal("compute".to_string(), "primal-1".to_string());

    assert_eq!(designer.validation, ValidationResult::Valid);
}

#[tokio::test]
async fn test_primal_unassignment() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut designer = NicheDesigner::new(event_handler);

    let template = NicheTemplate {
        id: "template-1".to_string(),
        name: "Test Template".to_string(),
        description: "Test".to_string(),
        required_primals: vec!["compute".to_string()],
        optional_primals: vec![],
        metadata: serde_json::json!({}),
    };

    let primal = Primal {
        id: "primal-1".to_string(),
        name: "Test Primal".to_string(),
        health: Health::Healthy,
        capabilities: vec!["compute".to_string()],
        load: 0.5,
        assigned_devices: vec![],
        metadata: serde_json::json!({}),
    };

    designer.refresh(vec![template.clone()], vec![primal]).await;
    designer.select_template(template);
    designer.assign_primal("compute".to_string(), "primal-1".to_string());

    assert_eq!(designer.assigned_primals.len(), 1);

    designer.unassign_primal("compute");

    assert_eq!(designer.assigned_primals.len(), 0);
    assert_ne!(designer.validation, ValidationResult::Valid);
}

#[tokio::test]
async fn test_capability_mismatch() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut designer = NicheDesigner::new(event_handler);

    let primal = Primal {
        id: "primal-1".to_string(),
        name: "Compute Primal".to_string(),
        health: Health::Healthy,
        capabilities: vec!["compute".to_string()], // Only has compute
        load: 0.5,
        assigned_devices: vec![],
        metadata: serde_json::json!({}),
    };

    designer.refresh(vec![], vec![primal]).await;

    // Try to assign to storage capability (which the primal doesn't have)
    designer.assign_primal("storage".to_string(), "primal-1".to_string());

    // Should not be assigned
    assert_eq!(designer.assigned_primals.len(), 0);
}

#[tokio::test]
async fn test_optional_primals() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut designer = NicheDesigner::new(event_handler);

    let template = NicheTemplate {
        id: "template-1".to_string(),
        name: "Test Template".to_string(),
        description: "Test".to_string(),
        required_primals: vec!["compute".to_string()],
        optional_primals: vec!["storage".to_string()],
        metadata: serde_json::json!({}),
    };

    let compute_primal = Primal {
        id: "primal-1".to_string(),
        name: "Compute Primal".to_string(),
        health: Health::Healthy,
        capabilities: vec!["compute".to_string()],
        load: 0.5,
        assigned_devices: vec![],
        metadata: serde_json::json!({}),
    };

    designer
        .refresh(vec![template.clone()], vec![compute_primal])
        .await;
    designer.select_template(template);
    designer.assign_primal("compute".to_string(), "primal-1".to_string());

    // Should be valid even without optional primal
    assert_eq!(designer.validation, ValidationResult::Valid);
}

#[tokio::test]
async fn test_multiple_assignments() {
    let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
    let mut designer = NicheDesigner::new(event_handler);

    let template = NicheTemplate {
        id: "template-1".to_string(),
        name: "Test Template".to_string(),
        description: "Test".to_string(),
        required_primals: vec!["compute".to_string(), "storage".to_string()],
        optional_primals: vec![],
        metadata: serde_json::json!({}),
    };

    let compute_primal = Primal {
        id: "primal-1".to_string(),
        name: "Compute Primal".to_string(),
        health: Health::Healthy,
        capabilities: vec!["compute".to_string()],
        load: 0.5,
        assigned_devices: vec![],
        metadata: serde_json::json!({}),
    };

    let storage_primal = Primal {
        id: "primal-2".to_string(),
        name: "Storage Primal".to_string(),
        health: Health::Healthy,
        capabilities: vec!["storage".to_string()],
        load: 0.3,
        assigned_devices: vec![],
        metadata: serde_json::json!({}),
    };

    designer
        .refresh(vec![template.clone()], vec![compute_primal, storage_primal])
        .await;
    designer.select_template(template);
    designer.assign_primal("compute".to_string(), "primal-1".to_string());
    designer.assign_primal("storage".to_string(), "primal-2".to_string());

    assert_eq!(designer.assigned_primals.len(), 2);
    assert_eq!(designer.validation, ValidationResult::Valid);
}
