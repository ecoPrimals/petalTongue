# 🌸 petalTongue — Start Here

Welcome to petalTongue, the Universal UI and Visualization System for ecoPrimals!

---

## 🎯 What is petalTongue?

petalTongue is the **visualization primal** that shows how ecoPrimals interact. It provides:

- **Interactive graph** of primal topology
- **Real-time flow animation** of messages between primals
- **Multi-view dashboard** (topology, timeline, traffic, health)
- **REST + WebSocket API** for multiple consumers

**Name Origin**: "petal" (delicate, visual) + "tongue" (speaks/tastes ecosystem state)

---

## 🚀 Quick Start

### 1. Prerequisites

- Rust 1.75+ (edition 2024)
- cargo
- Linux/macOS/Windows

### 2. Build

```bash
cd petalTongue
cargo build
```

### 3. Run Tests

```bash
cargo test
```

### 4. Run petalTongue

```bash
# Start the visualization UI
cargo run --release

# Or run a specific crate
cargo run -p petal-tongue-ui
```

---

## 📁 Project Structure

```
petalTongue/
├── crates/
│   ├── petal-tongue-core/        # Core traits, types, config
│   ├── petal-tongue-graph/       # Graph rendering engine
│   ├── petal-tongue-animation/   # Flow animation system
│   ├── petal-tongue-telemetry/   # Event streaming and integration
│   ├── petal-tongue-api/         # REST + WebSocket API server
│   └── petal-tongue-ui/          # UI components (egui-based)
├── specs/                         # Specifications
├── showcase/                      # Demo applications
├── README.md                      # Overview
├── STATUS.md                      # Current status
├── WHATS_NEXT.md                 # Roadmap
└── START_HERE.md                 # This file
```

---

## 🧱 Architecture

### Core Concepts

1. **Graph Model**
   - Nodes = Primals (BearDog, ToadStool, Songbird, etc.)
   - Edges = Connections (API calls, capability relationships)
   - Layout = Force-directed, hierarchical, or circular

2. **Animation**
   - Particles = Messages flowing between primals
   - Color = Message type
   - Speed = Traffic volume

3. **Telemetry**
   - Event streams from primals
   - Discovery events (new primal found)
   - Health updates (primal status changed)
   - Traffic events (API call made)

4. **API**
   - REST endpoints for topology queries
   - WebSocket streams for live updates
   - BearDog authentication

### Data Flow

```
┌─────────────┐
│  biomeOS    │────────┐
│  (client)   │        │
└─────────────┘        │
                       ▼
┌─────────────┐   ┌─────────────────┐   ┌─────────────┐
│   Songbird  │──▶│  petalTongue    │──▶│   Display   │
│ (discovery) │   │  (visualization)│   │  (egui UI)  │
└─────────────┘   └─────────────────┘   └─────────────┘
                       ▲
                       │
┌─────────────┐        │
│  ToadStool  │────────┘
│  (events)   │
└─────────────┘
```

---

## 🛠️ Development Workflow

### Adding a New Feature

1. **Write specification** in `specs/`
2. **Implement in appropriate crate**:
   - Core logic → `petal-tongue-core/`
   - Graph rendering → `petal-tongue-graph/`
   - Animation → `petal-tongue-animation/`
   - Telemetry → `petal-tongue-telemetry/`
   - API → `petal-tongue-api/`
   - UI → `petal-tongue-ui/`
3. **Add tests** (aim for 80%+ coverage)
4. **Update documentation**
5. **Run linting**:
   ```bash
   cargo clippy --all -- -D warnings
   cargo fmt --all
   ```

### Running Specific Tests

```bash
# All tests
cargo test --all

# Specific crate
cargo test -p petal-tongue-graph

# Specific test
cargo test test_force_directed_layout
```

### Building Documentation

```bash
cargo doc --open
```

---

## 🔍 Key Files to Explore

### 1. Core Types (`petal-tongue-core/src/`)
- `lib.rs` - Main primal struct and traits
- `config.rs` - Configuration management
- `error.rs` - Error types

### 2. Graph Rendering (`petal-tongue-graph/src/`)
- `lib.rs` - Graph rendering logic (TODO)
- `nodes.rs` - Node rendering (TODO)
- `edges.rs` - Edge rendering (TODO)
- `layout.rs` - Layout algorithms (TODO)

### 3. UI Components (`petal-tongue-ui/src/`)
- `lib.rs` - UI application (TODO)
- `topology.rs` - Topology view (TODO)
- `timeline.rs` - Timeline view (TODO)

### 4. API (`petal-tongue-api/src/`)
- `lib.rs` - API server (TODO)
- `rest.rs` - REST endpoints (TODO)
- `websocket.rs` - WebSocket streams (TODO)

---

## 🧪 Testing Strategy

### Unit Tests
- Individual functions and methods
- Mock external dependencies
- Fast execution

### Integration Tests
- Cross-crate functionality
- Real telemetry integration
- Songbird discovery integration

### E2E Tests
- Full visualization pipeline
- Multi-primal scenarios
- Performance benchmarks

### Visual Regression Tests (Future)
- Screenshot comparison
- Layout consistency
- Animation correctness

---

## 📚 Key Documents

| Document | Purpose |
|----------|---------|
| [README.md](./README.md) | Project overview and quick start |
| [STATUS.md](./STATUS.md) | Current implementation status |
| [WHATS_NEXT.md](./WHATS_NEXT.md) | Roadmap and future plans |
| [specs/PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md](./specs/PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md) | Full specification (50KB) |

---

## 🌐 Integration with Ecosystem

### BearDog (Security & Identity)
- **What**: Cryptographic identity and signatures
- **Usage**: API authentication, primal verification
- **Status**: To be integrated

### Songbird (Discovery)
- **What**: Service discovery and capability announcement
- **Usage**: Discover primals to visualize
- **Status**: To be integrated

### ToadStool (Compute & Events)
- **What**: Compute execution and event streaming
- **Usage**: Event source for real-time updates
- **Status**: To be integrated

### biomeOS (Orchestration)
- **What**: Primal orchestration and chimera mixing
- **Usage**: Primary consumer of petalTongue
- **Status**: Will use petalTongue as client

---

## 💡 Tips for New Contributors

### First Contribution Ideas
1. Add unit tests for existing code
2. Implement a missing layout algorithm
3. Add a new view to the dashboard
4. Improve documentation
5. Fix clippy warnings

### Code Style
- Follow Rust edition 2024 idioms
- Use `clippy::pedantic` lints
- Document all public APIs
- Write tests for new features
- Use `tracing` for logging, not `println!`

### Getting Help
- Read the specification: `specs/PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md`
- Check existing implementations in mature primals (BearDog, ToadStool)
- Look at SourDough conventions: `../sourDough/CONVENTIONS.md`

---

## 🎨 UI Development

### egui Basics

petalTongue uses **egui** (immediate mode GUI):

```rust
use egui::*;

pub fn render_topology(ui: &mut Ui, graph: &TopologyGraph) {
    ui.heading("Primal Topology");
    
    // Render graph
    for node in &graph.nodes {
        ui.horizontal(|ui| {
            ui.label(&node.name);
            ui.label(format!("Health: {}", node.health));
        });
    }
}
```

### Adding a New View

1. Create `petal-tongue-ui/src/views/my_view.rs`
2. Implement the view struct and `render()` method
3. Add to `petal-tongue-ui/src/views/mod.rs`
4. Integrate into main app

---

## 🐛 Debugging

### Common Issues

**Issue**: UI not rendering
- **Fix**: Check `cargo run` output for errors
- **Fix**: Ensure eframe is initialized correctly

**Issue**: Graph layout looks wrong
- **Fix**: Adjust layout algorithm parameters
- **Fix**: Check node/edge data structure

**Issue**: Live updates not working
- **Fix**: Verify telemetry integration
- **Fix**: Check WebSocket connection

### Debugging Tools

```bash
# Run with tracing enabled
RUST_LOG=petal_tongue=debug cargo run

# Run with backtrace
RUST_BACKTRACE=1 cargo run

# Run tests with output
cargo test -- --nocapture
```

---

## 🚀 Next Steps

1. **Read the specification**: `specs/PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md`
2. **Check current status**: `STATUS.md`
3. **See roadmap**: `WHATS_NEXT.md`
4. **Build and run**: `cargo build && cargo run`
5. **Pick a task**: Check `WHATS_NEXT.md` for Phase 1 tasks

---

## 📞 Contact & Contributions

- **Team**: ecoPrimals petalTongue team
- **Repository**: https://github.com/ecoPrimals/petalTongue (future)
- **License**: AGPL-3.0
- **Contributions**: Welcome! Follow the conventions in `../sourDough/CONVENTIONS.md`

---

*petalTongue: Let's visualize the ecosystem together! 🌸*

