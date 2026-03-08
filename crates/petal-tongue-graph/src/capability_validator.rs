// SPDX-License-Identifier: AGPL-3.0-only
//! Capability-based edge validation
//!
//! Validates connections between primals based on their capabilities,
//! following TRUE PRIMAL principles (no hardcoded types).

use petal_tongue_core::PrimalInfo;

/// Result of validating a potential edge connection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Connection is valid
    Valid,
    /// Connection is valid but with a warning
    Warning(String),
    /// Connection is invalid
    Invalid(String),
}

/// Validate if two primals can be connected based on their capabilities
///
/// # TRUE PRIMAL Design
///
/// This function does NOT check hardcoded primal types.
/// Instead, it validates based on discovered capabilities:
/// - Provider capabilities (what a primal offers)
/// - Consumer capabilities (what a primal requires)
/// - Bidirectional capabilities (coordination, discovery)
///
/// # Arguments
///
/// * `from` - Source primal
/// * `to` - Target primal
///
/// # Returns
///
/// Validation result indicating if connection is valid, with optional warnings
pub fn validate_connection(from: &PrimalInfo, to: &PrimalInfo) -> ValidationResult {
    // Allow self-loops for testing/debugging
    if from.id == to.id {
        return ValidationResult::Warning("Self-connection detected".to_string());
    }

    // Discover what capabilities the source provides
    let from_provides = discover_provided_capabilities(from);

    // Discover what capabilities the target requires
    let to_requires = discover_required_capabilities(to);

    // Check for capability matches
    let matches: Vec<&String> = from_provides
        .iter()
        .filter(|cap| to_requires.contains(cap))
        .collect();

    if matches.is_empty() {
        // Check for bidirectional capabilities (coordination, discovery)
        let from_coord = has_coordination_capability(from);
        let to_coord = has_coordination_capability(to);

        if from_coord || to_coord {
            // Coordination primals can connect to anything
            ValidationResult::Valid
        } else {
            // No clear capability match - allow but warn
            ValidationResult::Warning(format!(
                "No explicit capability match between '{}' and '{}'",
                from.name, to.name
            ))
        }
    } else {
        // Valid connection - source provides what target needs
        ValidationResult::Valid
    }
}

/// Discover what capabilities a primal provides to others
fn discover_provided_capabilities(primal: &PrimalInfo) -> Vec<String> {
    let mut provided = Vec::new();

    for capability in &primal.capabilities {
        let cap_lower = capability.to_lowercase();

        // Provider patterns (what this primal offers)
        if cap_lower.contains("provider")
            || cap_lower.contains("security")
            || cap_lower.contains("discovery")
            || cap_lower.contains("compute")
            || cap_lower.contains("storage")
            || cap_lower.contains("auth")
        {
            provided.push(capability.clone());
        }
    }

    provided
}

/// Discover what capabilities a primal requires from others
fn discover_required_capabilities(primal: &PrimalInfo) -> Vec<String> {
    let mut required = Vec::new();

    for capability in &primal.capabilities {
        let cap_lower = capability.to_lowercase();

        // Consumer patterns (what this primal needs)
        if cap_lower.contains("consumer")
            || cap_lower.contains("client")
            || cap_lower.contains("require")
        {
            required.push(capability.clone());
        }

        // Also match provider capabilities that this primal might consume
        // (e.g., a primal listing "security" might need to connect to security provider)
        if cap_lower.contains("security")
            || cap_lower.contains("auth")
            || cap_lower.contains("discovery")
        {
            required.push(capability.clone());
        }
    }

    required
}

/// Check if a primal has coordination capabilities
fn has_coordination_capability(primal: &PrimalInfo) -> bool {
    primal.capabilities.iter().any(|cap| {
        let cap_lower = cap.to_lowercase();
        cap_lower.contains("coordination")
            || cap_lower.contains("coordinate")
            || cap_lower.contains("orchestrate")
            || cap_lower.contains("nucleus")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::{PrimalHealthStatus, Properties};

    fn create_test_primal(id: &str, name: &str, capabilities: Vec<&str>) -> PrimalInfo {
        PrimalInfo {
            id: id.to_string(),
            name: name.to_string(),
            primal_type: "test".to_string(),
            endpoint: format!("test://{}", id),
            capabilities: capabilities
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            health: PrimalHealthStatus::Healthy,
            last_seen: 0,
            endpoints: None,
            metadata: None,
            properties: Properties::new(),
            trust_level: None,
            family_id: None,
        }
    }

    #[test]
    fn test_valid_connection_provider_to_consumer() {
        let provider = create_test_primal(
            "provider",
            "Security Provider",
            vec!["security-provider", "auth"],
        );
        let consumer = create_test_primal(
            "consumer",
            "Service Consumer",
            vec!["security-consumer", "api"],
        );

        // Should allow - but might warn since capabilities don't exactly match
        let result = validate_connection(&provider, &consumer);
        assert!(
            matches!(
                result,
                ValidationResult::Valid | ValidationResult::Warning(_)
            ),
            "Provider should connect to consumer"
        );
    }

    #[test]
    fn test_coordination_primal_connects_to_anything() {
        let nucleus = create_test_primal(
            "nucleus",
            "NUCLEUS",
            vec!["coordination", "graph-execution"],
        );
        let service = create_test_primal("service", "Random Service", vec!["api"]);

        let result = validate_connection(&nucleus, &service);
        assert_eq!(
            result,
            ValidationResult::Valid,
            "Coordination primal should connect to anything"
        );

        let result = validate_connection(&service, &nucleus);
        assert_eq!(
            result,
            ValidationResult::Valid,
            "Anything should connect to coordination primal"
        );
    }

    #[test]
    fn test_self_connection_warning() {
        let primal = create_test_primal("test", "Test", vec!["api"]);

        let result = validate_connection(&primal, &primal);
        assert!(
            matches!(result, ValidationResult::Warning(_)),
            "Self-connection should warn"
        );
    }

    #[test]
    fn test_capability_discovery() {
        let primal = create_test_primal(
            "test",
            "Test",
            vec!["security-provider", "compute", "api-consumer"],
        );

        let provided = discover_provided_capabilities(&primal);
        assert!(
            provided.contains(&"security-provider".to_string()),
            "Should discover security provider"
        );
        assert!(
            provided.contains(&"compute".to_string()),
            "Should discover compute capability"
        );

        let required = discover_required_capabilities(&primal);
        assert!(
            required.contains(&"api-consumer".to_string()),
            "Should discover consumer capability"
        );
    }

    #[test]
    fn test_no_match_gives_warning() {
        let primal1 = create_test_primal("p1", "Primal 1", vec!["custom-a"]);
        let primal2 = create_test_primal("p2", "Primal 2", vec!["custom-b"]);

        let result = validate_connection(&primal1, &primal2);
        assert!(
            matches!(result, ValidationResult::Warning(_)),
            "No capability match should give warning"
        );
    }
}
