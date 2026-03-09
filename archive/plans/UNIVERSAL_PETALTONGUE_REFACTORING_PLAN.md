# 🏗️ Universal petalTongue Refactoring Plan

**Date**: January 3, 2026  
**Goal**: Transform petalTongue from ecosystem-specific to universal  
**Approach**: Adapter pattern + Capability discovery

---

## 🎯 Architecture: 3-Layer Composition

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 3: Ecosystem-Specific Adapters (ecoPrimals-specific)│
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐  │
│  │ TrustAdapter │ │FamilyAdapter │ │ CapabilityAdapter │  │
│  │ (trust 0-3)  │ │  (DNA/iidn)  │ │   (icon maps)     │  │
│  └──────────────┘ └──────────────┘ └───────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Adapter Registry (runtime composition)            │
│  • Discovers ecosystem capabilities                         │
│  • Loads adapters dynamically                               │
│  • Routes property rendering                                │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Universal petalTongue Core (self-knowledge only)  │
│  • Generic graph rendering                                  │
│  • Property display (any key-value)                         │
│  • Interaction (click, hover)                               │
│  • Modalities (visual, audio)                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 📁 New File Structure

```
crates/petal-tongue-core/
├── src/
│   ├── types.rs                  # ✅ Generic only
│   ├── graph_engine.rs           # ✅ Generic only
│   └── property.rs               # NEW: Generic property system

crates/petal-tongue-adapters/     # NEW CRATE!
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Adapter registry
│   ├── trait.rs                  # PropertyAdapter trait
│   ├── ecoprimal_trust.rs        # ecoPrimals trust adapter
│   ├── ecoprimal_family.rs       # ecoPrimals family adapter
│   └── ecoprimal_capabilities.rs # ecoPrimals capability icons

crates/petal-tongue-ui/
├── src/
│   ├── app.rs                    # Uses adapters, not hardcoding
│   └── adapter_loader.rs         # NEW: Dynamic adapter loading
```

---

## 🔧 Step 1: Create Generic Property System

### File: `crates/petal-tongue-core/src/property.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic property value (no ecosystem knowledge)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, PropertyValue>),
    Array(Vec<PropertyValue>),
}

/// Generic node with arbitrary properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub properties: HashMap<String, PropertyValue>,  // ✅ Generic!
    pub health: HealthStatus,
    pub last_seen: u64,
}

/// Health status (generic, not ecosystem-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}
```

**Key**: `properties` is a HashMap - petalTongue doesn't know what's in it!

---

## 🔧 Step 2: Create Adapter Trait

### File: `crates/petal-tongue-adapters/src/trait.rs`

```rust
use egui::Ui;
use petal_tongue_core::property::PropertyValue;

/// Trait for rendering ecosystem-specific properties
pub trait PropertyAdapter: Send + Sync {
    /// Name of this adapter (e.g., "ecoprimal-trust")
    fn name(&self) -> &str;
    
    /// Which property keys this adapter handles
    fn handles(&self, property_key: &str) -> bool;
    
    /// Render this property in the UI
    fn render(&self, 
        property_key: &str, 
        value: &PropertyValue, 
        ui: &mut Ui,
        palette: &ColorPalette,
    );
    
    /// Optional: Provide visual decoration (e.g., badge on node)
    fn node_decoration(
        &self,
        properties: &HashMap<String, PropertyValue>
    ) -> Option<NodeDecoration>;
}

pub struct NodeDecoration {
    pub badge: Option<String>,     // Emoji badge
    pub color: Option<Color32>,    // Node fill color
    pub ring_color: Option<Color32>, // Ring color
}
```

---

## 🔧 Step 3: Create Trust Adapter

### File: `crates/petal-tongue-adapters/src/ecoprimal_trust.rs`

```rust
use super::trait::PropertyAdapter;

/// ecoPrimals-specific trust visualization
pub struct EcoPrimalTrustAdapter {
    // Configuration (discovered from ecosystem)
    min_level: u8,
    max_level: u8,
    level_names: Vec<String>,
    level_emojis: Vec<String>,
    level_colors: Vec<Color32>,
}

impl EcoPrimalTrustAdapter {
    /// Create from ecosystem capability spec
    pub fn from_capability_spec(spec: &CapabilitySpec) -> Self {
        // Parse trust spec from ecosystem
        // spec might say: "levels": [0,1,2,3], "names": [...], etc.
        Self {
            min_level: spec.get("min").as_u8().unwrap_or(0),
            max_level: spec.get("max").as_u8().unwrap_or(3),
            level_names: spec.get("names").as_vec_string().unwrap_or_default(),
            level_emojis: spec.get("emojis").as_vec_string().unwrap_or_default(),
            level_colors: parse_colors(spec.get("colors")),
        }
    }
}

impl PropertyAdapter for EcoPrimalTrustAdapter {
    fn name(&self) -> &str { "ecoprimal-trust" }
    
    fn handles(&self, property_key: &str) -> bool {
        property_key == "trust_level"
    }
    
    fn render(&self, _key: &str, value: &PropertyValue, ui: &mut Ui, _palette: &ColorPalette) {
        if let PropertyValue::Number(level) = value {
            let level_idx = *level as usize;
            if level_idx < self.level_names.len() {
                ui.label(self.level_emojis[level_idx]);
                ui.label(&self.level_names[level_idx]);
                // ... render with ecosystem-provided config
            }
        }
    }
    
    fn node_decoration(&self, properties: &HashMap<String, PropertyValue>) -> Option<NodeDecoration> {
        if let Some(PropertyValue::Number(level)) = properties.get("trust_level") {
            let level_idx = *level as usize;
            Some(NodeDecoration {
                badge: Some(self.level_emojis.get(level_idx)?.clone()),
                color: Some(self.level_colors.get(level_idx)?.clone()),
                ring_color: None,
            })
        } else {
            None
        }
    }
}
```

**Key**: Configuration comes FROM the ecosystem, not hardcoded!

---

## 🔧 Step 4: Create Adapter Registry

### File: `crates/petal-tongue-adapters/src/lib.rs`

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct AdapterRegistry {
    adapters: Arc<RwLock<Vec<Box<dyn PropertyAdapter>>>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Register an adapter (at runtime!)
    pub fn register(&mut self, adapter: Box<dyn PropertyAdapter>) {
        self.adapters.write().unwrap().push(adapter);
    }
    
    /// Find adapter for a property
    pub fn find_adapter(&self, property_key: &str) -> Option<Box<dyn PropertyAdapter + '_>> {
        let adapters = self.adapters.read().unwrap();
        adapters.iter()
            .find(|a| a.handles(property_key))
            .map(|a| a.clone())  // Or use Arc
    }
    
    /// Render property (with adapter or generic fallback)
    pub fn render_property(
        &self,
        key: &str,
        value: &PropertyValue,
        ui: &mut Ui,
        palette: &ColorPalette,
    ) {
        if let Some(adapter) = self.find_adapter(key) {
            // Use ecosystem-specific adapter
            adapter.render(key, value, ui, palette);
        } else {
            // Generic fallback
            self.render_generic_property(key, value, ui);
        }
    }
    
    fn render_generic_property(&self, key: &str, value: &PropertyValue, ui: &mut Ui) {
        // Simple key-value display (no assumptions)
        ui.horizontal(|ui| {
            ui.label(format!("{}: ", key));
            match value {
                PropertyValue::String(s) => ui.label(s),
                PropertyValue::Number(n) => ui.label(format!("{}", n)),
                PropertyValue::Boolean(b) => ui.label(format!("{}", b)),
                _ => ui.label("..."),
            };
        });
    }
}
```

---

## 🔧 Step 5: Update app.rs to Use Adapters

### File: `crates/petal-tongue-ui/src/app.rs`

**OLD** (hardcoded):
```rust
if let Some(trust_level) = info.trust_level {
    let (emoji, text, color) = match trust_level {
        0 => ("⚫", "None", gray),
        // ... hardcoded
    };
}
```

**NEW** (adapter-based):
```rust
// Generic property rendering
for (key, value) in &node.properties {
    self.adapter_registry.render_property(key, value, ui, palette);
}
```

**Result**: app.rs has ZERO knowledge of trust, family, etc.!

---

## 🔧 Step 6: Runtime Adapter Loading

### File: `crates/petal-tongue-ui/src/adapter_loader.rs`

```rust
use petal_tongue_adapters::*;

pub fn load_adapters(ecosystem_capabilities: &[String]) -> AdapterRegistry {
    let mut registry = AdapterRegistry::new();
    
    // Discover which adapters to load based on ecosystem capabilities
    if ecosystem_capabilities.contains(&"trust-management".to_string()) {
        // Query ecosystem for trust spec
        if let Some(trust_spec) = query_capability_spec("trust-management") {
            let adapter = EcoPrimalTrustAdapter::from_capability_spec(&trust_spec);
            registry.register(Box::new(adapter));
        }
    }
    
    if ecosystem_capabilities.contains(&"genetic-lineage".to_string()) {
        if let Some(family_spec) = query_capability_spec("genetic-lineage") {
            let adapter = EcoPrimalFamilyAdapter::from_capability_spec(&family_spec);
            registry.register(Box::new(adapter));
        }
    }
    
    // ... load more adapters based on discovery
    
    registry
}
```

**Key**: Adapters are loaded AT RUNTIME based on what ecosystem provides!

---

## 📋 Migration Steps

### Session 1: Infrastructure
1. ✅ Document violations (done!)
2. ⏳ Create `petal-tongue-adapters` crate
3. ⏳ Implement `PropertyAdapter` trait
4. ⏳ Implement `AdapterRegistry`

### Session 2: Extract Trust
1. Create `EcoPrimalTrustAdapter`
2. Move trust rendering from app.rs to adapter
3. Test with runtime loading

### Session 3: Extract Family
1. Create `EcoPrimalFamilyAdapter`
2. Move family rendering to adapter
3. Test composition

### Session 4: Extract Capabilities
1. Create `EcoPrimalCapabilityAdapter`
2. Move icon mapping to adapter
3. Make it discoverable

### Session 5: Genericize Core
1. Replace `PrimalInfo` with `GenericNode`
2. Update all core types
3. Remove ecosystem-specific fields

### Session 6: Integration
1. Wire adapters into app.rs
2. Remove all hardcoding
3. Test with generic and specific ecosystems

---

## 🎯 Success Criteria

### After Refactoring:

**petalTongue core should**:
```bash
# No trust knowledge:
$ grep -r "trust_level" crates/petal-tongue-core/
# (no results)

# No family knowledge:
$ grep -r "family_id" crates/petal-tongue-core/
# (no results)

# No hardcoded icons:
$ grep -r "if.*contains.*security" crates/petal-tongue-ui/src/app.rs
# (no results)
```

**petalTongue should work with**:
- ecoPrimals (with trust, family, entropy adapters)
- Kubernetes (with pod, service, deployment adapters)
- Generic graphs (with NO adapters, just key-value display)

---

## 💡 Key Benefits

### 1. Universal
```rust
// Works with ANY primal system:
let node = GenericNode { properties: any_hashmap };
renderer.render(node);  // Just works!
```

### 2. Extensible
```rust
// New ecosystems just add adapters:
registry.register(Box::new(MyCustomAdapter::new()));
// No petalTongue code changes needed!
```

### 3. Maintainable
```rust
// Core stays stable:
// crates/petal-tongue-core/ - never changes

// Adapters evolve independently:
// crates/petal-tongue-adapters/ - evolve freely
```

---

## 📊 Estimated Effort

| Phase | Time | LOC | Complexity |
|-------|------|-----|------------|
| Infrastructure | 3-4 hours | ~300 | Medium |
| Trust Adapter | 2-3 hours | ~200 | Low |
| Family Adapter | 1-2 hours | ~100 | Low |
| Capability Adapter | 1-2 hours | ~100 | Low |
| Core Genericization | 4-5 hours | ~400 | High |
| Integration | 3-4 hours | ~200 | Medium |
| **TOTAL** | **14-20 hours** | **~1300** | **High** |

**Timeline**: 2-3 focused sessions

---

## 🎊 Vision Realized

> **"petalTongue: A Universal Primal Visualization Engine"**

**NOT**:
- An ecoPrimals UI
- A Kubernetes dashboard
- A specific tool

**BUT**:
- A universal graph visualization primal
- That discovers ecosystem capabilities
- And composes appropriate UI

**Usable by**:
- ecoPrimals ✅
- Any primal ecosystem ✅
- Non-primal systems (with adapters) ✅

---

**Status**: 📋 Plan Complete  
**Priority**: HIGH  
**Impact**: Architectural alignment  
**Ready**: To execute!

🌸 **petalTongue: From specific to universal!** 🚀

