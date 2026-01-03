# 🌸 petalTongue - Universal Primal Visualization Engine

**Status**: Production-Ready Foundations | Multi-Instance Architecture Complete  
**Version**: 0.1.0  
**License**: AGPL-3.0  

---

## 🎯 Overview

petalTongue is a **universal, multimodal, and accessible** user interface for the ecoPrimals ecosystem. It provides real-time visualization, monitoring, and interaction with distributed primal networks through an extensible adapter-based architecture.

### Key Features

✅ **Universal Visualization Engine**
- Adapter-based rendering (ecosystem-agnostic)
- Real-time graph topology with force-directed layout
- Trust visualization (0-3: None, Limited, Elevated, Full)
- Family relationship display with genetic lineage
- Capability badges for primal features

✅ **Multi-Instance Architecture** *(New: Jan 3, 2026)*
- UUID-based instance tracking and management
- File-backed registry with automatic cleanup
- Complete state persistence (never lose work)
- Auto-save every 30 seconds with crash-safe writes
- Inter-process communication via Unix domain sockets
- CLI tools for instance management

✅ **Multimodal Data Stream Sonification**
- Audio representation of live data feeds
- Adaptive and simultaneous modalities
- Accessible audio alternatives

✅ **Full Accessibility**
- Multiple color schemes
- Configurable font sizes
- Keyboard shortcuts
- Screen reader support
- Audio-first alternatives

✅ **Live Data Integration**
- BiomeOS API integration
- mDNS primal discovery
- Real-time health monitoring
- Sandbox scenarios for demonstration

---

## 🏗️ Architecture

### Core Components

```
petalTongue/
├── crates/
│   ├── petal-tongue-core/      # Core types, graph engine, instance mgmt
│   ├── petal-tongue-ui/         # Main UI application (eframe/egui)
│   ├── petal-tongue-api/        # BiomeOS API client
│   ├── petal-tongue-discovery/  # Primal discovery (mDNS, HTTP, mock)
│   ├── petal-tongue-audio/      # Audio system & sonification
│   ├── petal-tongue-entropy/    # Human entropy integration
│   ├── petal-tongue-adapters/   # Ecosystem-specific adapters
│   ├── petal-tongue-ipc/        # Inter-process communication (NEW)
│   └── petal-tongue-cli/        # CLI management tools (NEW)
```

### Multi-Instance System *(New)*

**Instance Management**:
- Each petalTongue instance has a unique UUID
- Registry tracks all instances: `~/.local/share/petaltongue/instances.ron`
- Process liveness checking via Unix signals
- Automatic garbage collection of dead instances

**State Persistence**:
- Complete application state capture (graph, UI, settings)
- Sessions saved: `~/.local/share/petaltongue/sessions/{uuid}.ron`
- Auto-save every 30 seconds + on significant changes
- Atomic writes (crash-safe)
- Export/import for machine transfer
- Merge operations for combining sessions

**IPC Layer**:
- Unix domain socket server per instance
- Sockets: `/tmp/petaltongue/{uuid}.sock` or `/run/user/{uid}/petaltongue/{uuid}.sock`
- Commands: GetStatus, TransferState, Show, Hide, Shutdown
- CLI tool for remote management

---

## 🚀 Getting Started

### Prerequisites

- Rust 1.75+ (2024 edition)
- Linux (X11 or Wayland)
- Audio support (optional)

### Building

```bash
# Build all components
cargo build --release

# Build main UI
cargo build --release --bin petal-tongue

# Build CLI tool
cargo build --release --bin petaltongue
```

### Running

```bash
# Launch with live data
./target/release/petal-tongue

# Launch with sandbox scenario
SANDBOX_SCENARIO=trust-demo ./target/release/petal-tongue

# Launch in showcase mode
SHOWCASE_MODE=true ./target/release/petal-tongue
```

### CLI Management

```bash
# List all running instances
./target/release/petaltongue list

# Show instance details
./target/release/petaltongue show <instance-id>

# Bring window to front
./target/release/petaltongue raise <instance-id>

# Check instance responsiveness
./target/release/petaltongue ping <instance-id>

# Clean up dead instances
./target/release/petaltongue gc --force

# Status summary
./target/release/petaltongue status
```

---

## 🎨 UI Controls

### Navigation
- **Mouse Drag**: Pan the graph
- **Scroll**: Zoom in/out
- **Click Node**: Select and highlight
- **Shift+T**: Toggle trust dashboard

### Layout Algorithms
- Force-Directed (physics-based)
- Hierarchical (tree-like)
- Circular (nodes in circle)

### Panels
- **Left**: Discovery controls, layout selection
- **Right**: Trust dashboard, filters
- **Bottom**: Status, help

---

## 📊 Sandbox Scenarios

petalTongue includes demonstration scenarios for testing and showcasing features:

**Available Scenarios**:
- `trust-demo` - Trust visualization with 4 trust levels
- `complex` - Large network (50+ nodes)
- `simple` - Basic network (5 nodes)

**Location**: `sandbox/*.json`

**Usage**:
```bash
SANDBOX_SCENARIO=trust-demo ./target/release/petal-tongue
```

---

## 🔧 Configuration

### Environment Variables

- `SHOWCASE_MODE=true` - Enable showcase mode
- `SANDBOX_SCENARIO=<name>` - Load sandbox scenario
- `RUST_LOG=debug` - Enable debug logging
- `XDG_DATA_HOME` - Override data directory (default: `~/.local/share`)

### Data Directories (XDG-compliant)

```
~/.local/share/petaltongue/
├── instances.ron         # Instance registry
└── sessions/            # Session state files
    └── {uuid}.ron
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test --package petal-tongue-core
cargo test --package petal-tongue-ipc

# Run with output
cargo test -- --nocapture
```

---

## 📚 Documentation

### User Documentation
- **DEMO_GUIDE.md** - Complete user guide with examples
- **README.md** - This file

### Technical Documentation
- **STATUS.md** - Current implementation status
- **ARCHITECTURE.md** - System architecture and design
- **DEEP_DEBT_ROADMAP.md** - Development roadmap

### Session Reports (January 3, 2026)
- **FINAL_SESSION_SUMMARY_JAN_3_2026.md** - Executive summary
- **DEEP_DEBT_SESSION_COMPLETE.md** - Comprehensive technical details
- **PHASES_1_2_COMPLETE.md** - Phases 1-2 implementation
- **PHASE_1_COMPLETE.md** - Instance management details
- **INSTANCE_MANAGEMENT_ARCHITECTURE.md** - Architecture analysis

---

## 🛠️ Development

### Project Principles

All code follows these deep debt principles:

1. **Modern Idiomatic Rust** - Zero unsafe code, proper error handling
2. **Smart Refactoring** - Clean modules, appropriate crate boundaries
3. **Self-Knowledge Only** - Each component knows only itself
4. **Runtime Discovery** - No hardcoded assumptions
5. **No Hardcoding** - XDG-compliant, environment-driven
6. **No Mocks in Production** - Real implementations only
7. **Capability-Based** - Extensible, dynamic behavior

### Code Quality

- ✅ Zero unsafe code
- ✅ Comprehensive test coverage
- ✅ Full documentation (>250 inline docs)
- ✅ Clippy-clean
- ✅ Modern async/await with tokio
- ✅ Proper error handling with thiserror

### Recent Work (January 3, 2026)

**Phase 1: Instance Management** ✅
- 650 lines, 6 tests
- UUID-based tracking, file-backed registry
- Process liveness checking, auto cleanup

**Phase 2: State Persistence** ✅
- 750 lines, 4 tests
- Complete state capture, auto-save
- Export/import, merge operations

**Phase 3: IPC Layer** ✅
- 1,050 lines, 5 tests
- Unix socket server/client
- CLI management tool

**Total**: 2,450 lines of production Rust, 15 tests, 8 documentation files

---

## 🌟 Roadmap

### Completed ✅
- Universal adapter-based architecture
- Trust visualization with dashboard
- Multi-instance management system
- Complete state persistence
- IPC infrastructure and CLI tools
- Sandbox scenarios for demonstration

### In Progress 🔨
- Integration of IPC into main application
- Phase 4: Window management & auto-recovery

### Planned 📋
- Advanced filtering and search
- Graph export (GraphML, JSON)
- Plugin system for custom adapters
- Web-based remote UI
- Collaborative multi-user sessions

---

## 🤝 Contributing

petalTongue follows the ecoPrimals development principles:

- **No technical debt** - Clean code from the start
- **Production quality** - Every commit is production-ready
- **Comprehensive testing** - Full test coverage
- **Complete documentation** - Inline docs + guides
- **Capability-based** - Extensible by design

---

## 📄 License

AGPL-3.0 - See LICENSE file for details

---

## 🙏 Acknowledgments

Built with:
- [egui](https://github.com/emilk/egui) - Immediate mode GUI
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - App framework
- [tokio](https://tokio.rs/) - Async runtime
- [serde](https://serde.rs/) - Serialization

---

## 📞 Contact

Part of the ecoPrimals ecosystem  
Repository: https://github.com/ecoPrimals/petalTongue

---

*Last updated: January 3, 2026*  
*Status: Production-Ready Foundations*  
*Version: 0.1.0*

🌸 **Universal. Accessible. Multimodal.** 🚀
