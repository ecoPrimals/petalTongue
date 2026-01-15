# 🌸 benchTop Architecture - The ecoPrimals Desktop

**Version**: 1.0.0  
**Date**: January 15, 2026  
**Status**: Demonstration Ready

---

## 🎯 Vision

**benchTop** is the ecoPrimals signature desktop environment - a smooth, modern, adaptive UI that showcases the full power of primal coordination.

### Design Goals

1. **Smooth**: 60+ FPS, buttery animations, instant response
2. **Intuitive**: No manual needed - click, drag, explore
3. **Beautiful**: Modern design, thoughtful UX, delightful interactions
4. **Powerful**: Full system visibility and control
5. **Adaptive**: Learns from usage, evolves over time

---

## 🏗️ Architecture

### Three-Layer Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                      PRESENTATION LAYER                          │
│  ┌─────────────┬──────────────┬──────────────┬────────────────┐ │
│  │   Visual    │    Audio     │   Terminal   │    Haptic      │ │
│  │   (egui)    │ (sonification│    (TUI)     │   (future)     │ │
│  └─────────────┴──────────────┴──────────────┴────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                    COORDINATION LAYER                            │
│  ┌──────────────┬───────────────┬──────────────┬──────────────┐ │
│  │  Neural API  │  RootPulse    │   Discovery  │   Learning   │ │
│  │  (graphs)    │  (temporal)   │  (Songbird)  │  (adaptive)  │ │
│  └──────────────┴───────────────┴──────────────┴──────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                      PRIMAL LAYER                                │
│  ┌──────────┬──────────┬──────────┬──────────┬────────────────┐│
│  │ BearDog  │ Songbird │Toadstool │ NestGate │   ... more     ││
│  │(security)│(discovery│ (compute)│ (storage)│                ││
│  └──────────┴──────────┴──────────┴──────────┴────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

**Top-Down** (User → Primals):
1. User interaction (click, drag, type)
2. benchTop processes intent
3. Neural API coordinates primals
4. Primals execute operations
5. Results flow back up

**Bottom-Up** (Primals → User):
1. Primals emit metrics/events
2. Neural API aggregates data
3. benchTop visualizes updates
4. User sees real-time changes

---

## 🎨 UI Components

### 1. **Live Ecosystem View**

**Purpose**: Real-time visualization of all running primals

**Features**:
- Animated primal nodes (breathing effect)
- Live connection pulses (data flow visualization)
- Health-based coloring (green → yellow → red)
- CPU/memory sparklines per primal
- Adaptive force-directed layout
- Capability badges with icons
- Click-to-inspect details panel

**Interactions**:
- Click node → Inspect details
- Drag node → Manual positioning
- Shift+Drag → Pan canvas
- Scroll → Zoom
- Ctrl+Click → Multi-select
- Right-click → Context menu

### 2. **Graph Builder Studio**

**Purpose**: Visual graph construction with Neural API execution

**Layout**:
```
┌─────────────┬─────────────────────────────┬─────────────────┐
│   Palette   │          Canvas             │   Properties    │
│             │                             │                 │
│ [PrimalStart│   ┌──────┐       ┌──────┐  │ Selected Node:  │
│ [Verification   │ Node │──────>│ Node │  │   PrimalStart   │
│ [WaitFor]   │   └──────┘       └──────┘  │                 │
│ [Conditional│                             │ Parameters:     │
│             │                             │   primal_name:  │
│ Search: ___ │   [Grid snapping enabled]   │   [nucleus   ]  │
│             │                             │                 │
│             │                             │ [Apply] [Reset] │
└─────────────┴─────────────────────────────┴─────────────────┘
```

**Features**:
- Drag-and-drop node creation
- Smooth Bézier curve edges
- Real-time validation (cycle detection, parameter checks)
- Execution monitoring with animation
- Save/load from Neural API
- Error visualization

### 3. **RootPulse Visualization**

**Purpose**: Temporal coordination and version control

**Layout**:
```
┌─────────────────────────────────────────────────────────────────┐
│  rhizoCrypt DAG - Present/Future (Branching possibilities)      │
│                                                                  │
│     ◉─────◉─────◉     Feature branch                            │
│           │      \                                               │
│           │       ◉─────◉  Merge                                │
│           │              /                                       │
│     ◉─────◉─────◉─────◉─────◉  Main branch                     │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│  LoamSpine Linear - The Past (Immutable history)                │
│                                                                  │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│  ◆        ◆        ◆        ◆        ◆                          │
│  │        │        │        │        │                          │
│  Crypto   Atomic   Causal   Consensus                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Features**:
- DAG timeline (branching possibilities)
- Linear history (immutable past)
- Multiple anchor types (crypto, atomic, causal, consensus)
- SweetGrass semantic attribution
- Temporal dehydration (nanoseconds → years)
- Branch merge animations

### 4. **Neural API Learning Dashboard**

**Purpose**: Visualize adaptive optimization and pattern discovery

**Features**:
- Usage heatmap (frequently used paths)
- Learning curves (optimization over time)
- Pattern discovery (emerging workflows)
- Pathway optimization (route changes)
- Bidirectional feedback loops
- Performance improvements graph

---

## 🎮 User Experience

### Startup Experience

**Awakening Sequence** (3 seconds):
1. Black screen (0-500ms)
2. petalTongue logo fade-in (500-1000ms)
3. Flower bloom animation (1000-2000ms)
4. Ecosystem reveal (2000-3000ms)
5. Ready state (3000ms+)

**First Impression**:
- Beautiful, smooth, professional
- Living, breathing ecosystem
- Clear what to do next
- Delightful animations

### Keyboard Shortcuts

**Always Available**:
- `P` - Proprioception Panel (SAME DAVE self-awareness)
- `M` - Metrics Dashboard (CPU, memory, system stats)
- `G` - Graph Builder (visual graph construction)
- `R` - RootPulse (version control timeline)
- `D` - Deployment Manager (spore deployment)
- `H` - Help overlay (all shortcuts)
- `Esc` - Close panels / Cancel action

**Context-Specific**:
- `Ctrl+A` - Select all
- `Ctrl+C` - Copy
- `Ctrl+V` - Paste
- `Delete` - Remove selected
- `Ctrl+Z` - Undo
- `Ctrl+Y` - Redo
- `Ctrl+S` - Save
- `Ctrl+O` - Open

### Mouse Interactions

**Single Click**:
- Select item
- Open panel
- Execute action

**Drag**:
- Move item
- Pan canvas (with Shift)
- Create connection (with Ctrl)

**Scroll**:
- Zoom canvas
- Scroll panels

**Right Click**:
- Context menu
- Quick actions

---

## 🎯 Performance Targets

### Frame Rate
- **Target**: 60 FPS constant
- **Minimum**: 30 FPS (degraded mode)
- **Measurement**: Real-time FPS counter in debug mode

### Latency
- **Click response**: <16ms (1 frame)
- **Hover feedback**: <32ms (2 frames)
- **Panel open**: <300ms (perceived instant)
- **Graph load**: <500ms (smooth)
- **Data update**: <100ms (real-time feel)

### Resource Usage
- **CPU (idle)**: <5%
- **CPU (active)**: <15%
- **Memory (base)**: <100 MB
- **Memory (loaded)**: <200 MB
- **GPU**: Hardware accelerated where available

---

## 🌈 Visual Design System

### Color Palette (Catppuccin Mocha)

```rust
Background:     #1e1e2e  // Dark blue-gray base
Surface:        #313244  // Slightly lighter
Overlay:        #45475a  // Modal backgrounds

Accent:         #89b4fa  // Blue - primary actions
Success:        #a6e3a1  // Green - healthy, complete
Warning:        #f9e2af  // Yellow - degraded, caution
Error:          #f38ba8  // Red - critical, failed

Text:           #cdd6f4  // Light gray - primary text
Text Dim:       #6c7086  // Medium gray - secondary text
Text Disabled:  #45475a  // Dark gray - disabled

// Primal Type Colors
Security:       #f38ba8  // Red (BearDog)
Discovery:      #89b4fa  // Blue (Songbird)
Compute:        #f9e2af  // Yellow (Toadstool)
Storage:        #a6e3a1  // Green (NestGate)
Visualization:  #cba6f7  // Purple (petalTongue)
Intelligence:   #fab387  // Orange (Squirrel)
Temporal:       #94e2d5  // Teal (RootPulse)
Coordination:   #f5c2e7  // Pink (NUCLEUS)
```

### Typography

**Font Families**:
- **Primary**: Inter (modern, clean, readable)
- **Monospace**: JetBrains Mono (code, terminal, data)
- **Display**: Inter Display (large headings)

**Sizes**:
- **XL**: 24px (main headings)
- **L**: 20px (section headings)
- **M**: 16px (body text, buttons)
- **S**: 14px (labels, secondary)
- **XS**: 12px (metadata, timestamps)

**Weights**:
- **Regular**: 400 (body text)
- **Medium**: 500 (labels)
- **Semibold**: 600 (headings)
- **Bold**: 700 (emphasis)

### Spacing System

**Base**: 8px (0.5rem)

- **XXS**: 4px (0.25rem) - tight spacing
- **XS**: 8px (0.5rem) - compact spacing
- **S**: 12px (0.75rem) - small gaps
- **M**: 16px (1rem) - standard spacing
- **L**: 24px (1.5rem) - section spacing
- **XL**: 32px (2rem) - large gaps
- **XXL**: 48px (3rem) - major sections

### Border Radius

- **Small**: 4px (buttons, inputs)
- **Medium**: 8px (cards, panels)
- **Large**: 12px (modals, windows)
- **Round**: 50% (circular buttons, avatars)

---

## 🎬 Animation Principles

### Timing

**Duration**:
- **Instant**: 0ms (immediate feedback)
- **Quick**: 150ms (hover, highlight)
- **Standard**: 300ms (panel open/close, transitions)
- **Slow**: 500ms (page changes, major transitions)
- **Celebration**: 1000ms (success animations)

**Easing Functions**:
- **Ease-in**: Acceleration (things appearing)
- **Ease-out**: Deceleration (things disappearing)
- **Ease-in-out**: Smooth (most transitions)
- **Spring**: Elastic (playful, celebratory)
- **Linear**: Constant (progress indicators)

### Animation Patterns

**Fade**:
- Opacity transitions
- Subtle, professional
- Use for: modal overlays, tooltips

**Slide**:
- Position transitions
- Directional feedback
- Use for: panels, sidebars

**Scale**:
- Size transitions
- Attention-grabbing
- Use for: buttons, selections

**Pulse**:
- Rhythmic scaling/opacity
- Breathing effect
- Use for: live nodes, activity indicators

**Flow**:
- Particle movement
- Data visualization
- Use for: connections, execution paths

---

## 🚀 Implementation Status

### Phase 1: Core UI (Complete)
- ✅ Live Ecosystem View
- ✅ Graph Builder Studio
- ✅ Neural API Integration
- ✅ Keyboard shortcuts
- ✅ Basic animations

### Phase 2: Advanced Features (In Progress)
- 🔄 RootPulse Visualization
- 🔄 Deployment Theater
- 🔄 Multi-modal Studio
- 🔄 Advanced animations
- 🔄 Learning dashboard

### Phase 3: Polish (Planned)
- 🔵 Custom themes
- 🔵 Accessibility enhancements
- 🔵 Performance optimization
- 🔵 Collaborative features
- 🔵 Federation visualization

---

## 📚 Related Documentation

- [sandbox/README.md](README.md) - Demonstration overview
- [Neural API Whitepaper](/home/eastgate/Development/ecoPrimals/whitePaper/neuralAPI/)
- [RootPulse Whitepaper](/home/eastgate/Development/ecoPrimals/whitePaper/RootPulse/)
- [petalTongue Architecture](../specs/)

---

**benchTop**: The desktop environment that grows with you 🌸✨

🎨 **Smooth. Beautiful. Powerful. Adaptive.**

