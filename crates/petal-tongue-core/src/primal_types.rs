//! Primal Type System - Capability-Based
//!
//! This module provides capability-based primal identification.
//! **NEVER hardcode primal names in logic** - use capabilities instead.
//!
//! # Design Philosophy
//!
//! Primals are identified by **what they can do**, not **what they're named**.
//! Names are for display only. Capabilities are for logic.
//!
//! # Example
//!
//! ```rust,ignore
//! // ❌ BAD: Hardcoded name check
//! if primal.primal_type == "ToadStool" { ... }
//!
//! // ✅ GOOD: Capability-based check
//! if primal.has_capability("compute.container") { ... }
//! ```

use crate::PrimalInfo;

/// Capability categories for primals
///
/// These are the fundamental types of capabilities primals can have.
/// Any primal can implement any capability.
pub mod capability_categories {
    /// Compute capabilities (ToadStool-like)
    pub const COMPUTE: &[&str] = &[
        "compute.container",
        "compute.process",
        "compute.python",
        "compute.execute",
    ];

    /// Discovery/orchestration capabilities (Songbird-like)
    pub const DISCOVERY: &[&str] = &[
        "discovery.primals",
        "discovery.services",
        "orchestration.workflow",
        "orchestration.routing",
    ];

    /// Storage capabilities (NestGate-like)
    pub const STORAGE: &[&str] = &[
        "storage.filesystem",
        "storage.object",
        "storage.database",
        "storage.cache",
    ];

    /// Security capabilities (BearDog-like)
    pub const SECURITY: &[&str] = &[
        "security.auth",
        "security.signing",
        "security.encryption",
        "security.identity",
    ];

    /// AI/ML capabilities (Squirrel-like)
    pub const AI: &[&str] = &["ai.inference", "ai.training", "ai.nlp", "ai.vision"];
}

/// Primal capability queries
///
/// Extension trait for querying primal capabilities.
pub trait PrimalCapabilities {
    /// Check if primal has a specific capability
    fn has_capability(&self, capability: &str) -> bool;

    /// Check if primal has any capability in a category
    fn has_any_capability(&self, capabilities: &[&str]) -> bool;

    /// Check if primal provides compute capabilities
    fn is_compute_provider(&self) -> bool {
        self.has_any_capability(capability_categories::COMPUTE)
    }

    /// Check if primal provides discovery capabilities
    fn is_discovery_provider(&self) -> bool {
        self.has_any_capability(capability_categories::DISCOVERY)
    }

    /// Check if primal provides storage capabilities
    fn is_storage_provider(&self) -> bool {
        self.has_any_capability(capability_categories::STORAGE)
    }

    /// Check if primal provides security capabilities
    fn is_security_provider(&self) -> bool {
        self.has_any_capability(capability_categories::SECURITY)
    }

    /// Check if primal provides AI capabilities
    fn is_ai_provider(&self) -> bool {
        self.has_any_capability(capability_categories::AI)
    }

    /// Get display name for primal type
    ///
    /// Returns the `primal_type` field. **For display only, never for logic.**
    fn display_type(&self) -> &str;
}

impl PrimalCapabilities for PrimalInfo {
    fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    fn has_any_capability(&self, capabilities: &[&str]) -> bool {
        capabilities.iter().any(|cap| self.has_capability(cap))
    }

    fn display_type(&self) -> &str {
        &self.primal_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PrimalHealthStatus;

    fn test_primal(capabilities: Vec<String>) -> PrimalInfo {
        PrimalInfo {
            id: "test".to_string(),
            name: "Test Primal".to_string(),
            primal_type: "TestType".to_string(),
            endpoint: "http://test:8000".to_string(),
            health: PrimalHealthStatus::Healthy,
            trust_level: None,
            family_id: None,
            capabilities,
            last_seen: 0, // Unix timestamp
        }
    }

    #[test]
    fn test_compute_detection() {
        let primal = test_primal(vec!["compute.container".to_string()]);
        assert!(primal.is_compute_provider());
        assert!(!primal.is_storage_provider());
    }

    #[test]
    fn test_discovery_detection() {
        let primal = test_primal(vec!["discovery.primals".to_string()]);
        assert!(primal.is_discovery_provider());
        assert!(!primal.is_compute_provider());
    }

    #[test]
    fn test_storage_detection() {
        let primal = test_primal(vec!["storage.filesystem".to_string()]);
        assert!(primal.is_storage_provider());
        assert!(!primal.is_discovery_provider());
    }

    #[test]
    fn test_multiple_capabilities() {
        let primal = test_primal(vec![
            "compute.container".to_string(),
            "storage.cache".to_string(),
        ]);
        assert!(primal.is_compute_provider());
        assert!(primal.is_storage_provider());
    }

    #[test]
    fn test_display_type_for_display_only() {
        let primal = test_primal(vec![]);
        // primal_type field exists for display/logging
        assert_eq!(primal.display_type(), "TestType");
        // But never use it for logic decisions
    }
}
