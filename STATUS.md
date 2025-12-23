# 🌸 petalTongue — Project Status

**Last Updated**: December 23, 2025  
**Version**: 0.1.0  
**Status**: 🌱 **Scaffolded** — Ready for Implementation  
**Grade**: N/A (Pre-implementation)

---

## 📊 Current State

### Build Status
| Metric | Status |
|--------|--------|
| **Compilation** | ✅ Clean (scaffolded) |
| **Tests** | ⬜ 0/0 (not yet implemented) |
| **Linting** | ✅ Clean (pedantic clippy) |
| **Documentation** | 🟡 Scaffold + specification only |

### Implementation Progress

| Component | Status | Notes |
|-----------|--------|-------|
| **Core Traits** | ✅ Scaffolded | Basic `PrimalLifecycle`, `PrimalHealth` |
| **Configuration** | ✅ Scaffolded | Basic `PetalTongueConfig` |
| **Error Types** | ✅ Scaffolded | Basic `PetalTongueError` |
| **Graph Rendering** | ⬜ Not Started | egui_graphs integration |
| **Animation System** | ⬜ Not Started | Flow visualization |
| **Telemetry** | ⬜ Not Started | Event streaming |
| **API Server** | ⬜ Not Started | REST + WebSocket |
| **UI Components** | 🟡 Partial | Moving from biomeOS |

---

## 🎯 What petalTongue Does

petalTongue is the **Universal Representation System** — translates ecosystem state into any modality a human can perceive:

```
┌─────────────────────────────────────────────────────────────────┐
│                        petalTongue                               │
│           (Universal Representation System)                      │
│                                                                  │
│  Any Input → Any Output → Any Human                             │
│                                                                  │
│  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐      │
│  │ Visual  │  │  Audio   │  │  Haptic  │  │    AI       │      │
│  │ (2D/3D) │  │(Sonify)  │  │(Vibrate) │  │ (Narrate)   │      │
│  └─────────┘  └──────────┘  └──────────┘  └─────────────┘      │
│                                                                  │
│  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐      │
│  │  VR/AR  │  │  Voice   │  │  Screen  │  │ Projection  │      │
│  │(Spatial)│  │(Control) │  │  Reader  │  │(Planetarium)│      │
│  └─────────┘  └──────────┘  └──────────┘  └─────────────┘      │
└─────────────────────────────────────────────────────────────────┘
```

**Revolutionary Concepts**:
- **Multi-modal** — Visual, audio, haptic, spatial, AI-narrated
- **Universal accessibility** — Works equally well for blind, deaf, any user
- **AI-first** — Intelligent translation of topology to human perception
- **Environment-aware** — Adapts to desktop, VR, planetarium, terminal
- **Human dignity** — Celebrates diversity, not just "accommodates" disability

---

## 📁 Project Structure

```
petalTongue/
├── Cargo.toml                     # Workspace manifest
├── README.md                      # Project overview
├── STATUS.md                      # This file
├── WHATS_NEXT.md                 # Roadmap
├── START_HERE.md                 # Developer guide
├── crates/
│   ├── petal-tongue-core/        # Core traits, types
│   ├── petal-tongue-graph/       # Graph rendering engine
│   ├── petal-tongue-animation/   # Flow animation system
│   ├── petal-tongue-telemetry/   # Event streaming
│   ├── petal-tongue-api/         # REST + WebSocket API
│   └── petal-tongue-ui/          # UI components (egui)
└── specs/
    └── PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md
```

---

## 🔗 Dependencies

### Gen 1 Primals (Required)
| Primal | Purpose | Status |
|--------|---------|--------|
| **BearDog** | DIDs, Signatures | ✅ Ready |
| **Songbird** | Service Discovery | ✅ Ready |
| **ToadStool** | Event Source | ✅ Ready |
| **NestGate** | Optional Storage | ✅ Ready |

### Phase 2 Integration
| Component | Relationship | Status |
|-----------|--------------|--------|
| **biomeOS** | Primary consumer | ✅ Ready (will become client) |
| **RhizoCrypt** | Optional DAG queries | 🌱 Scaffolded |

---

## 📈 Metrics

```
Lines of Code:       ~200 (scaffold)
Test Coverage:       0% (no tests yet)
Unsafe Blocks:       0
Files:               15 source files
Dependencies:        egui, egui_graphs, petgraph, sourdough-core
```

---

## 🚀 Next Milestone

**Phase 1: Core Architecture + UI Migration** (Target: Week 1)

1. Move UI code from biomeOS to petalTongue
2. Set up egui_graphs integration
3. Basic graph rendering
4. Compile and run

See [WHATS_NEXT.md](./WHATS_NEXT.md) for full roadmap.

---

## 📚 Key Documents

| Document | Purpose |
|----------|---------|
| [README.md](./README.md) | Project overview |
| [START_HERE.md](./START_HERE.md) | Developer onboarding |
| [WHATS_NEXT.md](./WHATS_NEXT.md) | Implementation roadmap |
| [specs/PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md](./specs/PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md) | Full specification |

---

## 🌟 Origin Story

petalTongue was born from biomeOS's UI layer. As the visualization needs grew complex and multiple consumers emerged, it became clear that visualization deserved to be its own primal.

**Name Origin**: "petal" (delicate, visual) + "tongue" (speaks/tastes ecosystem state)

**Migration**: December 23, 2025 - Extracted from biomeOS, scaffolded as independent primal.

---

*petalTongue: The visual tongue that speaks the ecosystem's story.*

