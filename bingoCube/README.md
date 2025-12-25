# 🎲 BingoCube

**Human-Verifiable Cryptographic Commitment System**

> A cross-primal tool for creating memorable, visual, and auditory cryptographic patterns.

---

## 🎯 **What is BingoCube?**

BingoCube is a **pure tool** (not a primal) that any primal in the ecoPrimals ecosystem can use for:

- 🔐 **Identity verification** (BearDog)
- 🤝 **Peer trust stamps** (Songbird)
- 📄 **Content fingerprints** (NestGate)
- ⚙️ **Computation proofs** (ToadStool)
- 🎨 **Visual representation** (petalTongue)

---

## 🏗️ **Architecture**

```
bingoCube/
├── core/         # Pure cryptographic tool (no dependencies)
├── adapters/     # Optional visualization helpers
├── demos/        # Interactive demonstrations
└── whitePaper/   # Mathematical foundations & use cases
```

### **Core** (Required)
Pure Rust cryptographic primitives:
- 2-board cross-binding algorithm
- BLAKE3 hashing
- Progressive reveal (x: 0.0 → 1.0)
- Verification primitives

**No Dependencies**: Works standalone, no GUI, no audio, no primal coupling.

### **Adapters** (Optional)
Helpers for systems that want to visualize BingoCubes:
- `visual` - egui rendering
- `audio` - sonification
- `animation` - smooth transitions

**Feature-gated**: Only include what you need.

### **Demos** (Optional)
Interactive demonstrations showing multi-modal capabilities.

---

## 📦 **Usage**

### As a Pure Tool (Any Primal)

```rust
use bingocube_core::{BingoCube, Config};

// Generate identity cube
let cube = BingoCube::from_seed(identity_bytes, Config::default())?;

// Get progressive reveal
let subcube = cube.subcube(0.5)?; // 50% reveal

// You now have:
// - subcube.revealed: HashMap<(row, col), color>
// - subcube.size: usize
// - subcube.x: f64

// Use it however you want:
// - Serialize and send to peers
// - Verify authenticity
// - Store in database
// - Convert to QR-like format
```

### With Visualization (Optional)

```rust
use bingocube_adapters::visual::BingoCubeVisualAdapter;

// In your egui UI:
BingoCubeVisualAdapter::render(&cube, x, ui);
```

---

## 🎨 **For Visualization Systems (like petalTongue)**

petalTongue can render BingoCubes but doesn't "own" them:

```rust
use bingocube_adapters::visual::BingoCubeVisualAdapter;

// petalTongue's job: render ANY data structure
impl UniversalRenderer for PetalTongue {
    fn render_bingocube(&self, cube: &BingoCube, ui: &mut Ui) {
        BingoCubeVisualAdapter::render(cube, self.reveal_level, ui);
    }
}
```

---

## 🔐 **Security Properties**

- **Forgery Probability**: ~2^-50 at x=0.5, ~2^-100 at x=1.0
- **Collision Resistance**: BLAKE3 (256-bit)
- **Progressive Trust**: Nested masks (x=0.2 ⊂ x=0.5 ⊂ x=1.0)
- **Deterministic**: Same seed → same cube

---

## 🚀 **Quick Start**

```bash
# Run interactive demo
cd bingoCube/demos
cargo run --release

# Build just the core tool
cd bingoCube/core
cargo build --release

# Build adapters with visual support
cd bingoCube/adapters
cargo build --release --features visual
```

---

## 📚 **Documentation**

- **Whitepaper**: `whitePaper/BingoCube-Overview.md`
- **Math**: `whitePaper/BingoCube-Mathematical-Foundation.md`
- **Use Cases**: `whitePaper/BingoCube-Ecosystem-Examples.md`

---

## 🎯 **Design Principles**

1. **Pure Tool**: No primal dependencies
2. **Self-Contained**: Core has minimal dependencies
3. **Optional Rendering**: Adapters are feature-gated
4. **Use Cases Emerge**: Any primal can use it their way

---

## 🔄 **Future: Separate Repository**

BingoCube is designed to be extracted into its own repo:

```
github.com/ecoPrimals/bingoCube/
  ├── core/         # Pure tool
  ├── adapters/     # Visualization helpers
  └── demos/        # Interactive demos
```

Any primal can then:
```toml
[dependencies]
bingocube-core = { git = "https://github.com/ecoPrimals/bingoCube" }
```

---

## ✅ **Status**

- ✅ Core: Production ready (600 lines, 7 tests)
- ✅ Adapters: Complete (visual, audio, animation)
- ✅ Demos: Interactive demo working
- ✅ Whitepaper: ~110 pages comprehensive
- ✅ Tests: 22 passing (100% success)

**Ready to use!** 🚀

---

**License**: AGPL-3.0  
**Project**: ecoPrimals  
**Contact**: See main petalTongue repository

