# BingoCube Tool Use Showcase

**Scenario**: Demonstrate petalTongue using BingoCube as an external tool  
**Status**: ✅ Ready  
**Type**: Primal Tool Use (not primal-to-primal interaction)  
**Duration**: 5-10 minutes

---

## Overview

This showcase demonstrates how petalTongue uses BingoCube as a standalone cryptographic tool. This is a **"primal tool use"** pattern, not a primal-to-primal interaction.

### Key Concepts

**BingoCube as a Tool:**
- Standalone cryptographic library
- Any primal can import and use it
- Provides pure crypto primitives
- No primal dependencies

**petalTongue as Universal Visualizer:**
- Can render any data structure
- Uses `bingocube-adapters` for rendering
- Demonstrates tool integration pattern
- Shows clean separation of concerns

---

## What You'll See

### 1. **Tool Integration**
- petalTongue imports `bingocube-core` and `bingocube-adapters`
- Uses the tool's API to generate BingoCubes
- Renders using optional adapters
- Clean, capability-based integration

### 2. **Interactive Controls**
- **Seed Input**: Generate different patterns
- **Reveal Slider**: Progressive reveal (0-100%)
- **Generate Button**: Create new BingoCubes
- **Toggle**: Switch between graph and tool views

### 3. **Visual Rendering**
- 5×5 color grid (default configuration)
- Cryptographic hash-based colors
- Progressive reveal visualization
- Real-time updates

---

## How to Run

### Quick Start:
```bash
cd /path/to/petalTongue
./showcase/local/07-bingocube-visualization/demo.sh
```

### Manual Run:
```bash
# Build
cargo build --release -p petal-tongue-ui

# Run
cargo run --release -p petal-tongue-ui

# In the UI:
# 1. Click "🎲 BingoCube Tool" in top menu
# 2. Interact with controls
# 3. Toggle back to graph view anytime
```

---

## Architecture Demonstrated

### Tool Pattern:
```
┌─────────────────────────────────────────┐
│  BingoCube (Standalone Tool)            │
│  • Pure crypto primitives               │
│  • No primal dependencies               │
│  • Any primal can use it                │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│  BingoCube Adapters (Optional)          │
│  • Visual rendering helpers             │
│  • Feature-gated                        │
│  • Used by visualization systems        │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│  petalTongue (Universal Visualizer)     │
│  • Imports bingocube-core               │
│  • Uses bingocube-adapters              │
│  • Demonstrates tool use pattern        │
└─────────────────────────────────────────┘
```

### Code Integration:
```rust
// In petalTongue UI:
use bingocube_core::{BingoCube, Config};
use bingocube_adapters::visual::BingoCubeVisualRenderer;

// Generate a BingoCube (using the tool)
let cube = BingoCube::from_seed(b"alice", Config::default())?;

// Render it (using the adapter)
let renderer = BingoCubeVisualRenderer::new();
renderer.render(ui, &cube);
```

---

## What This Is NOT

This is **NOT** a primal-to-primal interaction. This is a primal using a tool.

**Primal-to-Primal** would be:
- petalTongue ←→ BearDog (identity verification)
- petalTongue ←→ Songbird (discovery)
- petalTongue ←→ NestGate (content addressing)

**Primal Tool Use** is:
- petalTongue uses BingoCube (this demo)
- BearDog uses BingoCube (identity)
- Songbird uses BingoCube (trust stamps)
- Any primal uses any tool

---

## Use Cases

### 1. **Identity Verification** (BearDog)
```rust
// BearDog generates identity cube
let identity = BingoCube::from_seed(genetics_hash, config)?;
// User verifies visually or via audio
```

### 2. **Trust Stamps** (Songbird)
```rust
// Songbird creates trust stamp
let stamp = BingoCube::from_seed(peer_id, config)?;
// Peers verify the stamp
```

### 3. **Content Fingerprints** (NestGate)
```rust
// NestGate fingerprints content
let fingerprint = BingoCube::from_seed(content_hash, config)?;
// Users verify content authenticity
```

### 4. **Computation Proofs** (ToadStool)
```rust
// ToadStool proves computation
let proof = BingoCube::from_seed(result_hash, config)?;
// Verifiers check the proof
```

---

## Interaction Patterns Discovered

### ✅ **What Works Well:**
- Clean import of tool crates
- Adapter pattern for rendering
- Optional dependencies (feature gates)
- Tool independence

### 🔍 **Potential Gaps:**
- (To be discovered during testing)
- (Document as we find them)
- (Evolution opportunities)

---

## Success Criteria

- ✅ petalTongue can import BingoCube
- ✅ BingoCube generates correctly
- ✅ Adapters render properly
- ✅ UI controls work smoothly
- ✅ Toggle between views works
- ✅ Pattern demonstrates tool use clearly

---

## Next Steps

After running this demo:
1. Test with different seeds
2. Verify progressive reveal works
3. Document any interaction gaps found
4. Consider audio sonification integration
5. Plan for other tool integrations

---

**This showcase demonstrates the foundation for all primal tool use in the ecosystem!** 🎉
