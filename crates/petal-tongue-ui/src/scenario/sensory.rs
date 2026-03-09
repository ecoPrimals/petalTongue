// SPDX-License-Identifier: AGPL-3.0-only
//! Sensory capability configuration for adaptive rendering
//!
//! Allows scenarios to define required and optional capabilities,
//! enabling cross-device support and graceful degradation.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Sensory capability configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensoryConfig {
    /// Required capabilities (scenario won't work without these)
    #[serde(default)]
    pub required_capabilities: CapabilityRequirements,

    /// Optional capabilities (enhanced experience if available)
    #[serde(default)]
    pub optional_capabilities: CapabilityRequirements,

    /// UI complexity hint ("auto", "minimal", "simple", "standard", "rich", "immersive")
    /// "auto" means detect based on discovered capabilities
    #[serde(default = "default_complexity_hint")]
    pub complexity_hint: String,
}

fn default_complexity_hint() -> String {
    "auto".to_string()
}

impl SensoryConfig {
    /// Validate sensory configuration
    pub fn validate(&self) -> Result<()> {
        // Validate complexity hint
        let valid_hints = ["auto", "minimal", "simple", "standard", "rich", "immersive"];
        if !valid_hints.contains(&self.complexity_hint.as_str()) {
            anyhow::bail!(
                "Invalid complexity_hint '{}'. Must be one of: {}",
                self.complexity_hint,
                valid_hints.join(", ")
            );
        }

        // Validate capability requirements
        self.required_capabilities.validate("required")?;
        self.optional_capabilities.validate("optional")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensory_config_deserialize_default_hint() {
        let c: SensoryConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(c.complexity_hint, "auto");
    }

    #[test]
    fn sensory_config_validate_invalid_hint_fails() {
        let c = SensoryConfig {
            required_capabilities: CapabilityRequirements::default(),
            optional_capabilities: CapabilityRequirements::default(),
            complexity_hint: "invalid".to_string(),
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn sensory_config_validate_valid_hints() {
        for hint in ["auto", "minimal", "simple", "standard", "rich", "immersive"] {
            let c = SensoryConfig {
                required_capabilities: CapabilityRequirements::default(),
                optional_capabilities: CapabilityRequirements::default(),
                complexity_hint: hint.to_string(),
            };
            assert!(c.validate().is_ok(), "hint {hint} should be valid");
        }
    }

    #[test]
    fn capability_requirements_invalid_output_fails() {
        let r = CapabilityRequirements {
            outputs: vec!["invalid".to_string()],
            inputs: vec![],
        };
        assert!(r.validate("required").is_err());
    }

    #[test]
    fn capability_requirements_invalid_input_fails() {
        let r = CapabilityRequirements {
            outputs: vec![],
            inputs: vec!["invalid".to_string()],
        };
        assert!(r.validate("optional").is_err());
    }

    #[test]
    fn capability_requirements_valid() {
        let r = CapabilityRequirements {
            outputs: vec!["visual".to_string(), "audio".to_string()],
            inputs: vec!["pointer".to_string(), "keyboard".to_string()],
        };
        assert!(r.validate("required").is_ok());
    }

    #[test]
    fn sensory_config_serialization() {
        let c = SensoryConfig {
            required_capabilities: CapabilityRequirements {
                outputs: vec!["visual".to_string()],
                inputs: vec![],
            },
            optional_capabilities: CapabilityRequirements::default(),
            complexity_hint: "simple".to_string(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: SensoryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.complexity_hint, "simple");
        assert_eq!(parsed.required_capabilities.outputs, vec!["visual"]);
    }
}

/// Capability requirements for a scenario
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityRequirements {
    /// Required/optional output modalities: "visual", "audio", "haptic"
    #[serde(default)]
    pub outputs: Vec<String>,

    /// Required/optional input modalities: "pointer", "keyboard", "touch", "gesture", "audio"
    #[serde(default)]
    pub inputs: Vec<String>,
}

impl CapabilityRequirements {
    /// Validate capability requirements
    pub fn validate(&self, context: &str) -> Result<()> {
        // Valid output modalities
        let valid_outputs = ["visual", "audio", "haptic"];
        for output in &self.outputs {
            if !valid_outputs.contains(&output.as_str()) {
                anyhow::bail!(
                    "Invalid {} output capability '{}'. Must be one of: {}",
                    context,
                    output,
                    valid_outputs.join(", ")
                );
            }
        }

        // Valid input modalities
        let valid_inputs = ["pointer", "keyboard", "touch", "gesture", "audio"];
        for input in &self.inputs {
            if !valid_inputs.contains(&input.as_str()) {
                anyhow::bail!(
                    "Invalid {} input capability '{}'. Must be one of: {}",
                    context,
                    input,
                    valid_inputs.join(", ")
                );
            }
        }

        Ok(())
    }
}
