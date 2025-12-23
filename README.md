# 🌸 petalTongue - Universal Representation System

**"Any topology, any modality, any human."**

petalTongue is a revolutionary accessibility-first visualization system for distributed ecosystems. It represents the same information through multiple sensory modalities—visual, audio, haptic, VR/AR—ensuring that sighted, blind, deaf, and neurodiverse users all have equal access to rich system insights.

[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-demo--ready-green.svg)](https://github.com/ecoPrimals/petalTongue)

---

## 🌟 Why petalTongue?

Traditional system monitoring tools assume sighted users. petalTongue reimagines observability:

- **Blind SRE**: Hears healthy primals as harmonic bass and chimes, warnings as off-key notes, critical systems as dissonant alarms
- **Sighted Engineer**: Sees a color-coded graph with interactive zoom, pan, and selection
- **Deaf Analyst**: Uses haptic feedback and visual cues (future)
- **VR Operations Team**: Works in immersive 3D spatial environments (future)

**Both users get identical information, just different sensory channels.**

This isn't accommodation—it's **celebration** of human diversity.

---

## ✨ Features

### Current (Month 1 - ✅ Complete)

- **🎨 Visual Modality**
  - Interactive 2D graph visualization
  - Health-based color coding (green/yellow/red/gray)
  - Pan, zoom, click-to-select
  - Multiple layout algorithms (force-directed, hierarchical, circular)
  - Real-time statistics overlay

- **🎵 Audio Modality**
  - Sonification engine mapping primals to instruments:
    - 🐻 Security (BearDog) → Deep Bass
    - 🍄 Compute (ToadStool) → Rhythmic Drums
    - 🐦 Discovery (Songbird) → Light Chimes
    - 🏠 Storage (NestGate) → Sustained Strings
    - 🐿️ AI (Squirrel) → High Synth
  - Health → Pitch (harmonic/off-key/dissonant)
  - Position → Stereo panning (left/center/right)
  - Activity → Volume
  - AI narration with soundscape descriptions

- **🏗️ Architecture**
  - Modality-agnostic graph engine
  - Clean separation of concerns
  - Extensible for future modalities
  - Production-quality Rust (zero unsafe code)

### Future (Months 2-4)

- **🤚 Haptic**: Vibration patterns for alerts
- **🥽 VR/AR**: Immersive 3D spatial visualization
- **📱 Mobile**: Touch-optimized interface
- **🗣️ Voice**: AI-powered narration with voice
- **♿ Screen Reader**: Rich semantic descriptions
- **🏟️ Planetarium**: Large-scale projection mapping

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (2024 edition)
- Linux, macOS, or Windows

### Installation

```bash
git clone git@github.com:ecoPrimals/petalTongue.git
cd petalTongue
cargo build --release
```

### Run the Demo

```bash
cargo run --release -p petal-tongue-ui
```

This opens a window with:
- **Left Panel**: Controls and health legend
- **Center**: Interactive visual graph
- **Right Panel**: Audio descriptions and instrument mapping

### Try It

1. **Pan**: Drag with mouse
2. **Zoom**: Scroll wheel
3. **Select**: Click a node to see its audio description
4. **Layout**: Switch between Force-Directed, Hierarchical, Circular
5. **Volume**: Adjust master volume slider
6. **Audio**: Toggle audio on/off

---

## 📊 Architecture

petalTongue follows an evolutionary architecture: **Start Concrete → Stabilize → Abstract → Infinite**

```
┌─────────────────────────────────────────┐
│         GraphEngine (Abstract)          │
│  • Nodes, edges, positions             │
│  • Layout algorithms                    │
│  • NO RENDERING KNOWLEDGE               │
└──────────────────┬──────────────────────┘
                   │
         ┌─────────┴─────────┐
         ▼                   ▼
  ┌─────────────┐     ┌─────────────┐
  │   Visual    │     │   Audio     │
  │  Renderer   │     │  Renderer   │
  │             │     │             │
  │ Nodes →     │     │ Nodes →     │
  │ Circles     │     │ Instruments │
  │             │     │             │
  │ Health →    │     │ Health →    │
  │ Colors      │     │ Pitch       │
  │             │     │             │
  │ Position →  │     │ Position →  │
  │ Screen XY   │     │ Stereo Pan  │
  └─────────────┘     └─────────────┘
         ▲                   ▲
         └───────────┬───────┘
                     │
         ┌───────────┴────────────┐
         │   UI Application       │
         │  (Integrates Both)     │
         └────────────────────────┘
```

**Key Insight**: The graph engine knows **nothing** about rendering. This allows us to add new modalities (haptic, VR, AR) without changing the core.

---

## 🧬 Project Structure

```
petalTongue/
├── crates/
│   ├── petal-tongue-core/      # Core types, graph engine (450+ LOC)
│   ├── petal-tongue-graph/     # Visual & audio renderers (700+ LOC)
│   ├── petal-tongue-ui/        # Desktop application (350+ LOC)
│   ├── petal-tongue-animation/ # Flow animation (future)
│   ├── petal-tongue-telemetry/ # Event streaming (future)
│   └── petal-tongue-api/       # REST + WebSocket API (future)
├── specs/                       # Technical specifications
├── VISION_SUMMARY.md           # 5-minute vision overview
├── EVOLUTION_PLAN.md           # 4-month phased roadmap
├── UNIVERSAL_UI_EVOLUTION.md   # Full vision (10K words)
└── STATUS.md                   # Current progress
```

---

## 🎯 Use Cases

### Scenario 1: Blind SRE Monitoring Production

**Morning standup**:
```
SRE (using petalTongue):
"I hear the ecosystem is mostly harmonic—bass is steady (security),
drums are rhythmic (compute), chimes are clear (discovery).

But there's a slightly off-key note in the drums. Let me check...
[clicks ToadStool node]

'ToadStool Compute: Warning status, volume at 70%, centered position.'

Ah, ToadStool has a warning. I'll investigate."
```

**Same information as sighted engineer, different modality.**

### Scenario 2: Conference Presentation

**Planetarium mode** (future):
```
Presenter: "Let's visualize our distributed system at scale."
[Projects petalTongue graph onto planetarium dome]
Audience: Sees ecosystem topology in immersive 360° view
          Hears sonification as ambient soundscape
          Experiences system health as multi-sensory environment
```

### Scenario 3: Accessibility Training

**Diversity workshop**:
```
Instructor: "Experience how a blind engineer monitors systems."
[Attendees close eyes, listen to petalTongue sonification]
Attendees: "Wow, I can actually 'hear' which services are unhealthy!"
          "This is more intuitive than I expected."
          "Why don't all tools work this way?"
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test --all

# Run with coverage
cargo llvm-cov --all --lcov --output-path lcov.info

# Run specific crate tests
cargo test -p petal-tongue-core
cargo test -p petal-tongue-graph
```

**Current Coverage**: 24 unit tests (100% passing)

---

## 🛠️ Development

### Dependencies

- `eframe`/`egui` - UI framework
- `egui_graphs` - Graph visualization
- `petgraph` - Graph data structures
- `tokio` - Async runtime
- `sourdough-core` - ecoPrimals common traits

### Code Quality

```bash
# Lint (pedantic)
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all

# Documentation
cargo doc --no-deps --open
```

**Standards**:
- Zero unsafe code
- Pedantic clippy compliance
- Comprehensive documentation
- Test-driven development

---

## 📈 Roadmap

See [EVOLUTION_PLAN.md](./EVOLUTION_PLAN.md) for the full 4-month roadmap.

### Month 1: Concrete Systems ✅ **COMPLETE**
- [x] Graph engine
- [x] Visual renderer
- [x] Audio renderer
- [x] UI integration
- [x] Demo ready

### Month 2: Stabilization & Polish (Current)
- [ ] Refine rendering quality
- [ ] Performance optimization (large graphs)
- [ ] Integration tests
- [ ] User feedback

### Month 3: Abstraction
- [ ] Define `RepresentationModality` trait
- [ ] Refactor renderers to use trait
- [ ] Prepare for new modalities

### Month 4+: Expansion
- [ ] Haptic renderer
- [ ] VR/AR renderer
- [ ] Voice/screen reader
- [ ] Real-time live data
- [ ] BiomeOS integration

---

## 🌍 Impact

petalTongue aims to:

- **Open careers** to blind engineers in DevOps and SRE roles
- **Set new standards** for inclusive design in observability tools
- **Prove** that accessibility can be elegant, not an afterthought
- **Inspire** the industry to rethink "visualization" as "representation"
- **Celebrate** human diversity as strength, not limitation

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [00_START_HERE.md](./00_START_HERE.md) | Navigation hub |
| [VISION_SUMMARY.md](./VISION_SUMMARY.md) | 5-minute overview |
| [UNIVERSAL_UI_EVOLUTION.md](./UNIVERSAL_UI_EVOLUTION.md) | Full vision (10K words) |
| [EVOLUTION_PLAN.md](./EVOLUTION_PLAN.md) | 4-month roadmap |
| [STATUS.md](./STATUS.md) | Current progress |
| [specs/](./specs/) | Technical specifications |

---

## 🤝 Contributing

petalTongue is part of the [ecoPrimals](https://github.com/ecoPrimals) ecosystem. We welcome contributions!

### Guidelines

1. **Accessibility First**: Every feature must work for blind, deaf, and sighted users
2. **Zero Unsafe**: Production code must be safe Rust
3. **Test Coverage**: Aim for 90%+ coverage
4. **Documentation**: Every public item must be documented
5. **Human Dignity**: Technology that celebrates diversity

See [START_HERE.md](./START_HERE.md) for developer onboarding.

---

## 📜 License

AGPL-3.0 - See [LICENSE](../LICENSE) for details.

---

## 🙏 Acknowledgments

petalTongue was born from the [biomeOS](../biomeOS/) project and scaffolded using [sourDough](../sourDough/), the ecoPrimals starter culture.

**Name Origin**: "petal" (delicate, visual) + "tongue" (speaks/tastes ecosystem state)

---

## 📬 Contact

- **Repository**: https://github.com/ecoPrimals/petalTongue
- **Parent Ecosystem**: https://github.com/ecoPrimals
- **Issues**: https://github.com/ecoPrimals/petalTongue/issues

---

*"Any topology, any modality, any human."* 🌸

**petalTongue is not just a UI—it's a new way of experiencing distributed systems.**
