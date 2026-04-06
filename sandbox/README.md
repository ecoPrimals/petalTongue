# 🌸 petalTongue Sandbox - benchTop Demonstration

**The ecoPrimals Signature Desktop Environment**

---

## 🎯 Purpose

This sandbox demonstrates **petalTongue as benchTop** - the smooth, modern desktop environment for ecoPrimals. Think popOS cosmic meets steamOS meets Discord, with Rust ownership and live evolution.

### What is benchTop?

**benchTop** is the ecoPrimals signature desktop interface:
- **Smooth**: 60+ FPS, buttery animations, instant response
- **Intuitive**: Click, drag, explore - no manuals needed
- **Adaptive**: Learns from usage, evolves over time
- **Beautiful**: Modern design, thoughtful UX, delightful interactions
- **Powerful**: Full system visibility and control

---

## 🎨 Demonstration Scenarios

### 1. **Live Ecosystem** (`scenarios/live-ecosystem.json`)
**Showcase**: Real-time primal topology with Neural API coordination

**What You See**:
- Animated primal nodes (breathing effect)
- Live connection pulses (data flow)
- Health-based coloring (green → yellow → red)
- CPU/memory sparklines
- Adaptive layout (force-directed graph)
- Capability badges
- Click-to-inspect details

**Neural API Integration**:
- Real-time proprioception data
- System metrics dashboard
- Learning pattern visualization
- Adaptive pathway optimization

**User Experience**:
```
Open petalTongue → See living ecosystem
Click BearDog → View security metrics
Click Songbird → See discovery patterns
Press M → Metrics dashboard slides in
Press P → Proprioception panel appears
Press G → Graph builder opens
```

### 2. **Graph Builder Studio** (`scenarios/graph-studio.json`)
**Showcase**: Visual graph construction with Neural API execution

**What You See**:
- Three-panel layout (Palette | Canvas | Properties)
- Drag-and-drop node creation
- Smooth Bézier edges
- Real-time validation
- Execution monitoring
- Save/load from Neural API

**Key Features**:
- Node palette with search
- Grid snapping
- Multi-selection
- Parameter validation
- Execution status
- Error visualization

**User Experience**:
```
Press G → Graph Builder opens
Drag "PrimalStart" → Drops on canvas
Drag "Verification" → Connect with edge
Edit properties → Real-time validation
Click "Execute" → Watch graph run
See results → Animated feedback
```

### 3. **RootPulse Visualization** (`scenarios/rootpulse-demo.json`)
**Showcase**: Temporal coordination and version control visualization

**What You See**:
- DAG timeline (rhizoCrypt - branching possibilities)
- Linear history (LoamSpine - immutable past)
- Commit graph with SweetGrass attribution
- Real-time collaboration
- Dehydration animation (temporal collapse)
- Multi-anchor ordering

**Temporal Architecture**:
- **Present/Future**: DAG with branching
- **Past**: Linear with anchors
- **Dehydration**: Flexible timescales
- **Attribution**: Semantic contributions

**User Experience**:
```
Open RootPulse view → See DAG timeline
Watch new commit → Branch animation
Click commit → See contributors (SweetGrass)
View dehydration → Time collapse animation
See anchors → Multiple orderings
```

---

## Interactive Demonstrations

### **benchTop Experience** (`demo-benchtop.sh`)

```bash
./sandbox/demo-benchtop.sh
```

---

## 🎨 Design Language

### Visual Style

**Color Palette**:
```
Background:    #1e1e2e (dark blue-gray)
Accent:        #89b4fa (blue)
Success:       #a6e3a1 (green)
Warning:       #f9e2af (yellow)
Error:         #f38ba8 (red)
Text:          #cdd6f4 (light gray)
Text Dim:      #6c7086 (medium gray)
```

**Typography**:
- **Primary**: Inter (modern, clean)
- **Monospace**: JetBrains Mono (code, terminal)
- **Sizes**: 16px base, 1.5 line height

**Spacing**:
- **Small**: 8px
- **Medium**: 16px
- **Large**: 24px
- **XLarge**: 32px

### Animation Principles

**Timing**:
- **Quick**: 150ms (hover, click)
- **Standard**: 300ms (panel open/close)
- **Slow**: 500ms (page transitions)
- **Celebration**: 1000ms (success animations)

**Easing**:
- **In**: Ease-in (acceleration)
- **Out**: Ease-out (deceleration)
- **InOut**: Ease-in-out (smooth)
- **Spring**: Elastic (playful)

**Patterns**:
- **Fade**: Opacity transitions
- **Slide**: Position transitions
- **Scale**: Size transitions
- **Pulse**: Breathing effects
- **Flow**: Data movement

---

## 🚀 Running Demonstrations

### Quick Start

**1. Build petalTongue**:
```bash
cd petalTongue  # from your ecoPrimals workspace
cargo build --release
```

**2. Run benchTop Demo**:
```bash
./sandbox/demo-benchtop.sh
```

**3. Explore Features**:
- Press `H` - Help overlay
- Press `P` - Proprioception
- Press `M` - Metrics
- Press `G` - Graph Builder
- Click nodes - Inspect details
- Drag canvas - Pan view
- Scroll - Zoom

### Individual Scenarios

**Live Ecosystem**:
```bash
./target/release/petaltongue ui \
  --scenario sandbox/scenarios/live-ecosystem.json
```

**Graph Builder**:
```bash
./target/release/petaltongue ui \
  --scenario sandbox/scenarios/graph-studio.json
```

**RootPulse**:
```bash
./target/release/petaltongue ui \
  --scenario sandbox/scenarios/rootpulse-demo.json
```

**Neural API**:
```bash
./target/release/petaltongue ui \
  --scenario sandbox/scenarios/neural-api-test.json
```

---

## 📊 Performance Targets

### Frame Rate
- **Target**: 60 FPS constant
- **Minimum**: 30 FPS (degraded)
- **Measurement**: Real-time FPS counter

### Response Time
- **Click**: <16ms (1 frame)
- **Hover**: <32ms (2 frames)
- **Panel open**: <300ms (smooth)
- **Graph load**: <500ms (perceived instant)

### Resource Usage
- **CPU**: <5% idle, <15% active
- **Memory**: <100 MB base, <200 MB with data
- **GPU**: Hardware accelerated where available

---

## 🎯 User Experience Goals

### Delight Factor

**"Wow" Moments**:
1. **First Open**: Smooth awakening animation
2. **Live Topology**: Breathing, pulsing ecosystem
3. **Graph Execution**: Animated flow visualization
4. **Deployment**: Celebration on success
5. **Learning**: Visible optimization over time

**Intuitive Interactions**:
- Click → Inspect
- Drag → Move/Pan
- Scroll → Zoom
- Ctrl+Click → Multi-select
- Shift+Drag → Pan
- Delete → Remove
- Ctrl+A → Select all

**Keyboard Shortcuts** (always visible with `H`):
- `P` - Proprioception
- `M` - Metrics
- `G` - Graph Builder
- `R` - RootPulse
- `D` - Deployment
- `H` - Help
- `Esc` - Close/Cancel

---

## 🌟 Future Enhancements

### Completed
- Live ecosystem visualization
- Graph builder UI
- Neural API integration
- Basic animations
- RootPulse visualization
- Domain themes

### Next
- Deployment theater
- Multi-modal studio
- Advanced animations
- AI interaction (via agent adapter)
- Federation visualization

---

## 📚 Related Documentation

- Neural API Whitepaper — see `ecoPrimals/whitePaper/neuralAPI/`
- RootPulse Whitepaper — see `ecoPrimals/whitePaper/RootPulse/`
- petalTongue Human Interface — see `biomeOS/docs/PETALTONGUE_HUMAN_INTERFACE.md`
- biomeOS Architecture — see `biomeOS/docs/ARCHITECTURE_LAYERS.md`

---

## 🎨 Design Inspiration

- **popOS cosmic**: Modern, smooth, thoughtful
- **steamOS**: Gaming-focused, clean, powerful
- **Discord**: Communication, sleek, intuitive
- **VS Code**: Developer-friendly, extensible
- **Figma**: Collaborative, real-time, beautiful

---

**benchTop**: The desktop environment that grows with you 🌸✨

🚀 **Smooth. Beautiful. Powerful. Adaptive.**
