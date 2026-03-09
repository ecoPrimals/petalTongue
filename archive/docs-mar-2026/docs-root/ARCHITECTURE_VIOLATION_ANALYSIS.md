# 🔴 Architecture Violation Analysis

**Date**: January 3, 2026  
**Issue**: Hardcoding & Vendor Lock-in in petalTongue  
**Severity**: HIGH (violates core principles)

---

## 🚨 Identified Violations

### 1. Hardcoded Ecosystem Knowledge

**Violation**: petalTongue "knows" about trust levels and family IDs

```rust
// In app.rs - HARDCODED KNOWLEDGE:
if let Some(trust_level) = info.trust_level {
    let (emoji, text, color) = match trust_level {
        0 => ("⚫", "None", gray),
        1 => ("🟡", "Limited", yellow),
        2 => ("🟠", "Elevated", orange),
        3 => ("🟢", "Full", green),
        _ => ...
    };
}

if let Some(family_id) = &info.family_id {
    // Assumes "family_id" is meaningful
    ui.label(format!("Family: {}", family_id));
}
```

**Problem**: 
- petalTongue "knows" trust is 0-3 scale
- petalTongue "knows" what "family_id" means
- petalTongue "knows" specific emoji mappings

**Principle Violated**: 
> "Primal code only has self knowledge and discovers other primals at runtime"

---

### 2. Hardcoded Capability Icons

**Violation**: Capability-to-icon mapping is hardcoded

```rust
fn get_capability_icon(&self, capability: &str) -> &'static str {
    if capability.contains("security") { "🔒" }
    else if capability.contains("storage") { "💾" }
    else if capability.contains("ai") { "🧠" }
    // ... 11 hardcoded mappings
}
```

**Problem**:
- Assumes "security" always means 🔒
- New capability types require code changes
- Not discoverable at runtime

---

### 3. Hardcoded Data Structures

**Violation**: `PrimalInfo` has ecosystem-specific fields

```rust
pub struct PrimalInfo {
    // Generic (OK):
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
    
    // Ecosystem-specific (VIOLATION):
    pub trust_level: Option<u8>,  // ❌ Assumes trust exists
    pub family_id: Option<String>, // ❌ Assumes family exists
}
```

**Problem**:
- Core type assumes specific ecosystem features
- Not extensible to other ecosystems
- Vendor lock-in

---

## 🎯 Correct Architecture

### Core Principle:
> **petalTongue should only have self-knowledge. Ecosystem-specific features (trust, family, entropy) are COMPOSED on top.**

---

### Layer 1: Universal petalTongue (Self-Knowledge Only)

**What petalTongue knows about ITSELF**:
```rust
// petalTongue's self-knowledge:
pub struct PetalTonguePrimal {
    name: "petalTongue",
    capabilities: [
        "visualization/2d",
        "visualization/audio",
        "graph-rendering",
        "node-display",
        "capability-discovery"
    ]
}
```

**What petalTongue can DO** (without ecosystem knowledge):
1. Render arbitrary graph structures
2. Display node properties (any key-value pairs)
3. Visualize connections
4. Provide interaction (click, hover)
5. Discover capabilities from other primals

---

### Layer 2: Generic Data Model

**No hardcoded fields**:
```rust
pub struct GenericNode {
    pub id: String,
    pub name: String,
    pub properties: HashMap<String, Property>,  // ✅ Generic!
    pub capabilities: Vec<String>,
}

pub enum Property {
    String(String),
    Number(f64),
    Boolean(bool),
    Nested(HashMap<String, Property>),
}
```

**Benefits**:
- Works with ANY ecosystem
- Extensible without code changes
- No assumptions about data meaning

---

### Layer 3: Capability-Based Composition

**Trust visualization = Optional capability**:
```rust
// Trust is discovered, not assumed:
if ecosystem.has_capability("trust-management") {
    let trust_spec = ecosystem.get_capability_spec("trust-management");
    // trust_spec tells us: range, colors, meanings
    render_trust_ui(node, trust_spec);
}
```

**Family lineage = Optional capability**:
```rust
if ecosystem.has_capability("genetic-lineage") {
    let lineage_spec = ecosystem.get_capability_spec("genetic-lineage");
    render_lineage_ui(node, lineage_spec);
}
```

**Icon mapping = Discovered**:
```rust
// Icons come FROM the ecosystem, not hardcoded:
let icon = ecosystem.get_icon_for_capability(capability)
    .unwrap_or("⚙️");  // Generic fallback
```

---

## 🏗️ Proposed Refactoring

### Phase 1: Extract Ecosystem Knowledge

**Goal**: Move trust/family knowledge OUT of petalTongue core

**Steps**:
1. Create `ecosystem_adapters/` module
2. Move trust logic to `trust_adapter.rs`
3. Move family logic to `family_adapter.rs`
4. Move capability icons to `capability_adapter.rs`

**Result**:
```
petalTongue/
├── core/           # Generic, universal (self-knowledge only)
│   ├── graph.rs
│   ├── node.rs
│   └── renderer.rs
├── adapters/       # Ecosystem-specific (composed)
│   ├── trust.rs
│   ├── family.rs
│   └── capabilities.rs
└── main.rs         # Composes based on discovery
```

---

### Phase 2: Capability Discovery

**Runtime discovery instead of compile-time knowledge**:

```rust
// At startup, petalTongue asks:
let ecosystem_caps = discover_ecosystem_capabilities();

// If trust exists, get its spec:
if let Some(trust_cap) = ecosystem_caps.get("trust-management") {
    let trust_ui = TrustUIAdapter::new(trust_cap.spec);
    register_adapter(trust_ui);
}

// If family exists, get its spec:
if let Some(family_cap) = ecosystem_caps.get("genetic-lineage") {
    let family_ui = FamilyUIAdapter::new(family_cap.spec);
    register_adapter(family_ui);
}
```

---

### Phase 3: Property-Based Rendering

**Generic property display**:

```rust
// petalTongue core doesn't know what properties mean
fn render_node_properties(node: &GenericNode) {
    for (key, value) in &node.properties {
        // Ask ecosystem adapters if they want to render this
        if let Some(adapter) = find_adapter_for_property(key) {
            adapter.render_property(key, value, ui);
        } else {
            // Generic fallback
            render_generic_property(key, value, ui);
        }
    }
}
```

---

## 🎊 Benefits of Correct Architecture

### 1. Universal
- Works with ANY primal ecosystem
- Not tied to ecoPrimals specifics
- Reusable across projects

### 2. Extensible
- New features (like entropy) don't require petalTongue changes
- Ecosystem can evolve independently
- Plug-and-play adapters

### 3. Principled
- ✅ Self-knowledge only
- ✅ Runtime discovery
- ✅ Capability-based
- ✅ No hardcoding
- ✅ No vendor lock-in

### 4. Maintainable
- Clear separation of concerns
- Generic core = stable
- Adapters = evolvable

---

## 📋 Migration Path

### Immediate (This Session):
1. Document the violations ✅ (this doc)
2. Design the refactoring
3. Create adapter architecture

### Phase 1 (Next Session):
1. Create `ecosystem_adapters/` module
2. Extract trust logic
3. Extract family logic
4. Extract capability icons

### Phase 2:
1. Implement capability discovery
2. Make PrimalInfo generic
3. Runtime adapter registration

### Phase 3:
1. Remove all hardcoded ecosystem knowledge
2. Pure generic core
3. 100% composed functionality

---

## 🎯 Success Criteria

**When complete, petalTongue should**:

```rust
// ✅ Have zero knowledge of trust:
grep -r "trust_level" core/  # Returns nothing

// ✅ Have zero knowledge of family:
grep -r "family_id" core/  # Returns nothing

// ✅ Have zero hardcoded mappings:
grep -r "if.*contains.*security" core/  # Returns nothing

// ✅ Be usable with ANY ecosystem:
cargo test ecosystem_agnostic_tests  # All pass
```

---

## 💡 Key Insight

> **petalTongue is NOT an "ecoPrimals UI"**  
> **petalTongue is a "Universal Primal Visualization Engine"**  
> **that CAN visualize ecoPrimals (and anything else)**

---

**Status**: 🔴 Violations Identified  
**Priority**: HIGH (architectural debt)  
**Impact**: Makes petalTongue truly universal  
**Timeline**: 2-3 refactoring sessions

---

🌸 **petalTongue: Evolving from specific to universal!** 🚀

