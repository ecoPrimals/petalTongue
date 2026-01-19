# 🌸 petalTongue

**The Universal Representation System for ecoPrimals**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-16%2F16-brightgreen.svg)](./tests)
[![UniBin](https://img.shields.io/badge/UniBin-✅-brightgreen.svg)](./ECOBUD_PHASE_1_COMPLETE.md)
[![ecoBin](https://img.shields.io/badge/ecoBin-80%25-green.svg)](./ECOBIN_MIGRATION_COMPLETE_JAN_18_2026.md)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](./LICENSE)

> **"A sensory coordination layer that composes primal capabilities into coherent experiences"**

---

## 🚀 **Quick Start**

```bash
# Build the UniBin
cargo build --release

# Run desktop GUI
./target/release/petaltongue ui

# Run terminal UI (Pure Rust!)
./target/release/petaltongue tui

# Run web server (Pure Rust!)
./target/release/petaltongue web --bind 0.0.0.0:8080

# Check system status (Pure Rust!)
./target/release/petaltongue status
```

**See**: [START_HERE.md](./START_HERE.md) for detailed setup

---

## 🌱 **What is petalTongue?**

petalTongue is **ecoPrimals' universal UI platform** - a single unified binary with 5 modes:

```
petaltongue                    (5.5M, 1 binary)
├── ui        ⚠️  Optional     Desktop GUI (egui/wayland)
├── tui       ✅ Pure Rust     Terminal UI (ratatui)
├── web       ✅ Pure Rust     Web server (axum)
├── headless  ✅ Pure Rust     Rendering (SVG/PNG)
└── status    ✅ Pure Rust     System info (JSON/text)
```

**UniBin**: ✅ 1 binary, 5 modes  
**ecoBin**: ✅ 80% Pure Rust (4/5 modes)  
**Size**: 5.5M (84% smaller than before!)

---

## 💡 **Why petalTongue?**

### **Universal Representation**
- Works on **any device**: desktop, terminal, web, headless
- Adapts to **any capability**: visual, audio, haptic
- Renders **any format**: GUI, TUI, SVG, JSON

### **Unified Binary (UniBin)**
- **Before**: 3 separate binaries (38M+ total)
- **After**: 1 unified binary (5.5M) 🎉
- **84% size reduction** with more features!

### **Pure Rust (ecoBin)**
- **4 out of 5 modes**: 100% Pure Rust
- **No OpenSSL**, no dirs-sys, no platform-specific C
- **Only libc/libm/libgcc_s** (standard system libs)

### **TRUE PRIMAL**
- ✅ Zero Hardcoding (runtime discovery)
- ✅ Self-Knowledge Only (no assumptions)
- ✅ Live Evolution (adapt without recompilation)
- ✅ Graceful Degradation (fallback chains)

---

## 🎯 **Usage Examples**

### **Desktop GUI Mode**
```bash
# Launch with default scenario
petaltongue ui

# Load specific scenario
petaltongue ui --scenario sandbox/scenarios/paint-simple.json

# Disable audio
petaltongue ui --no-audio
```

### **Terminal UI Mode** (Pure Rust!)
```bash
# Basic TUI
petaltongue tui

# With scenario
petaltongue tui --scenario sandbox/scenarios/minimal.json

# Custom refresh rate
petaltongue tui --refresh-rate 30
```

### **Web Mode** (Pure Rust!)
```bash
# Default (localhost:8080)
petaltongue web

# Custom bind address
petaltongue web --bind 0.0.0.0:3000

# Open browser: http://localhost:8080
```

### **Headless Mode** (Pure Rust!)
```bash
# Render to stdout
petaltongue headless

# With custom bind
petaltongue headless --bind 0.0.0.0:9090 --workers 8
```

### **Status Mode** (Pure Rust!)
```bash
# Quick status
petaltongue status

# Verbose output
petaltongue status --verbose

# JSON format
petaltongue status --format json
```

---

## ✨ **Features**

### **Universal Panel System**
Embed any application as a panel:
- 🎮 Games (Doom MVP implemented!)
- 🌐 Web browsers (planned)
- 🎬 Video players (planned)
- 💻 Terminals (planned)
- 🎨 Custom tools

### **Sensory Capability System**
Discovers device I/O instead of hardcoding:
- **Outputs**: Visual (2D/3D), Audio, Haptic
- **Inputs**: Pointer, Keyboard, Touch, Gesture
- Adapts to: Desktop, phone, watch, VR, neural interfaces

### **Interactive Canvas**
- Create nodes (double-click)
- Connect nodes (drag)
- Delete nodes (Delete key)
- Capability validation

### **Neural API Integration**
- Real-time primal discovery
- SAME DAVE proprioception
- System metrics aggregation
- Single source of truth

---

## 🧬 **TRUE PRIMAL Architecture**

✅ **Zero Hardcoding** - Discover capabilities at runtime  
✅ **Self-Knowledge Only** - No assumptions about other primals  
✅ **Live Evolution** - Adapt without recompilation  
✅ **Graceful Degradation** - Work with what's available  
✅ **Modern Idiomatic Rust** - Async/await, Arc/RwLock  
✅ **Pure Rust Dependencies** - 80% Pure Rust  
✅ **Concurrent Testing** - 16 tests in 0.00s  

---

## 📊 **Status**

| Metric | Status | Details |
|--------|--------|---------|
| **Tests** | ✅ 16/16 | All parallel (0.00s) |
| **Build** | ✅ Clean | 12s release |
| **UniBin** | ✅ Complete | 1 binary, 5 modes |
| **ecoBin** | ✅ 80% | 4/5 modes Pure Rust |
| **Size** | ✅ 5.5M | 84% smaller |
| **Docs** | ✅ Comprehensive | 6 major docs |
| **Grade** | ✅ A++ | Outstanding! |

**Version**: 1.3.0 (ecoBud)  
**Last Updated**: January 19, 2026

---

## 🎉 **Recent Achievements**

### **January 19, 2026: UniBin Complete!**
- ✅ From 3 binaries (38M+) to 1 binary (5.5M)
- ✅ 84% size reduction
- ✅ 80% Pure Rust (4/5 modes)
- ✅ 16 tests passing in 0.00s
- ✅ Modern concurrent architecture

**Details**: [ECOBUD_PHASE_1_COMPLETE.md](./ECOBUD_PHASE_1_COMPLETE.md)

### **January 18, 2026: ecoBin Migration**
- ✅ Replaced `dirs` with `etcetera` (Pure Rust)
- ✅ Fixed `reqwest` to use `rustls-tls` (no OpenSSL)
- ✅ ARM64 builds verified

**Details**: [ECOBIN_MIGRATION_COMPLETE_JAN_18_2026.md](./ECOBIN_MIGRATION_COMPLETE_JAN_18_2026.md)

### **January 16, 2026: Doom Fully Playable!**
- ✅ 7 critical input/timing fixes
- ✅ Remote desktop compatibility
- ✅ Smooth movement at 60 Hz

**Details**: [DOOM_DEBUGGING_SESSION_JAN_16_2026.md](./DOOM_DEBUGGING_SESSION_JAN_16_2026.md)

---

## 📚 **Documentation**

| Document | Description |
|----------|-------------|
| [START_HERE.md](./START_HERE.md) | Quick start guide |
| [PROJECT_STATUS.md](./PROJECT_STATUS.md) | Current health & metrics |
| [ECOBUD_PHASE_1_COMPLETE.md](./ECOBUD_PHASE_1_COMPLETE.md) | UniBin achievement details |
| [ECOBLOSSOM_PHASE_2_PLAN.md](./ECOBLOSSOM_PHASE_2_PLAN.md) | Pure Rust GUI roadmap |
| [DUAL_UNIBIN_EXECUTION_PLAN.md](./DUAL_UNIBIN_EXECUTION_PLAN.md) | Implementation plan |
| [specs/](./specs/) | Technical specifications |

---

## 🚀 **Roadmap**

### **✅ Completed**
- [x] UniBin architecture (1 binary, 5 modes)
- [x] ecoBin migration (80% Pure Rust)
- [x] Doom panel (fully playable!)
- [x] Validation layer
- [x] Input focus system
- [x] Lifecycle hooks
- [x] Neural API integration

### **🚧 Current (ecoBud Shipped!)**
- [x] 1 binary, 5 modes
- [x] 80% Pure Rust
- [x] Production ready

### **🌸 Next (ecoBlossom Evolution)**
- [ ] GUI abstraction layer
- [ ] Pure Rust GUI prototype (DRM/smithay)
- [ ] 100% Pure Rust goal (6-12 months)

### **🔮 Future**
- [ ] More panels (web, video, terminal)
- [ ] Performance budgets
- [ ] Hot reloading
- [ ] Multi-window support

---

## 🛠️ **Development**

### **Prerequisites**
```bash
# Rust 1.75+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Optional: ARM64 target
rustup target add aarch64-unknown-linux-musl
```

### **Building**
```bash
# Debug build
cargo build

# Release build (all modes)
cargo build --release

# Pure Rust build (no GUI)
cargo build --release --no-default-features

# ARM64 build
cargo build --release --target aarch64-unknown-linux-musl
```

### **Testing**
```bash
# All tests (16 passing in 0.00s!)
cargo test

# Specific binary
cargo test --bin petaltongue

# With coverage
cargo llvm-cov --html
```

### **Running**
```bash
# Development
cargo run -- status

# Release
./target/release/petaltongue status --verbose
```

---

## 🤝 **Contributing**

Follow TRUE PRIMAL principles:
- ✅ Discover capabilities, don't hardcode
- ✅ Pure Rust, modern idioms
- ✅ Concurrent, no sleeps
- ✅ Comprehensive testing
- ✅ Clear documentation

See recent commits for examples.

---

## 🎓 **Philosophy**

> "it's a successfully fail" → "spectacular success!"

**Test-Driven Evolution**:
1. Build minimal
2. Discover gaps
3. Solve systematically
4. Document learnings
5. Repeat

Architecture emerges from reality, not speculation.

**ecoBud** (Now): Pragmatic, ships today  
**ecoBlossom** (Future): Aspirational, evolves over time

---

## 📄 **License**

See ecoPrimals project for licensing details.

---

## 🌟 **Quick Links**

- **Get Started**: [START_HERE.md](./START_HERE.md)
- **Current Status**: [PROJECT_STATUS.md](./PROJECT_STATUS.md)
- **UniBin Details**: [ECOBUD_PHASE_1_COMPLETE.md](./ECOBUD_PHASE_1_COMPLETE.md)
- **Future Vision**: [ECOBLOSSOM_PHASE_2_PLAN.md](./ECOBLOSSOM_PHASE_2_PLAN.md)

---

**Version**: 1.3.0 (ecoBud)  
**Status**: ✅ UniBin Complete, Production Ready  
**Updated**: January 19, 2026

🌸 **From 3 binaries to 1 - petalTongue evolved!** 🚀
