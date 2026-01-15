# 🧠 PetalTongue Neural API Evolution Roadmap
**Date**: January 15, 2026  
**Status**: ✅ **Integration Complete - Ready to Evolve**  
**Priority**: HIGH - Rich new capabilities available

---

## 🎯 EXECUTIVE SUMMARY

**What Changed**: BiomeOS evolved Neural API as the **central nervous system** for inter-primal coordination, and integrated PetalTongue to use it as the single source of truth.

**What We Now Have**:
- ✅ `NeuralApiProvider` integrated (280 lines)
- ✅ Real-time primal discovery via Neural API
- ✅ SAME DAVE proprioception data (self-awareness!)
- ✅ Aggregated system metrics
- ✅ Graceful fallback chain (Neural API → Songbird → Mock)

**What We Can Build**:
- 🚀 Proprioception visualization (system self-awareness)
- 🚀 Real-time metrics dashboard
- 🚀 Enhanced topology with health indicators
- 🚀 Visual graph builder
- 🚀 AI-powered graph generation (future)

**Impact**: Transform PetalTongue from "visualization tool" to "ecosystem orchestration UI"

---

## 📊 ARCHITECTURAL EVOLUTION

### Before: Fragmented Discovery
```
petalTongue → Songbird → individual primals
petalTongue → BearDog (direct)
petalTongue → ToadStool (direct)
```

### After: Coordinated Central System
```
petalTongue → Neural API ← (central coordinator)
                  ↓
    ┌──────────────┼──────────────┐
    ↓              ↓              ↓
  BearDog      Songbird      ToadStool
```

**Benefits**:
- ✅ Single query gets all primal state
- ✅ Built-in proprioception data
- ✅ Aggregated system metrics
- ✅ Consistent topology view
- ✅ Any UI can use same pattern

---

## 🎨 EVOLUTION OPPORTUNITIES (Prioritized)

### Priority 1: Proprioception Visualization ⭐⭐⭐ (HIGH VALUE)
**Timeline**: 1-2 weeks  
**Impact**: Revolutionary - system can show its own self-awareness

#### What We Get from Neural API:
```json
{
  "timestamp": "2026-01-15T01:44:00Z",
  "family_id": "nat0",
  "health": {
    "percentage": 100.0,
    "status": "healthy"  // healthy | degraded | critical
  },
  "confidence": 100.0,
  "self_awareness": {
    "knows_about": 3,
    "can_coordinate": true,
    "has_security": true,
    "has_discovery": true,
    "has_compute": true
  },
  "motor": {
    "can_deploy": true,
    "can_execute_graphs": true,
    "can_coordinate_primals": true
  },
  "sensory": {
    "active_sockets": 3,
    "last_scan": "2026-01-15T01:44:00Z"
  }
}
```

#### UI Design Concept:
```
╔═══════════════════════════════════════════════════════════╗
║ NUCLEUS Proprioception                    Health: 100% 💚 ║
╠═══════════════════════════════════════════════════════════╣
║ Self-Awareness (SAME DAVE):                               ║
║                                                           ║
║   👁️  SENSORY:       3 active sockets detected           ║
║                     Last scan: 2s ago                     ║
║                                                           ║
║   🧠 AWARENESS:      Knows about 3 primals                ║
║                     ✓ Security (BearDog)                  ║
║                     ✓ Discovery (Songbird)                ║
║                     ✓ Compute (ToadStool)                 ║
║                                                           ║
║   💪 MOTOR:          Can deploy ✓                         ║
║                     Can execute graphs ✓                  ║
║                     Can coordinate primals ✓              ║
║                                                           ║
║   ⚖️  EVALUATIVE:    Confidence: 100%                     ║
║                     Status: Healthy                       ║
╠═══════════════════════════════════════════════════════════╣
║ System Capabilities:                                      ║
║   ✅ Can coordinate primals                               ║
║   ✅ Has security services                                ║
║   ✅ Has discovery services                               ║
║   ✅ Has compute services                                 ║
╚═══════════════════════════════════════════════════════════╝
```

#### Implementation Tasks:

**Week 1: Data Integration**
- [ ] Add `ProprioceptionData` struct to `petal-tongue-core`
- [ ] Create `ProprioceptionPanel` widget in `petal-tongue-ui`
- [ ] Implement polling (every 5s) in app state
- [ ] Add error handling for missing proprioception

**Week 2: UI Polish**
- [ ] Design health indicator (color-coded circle)
- [ ] Add confidence meter (progress bar or gauge)
- [ ] Create SAME DAVE visualization (4 categories)
- [ ] Add timestamp display (relative time: "2s ago")
- [ ] Implement smooth transitions when values change

**Files to Create/Modify**:
- `crates/petal-tongue-core/src/proprioception.rs` (new)
- `crates/petal-tongue-ui/src/proprioception_panel.rs` (new)
- `crates/petal-tongue-ui/src/app.rs` (add panel)
- `crates/petal-tongue-ui/src/state.rs` (add polling)

**Acceptance Criteria**:
- [ ] Panel shows real-time proprioception data
- [ ] Health indicator updates smoothly
- [ ] SAME DAVE categories clearly visualized
- [ ] Works gracefully if Neural API unavailable
- [ ] Beautiful, polished UI (Steam/Discord quality)

---

### Priority 2: Real-Time Metrics Dashboard ⭐⭐⭐ (HIGH VALUE)
**Timeline**: 1-2 weeks  
**Impact**: Users can see system resource usage at a glance

#### What We Get from Neural API:
```json
{
  "timestamp": "2026-01-15T01:44:00Z",
  "system": {
    "cpu_percent": 16.5,
    "memory_used_mb": 32768,
    "memory_total_mb": 49152,
    "memory_percent": 66.7,
    "uptime_seconds": 86400
  },
  "neural_api": {
    "family_id": "nat0",
    "active_primals": 3,
    "graphs_available": 5,
    "active_executions": 0
  }
}
```

#### UI Design Concept:
```
╔═══════════════════════════════╗
║ System Metrics    [Refresh ⟳] ║
╠═══════════════════════════════╣
║ CPU:    [████████░░]  16.5%   ║
║         ▂▃▅▇▆▄▃▂ (last 30s)   ║
║                               ║
║ Memory: [██████████]  66.7%   ║
║         32 GB / 48 GB used    ║
║                               ║
║ Uptime: 1d 2h 34m             ║
║                               ║
║ Ecosystem Status:             ║
║   • Active Primals: 3         ║
║   • Available Graphs: 5       ║
║   • Running: 0                ║
╚═══════════════════════════════╝
```

#### Implementation Tasks:

**Week 1: Core Metrics**
- [ ] Add `SystemMetrics` struct to `petal-tongue-core`
- [ ] Create `MetricsDashboard` widget
- [ ] Implement ring buffer for CPU history (30 data points)
- [ ] Add memory bar visualization
- [ ] Format uptime (human-readable: "1d 2h 34m")

**Week 2: Polish & Features**
- [ ] Add mini line charts for CPU/memory trends
- [ ] Color-code based on thresholds (green <50%, yellow <80%, red >80%)
- [ ] Add refresh button and auto-refresh toggle
- [ ] Implement smooth value transitions (animate changes)
- [ ] Add tooltip on hover (show exact values)

**Files to Create/Modify**:
- `crates/petal-tongue-core/src/metrics.rs` (new)
- `crates/petal-tongue-ui/src/metrics_dashboard.rs` (new)
- `crates/petal-tongue-ui/src/widgets/metric_widget.rs` (new)
- `crates/petal-tongue-ui/src/app_panels.rs` (add dashboard)

**Acceptance Criteria**:
- [ ] Real-time CPU and memory display
- [ ] Historical sparklines (last 30 seconds)
- [ ] Color-coded thresholds
- [ ] Smooth animations on value changes
- [ ] Auto-refresh every 5s (configurable)

---

### Priority 3: Enhanced Topology Visualization ⭐⭐ (MEDIUM VALUE)
**Timeline**: 1 week  
**Impact**: Much clearer topology with health indicators

#### Features to Add:
1. **Color-code nodes by health**:
   - Green: Healthy (100%)
   - Yellow: Degraded (50-99%)
   - Red: Critical (<50%)

2. **Connection type labels**:
   - "security-provider"
   - "discovery"
   - "compute"
   - "coordination"

3. **Capability badges**:
   - Small icons showing primal capabilities
   - 🔒 Security, 🎵 Discovery, ⚡ Compute, etc.

4. **Animated data flow** (future):
   - Pulses along edges when queries happen
   - Visual indication of active communication

#### UI Design Concept:
```
    ┌───────────────┐
    │   BearDog     │ 💚 Healthy
    │  🔒 Security  │
    └───────┬───────┘
            │ [security-provider]
            ↓
    ┌───────────────┐
    │   Songbird    │ 💚 Healthy
    │  🎵 Discovery │
    └───────┬───────┘
            │ [coordination]
            ↓
    ┌───────────────┐
    │  ToadStool    │ 💛 Degraded
    │  ⚡ Compute   │
    └───────────────┘
```

#### Implementation Tasks:

**Week 1: Visual Enhancements**
- [ ] Add health color to node rendering
- [ ] Add capability icons to nodes
- [ ] Add edge type labels
- [ ] Improve node layout algorithm
- [ ] Add hover tooltips (show detailed info)

**Files to Modify**:
- `crates/petal-tongue-graph/src/visual_2d.rs` (enhance rendering)
- `crates/petal-tongue-ui/src/topology_view.rs` (add health colors)

**Acceptance Criteria**:
- [ ] Nodes show health status with color
- [ ] Edges show connection types
- [ ] Capability icons visible
- [ ] Hover shows detailed information
- [ ] Layout is clear and readable

---

### Priority 4: Visual Graph Builder ⭐⭐⭐ (HIGH VALUE - REVOLUTIONARY)
**Timeline**: 2-3 weeks  
**Impact**: Users can build and execute graphs from UI!

#### What This Enables:
Users can **visually design Neural API graphs** and execute them directly from PetalTongue. This is HUGE - it transforms PetalTongue from viewer to orchestrator!

#### Features:
1. **Drag-and-Drop Canvas**:
   - Drag nodes from palette
   - Drop on canvas
   - Connect nodes with edges

2. **Node Types Available**:
   - `primal_start` - Start a primal
   - `verification` - Verify primal health
   - `wait_for` - Wait for condition
   - `conditional` - Branch based on condition

3. **Parameter Forms**:
   - Each node has a form for parameters
   - Validation before save
   - Help text for each field

4. **Graph Operations**:
   - Save to Neural API
   - Load existing graphs
   - Execute from UI
   - Monitor execution status

#### UI Design Concept:
```
╔════════════════════════════════════════════════════════════╗
║ Graph Builder: my-deployment                [Save] [Execute]║
╠════════════════════════════════════════════════════════════╣
║                                                            ║
║  Node Palette:              Canvas:                        ║
║  ┌────────────┐            ┌──────────────┐               ║
║  │ Start      │            │ Start BearDog│               ║
║  │ Primal     │            │  family: nat0│               ║
║  ├────────────┤            └──────┬───────┘               ║
║  │ Verify     │                   │                       ║
║  │ Health     │                   ↓                       ║
║  ├────────────┤            ┌──────────────┐               ║
║  │ Wait For   │            │Start Songbird│               ║
║  ├────────────┤            │ depends: [1] │               ║
║  │ Conditional│            └──────────────┘               ║
║  └────────────┘                                           ║
║                                                            ║
║  [Drag nodes to canvas to build your graph]               ║
╚════════════════════════════════════════════════════════════╝
```

#### Implementation Tasks:

**Week 1: Core Infrastructure**
- [ ] Create `GraphBuilder` widget with canvas
- [ ] Implement node palette (draggable nodes)
- [ ] Add drag-and-drop functionality
- [ ] Implement edge drawing (connect nodes)
- [ ] Add node deletion/editing

**Week 2: Graph Logic**
- [ ] Implement parameter forms for each node type
- [ ] Add graph validation (check dependencies, cycles)
- [ ] Create graph serialization to Neural API format
- [ ] Implement save to Neural API (`neural_api.save_graph`)
- [ ] Implement load from Neural API

**Week 3: Execution & Polish**
- [ ] Add execute button (calls Neural API)
- [ ] Monitor execution status
- [ ] Show execution progress
- [ ] Add error handling and user feedback
- [ ] Polish UI (zoom, pan, grid snapping)

**Files to Create**:
- `crates/petal-tongue-ui/src/graph_builder/mod.rs`
- `crates/petal-tongue-ui/src/graph_builder/canvas.rs`
- `crates/petal-tongue-ui/src/graph_builder/node_palette.rs`
- `crates/petal-tongue-ui/src/graph_builder/parameter_form.rs`
- `crates/petal-tongue-ui/src/graph_builder/executor.rs`

**Acceptance Criteria**:
- [ ] Users can drag nodes onto canvas
- [ ] Users can connect nodes with edges
- [ ] Parameter forms validate input
- [ ] Graphs can be saved to Neural API
- [ ] Graphs can be loaded from Neural API
- [ ] Graphs can be executed from UI
- [ ] Execution status is monitored
- [ ] Beautiful, intuitive UI (Figma/Miro quality)

---

### Priority 5: AI Integration via Squirrel ⭐ (FUTURE - VISIONARY)
**Timeline**: 3-4 weeks (after Squirrel integration in biomeOS)  
**Impact**: Natural language → Graph generation

#### Vision:
```
User types: "Deploy a full NUCLEUS with GPU support"
  ↓
PetalTongue → Squirrel AI
  ↓
Squirrel generates graph:
  - Start BearDog (security)
  - Start Songbird (discovery)
  - Start ToadStool with GPU (compute)
  - Wire dependencies
  ↓
PetalTongue shows preview (in Graph Builder)
  ↓
User reviews and approves
  ↓
Neural API executes
  ↓
PetalTongue visualizes deployment in real-time
```

#### This Is Revolutionary:
- **Self-hosting evolution**: System evolves itself
- **Natural language**: No need to learn graph syntax
- **Intelligent**: Squirrel knows best practices
- **Iterative**: User can refine the graph

#### Implementation (Future):
- [ ] Add natural language input box
- [ ] Integrate with Squirrel API (when available)
- [ ] Preview generated graphs in Graph Builder
- [ ] Allow user refinement before execution
- [ ] Learn from user edits (feedback to Squirrel)

**Status**: Awaiting Squirrel integration in biomeOS ecosystem

---

## 🛠️ TECHNICAL IMPLEMENTATION GUIDE

### Accessing Neural API from PetalTongue

```rust
use petal_tongue_discovery::NeuralApiProvider;

// Discovery (already implemented in discovery chain)
let provider = NeuralApiProvider::discover(None).await?;

// Get standard primals (trait method)
let primals = provider.get_primals().await?;

// Get proprioception (Neural API specific)
let proprio = provider.get_proprioception().await?;

// Get metrics (Neural API specific)
let metrics = provider.get_metrics().await?;
```

### Adding New Neural API Methods

If biomeOS adds new methods, extend `NeuralApiProvider`:

```rust
// In crates/petal-tongue-discovery/src/neural_api_provider.rs

impl NeuralApiProvider {
    pub async fn get_new_feature(&self) -> Result<NewFeatureData> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "neural_api.new_feature".to_string(),
            params: serde_json::json!({}),
            id: self.next_request_id(),
        };
        
        self.send_request(&request).await
    }
}
```

### Polling Strategy

For real-time updates:

```rust
// In app state
struct AppState {
    proprioception: Option<ProprioceptionData>,
    metrics: Option<SystemMetrics>,
    last_update: Instant,
}

// In update loop
if last_update.elapsed() > Duration::from_secs(5) {
    // Refresh data
    if let Some(provider) = &self.neural_api_provider {
        self.proprioception = provider.get_proprioception().await.ok();
        self.metrics = provider.get_metrics().await.ok();
    }
    self.last_update = Instant::now();
}
```

### Error Handling

Always gracefully handle Neural API unavailability:

```rust
match neural_api_provider.get_proprioception().await {
    Ok(data) => {
        // Show data
    }
    Err(e) => {
        tracing::warn!("Neural API unavailable: {}", e);
        // Show "No proprioception data" message
        // Fall back to basic topology view
    }
}
```

---

## 📋 IMPLEMENTATION ROADMAP

### Phase 1: Foundation (Weeks 1-2)
**Goal**: Display proprioception and metrics

- [ ] Week 1: Proprioception panel
- [ ] Week 2: Metrics dashboard
- [ ] Deliverable: PetalTongue shows system self-awareness

### Phase 2: Enhanced Visualization (Week 3)
**Goal**: Better topology understanding

- [ ] Week 3: Enhanced topology (health colors, capability badges)
- [ ] Deliverable: Beautiful, informative topology view

### Phase 3: Orchestration (Weeks 4-6)
**Goal**: Users can build and execute graphs

- [ ] Week 4: Graph builder core
- [ ] Week 5: Graph logic and validation
- [ ] Week 6: Execution and monitoring
- [ ] Deliverable: Full graph builder with execution

### Phase 4: AI Integration (Weeks 7-8+)
**Goal**: Natural language graph generation

- [ ] Week 7+: Squirrel integration (when available)
- [ ] Deliverable: Natural language → graph execution

---

## 🎨 DESIGN GUIDELINES

### Follow TRUE PRIMAL Principles:
1. **Discovery Over Configuration**: Don't hardcode
2. **Graceful Degradation**: Work without Neural API
3. **Capability-Based**: Query what primals can do
4. **Beautiful**: Steam/Discord/VS Code quality
5. **Fast**: <100ms updates

### UI Inspiration:
- **Steam Library**: Available graphs, execution status
- **Discord Status**: Health indicators, active primals
- **VS Code Panels**: Modular, resizable, elegant
- **Figma Canvas**: Drag-drop graph builder
- **Grafana Dashboards**: Metrics visualization

---

## ✅ SUCCESS CRITERIA

### User Delight:
- [ ] "Wow, I can see the system thinking!"
- [ ] "This is so much clearer than before"
- [ ] "I can deploy entire ecosystems from here!"

### Technical Excellence:
- [ ] <100ms to update all visualizations
- [ ] Works even if Neural API is down
- [ ] Smooth animations and transitions
- [ ] Polished, professional UI

### Ecosystem Impact:
- [ ] Other primals want to integrate with PetalTongue
- [ ] Users prefer PetalTongue over CLI
- [ ] Demonstrations wow new users

---

## 📚 DOCUMENTATION TO CREATE

As we implement, document:

1. **User Guide**: How to use new features
2. **Developer Guide**: How to extend UI
3. **API Integration**: How to add new Neural API methods
4. **Design System**: UI components and patterns
5. **Performance Guide**: Optimization tips

---

## 🔄 COORDINATION WITH BIOMEOS

### What We Need from BiomeOS:
- ✅ Current Neural API endpoints (already provided)
- ⏳ Graph save/load API (for graph builder)
- ⏳ Execution monitoring API (for status updates)
- ⏳ Squirrel integration (future AI features)

### What We'll Provide:
- Beautiful UI for Neural API data
- Graph builder (visual tool)
- Real-time monitoring
- User feedback and feature requests

### Communication:
- Weekly syncs to align on features
- Shared documentation in wateringHole
- GitHub issues for bugs/features

---

## 🎯 IMMEDIATE NEXT STEPS

**This Week**:
1. [ ] Review Neural API integration code
2. [ ] Design proprioception panel (sketch/wireframe)
3. [ ] Create `proprioception.rs` in core
4. [ ] Start implementing proprioception panel

**Next Week**:
1. [ ] Complete proprioception panel
2. [ ] Design metrics dashboard
3. [ ] Start metrics implementation

**By End of Month**:
1. [ ] Proprioception panel live
2. [ ] Metrics dashboard live
3. [ ] Enhanced topology colors
4. [ ] Demo video for biomeOS team

---

## 🎊 CONCLUSION

**We now have incredible capabilities** thanks to Neural API integration:

**Before**:
- Basic primal list
- Static topology
- No system awareness
- No metrics

**After (with this roadmap)**:
- System self-awareness visualization (SAME DAVE)
- Real-time metrics dashboard
- Health-coded topology
- Visual graph builder
- (Future) AI-powered orchestration

**This transforms PetalTongue** from "viewer" to "orchestrator" - a true command center for the ecoPrimals ecosystem!

**Timeline**: 6-8 weeks to complete priorities 1-4  
**Impact**: Revolutionary user experience  
**Excitement Level**: 🚀🚀🚀

---

**Created**: January 15, 2026  
**Owner**: PetalTongue Team  
**Status**: ✅ **Ready to Build**

🌸 **Let's evolve PetalTongue into something extraordinary!** 🧠✨

