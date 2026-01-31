# 🔨 Scenario Refactoring Plan - Smart Modular Split

**File**: `crates/petal-tongue-ui/src/scenario.rs` (1,081 lines)  
**Target**: <1000 lines per module  
**Strategy**: Logical separation by responsibility

---

## 📋 Current Structure Analysis

**Lines 1-430**: Type definitions and validation (43% of file)
- Scenario, UiConfig, PanelVisibility, FeatureFlags
- CustomPanelConfig, AnimationConfig, PerformanceConfig
- Ecosystem, PrimalDefinition, Position, PrimalMetrics
- ScenarioProprioception, SelfAwareness, Motor, Sensory
- NeuralApiConfig, SensoryConfig, CapabilityRequirements

**Lines 431-700**: Implementation (Scenario methods) (25% of file)
- load(), validate(), to_primal_infos()
- primal_count(), extract_metrics()
- sensory adaptation logic

**Lines 701-1081**: Conversion and helpers (32% of file)
- Conversion from PrimalDefinition → PrimalInfo
- Mock/fixture generation
- Test helpers

---

## 🎯 Proposed Module Structure

```
crates/petal-tongue-ui/src/scenario/
├── mod.rs              (~150 lines) - Public API & re-exports
├── types.rs            (~350 lines) - Core data structures
├── config.rs           (~200 lines) - UI & performance config
├── ecosystem.rs        (~180 lines) - Primal definitions & metrics
├── sensory.rs          (~150 lines) - Sensory capability config
├── loader.rs           (~200 lines) - Loading & validation logic
├── convert.rs          (~180 lines) - Type conversions
└── fixtures.rs         (~150 lines) - Test fixtures (test-only)
```

**Total**: ~1,560 lines across 8 files (avg ~195 lines/file)
**Reduction**: Well under 1000 lines per file ✅

---

## 📦 Module Breakdown

### 1. `mod.rs` - Public API (~150 lines)

**Purpose**: Module root, re-exports, public API

```rust
//! Scenario loading system for benchTop demonstrations
//!
//! # Architecture
//!
//! This module is organized as follows:
//! - `types`: Core Scenario struct and main types
//! - `config`: UI configuration (panels, animations, performance)
//! - `ecosystem`: Primal definitions and metrics
//! - `sensory`: Sensory capability configuration
//! - `loader`: Loading and validation logic
//! - `convert`: Type conversions
//! - `fixtures`: Test fixtures (test-only)

pub mod types;
pub mod config;
pub mod ecosystem;
pub mod sensory;
pub mod loader;
pub mod convert;

#[cfg(test)]
pub mod fixtures;

// Re-export main types for convenience
pub use types::Scenario;
pub use config::{UiConfig, PanelVisibility, FeatureFlags, AnimationConfig, PerformanceConfig};
pub use ecosystem::{Ecosystem, PrimalDefinition, Position, PrimalMetrics};
pub use sensory::{SensoryConfig, CapabilityRequirements};
pub use loader::ScenarioLoader;
pub use convert::ToPrimalInfo;
```

---

### 2. `types.rs` - Core Structures (~350 lines)

**Purpose**: Main Scenario struct and primary types

**Contents**:
- `Scenario` struct
- `NeuralApiConfig`
- `ScenarioProprioception`
- `SelfAwareness`, `Motor`, `Sensory`
- Core trait implementations
- Documentation

**Example**:
```rust
//! Core scenario types

use serde::{Deserialize, Serialize};
use crate::scenario::config::UiConfig;
use crate::scenario::ecosystem::Ecosystem;
use crate::scenario::sensory::SensoryConfig;

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
    #[serde(default)]
    pub sensory_config: SensoryConfig,
}

impl Scenario {
    /// Get number of primals in scenario
    pub fn primal_count(&self) -> usize {
        self.ecosystem.primals.len()
    }
    
    // ... other methods ...
}

// Neural API config
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NeuralApiConfig {
    // ... fields ...
}

// Proprioception types
// ... rest of types ...
```

---

### 3. `config.rs` - UI Configuration (~200 lines)

**Purpose**: UI-related configuration structures

**Contents**:
- `UiConfig`
- `PanelVisibility`
- `FeatureFlags`
- `CustomPanelConfig`
- `AnimationConfig`
- `PerformanceConfig`
- Validation logic

**Example**:
```rust
//! UI configuration types

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// UI configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub layout: String,
    #[serde(default)]
    pub show_panels: PanelVisibility,
    #[serde(default)]
    pub animations: AnimationConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
    #[serde(default)]
    pub features: FeatureFlags,
    #[serde(default)]
    pub custom_panels: Vec<CustomPanelConfig>,
}

impl UiConfig {
    pub fn validate(&self) -> Result<()> {
        // ... validation logic ...
    }
}

// Rest of config types...
```

---

### 4. `ecosystem.rs` - Primal Definitions (~180 lines)

**Purpose**: Ecosystem and primal-related structures

**Contents**:
- `Ecosystem`
- `PrimalDefinition`
- `Position`
- `PrimalMetrics`
- Primal-specific logic

**Example**:
```rust
//! Ecosystem and primal definition types

use serde::{Deserialize, Serialize};
use crate::scenario::types::ScenarioProprioception;

/// Ecosystem containing primals
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ecosystem {
    #[serde(default)]
    pub primals: Vec<PrimalDefinition>,
}

/// Single primal definition in scenario
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

// Position, PrimalMetrics...
```

---

### 5. `sensory.rs` - Sensory Config (~150 lines)

**Purpose**: Sensory capability configuration

**Contents**:
- `SensoryConfig`
- `CapabilityRequirements`
- Validation logic
- Capability matching

**Example**:
```rust
//! Sensory capability configuration for adaptive rendering

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Sensory capability configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensoryConfig {
    #[serde(default)]
    pub required_capabilities: CapabilityRequirements,
    #[serde(default)]
    pub optional_capabilities: CapabilityRequirements,
    #[serde(default = "default_complexity_hint")]
    pub complexity_hint: String,
}

impl SensoryConfig {
    pub fn validate(&self) -> Result<()> {
        // ... validation ...
    }
}

// CapabilityRequirements...
```

---

### 6. `loader.rs` - Loading Logic (~200 lines)

**Purpose**: Scenario loading and validation

**Contents**:
- `ScenarioLoader` struct
- File loading logic
- Validation orchestration
- Error handling

**Example**:
```rust
//! Scenario loading and validation logic

use anyhow::{Context, Result};
use std::path::Path;
use crate::scenario::types::Scenario;

/// Scenario loader with validation
pub struct ScenarioLoader;

impl ScenarioLoader {
    /// Load scenario from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Scenario> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read scenario file: {}", path.display()))?;
        
        Self::from_json(&content)
            .with_context(|| format!("Failed to parse scenario: {}", path.display()))
    }
    
    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Scenario> {
        let scenario: Scenario = serde_json::from_str(json)
            .context("Failed to deserialize JSON")?;
        
        Self::validate(&scenario)?;
        
        Ok(scenario)
    }
    
    /// Validate scenario
    pub fn validate(scenario: &Scenario) -> Result<()> {
        scenario.ui_config.validate()?;
        scenario.sensory_config.validate()?;
        // ... more validation ...
        Ok(())
    }
}
```

---

### 7. `convert.rs` - Type Conversions (~180 lines)

**Purpose**: Convert scenario types to runtime types

**Contents**:
- `ToPrimalInfo` trait
- PrimalDefinition → PrimalInfo conversion
- Metrics extraction
- Position mapping

**Example**:
```rust
//! Type conversion utilities

use petal_tongue_core::{PrimalInfo, PrimalHealthStatus, SystemMetrics};
use crate::scenario::ecosystem::PrimalDefinition;
use crate::scenario::types::Scenario;

/// Convert scenario types to runtime primal info
pub trait ToPrimalInfo {
    fn to_primal_info(&self) -> PrimalInfo;
}

impl ToPrimalInfo for PrimalDefinition {
    fn to_primal_info(&self) -> PrimalInfo {
        let health = match self.status.as_str() {
            "healthy" => PrimalHealthStatus::Healthy,
            "degraded" => PrimalHealthStatus::Degraded,
            "down" => PrimalHealthStatus::Down,
            _ => PrimalHealthStatus::Unknown,
        };
        
        PrimalInfo::new(
            self.id.clone(),
            self.name.clone(),
            self.primal_type.clone(),
            format!("mock://{}:8080", self.id),
            self.capabilities.clone(),
            health,
            0, // mock last_seen
        )
    }
}

impl Scenario {
    /// Convert all primals to PrimalInfo
    pub fn to_primal_infos(&self) -> Vec<PrimalInfo> {
        self.ecosystem.primals
            .iter()
            .map(|p| p.to_primal_info())
            .collect()
    }
}
```

---

### 8. `fixtures.rs` - Test Fixtures (~150 lines, test-only)

**Purpose**: Mock scenario generation for tests

**Contents**:
- `create_mock_scenario()`
- `create_minimal_scenario()`
- Test helper functions
- Conditional compilation (#[cfg(test)])

**Example**:
```rust
//! Test fixtures for scenario testing

#![cfg(test)]

use crate::scenario::types::Scenario;
use crate::scenario::config::UiConfig;
use crate::scenario::ecosystem::{Ecosystem, PrimalDefinition, Position};

/// Create a mock scenario for testing
pub fn create_mock_scenario() -> Scenario {
    Scenario {
        name: "Test Scenario".to_string(),
        description: "A test scenario".to_string(),
        version: "1.0.0".to_string(),
        mode: "test".to_string(),
        ui_config: UiConfig::default(),
        ecosystem: Ecosystem {
            primals: vec![
                create_mock_primal("beardog-1", "BearDog", 100.0, 100.0),
                create_mock_primal("songbird-1", "Songbird", 200.0, 100.0),
            ],
        },
        neural_api: Default::default(),
        sensory_config: Default::default(),
    }
}

fn create_mock_primal(id: &str, name: &str, x: f32, y: f32) -> PrimalDefinition {
    // ... mock primal creation ...
}
```

---

## 🔄 Migration Steps

### Phase 1: Create Module Structure (30 min)

1. Create `scenario/` directory
2. Create empty module files with headers
3. Add `mod scenario;` to lib.rs

### Phase 2: Extract Types (1 hour)

1. Move core types to `types.rs`
2. Move config types to `config.rs`
3. Move ecosystem types to `ecosystem.rs`
4. Move sensory types to `sensory.rs`
5. Fix imports in each file

### Phase 3: Extract Logic (1 hour)

1. Move loading logic to `loader.rs`
2. Move conversion logic to `convert.rs`
3. Move test fixtures to `fixtures.rs`
4. Fix cross-module imports

### Phase 4: Update Imports (30 min)

1. Update `mod.rs` with re-exports
2. Update imports in consuming code
3. Fix any broken references

### Phase 5: Validation (30 min)

1. Run `cargo build`
2. Run `cargo test`
3. Fix any compilation errors
4. Verify all tests pass

**Total Effort**: ~3.5 hours

---

## ✅ Benefits

1. **Maintainability**: Clear module boundaries
2. **Readability**: Each file has single responsibility
3. **Testing**: Easier to test isolated components
4. **Compliance**: All files <1000 lines
5. **Modern**: Idiomatic Rust module organization

---

## 📊 Before vs After

| Metric | Before | After |
|--------|--------|-------|
| **Files** | 1 | 8 |
| **Largest File** | 1,081 lines | ~350 lines |
| **Avg Lines/File** | 1,081 | ~195 |
| **Compliance** | ❌ Over limit | ✅ Under limit |
| **Test Isolation** | Mixed | Separate module |

---

## 🎯 Next Steps

1. Execute Phase 1-5 refactoring
2. Run full test suite
3. Update documentation
4. Commit with message: "refactor(ui): split scenario.rs into modules"

---

*Smart refactoring plan - preserves all functionality, improves structure*
