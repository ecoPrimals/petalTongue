# 🗺️ petalTongue Navigation

**Quick reference for finding your way around the codebase**

---

## 📦 **Root Structure**

```
petalTongue/
├── src/                    # UniBin entry point
│   ├── main.rs            # Main entry, subcommand routing
│   ├── ui_mode.rs         # Desktop GUI mode
│   ├── tui_mode.rs        # Terminal UI mode (Pure Rust!)
│   ├── web_mode.rs        # Web server mode (Pure Rust!)
│   ├── headless_mode.rs   # Headless mode (Pure Rust!)
│   └── cli_mode.rs        # Status command (Pure Rust!)
│
├── crates/                 # Workspace crates
│   ├── petal-tongue-ui/          # Desktop UI (egui)
│   ├── petal-tongue-tui/         # Terminal UI (ratatui)
│   ├── petal-tongue-core/        # Core types & logic
│   ├── petal-tongue-discovery/   # Primal discovery
│   ├── petal-tongue-graph/       # Graph data structures
│   ├── petal-tongue-ui-core/     # UI abstractions
│   └── ...                       # Additional crates
│
├── web/                    # Web frontend
│   └── index.html         # Modern responsive UI
│
├── sandbox/                # Demo scenarios
│   └── scenarios/         # JSON scenario configs
│
├── specs/                  # Technical specifications
├── docs/                   # Detailed documentation
├── archive/                # Historical documentation
├── tests/                  # Integration tests
└── *.md                    # Root documentation
```

---

## 📚 **Essential Documents**

### **Getting Started**
- `README.md` - Project overview
- `START_HERE.md` - Setup guide
- `DEPLOYMENT_READY.md` - Deploy guide

### **Current Status**
- `PROJECT_STATUS.md` - Health & metrics
- `CHANGELOG.md` - Version history

### **UniBin Evolution**
- `ECOBUD_PHASE_1_COMPLETE.md` - Implementation
- `ECOBLOSSOM_PHASE_2_PLAN.md` - Future vision
- `UNIBIN_EVOLUTION_COMPLETE_JAN_19_2026.md` - Summary

---

## 🗂️ **Crate Purposes**

| Crate | Purpose | Pure Rust? |
|-------|---------|------------|
| `petal-tongue-ui` | Desktop GUI (egui) | ⚠️ No |
| `petal-tongue-tui` | Terminal UI (ratatui) | ✅ Yes |
| `petal-tongue-core` | Core types, traits | ✅ Yes |
| `petal-tongue-discovery` | Primal discovery | ✅ Yes |
| `petal-tongue-graph` | Graph engine | ✅ Yes |
| `petal-tongue-ui-core` | UI abstractions | ✅ Yes |
| `petal-tongue-adapters` | External integrations | ✅ Yes |
| `petal-tongue-ipc` | Inter-process comm | ✅ Yes |
| `petal-tongue-entropy` | Randomness/chaos | ✅ Yes |
| `doom-core` | Doom integration | ✅ Yes |

---

## 🎯 **Common Tasks**

### **Run petalTongue**
```bash
# From root
./target/release/petaltongue <mode>

# Or with cargo
cargo run --bin petaltongue -- <mode>
```

### **Build**
```bash
# Release build
cargo build --release

# Pure Rust build (no GUI)
cargo build --release --no-default-features
```

### **Test**
```bash
# All tests
cargo test

# UniBin tests
cargo test --bin petaltongue

# Specific crate
cargo test --package petal-tongue-core
```

### **Add a new mode**
1. Create `src/my_mode.rs`
2. Add to `src/main.rs` enum
3. Implement handler
4. Add tests

---

## 🔍 **Finding Code**

### **UI Rendering**
→ `crates/petal-tongue-ui/src/`
→ `crates/petal-tongue-ui-core/src/`

### **Graph Logic**
→ `crates/petal-tongue-graph/src/`
→ `crates/petal-tongue-core/src/graph.rs`

### **Discovery**
→ `crates/petal-tongue-discovery/src/`

### **Doom Integration**
→ `crates/petal-tongue-ui/src/panels/doom_panel.rs`
→ `crates/doom-core/src/`

### **Web Server**
→ `src/web_mode.rs`
→ `web/index.html`

---

## 📖 **Documentation Locations**

### **Root**
Current, essential documentation

### **specs/**
Technical specifications:
- Architecture
- Design decisions
- Specifications

### **docs/**
Detailed documentation:
- `architecture/` - System design
- `features/` - Feature docs
- `guides/` - How-to guides
- `operations/` - Deployment
- `sessions/` - Dev sessions

### **archive/**
Historical documentation:
- Completed sessions
- Evolution history
- Fossil record

---

## 🚀 **Quick Links**

| What | Where |
|------|-------|
| Main entry | `src/main.rs` |
| UI mode | `src/ui_mode.rs` |
| Web mode | `src/web_mode.rs` |
| Core types | `crates/petal-tongue-core/src/lib.rs` |
| Tests | `tests/` + `*/tests/` |
| Scenarios | `sandbox/scenarios/` |
| Web UI | `web/index.html` |
| Docs | Root `*.md` + `docs/` |
| Specs | `specs/` |

---

**Last Updated**: January 19, 2026  
**Version**: 1.3.0 (ecoBud)

🌸 Navigate with confidence! 🚀
