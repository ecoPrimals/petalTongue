# 🌸 Universal UI Evolution - petalTongue as the Face of ecoPrimals

**Date**: January 3, 2026  
**Goal**: Transform petalTongue into a comprehensive Universal User Interface  
**Principle**: Any Input → Any Output | Live Data | Full Accessibility

---

## 🎯 Vision

**petalTongue will be THE face of ecoPrimals:**
- Universal accessibility (blind, deaf, color-blind, motor disabilities)
- Live data visualization with real-time graphs
- Full UI capabilities (not just a viewer)
- BearDog integration (slice/tunnel when needed)
- Customizable for every user's needs

---

## 📊 Current State (Already Implemented)

### ✅ Live Data Sources
- System Monitor with real-time graphs (CPU, memory)
- Process Viewer with live process list
- Network topology discovery (mDNS)
- Audio entropy capture (real microphone)
- Sparkline graphs for history

### ✅ Multi-Modal Output
- Visual 2D graph
- Audio sonification
- Text descriptions
- Timeline view
- Traffic view

### ✅ Tool Integration
- System Monitor
- Process Viewer
- Graph Metrics
- BingoCube integration

---

## 🚀 Evolution Plan

### Phase 1: Accessibility Controls (Immediate)

**Color Schemes**:
- Default (current)
- High Contrast
- Color-Blind Friendly (Deuteranopia, Protanopia, Tritanopia)
- Dark Mode variants
- Custom user-defined

**Font & Size**:
- Adjustable font sizes (small, medium, large, XL)
- Font family selection
- Line spacing control
- Text-to-speech integration

**Input Methods**:
- Keyboard shortcuts (comprehensive)
- Mouse/touchpad
- Screen reader support
- Voice commands (future)
- Single-switch scanning

### Phase 2: Real-Time Graphing (Next 2-3 days)

**System Metrics Integration**:
- Embed system monitor into main view
- Real-time CPU/memory graphs overlay
- Network bandwidth visualization
- Disk I/O graphs
- Custom metric streaming

**Live Data Proof**:
- Timestamps on all graphs
- "LIVE" indicators
- Update frequency display
- Data source labels
- No-data detection ("Waiting for data...")

**Graph Types**:
- Time series (sparklines, line charts)
- Bar charts (current values)
- Pie charts (resource allocation)
- Heat maps (historical patterns)
- Sankey diagrams (flow)

### Phase 3: Universal Controls (3-4 days)

**Settings Panel**:
- Accessibility settings
- Color scheme picker
- Font controls
- Audio volume/pitch
- Update frequency
- Data source selection

**Customization**:
- Save user preferences
- Multiple profiles
- Quick-switch modes
- Export/import settings

**Keyboard Navigation**:
- Full keyboard access
- Customizable shortcuts
- On-screen command palette
- Help overlay (? key)

### Phase 4: Live Data Dashboard (3-4 days)

**Main Dashboard**:
- Network topology (center)
- System metrics (sidebar)
- Recent events (bottom)
- Quick actions (top)
- All live, all real-time

**Data Streaming**:
- Real system metrics
- Real network discovery
- Real primal status
- Real entropy capture
- Zero mocks

**Visual Indicators**:
- "LIVE" badges on all graphs
- Timestamp displays
- Update animations
- Connection status
- Data source indicators

---

## 🎨 UI Component Architecture

### Main Layout

```
┌─────────────────────────────────────────────────────────────┐
│  ⚙️ Settings  |  🌐 Network  |  📊 Metrics  |  ♿ Access   │ Top Bar
├─────────────────────────────────────────────────────────────┤
│         │                                     │             │
│  Tool   │         Main Topology View          │   System   │
│  Panel  │    (Network Graph - LIVE)           │   Metrics  │
│         │                                     │   (LIVE)   │
│  • Sys  │    ● Bear Dog (healthy)            │            │
│  • Proc │    ● Songbird (healthy)            │  CPU: 45%  │
│  • Ent  │    ● Local (excellent)             │  [====   ] │
│         │                                     │            │
│         │    Connections: 3 active            │  Mem: 62%  │
│         │    Last update: 0.5s ago  [LIVE]   │  [======  ]│
│         │                                     │            │
├─────────────────────────────────────────────────────────────┤
│  Recent Events:                                              │
│  • 12:34:56 - Primal discovered: BearDog                     │
│  • 12:35:01 - Connection established                          │
│  • 12:35:12 - System: CPU spike detected (85%)               │
└─────────────────────────────────────────────────────────────┘
```

### Accessibility Panel

```
♿ Accessibility Settings
├── Color Scheme
│   ○ Default
│   ● High Contrast
│   ○ Deuteranopia
│   ○ Protanopia
│   ○ Tritanopia
│   ○ Custom...
│
├── Font Size
│   [ Small | ● Medium | Large | X-Large ]
│
├── Audio
│   Sonification: [ON]
│   Volume: [====░░] 80%
│   Narration: [ON]
│
├── Input
│   Keyboard Nav: [ON]
│   Screen Reader: [ON]
│   Mouse Required: [OFF]
│
└── [Save Preferences]
```

### Live Data Indicators

```
Every graph/metric shows:
┌────────────────────┐
│ CPU Usage   [LIVE] │
│ 45.2%       0.5s   │
│ [========░░░░░░░░] │
│ ▁▂▃▅▄▃▂ History    │
│                    │
│ Source: sysinfo    │
│ Update: 1s         │
└────────────────────┘
```

---

## 🔧 Technical Implementation

### 1. Accessibility Module

```rust
// crates/petal-tongue-ui/src/accessibility.rs

pub struct AccessibilitySettings {
    color_scheme: ColorScheme,
    font_size: FontSize,
    audio_enabled: bool,
    audio_volume: f32,
    narration_enabled: bool,
    keyboard_only: bool,
    screen_reader_mode: bool,
}

pub enum ColorScheme {
    Default,
    HighContrast,
    Deuteranopia,      // Red-green color blind
    Protanopia,        // Red-blind
    Tritanopia,        // Blue-yellow color blind
    Custom(CustomColors),
}

pub struct CustomColors {
    healthy: Color32,
    warning: Color32,
    error: Color32,
    background: Color32,
    text: Color32,
}
```

### 2. Live Data Dashboard

```rust
// crates/petal-tongue-ui/src/live_dashboard.rs

pub struct LiveDashboard {
    system_monitor: SystemMonitorTool,
    network_topology: TopologyView,
    recent_events: EventLog,
    update_indicators: HashMap<String, LiveIndicator>,
}

pub struct LiveIndicator {
    last_update: Instant,
    update_interval: Duration,
    data_source: String,
    is_live: bool,
}
```

### 3. Real-Time Graph Component

```rust
// crates/petal-tongue-ui/src/realtime_graph.rs

pub struct RealTimeGraph {
    data: VecDeque<DataPoint>,
    max_points: usize,
    live_indicator: bool,
    last_update: Instant,
    source_name: String,
}

pub struct DataPoint {
    timestamp: Instant,
    value: f64,
}
```

---

## 📋 Implementation Checklist

### Immediate (Next Session)

- [ ] Create `accessibility.rs` module
- [ ] Implement color scheme system
- [ ] Add font size controls
- [ ] Create settings panel UI
- [ ] Add "LIVE" indicators to all graphs
- [ ] Show timestamps on data
- [ ] Add data source labels

### Short-Term (2-3 days)

- [ ] Integrate system monitor into main view
- [ ] Add real-time graph overlays
- [ ] Implement keyboard shortcuts
- [ ] Create accessibility settings panel
- [ ] Add audio controls
- [ ] Test with screen readers

### Medium-Term (3-4 days)

- [ ] Implement all color schemes
- [ ] Add custom color picker
- [ ] Create user profiles
- [ ] Save/load preferences
- [ ] Full keyboard navigation
- [ ] Voice command integration (future)

---

## 🎯 Success Criteria

**Accessibility**:
- ✅ Color-blind users can distinguish all states
- ✅ Blind users can navigate entire UI
- ✅ Deaf users lose no information
- ✅ Motor disabilities can use single input
- ✅ Custom preferences persist

**Live Data**:
- ✅ All data sources labeled
- ✅ Timestamps on all graphs
- ✅ "LIVE" indicators visible
- ✅ Update frequency shown
- ✅ No-data states handled

**Universal UI**:
- ✅ Full functionality for all users
- ✅ Customizable to any need
- ✅ No mock data in production
- ✅ Professional appearance
- ✅ ecoPrimals "face" quality

---

🌸♿🎵 **petalTongue: The Universal Interface for ecoPrimals** 🎵♿🌸

Next: Implement accessibility controls and real-time graph enhancements!

