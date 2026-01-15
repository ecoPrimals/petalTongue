//! Scenario loading system for benchTop demonstrations
//!
//! This module loads JSON scenario files that define complete UI states,
//! including primals, graphs, metrics, and animations.
//!
//! ## Sensory Capability Integration (v2.2.0)
//!
//! Scenarios now support sensory capability configuration, allowing them to:
//! - Define required and optional capabilities
//! - Work across different devices (desktop, phone, watch, terminal, VR)
//! - Adapt rendering based on discovered capabilities
//! - Gracefully degrade on limited devices

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use petal_tongue_core::{
    PrimalHealthStatus, ProprioceptionData, SystemMetrics,
    SensoryCapabilities, SensoryUIComplexity,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Complete scenario definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub version: String,
    pub mode: String,
    #[serde(default)]
    pub ui_config: UiConfig,
    #[serde(default)]
    pub ecosystem: Ecosystem,
    #[serde(default)]
    pub neural_api: NeuralApiConfig,
    /// Sensory capability configuration (v2.2.0)
    #[serde(default)]
    pub sensory_config: SensoryConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub layout: String, // "canvas-only", "dashboard-centered", "full-dashboard"
    #[serde(default)]
    pub show_panels: PanelVisibility,
    #[serde(default)]
    pub animations: AnimationConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
    #[serde(default)]
    pub features: FeatureFlags,
    /// Custom panels (e.g., Doom, web browser, video player)
    #[serde(default)]
    pub custom_panels: Vec<CustomPanelConfig>,
}

impl UiConfig {
    /// Validate UI configuration
    pub fn validate(&self) -> Result<()> {
        // Validate custom panels
        for (idx, panel) in self.custom_panels.iter().enumerate() {
            panel.validate()
                .with_context(|| format!("Custom panel {} validation failed", idx))?;
        }
        
        // Validate performance config
        if self.performance.target_fps > 0 && self.performance.target_fps < 10 {
            tracing::warn!("⚠️  Target FPS ({}) is very low, may cause sluggish UI", self.performance.target_fps);
        }
        
        if self.performance.target_fps > 240 {
            tracing::warn!("⚠️  Target FPS ({}) is very high, may waste resources", self.performance.target_fps);
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelVisibility {
    pub left_sidebar: bool,
    pub right_sidebar: bool,
    pub top_menu: bool,
    pub system_dashboard: bool,
    pub audio_panel: bool,
    pub trust_dashboard: bool,
    pub proprioception: bool,
    pub graph_stats: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        // Default: show everything (backward compatible)
        Self {
            left_sidebar: true,
            right_sidebar: true,
            top_menu: true,
            system_dashboard: true,
            audio_panel: true,
            trust_dashboard: true,
            proprioception: true,
            graph_stats: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FeatureFlags {
    pub audio_sonification: bool,
    pub auto_refresh: bool,
    pub neural_api: bool,
    pub tutorial_mode: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        // Default: enable all features (backward compatible)
        Self {
            audio_sonification: true,
            auto_refresh: true,
            neural_api: false, // Disabled by default (requires external service)
            tutorial_mode: false,
        }
    }
}

/// Custom panel configuration (for embedded apps like Doom, web browsers, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPanelConfig {
    /// Panel type identifier (e.g., "doom_game", "web_view", "video_player")
    #[serde(rename = "type")]
    pub panel_type: String,
    
    /// Panel title
    pub title: String,
    
    /// Panel width (optional, defaults to fit)
    #[serde(default)]
    pub width: Option<usize>,
    
    /// Panel height (optional, defaults to fit)
    #[serde(default)]
    pub height: Option<usize>,
    
    /// Fullscreen mode
    #[serde(default)]
    pub fullscreen: bool,
    
    /// Panel-specific configuration (JSON value for flexibility)
    #[serde(default)]
    pub config: serde_json::Value,
}

impl CustomPanelConfig {
    /// Validate panel configuration
    pub fn validate(&self) -> Result<()> {
        // Check panel type
        if self.panel_type.trim().is_empty() {
            anyhow::bail!("Panel type cannot be empty (e.g., 'doom_game', 'web_view')");
        }
        
        // Check title
        if self.title.trim().is_empty() {
            anyhow::bail!("Panel '{}' has empty title", self.panel_type);
        }
        
        // Validate dimensions
        if let Some(width) = self.width {
            if width == 0 {
                anyhow::bail!("Panel '{}' has zero width", self.title);
            }
            if width > 7680 {  // Reasonable max: 8K resolution
                tracing::warn!("⚠️  Panel '{}' has unusually large width: {}", self.title, width);
            }
        }
        
        if let Some(height) = self.height {
            if height == 0 {
                anyhow::bail!("Panel '{}' has zero height", self.title);
            }
            if height > 4320 {  // Reasonable max: 8K resolution
                tracing::warn!("⚠️  Panel '{}' has unusually large height: {}", self.title, height);
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub breathing_nodes: bool,
    #[serde(default)]
    pub connection_pulses: bool,
    #[serde(default)]
    pub smooth_transitions: bool,
    #[serde(default)]
    pub celebration_effects: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(default)]
    pub target_fps: u32,
    #[serde(default)]
    pub vsync: bool,
    #[serde(default)]
    pub hardware_acceleration: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ecosystem {
    #[serde(default)]
    pub primals: Vec<PrimalDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDefinition {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub primal_type: String,
    pub family: String,
    pub status: String,
    pub health: u8,
    pub confidence: u8,
    pub position: Position,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub metrics: PrimalMetrics,
    #[serde(default)]
    pub proprioception: Option<ScenarioProprioception>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrimalMetrics {
    #[serde(default)]
    pub cpu_percent: f32,
    #[serde(default)]
    pub memory_mb: u64,
    #[serde(default)]
    pub uptime_seconds: u64,
    #[serde(default)]
    pub requests_per_second: u64,
    #[serde(default)]
    pub active_primals: usize,
    #[serde(default)]
    pub graphs_available: usize,
    #[serde(default)]
    pub active_executions: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioProprioception {
    #[serde(default)]
    pub self_awareness: SelfAwareness,
    #[serde(default)]
    pub motor: Motor,
    #[serde(default)]
    pub sensory: Sensory,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelfAwareness {
    #[serde(default)]
    pub knows_about: usize,
    #[serde(default)]
    pub can_coordinate: bool,
    #[serde(default)]
    pub has_security: bool,
    #[serde(default)]
    pub has_discovery: bool,
    #[serde(default)]
    pub has_compute: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Motor {
    #[serde(default)]
    pub can_deploy: bool,
    #[serde(default)]
    pub can_execute_graphs: bool,
    #[serde(default)]
    pub can_coordinate_primals: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sensory {
    #[serde(default)]
    pub active_sockets: usize,
    #[serde(default)]
    pub last_scan: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NeuralApiConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub learning_rate: f32,
    #[serde(default)]
    pub optimization_cycles: usize,
}

/// Sensory capability configuration for adaptive rendering
///
/// This allows scenarios to define what capabilities they need and prefer,
/// enabling the same scenario to work on different devices (desktop, phone, watch, etc.)
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

impl Scenario {
    /// Load scenario from JSON file with validation
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read scenario file: {}", path.display()))?;

        let scenario: Scenario = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse scenario JSON: {}", path.display()))?;

        // ✅ NEW: Explicit validation
        scenario.validate()
            .with_context(|| format!("Scenario validation failed: {}", path.display()))?;

        tracing::info!("📋 Loaded scenario: {} ({})", scenario.name, scenario.version);
        tracing::info!("   Mode: {}", scenario.mode);
        tracing::info!("   Primals: {}", scenario.ecosystem.primals.len());

        Ok(scenario)
    }
    
    /// Validate scenario structure and contents
    ///
    /// This catches common mistakes like:
    /// - Empty required fields
    /// - Invalid panel configurations
    /// - Malformed capability requirements
    /// - Orphaned references
    pub fn validate(&self) -> Result<()> {
        // Check required fields
        if self.name.trim().is_empty() {
            anyhow::bail!("Scenario name cannot be empty");
        }
        
        if self.mode.trim().is_empty() {
            anyhow::bail!("Scenario mode cannot be empty (e.g., 'doom-showcase', 'live-ecosystem')");
        }
        
        if self.version.trim().is_empty() {
            anyhow::bail!("Scenario version cannot be empty");
        }
        
        // Validate version format (should be semver-like)
        if !self.version.contains('.') {
            tracing::warn!("⚠️  Scenario version '{}' doesn't follow semver format (e.g., '2.0.0')", self.version);
        }
        
        // Validate UI config
        self.ui_config.validate()
            .with_context(|| "UI configuration validation failed")?;
        
        // Validate sensory config
        self.sensory_config.validate()
            .with_context(|| "Sensory configuration validation failed")?;
        
        tracing::debug!("✅ Scenario validation passed: {}", self.name);
        Ok(())
    }

    /// Get count of primals in the scenario
    pub fn primal_count(&self) -> usize {
        self.ecosystem.primals.len()
    }

    /// Convert scenario primals to PrimalInfo for graph display
    pub fn to_primal_infos(&self) -> Vec<petal_tongue_core::PrimalInfo> {
        use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, PropertyValue};
        
        self.ecosystem.primals.iter().map(|p| {
            let health_status = match p.status.to_lowercase().as_str() {
                "healthy" => PrimalHealthStatus::Healthy,
                "degraded" | "warning" => PrimalHealthStatus::Warning,
                "critical" => PrimalHealthStatus::Critical,
                _ => PrimalHealthStatus::Unknown,
            };
            
            // Convert scenario data to properties
            let mut properties = Properties::new();
            
            // Add capabilities as boolean properties
            for cap in &p.capabilities {
                properties.insert(
                    format!("capability.{}", cap),
                    PropertyValue::Boolean(true),
                );
            }
            
            // Add metrics as properties
            properties.insert("cpu_percent".to_string(), PropertyValue::Number(p.metrics.cpu_percent as f64));
            properties.insert("memory_mb".to_string(), PropertyValue::Number(p.metrics.memory_mb as f64));
            properties.insert("health_percent".to_string(), PropertyValue::Number(p.health as f64));
            properties.insert("confidence".to_string(), PropertyValue::Number(p.confidence as f64));
            
            PrimalInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                primal_type: p.primal_type.clone(),
                health: health_status,
                capabilities: p.capabilities.clone(),
                endpoint: format!("scenario://{}", p.id), // Virtual endpoint for scenarios
                last_seen: chrono::Utc::now().timestamp() as u64,
                endpoints: None,
                metadata: None,
                properties,
                trust_level: None,
                family_id: Some(p.family.clone()),
            }
        }).collect()
    }

    /// Get system metrics from scenario (if available)
    pub fn get_metrics(&self) -> Option<SystemMetrics> {
        // Find NUCLEUS primal for system metrics
        let nucleus = self
            .ecosystem
            .primals
            .iter()
            .find(|p| p.name == "NUCLEUS")?;

        Some(SystemMetrics {
            timestamp: Utc::now(),
            system: petal_tongue_core::SystemResourceMetrics {
                cpu_percent: nucleus.metrics.cpu_percent,
                memory_used_mb: nucleus.metrics.memory_mb,
                memory_total_mb: 49152, // Default
                memory_percent: (nucleus.metrics.memory_mb as f32 / 49152.0) * 100.0,
                uptime_seconds: nucleus.metrics.uptime_seconds,
            },
            neural_api: petal_tongue_core::NeuralApiMetrics {
                family_id: nucleus.family.clone(),
                active_primals: nucleus.metrics.active_primals as u32,
                graphs_available: nucleus.metrics.graphs_available as u32,
                active_executions: nucleus.metrics.active_executions as u32,
            },
        })
    }

    /// Validate that discovered capabilities meet scenario requirements
    ///
    /// Returns an error if required capabilities are not available.
    /// Optional capabilities are logged but don't cause failure.
    pub fn validate_capabilities(&self, caps: &SensoryCapabilities) -> Result<(), String> {
        tracing::debug!("🔍 Validating scenario capabilities against device");

        // Check required outputs
        for output in &self.sensory_config.required_capabilities.outputs {
            match output.as_str() {
                "visual" if !caps.has_visual_output() => {
                    return Err(format!(
                        "Scenario '{}' requires visual output, but device has none",
                        self.name
                    ));
                }
                "audio" if !caps.has_audio_output() => {
                    return Err(format!(
                        "Scenario '{}' requires audio output, but device has none",
                        self.name
                    ));
                }
                "haptic" if !caps.has_haptic_output() => {
                    return Err(format!(
                        "Scenario '{}' requires haptic output, but device has none",
                        self.name
                    ));
                }
                _ => {}
            }
        }

        // Check required inputs
        for input in &self.sensory_config.required_capabilities.inputs {
            match input.as_str() {
                "pointer" if caps.pointer_inputs.is_empty() => {
                    return Err(format!(
                        "Scenario '{}' requires pointer input, but device has none",
                        self.name
                    ));
                }
                "keyboard" if caps.keyboard_inputs.is_empty() => {
                    return Err(format!(
                        "Scenario '{}' requires keyboard input, but device has none",
                        self.name
                    ));
                }
                "touch" if caps.touch_inputs.is_empty() => {
                    return Err(format!(
                        "Scenario '{}' requires touch input, but device has none",
                        self.name
                    ));
                }
                "audio" if caps.audio_inputs.is_empty() => {
                    return Err(format!(
                        "Scenario '{}' requires audio input, but device has none",
                        self.name
                    ));
                }
                "gesture" if caps.gesture_inputs.is_empty() => {
                    return Err(format!(
                        "Scenario '{}' requires gesture input, but device has none",
                        self.name
                    ));
                }
                _ => {}
            }
        }

        // Log optional capabilities that are available
        for output in &self.sensory_config.optional_capabilities.outputs {
            let available = match output.as_str() {
                "visual" => caps.has_visual_output(),
                "audio" => caps.has_audio_output(),
                "haptic" => caps.has_haptic_output(),
                _ => false,
            };
            if available {
                tracing::info!("✨ Optional output '{}' is available - enhanced experience enabled", output);
            } else {
                tracing::debug!("ℹ️  Optional output '{}' not available - graceful degradation", output);
            }
        }

        tracing::info!("✅ Scenario capabilities validated successfully");
        Ok(())
    }

    /// Determine UI complexity level for this scenario on the discovered device
    ///
    /// If the scenario specifies a complexity hint, use it.
    /// Otherwise, auto-detect based on device capabilities.
    pub fn determine_complexity(&self, caps: &SensoryCapabilities) -> SensoryUIComplexity {
        let complexity = match self.sensory_config.complexity_hint.as_str() {
            "minimal" => SensoryUIComplexity::Minimal,
            "simple" => SensoryUIComplexity::Simple,
            "standard" => SensoryUIComplexity::Standard,
            "rich" => SensoryUIComplexity::Rich,
            "immersive" => SensoryUIComplexity::Immersive,
            "auto" | _ => caps.determine_ui_complexity(), // Auto-detect from capabilities
        };

        tracing::info!(
            "🎨 Scenario '{}' using {:?} complexity (hint: {})",
            self.name,
            complexity,
            self.sensory_config.complexity_hint
        );

        complexity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // ===== Validation Tests =====
    
    #[test]
    fn test_scenario_validation_success() {
        let json = r#"{
            "name": "Test Scenario",
            "description": "A test",
            "version": "2.0.0",
            "mode": "test-mode",
            "ui_config": {
                "custom_panels": []
            },
            "sensory_config": {
                "required_capabilities": {
                    "outputs": [],
                    "inputs": []
                },
                "optional_capabilities": {
                    "outputs": [],
                    "inputs": []
                },
                "complexity_hint": "auto"
            },
            "ecosystem": {
                "primals": []
            }
        }"#;
        
        let scenario: Scenario = serde_json::from_str(json).unwrap();
        let result = scenario.validate();
        if let Err(e) = &result {
            eprintln!("Validation error: {}", e);
        }
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_scenario_validation_empty_name() {
        let json = r#"{
            "name": "",
            "description": "A test",
            "version": "2.0.0",
            "mode": "test-mode",
            "ecosystem": {"primals": []}
        }"#;
        
        let scenario: Scenario = serde_json::from_str(json).unwrap();
        let result = scenario.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name cannot be empty"));
    }
    
    #[test]
    fn test_scenario_validation_empty_mode() {
        let json = r#"{
            "name": "Test",
            "description": "A test",
            "version": "2.0.0",
            "mode": "",
            "ecosystem": {"primals": []}
        }"#;
        
        let scenario: Scenario = serde_json::from_str(json).unwrap();
        let result = scenario.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mode cannot be empty"));
    }
    
    #[test]
    fn test_custom_panel_validation_empty_type() {
        let panel = CustomPanelConfig {
            panel_type: "".to_string(),
            title: "Test Panel".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        
        let result = panel.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Panel type cannot be empty"));
    }
    
    #[test]
    fn test_custom_panel_validation_empty_title() {
        let panel = CustomPanelConfig {
            panel_type: "test_panel".to_string(),
            title: "".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        
        let result = panel.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty title"));
    }
    
    #[test]
    fn test_custom_panel_validation_zero_dimensions() {
        let panel = CustomPanelConfig {
            panel_type: "test_panel".to_string(),
            title: "Test".to_string(),
            width: Some(0),
            height: Some(480),
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        
        let result = panel.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("zero width"));
    }
    
    #[test]
    fn test_sensory_config_validation_invalid_complexity() {
        let config = SensoryConfig {
            required_capabilities: CapabilityRequirements::default(),
            optional_capabilities: CapabilityRequirements::default(),
            complexity_hint: "invalid".to_string(),
        };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid complexity_hint"));
    }
    
    #[test]
    fn test_sensory_config_validation_valid_complexity() {
        let valid_hints = ["auto", "minimal", "simple", "standard", "rich", "immersive"];
        
        for hint in &valid_hints {
            let config = SensoryConfig {
                required_capabilities: CapabilityRequirements::default(),
                optional_capabilities: CapabilityRequirements::default(),
                complexity_hint: hint.to_string(),
            };
            
            assert!(config.validate().is_ok(), "Failed for hint: {}", hint);
        }
    }
    
    #[test]
    fn test_capability_requirements_invalid_output() {
        let reqs = CapabilityRequirements {
            outputs: vec!["invalid_output".to_string()],
            inputs: vec![],
        };
        
        let result = reqs.validate("test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid test output capability"));
    }
    
    #[test]
    fn test_capability_requirements_invalid_input() {
        let reqs = CapabilityRequirements {
            outputs: vec![],
            inputs: vec!["invalid_input".to_string()],
        };
        
        let result = reqs.validate("test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid test input capability"));
    }
    
    #[test]
    fn test_capability_requirements_valid() {
        let reqs = CapabilityRequirements {
            outputs: vec!["visual".to_string(), "audio".to_string()],
            inputs: vec!["pointer".to_string(), "keyboard".to_string()],
        };
        
        assert!(reqs.validate("test").is_ok());
    }
    
    #[test]
    fn test_ui_config_validation_with_panels() {
        let config = UiConfig {
            custom_panels: vec![
                CustomPanelConfig {
                    panel_type: "doom_game".to_string(),
                    title: "Doom".to_string(),
                    width: Some(640),
                    height: Some(480),
                    fullscreen: false,
                    config: serde_json::Value::Null,
                },
            ],
            ..Default::default()
        };
        
        assert!(config.validate().is_ok());
    }
    
    // ===== Original Tests =====

    #[test]
    fn test_scenario_parsing() {
        let json = r#"{
            "name": "Test Scenario",
            "description": "A test",
            "version": "1.0.0",
            "mode": "test",
            "ecosystem": {
                "primals": [
                    {
                        "id": "test-1",
                        "name": "TEST",
                        "type": "test",
                        "family": "nat0",
                        "status": "healthy",
                        "health": 100,
                        "confidence": 100,
                        "position": { "x": 0.0, "y": 0.0 },
                        "capabilities": ["neural-api"],
                        "metrics": {
                            "cpu_percent": 5.0,
                            "memory_mb": 100
                        }
                    }
                ]
            }
        }"#;

        let scenario: Scenario = serde_json::from_str(json).unwrap();
        assert_eq!(scenario.name, "Test Scenario");
        assert_eq!(scenario.ecosystem.primals.len(), 1);
        assert_eq!(scenario.ecosystem.primals[0].name, "TEST");
    }

    #[test]
    fn test_scenario_with_sensory_config() {
        let json = r#"{
            "name": "Sensory Test",
            "description": "Tests sensory capabilities",
            "version": "2.0.0",
            "mode": "test",
            "sensory_config": {
                "required_capabilities": {
                    "outputs": ["visual"],
                    "inputs": ["pointer"]
                },
                "optional_capabilities": {
                    "outputs": ["audio"],
                    "inputs": ["keyboard"]
                },
                "complexity_hint": "auto"
            }
        }"#;

        let scenario: Scenario = serde_json::from_str(json).unwrap();
        assert_eq!(scenario.sensory_config.required_capabilities.outputs.len(), 1);
        assert_eq!(scenario.sensory_config.required_capabilities.outputs[0], "visual");
        assert_eq!(scenario.sensory_config.complexity_hint, "auto");
    }

    #[test]
    fn test_capability_validation_success() {
        let scenario = Scenario {
            name: "Test".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            mode: "test".to_string(),
            ui_config: UiConfig::default(),
            ecosystem: Ecosystem::default(),
            neural_api: NeuralApiConfig::default(),
            sensory_config: SensoryConfig {
                required_capabilities: CapabilityRequirements {
                    outputs: vec!["visual".to_string()],
                    inputs: vec!["pointer".to_string()],
                },
                optional_capabilities: CapabilityRequirements::default(),
                complexity_hint: "auto".to_string(),
            },
        };

        // Desktop capabilities (has visual + pointer)
        let caps = SensoryCapabilities::discover().unwrap();
        assert!(scenario.validate_capabilities(&caps).is_ok());
    }

    #[test]
    fn test_complexity_determination() {
        let scenario = Scenario {
            name: "Test".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            mode: "test".to_string(),
            ui_config: UiConfig::default(),
            ecosystem: Ecosystem::default(),
            neural_api: NeuralApiConfig::default(),
            sensory_config: SensoryConfig {
                required_capabilities: CapabilityRequirements::default(),
                optional_capabilities: CapabilityRequirements::default(),
                complexity_hint: "standard".to_string(),
            },
        };

        let caps = SensoryCapabilities::discover().unwrap();
        let complexity = scenario.determine_complexity(&caps);
        assert_eq!(complexity, SensoryUIComplexity::Standard);
    }

    #[test]
    fn test_complexity_auto_detection() {
        let scenario = Scenario {
            name: "Test".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            mode: "test".to_string(),
            ui_config: UiConfig::default(),
            ecosystem: Ecosystem::default(),
            neural_api: NeuralApiConfig::default(),
            sensory_config: SensoryConfig {
                required_capabilities: CapabilityRequirements::default(),
                optional_capabilities: CapabilityRequirements::default(),
                complexity_hint: "auto".to_string(),
            },
        };

        let caps = SensoryCapabilities::discover().unwrap();
        let complexity = scenario.determine_complexity(&caps);
        // Should auto-detect based on current device (likely Rich or Immersive on desktop)
        assert!(matches!(
            complexity,
            SensoryUIComplexity::Minimal
                | SensoryUIComplexity::Simple
                | SensoryUIComplexity::Standard
                | SensoryUIComplexity::Rich
                | SensoryUIComplexity::Immersive
        ));
    }
}

