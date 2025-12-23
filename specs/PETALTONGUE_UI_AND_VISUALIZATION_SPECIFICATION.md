# petalTongue: UI & Visualization System Specification

**Version:** 1.0.0  
**Status:** Specification  
**Date:** December 23, 2025  
**Author:** BiomeOS Team  

---

## Executive Summary

**petalTongue** is the visual interface and interaction visualization system for the ecoPrimals ecosystem. Initially embedded within BiomeOS as the universal UI layer, petalTongue provides real-time visualization of primal interactions, service mesh topology, and ecosystem health. As it matures, petalTongue may evolve into an independent primal, simplifying BiomeOS by delegating all visualization concerns to a dedicated service.

### Key Capabilities

- **Real-Time Topology Visualization** - Live graph rendering of primal interactions
- **Interactive Service Mesh** - Drag, zoom, filter primal connections
- **Flow Animation** - Visual representation of API calls and data movement
- **Health Monitoring** - Color-coded status indicators across the ecosystem
- **Desktop Interface** - OS-like user experience with windows and taskbar
- **Multi-View Dashboard** - Comprehensive views for different aspects of the ecosystem

### Philosophy

> "If you can't see it, you can't understand it. If you can't understand it, you can't trust it."

petalTongue embodies the principle that **sovereignty requires visibility**. Users must see how their primals interact, what data flows where, and who has access to what. Visual transparency is not a luxury—it's a requirement for digital sovereignty.

---

## 1. Architecture Overview

### 1.1 Current State (Embedded in BiomeOS)

```
┌─────────────────────────────────────────────────────────────────┐
│                         BiomeOS                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    petalTongue                            │  │
│  │              (UI & Visualization Layer)                   │  │
│  │                                                           │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │  │
│  │  │  Desktop    │  │   Graph     │  │    Real-Time    │  │  │
│  │  │  Interface  │  │  Rendering  │  │   Telemetry     │  │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │  │
│  │                                                           │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │  │
│  │  │   Views     │  │ Interactive │  │   Animation     │  │  │
│  │  │  System     │  │  Controls   │  │    Engine       │  │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                             │                                   │
│  ┌──────────────────────────┴────────────────────────────────┐ │
│  │           Universal BiomeOS Manager API                    │ │
│  └────────────────────────────────────────────────────────────┘ │
│                             │                                   │
└─────────────────────────────┼───────────────────────────────────┘
                              │
                  ┌───────────┴───────────┐
                  │                       │
                  ▼                       ▼
         [Primal Discovery]      [Health Monitoring]
```

### 1.2 Future State (Independent Primal)

```
┌─────────────────────────────────────────────────────────────────┐
│                      petalTongue Primal                          │
│                  (Visualization Service)                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Capabilities:                                                  │
│  • "visualization.graph-rendering"                              │
│  • "visualization.real-time-topology"                           │
│  • "visualization.flow-animation"                               │
│  • "ui.desktop-interface"                                       │
│  • "ui.primal-interaction"                                      │
│                                                                 │
│  API Endpoints:                                                 │
│  • GET  /api/v1/topology                                        │
│  • GET  /api/v1/flow-events                                     │
│  • POST /api/v1/render-graph                                    │
│  • WS   /api/v1/live-topology                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                  ┌───────────┴───────────┐
                  │                       │
                  ▼                       ▼
         ┌────────────────┐      ┌────────────────┐
         │    BiomeOS     │      │  Any Primal    │
         │  (simplified)  │      │ (via Songbird) │
         └────────────────┘      └────────────────┘
```

**Evolution Criteria:** petalTongue becomes an independent primal when:
1. Visualization complexity justifies separation
2. Multiple consumers need visualization services
3. UI rendering becomes a bottleneck
4. Independent scaling is required

---

## 2. Core Components

### 2.1 Desktop Interface System

**Purpose:** Provide an OS-like user experience for managing the ecosystem.

**Features:**
- **Taskbar** - Always-visible system status and app shortcuts
- **Window Manager** - Multiple windows, resize, minimize, maximize
- **Application Launcher** - Quick access to all views
- **Notification System** - Real-time alerts and messages
- **System Tray** - Background processes and quick actions

**Implementation:**
```rust
pub struct DesktopInterface {
    taskbar: Taskbar,
    window_manager: WindowManager,
    launcher: ApplicationLauncher,
    notifications: NotificationSystem,
    background: BackgroundRenderer,
}

impl DesktopInterface {
    pub fn render(&mut self, ctx: &egui::Context) {
        self.render_background(ctx);
        self.window_manager.render_all_windows(ctx);
        self.taskbar.render(ctx);
        self.notifications.render(ctx);
        
        if self.launcher.is_visible() {
            self.launcher.render(ctx);
        }
    }
}
```

### 2.2 Graph Rendering Engine

**Purpose:** Visualize primals as nodes and their interactions as edges.

**Features:**
- **Node Rendering** - Primals as interactive graph nodes
- **Edge Rendering** - Connections with flow indicators
- **Layout Algorithms** - Force-directed, hierarchical, circular
- **Color Coding** - Health status visualization
- **Label Management** - Clear, non-overlapping labels

**Node Types:**
```rust
pub enum NodeType {
    Primal {
        primal_type: PrimalType,
        health: Health,
        capabilities: Vec<String>,
    },
    Chimera {
        composition: ChimeraComposition,
        health: Health,
    },
    External {
        service_type: String,
        status: ServiceStatus,
    },
}
```

**Edge Types:**
```rust
pub enum EdgeType {
    CapabilityInvocation {
        capability: String,
        request_count: u64,
        avg_latency_ms: f64,
    },
    DataFlow {
        data_type: String,
        bytes_per_second: u64,
    },
    Discovery {
        last_discovery: Timestamp,
    },
    Federation {
        federation_type: String,
    },
}
```

### 2.3 Real-Time Flow Animation

**Purpose:** Show live message flow between primals.

**Features:**
- **Particle System** - Animated particles along edges
- **Flow Direction** - Visual indicators of request/response
- **Traffic Volume** - Particle density indicates load
- **Message Types** - Color-coded by API call type
- **Timeline View** - Sequence diagram of interactions

**Animation System:**
```rust
pub struct FlowAnimator {
    particles: Vec<FlowParticle>,
    active_flows: HashMap<EdgeId, FlowState>,
    animation_speed: f32,
}

pub struct FlowParticle {
    position: Vec2,         // Current position on edge
    progress: f32,          // 0.0 to 1.0 along edge
    color: Color32,         // Message type color
    size: f32,              // Priority/size indicator
    message_type: String,   // API call name
}

impl FlowAnimator {
    pub fn update(&mut self, dt: f32, telemetry: &TelemetryStream) {
        // Create particles for new API calls
        for event in telemetry.events() {
            self.spawn_particle_for_event(event);
        }
        
        // Update particle positions
        for particle in &mut self.particles {
            particle.progress += dt * self.animation_speed;
        }
        
        // Remove completed particles
        self.particles.retain(|p| p.progress < 1.0);
    }
    
    pub fn render(&self, painter: &egui::Painter, graph: &PrimalGraph) {
        for particle in &self.particles {
            let pos = graph.calculate_position(particle.position, particle.progress);
            painter.circle_filled(pos, particle.size, particle.color);
        }
    }
}
```

### 2.4 Interactive Controls

**Purpose:** Enable user interaction with the topology.

**Features:**
- **Pan & Zoom** - Navigate large topologies
- **Node Selection** - Click to view details
- **Drag & Drop** - Reposition nodes
- **Filtering** - Show/hide by type, health, capability
- **Search** - Find primals by name or attribute
- **Context Menus** - Right-click actions

**Interaction Handler:**
```rust
pub struct InteractionHandler {
    camera: Camera2D,
    selected_node: Option<NodeId>,
    drag_state: Option<DragState>,
    filter: TopologyFilter,
}

impl InteractionHandler {
    pub fn handle_input(&mut self, response: &egui::Response, graph: &mut PrimalGraph) {
        // Zoom
        if response.hovered() {
            self.camera.zoom *= response.ctx.input(|i| i.zoom_delta());
        }
        
        // Pan
        if response.dragged_by(egui::PointerButton::Middle) {
            self.camera.offset += response.drag_delta();
        }
        
        // Select node
        if response.clicked() {
            if let Some(node_id) = self.find_node_at(response.interact_point) {
                self.selected_node = Some(node_id);
            }
        }
        
        // Drag node
        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some(node_id) = self.selected_node {
                let delta = self.camera.screen_to_world(response.drag_delta());
                graph.move_node(node_id, delta);
            }
        }
    }
}
```

### 2.5 Multi-View System

**Purpose:** Different perspectives on the ecosystem.

**Views:**

| View | Purpose | Primary Widget |
|------|---------|----------------|
| **Dashboard** | System overview, metrics | Charts, gauges |
| **Topology** | Primal graph visualization | Interactive graph |
| **Timeline** | Interaction sequences | Sequence diagram |
| **Health** | Status monitoring | Heat map |
| **Traffic** | Data flow analysis | Sankey diagram |
| **Capabilities** | Capability mesh | Matrix view |
| **Logs** | Event stream | Scrolling text |
| **Alerts** | Issue tracking | Alert list |

**View Interface:**
```rust
pub trait View {
    fn title(&self) -> &str;
    fn render(&mut self, ui: &mut egui::Ui, ctx: &ViewContext);
    fn update(&mut self, telemetry: &TelemetryUpdate);
    fn supports_filtering(&self) -> bool { false }
    fn supports_search(&self) -> bool { false }
}

pub struct ViewContext {
    pub api: Arc<BiomeOSApi>,
    pub state: Arc<Mutex<AppState>>,
    pub selected_primal: Option<String>,
    pub time_range: TimeRange,
}
```

---

## 3. Graph Visualization Specification

### 3.1 Node Rendering

**Visual Design:**

```
┌─────────────────────────────────────┐
│         [ToadStool Icon]            │  ← Primal type icon
│                                     │
│         ToadStool-1                 │  ← Primal name
│         Compute                     │  ← Primal type
│                                     │
│  🟢 Healthy    ⚡ 5 caps           │  ← Status + capabilities
│  📊 1.2k req/s  ⏱️ 12ms            │  ← Metrics
└─────────────────────────────────────┘
   ↑
   └─ Border color indicates health
```

**Node States:**

```rust
pub struct NodeVisualState {
    pub health_color: Color32,
    pub border_thickness: f32,
    pub glow_intensity: f32,
    pub icon: IconType,
    pub size: NodeSize,
}

impl NodeVisualState {
    pub fn from_health(health: &Health) -> Self {
        match health {
            Health::Healthy => Self {
                health_color: Color32::GREEN,
                border_thickness: 2.0,
                glow_intensity: 0.5,
                // ...
            },
            Health::Degraded => Self {
                health_color: Color32::YELLOW,
                border_thickness: 3.0,
                glow_intensity: 1.0,
                // ...
            },
            Health::Unhealthy => Self {
                health_color: Color32::RED,
                border_thickness: 4.0,
                glow_intensity: 1.5,
                // ...
            },
        }
    }
}
```

### 3.2 Edge Rendering

**Visual Design:**

```
[Node A] ═══════════════════════════> [Node B]
          ↑         ↑         ↑
          │         │         │
        Color    Thickness  Direction
        (type)   (volume)   (flow)
```

**Edge Properties:**

```rust
pub struct EdgeVisualState {
    pub color: Color32,           // Message type
    pub thickness: f32,           // Traffic volume
    pub style: LineStyle,         // Solid, dashed, dotted
    pub arrow_size: f32,          // Flow direction indicator
    pub animated: bool,           // Show particles
    pub label: Option<String>,    // Edge label (capability)
}

pub enum LineStyle {
    Solid,                        // Normal connection
    Dashed,                       // Intermittent connection
    Dotted,                       // Discovery only
    Gradient(Color32, Color32),   // Bi-directional flow
}
```

### 3.3 Layout Algorithms

**Force-Directed Layout (Default):**

```rust
pub struct ForceDirectedLayout {
    repulsion_strength: f32,      // How much nodes push apart
    attraction_strength: f32,     // How much edges pull together
    center_gravity: f32,          // Pull toward center
    iterations: usize,            // Simulation steps
    damping: f32,                 // Energy loss per step
}

impl ForceDirectedLayout {
    pub fn calculate_layout(&mut self, graph: &PrimalGraph) -> HashMap<NodeId, Vec2> {
        let mut positions = self.initialize_positions(graph);
        
        for _ in 0..self.iterations {
            // Apply repulsion between all nodes
            for (i, node_a) in graph.nodes().enumerate() {
                for node_b in graph.nodes().skip(i + 1) {
                    let repulsion = self.calculate_repulsion(node_a, node_b, &positions);
                    positions.get_mut(&node_a.id).unwrap().add(repulsion);
                    positions.get_mut(&node_b.id).unwrap().sub(repulsion);
                }
            }
            
            // Apply attraction along edges
            for edge in graph.edges() {
                let attraction = self.calculate_attraction(edge, &positions);
                positions.get_mut(&edge.from).unwrap().add(attraction);
                positions.get_mut(&edge.to).unwrap().sub(attraction);
            }
            
            // Apply damping
            for pos in positions.values_mut() {
                *pos *= self.damping;
            }
        }
        
        positions
    }
}
```

**Hierarchical Layout:**

```
Level 0:              [BiomeOS]
                          │
                    ┌─────┼─────┐
Level 1:        [ToadStool] [Songbird] [NestGate]
                    │           │
                ┌───┴───┐   ┌───┴───┐
Level 2:    [Worker1] [Worker2] [Cache] [DB]
```

**Circular Layout:**

```
          [Node1]
      [Node2]  [Node3]
    [Node8]  BiomeOS  [Node4]
      [Node7]  [Node5]
          [Node6]
```

### 3.4 Color Scheme

**Health Colors:**
- 🟢 **Healthy** - `#00FF00` (Green)
- 🟡 **Degraded** - `#FFFF00` (Yellow)
- 🔴 **Unhealthy** - `#FF0000` (Red)
- ⚪ **Unknown** - `#808080` (Gray)

**Primal Type Colors:**
- 🔵 **ToadStool** - `#0080FF` (Blue) - Compute
- 🟣 **Songbird** - `#8000FF` (Purple) - Orchestration
- 🟤 **NestGate** - `#FF8000` (Orange) - Storage
- 🟡 **BearDog** - `#FFD700` (Gold) - Security
- 🟢 **Squirrel** - `#00FF80` (Mint) - AI
- 🔷 **BiomeOS** - `#00FFFF` (Cyan) - Orchestration

**Message Type Colors:**
- 🟦 **Discovery** - Light Blue
- 🟪 **Capability** - Purple
- 🟧 **Data Transfer** - Orange
- 🟩 **Health Check** - Light Green
- 🟥 **Error** - Light Red

---

## 4. Real-Time Telemetry Integration

### 4.1 Telemetry Stream

**Purpose:** Ingest live events from the ecosystem.

**Event Types:**
```rust
pub enum TelemetryEvent {
    PrimalDiscovered {
        primal_id: String,
        primal_type: PrimalType,
        capabilities: Vec<String>,
        endpoint: String,
    },
    PrimalDisappeared {
        primal_id: String,
    },
    HealthUpdate {
        primal_id: String,
        health: Health,
        timestamp: Timestamp,
    },
    ApiCall {
        from: String,
        to: String,
        capability: String,
        latency_ms: f64,
        status_code: u16,
        timestamp: Timestamp,
    },
    DataTransfer {
        from: String,
        to: String,
        bytes: u64,
        data_type: String,
        timestamp: Timestamp,
    },
    CapabilityInvoked {
        primal_id: String,
        capability: String,
        timestamp: Timestamp,
    },
}
```

**Stream Handler:**
```rust
pub struct TelemetryStreamHandler {
    event_buffer: VecDeque<TelemetryEvent>,
    subscribers: Vec<Box<dyn TelemetrySubscriber>>,
    aggregator: TelemetryAggregator,
}

impl TelemetryStreamHandler {
    pub async fn process_events(&mut self) {
        while let Some(event) = self.event_buffer.pop_front() {
            // Aggregate metrics
            self.aggregator.process(&event);
            
            // Notify subscribers
            for subscriber in &mut self.subscribers {
                subscriber.on_event(&event);
            }
        }
    }
}

pub trait TelemetrySubscriber: Send + Sync {
    fn on_event(&mut self, event: &TelemetryEvent);
}
```

### 4.2 Performance Considerations

**Update Rate:**
- **Graph Layout:** 30 FPS (33ms per frame)
- **Animation:** 60 FPS (16ms per frame)
- **Telemetry:** Real-time (< 100ms latency)
- **Metrics Aggregation:** 1 Hz (1 second intervals)

**Optimization Strategies:**
1. **Incremental Layout** - Only recalculate changed nodes
2. **Level of Detail** - Simplify distant nodes
3. **Culling** - Don't render off-screen elements
4. **Batching** - Combine draw calls
5. **Caching** - Cache rendered nodes between frames

---

## 5. API Specification

### 5.1 Internal API (Current: Embedded)

**Interface to BiomeOS Core:**

```rust
pub trait VisualizationAPI {
    async fn get_topology(&self) -> Result<TopologySnapshot>;
    async fn get_primal_details(&self, primal_id: &str) -> Result<PrimalDetails>;
    async fn get_flow_events(&self, since: Timestamp) -> Result<Vec<FlowEvent>>;
    async fn subscribe_to_telemetry(&self) -> Result<TelemetryStream>;
}
```

### 5.2 External API (Future: Independent Primal)

**REST Endpoints:**

```
GET  /api/v1/topology
  → Returns current topology as graph structure
  Response: TopologySnapshot

GET  /api/v1/primals/{id}
  → Get detailed information about a specific primal
  Response: PrimalDetails

GET  /api/v1/flow-events?since={timestamp}
  → Get flow events since timestamp
  Response: Vec<FlowEvent>

POST /api/v1/render-graph
  Body: GraphRenderRequest
  → Render a custom graph visualization
  Response: SVG or PNG image

WS   /api/v1/live-topology
  → WebSocket stream of real-time topology updates
  Stream: TelemetryEvent
```

**Data Structures:**

```rust
#[derive(Serialize, Deserialize)]
pub struct TopologySnapshot {
    pub timestamp: Timestamp,
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct TopologyNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub health: String,
    pub position: Option<(f32, f32)>,
    pub capabilities: Vec<String>,
    pub metrics: HashMap<String, f64>,
}

#[derive(Serialize, Deserialize)]
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub traffic: TrafficMetrics,
    pub metadata: HashMap<String, Value>,
}
```

---

## 6. Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

**Goals:**
- Set up petalTongue module structure
- Integrate egui_graphs library
- Create basic graph rendering

**Deliverables:**
- [ ] Module structure in `ui/src/petaltongue/`
- [ ] `PrimalTopologyView` with static graph
- [ ] Node and edge rendering
- [ ] Basic layout algorithm (force-directed)

**Code Structure:**
```
ui/src/petaltongue/
├── mod.rs                    # Module exports
├── graph/                    # Graph rendering
│   ├── mod.rs
│   ├── nodes.rs
│   ├── edges.rs
│   └── layout.rs
├── views/                    # Visualization views
│   ├── topology.rs
│   ├── timeline.rs
│   └── traffic.rs
├── animation/                # Flow animation
│   ├── mod.rs
│   ├── particles.rs
│   └── flow_animator.rs
└── telemetry/                # Event integration
    ├── mod.rs
    ├── stream_handler.rs
    └── aggregator.rs
```

### Phase 2: Interactivity (Week 3-4)

**Goals:**
- Add interaction handlers
- Implement pan, zoom, select
- Add filtering and search

**Deliverables:**
- [ ] Pan and zoom functionality
- [ ] Node selection and details panel
- [ ] Drag and drop node repositioning
- [ ] Filter controls (by type, health, capability)
- [ ] Search functionality

### Phase 3: Real-Time (Week 5-6)

**Goals:**
- Integrate telemetry stream
- Implement flow animation
- Add timeline view

**Deliverables:**
- [ ] Telemetry stream integration
- [ ] Flow particle animation
- [ ] Real-time graph updates
- [ ] Timeline sequence diagram
- [ ] Traffic metrics overlay

### Phase 4: Polish (Week 7-8)

**Goals:**
- Performance optimization
- Visual polish
- Additional layout algorithms

**Deliverables:**
- [ ] Performance profiling and optimization
- [ ] Hierarchical and circular layouts
- [ ] Export functionality (PNG, SVG)
- [ ] Keyboard shortcuts
- [ ] Configuration persistence

### Phase 5: Evolution (Future)

**Goals:**
- Extract as independent primal
- Define capability contract
- Implement REST/WebSocket API

**Deliverables:**
- [ ] petalTongue as separate crate
- [ ] Capability registration
- [ ] REST API implementation
- [ ] WebSocket streaming
- [ ] Multi-client support

---

## 7. Testing Strategy

### 7.1 Unit Tests

**Graph Operations:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = PrimalNode::new("test-1", "ToadStool", Health::Healthy);
        assert_eq!(node.id, "test-1");
        assert_eq!(node.primal_type, "ToadStool");
    }

    #[test]
    fn test_edge_creation() {
        let edge = PrimalEdge::capability_invocation("a", "b", "compute");
        assert_eq!(edge.from, "a");
        assert_eq!(edge.to, "b");
    }

    #[test]
    fn test_force_directed_layout() {
        let mut graph = PrimalGraph::new();
        graph.add_node(PrimalNode::new("a", "ToadStool", Health::Healthy));
        graph.add_node(PrimalNode::new("b", "Songbird", Health::Healthy));
        graph.add_edge(PrimalEdge::capability_invocation("a", "b", "discover"));

        let layout = ForceDirectedLayout::default();
        let positions = layout.calculate_layout(&graph);

        assert_eq!(positions.len(), 2);
        assert!(positions.contains_key("a"));
        assert!(positions.contains_key("b"));
    }
}
```

### 7.2 Integration Tests

**Telemetry Integration:**
```rust
#[tokio::test]
async fn test_telemetry_stream_integration() {
    let mut handler = TelemetryStreamHandler::new();
    
    // Send discovery event
    handler.push_event(TelemetryEvent::PrimalDiscovered {
        primal_id: "test-1".to_string(),
        primal_type: PrimalType::ToadStool,
        capabilities: vec!["compute".to_string()],
        endpoint: "http://localhost:8080".to_string(),
    });

    handler.process_events().await;

    // Verify event was processed
    assert_eq!(handler.aggregator.primal_count(), 1);
}
```

### 7.3 Visual Tests

**Screenshot Comparison:**
- Capture graph rendering at key states
- Compare against reference images
- Detect visual regressions

**Performance Tests:**
- Measure frame rate with varying node counts
- Profile memory usage
- Benchmark layout algorithms

---

## 8. User Interface Mockups

### 8.1 Main Topology View

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ petalTongue - Primal Ecosystem Topology                              [_][□][×]│
├───────────────────────────────────────────────────────────────────────────────┤
│ [🔍 Search: ________] [Layout: ▼ Force] [Filter: All ▼] [📊 Metrics] [⚙️]    │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│                                                                               │
│                            ┌──────────────┐                                   │
│                            │   BiomeOS    │ ← Double-click for details       │
│                            │ Orchestrator │                                   │
│                            └──────┬───────┘                                   │
│                                   │                                           │
│                    ┌──────────────┼──────────────┐                            │
│                    │              │              │                            │
│                    ▼              ▼              ▼                            │
│            ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                │
│            │  ToadStool   │ │   Songbird   │ │   NestGate   │                │
│            │   🟢 100%    │ │   🟢 100%    │ │   🟡 85%     │                │
│            │  ⚡ 5 caps   │ │  ⚡ 8 caps   │ │  ⚡ 4 caps   │                │
│            └──────┬───────┘ └──────┬───────┘ └──────┬───────┘                │
│                   │                │                │                        │
│                   └────────────────┼────────────────┘                        │
│                                    │                                         │
│                                    ▼                                         │
│                            ┌──────────────┐                                   │
│                            │   BearDog    │                                   │
│                            │   🟢 100%    │                                   │
│                            │  ⚡ 6 caps   │                                   │
│                            └──────────────┘                                   │
│                                                                               │
│  ┌─ Selected: ToadStool-1 ──────────────────────────────────────────────┐   │
│  │ Health: Healthy (100%)                                                │   │
│  │ Type: Compute                                                         │   │
│  │ Endpoint: http://localhost:8080                                       │   │
│  │ Capabilities: 5 active                                                │   │
│  │   • container_runtime                                                 │   │
│  │   • manifest_parsing                                                  │   │
│  │   • workload_execution                                                │   │
│  │ Connections: 3 primals                                                │   │
│  │ Traffic: 1.2k req/s (avg latency: 12ms)                              │   │
│  │ [View Logs] [Configure] [Restart]                                    │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                               │
├───────────────────────────────────────────────────────────────────────────────┤
│ Zoom: [─────●─────] 100% │ Nodes: 5 │ Edges: 7 │ FPS: 60 │ Update: 0.5s ago │
└───────────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Flow Animation View

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ petalTongue - Real-Time Primal Interactions                          [_][□][×]│
├───────────────────────────────────────────────────────────────────────────────┤
│ [⏸️ Pause] [Speed: 1x ▼] [Filter: All ▼] [Timeline ▼]                        │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ToadStool ━━━━━━━━━●━●━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━> Songbird            │
│                      ↑ ↑                                                      │
│                      │ └─ "provision_storage" (5ms ago)                       │
│                      └─── "discover_service" (12ms ago)                       │
│                                                                               │
│  Songbird  ━━━━━━━━━━━━━━━━━━━━━━●━●━●━━━━━━━━━━━━━━━> NestGate             │
│                                  ↑ ↑ ↑                                        │
│                                  │ │ └─ "write_data" (2ms ago)                │
│                                  │ └─── "allocate_space" (4ms ago)            │
│                                  └───── "get_status" (8ms ago)                │
│                                                                               │
│  BearDog   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━●━━━━━━━━━━━━━> ToadStool            │
│                                         ↑                                     │
│                                         └─ "verify_signature" (15ms ago)      │
│                                                                               │
│  ┌─ Activity Log ────────────────────────────────────────────────────────┐   │
│  │ [12:34:58.123] ToadStool → Songbird: discover_service (OK, 8ms)      │   │
│  │ [12:34:58.456] Songbird → NestGate: provision_storage (OK, 12ms)     │   │
│  │ [12:34:58.789] BearDog → ToadStool: verify_signature (OK, 3ms)       │   │
│  │ [12:34:59.012] Songbird → NestGate: allocate_space (OK, 5ms)         │   │
│  │ [12:34:59.345] ToadStool → Songbird: register_workload (OK, 10ms)    │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                               │
│  Legend:                                                                      │
│    🟦 Discovery   🟪 Capability   🟧 Data Transfer   🟩 Health Check         │
│                                                                               │
├───────────────────────────────────────────────────────────────────────────────┤
│ Active Flows: 5 │ Messages/sec: 23 │ Avg Latency: 8ms │ Errors: 0           │
└───────────────────────────────────────────────────────────────────────────────┘
```

### 8.3 Timeline View

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ petalTongue - Interaction Timeline                                   [_][□][×]│
├───────────────────────────────────────────────────────────────────────────────┤
│ Time: [━━━━━━━━━━●━━━━━━━━━━━━━] Last 1 minute [◄] [►]                       │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  BiomeOS     │                                                                │
│              │                                                                │
│  ToadStool   │─────────●────────────────●─────────────────────●──────────    │
│              │         │                │                     │              │
│  Songbird    │         ▼                │                     ▼              │
│              │    discover_service      │                register_workload   │
│              │         │                ▼                     │              │
│  NestGate    │         │         provision_storage            │              │
│              │         │                │                     │              │
│  BearDog     │         ▼                │                     ▼              │
│              │    verify_signature      │              verify_registration   │
│              │                          ▼                                    │
│              │                   allocate_space                              │
│              │                                                                │
│              └───┬─────────┬─────────┬─────────┬─────────┬─────────┬────>   │
│                  0s        10s       20s       30s       40s       50s       │
│                                                                               │
│  ┌─ Selected Event ──────────────────────────────────────────────────────┐   │
│  │ Event: discover_service                                               │   │
│  │ From: ToadStool-1                                                     │   │
│  │ To: Songbird-1                                                        │   │
│  │ Timestamp: 12:34:58.123                                               │   │
│  │ Latency: 8ms                                                          │   │
│  │ Status: 200 OK                                                        │   │
│  │ Payload: {"capability": "compute", "version": "1.0"}                  │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                               │
├───────────────────────────────────────────────────────────────────────────────┤
│ Total Events: 127 │ Time Range: 60s │ [Export CSV] [Export JSON]            │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## 9. Configuration

### 9.1 User Configuration

```yaml
# ~/.config/biomeos/petaltongue.yaml

visualization:
  # Layout algorithm
  layout:
    type: "force-directed"  # force-directed, hierarchical, circular
    iterations: 100
    repulsion_strength: 1000.0
    attraction_strength: 0.1
    
  # Visual appearance
  appearance:
    theme: "dark"  # dark, light, auto
    node_size: "medium"  # small, medium, large
    edge_thickness_multiplier: 1.0
    animation_speed: 1.0
    show_labels: true
    show_metrics: true
    
  # Performance
  performance:
    max_fps: 60
    update_interval_ms: 500
    max_particles: 1000
    lod_enabled: true
    
  # Filtering
  filters:
    show_healthy: true
    show_degraded: true
    show_unhealthy: true
    show_unknown: false
    primal_types:
      - "ToadStool"
      - "Songbird"
      - "NestGate"
      - "BearDog"
      - "Squirrel"
      
  # Telemetry
  telemetry:
    enable_real_time: true
    buffer_size: 10000
    aggregation_interval_ms: 1000
```

### 9.2 Primal Configuration (Future)

```yaml
# When petalTongue becomes an independent primal

primal:
  name: "petalTongue"
  type: "visualization"
  version: "1.0.0"
  
capabilities:
  - category: "visualization"
    name: "graph-rendering"
    version: "1.0.0"
  - category: "visualization"
    name: "real-time-topology"
    version: "1.0.0"
  - category: "visualization"
    name: "flow-animation"
    version: "1.0.0"
  - category: "ui"
    name: "desktop-interface"
    version: "1.0.0"

api:
  rest:
    port: 8090
    endpoints:
      - path: "/api/v1/topology"
        method: "GET"
      - path: "/api/v1/primals/{id}"
        method: "GET"
      - path: "/api/v1/render-graph"
        method: "POST"
  websocket:
    port: 8091
    endpoint: "/api/v1/live-topology"

telemetry:
  sources:
    - "songbird://localhost:8081/telemetry"
    - "biomeos://localhost:8000/events"
```

---

## 10. Success Criteria

### 10.1 Functional Requirements

- [ ] **Graph Visualization**
  - [ ] Display all discovered primals as nodes
  - [ ] Show connections between primals as edges
  - [ ] Color-code nodes by health status
  - [ ] Label nodes and edges clearly

- [ ] **Interactivity**
  - [ ] Pan and zoom the topology
  - [ ] Select nodes to view details
  - [ ] Drag nodes to reposition
  - [ ] Filter by type, health, capability
  - [ ] Search for specific primals

- [ ] **Real-Time Updates**
  - [ ] Update topology when primals discovered/removed
  - [ ] Update health status in real-time
  - [ ] Animate API calls between primals
  - [ ] Show traffic volume indicators

- [ ] **Performance**
  - [ ] Maintain 60 FPS with up to 50 nodes
  - [ ] Maintain 30 FPS with up to 200 nodes
  - [ ] Telemetry latency < 100ms
  - [ ] Layout calculation < 50ms

### 10.2 Non-Functional Requirements

- [ ] **Usability**
  - [ ] Intuitive controls (industry standard)
  - [ ] Clear visual hierarchy
  - [ ] Accessible (color-blind friendly)
  - [ ] Keyboard shortcuts available

- [ ] **Reliability**
  - [ ] Handle primal disconnections gracefully
  - [ ] Recover from telemetry stream interruptions
  - [ ] Degrade gracefully under load

- [ ] **Maintainability**
  - [ ] Clean separation of concerns
  - [ ] Comprehensive tests (>80% coverage)
  - [ ] Clear documentation
  - [ ] Extensible architecture

### 10.3 Evolution Criteria

petalTongue should become an independent primal when:

1. **Complexity Threshold**
   - Codebase exceeds 10,000 lines
   - Requires dedicated team
   - Has complex rendering pipeline

2. **Resource Requirements**
   - Visualization consumes >25% CPU
   - Memory usage >500MB
   - Impacts BiomeOS performance

3. **Reusability**
   - 3+ other primals want visualization
   - External services need topology data
   - API requests from outside BiomeOS

4. **Organizational**
   - Dedicated UI/UX team formed
   - Separate roadmap and release cycle
   - Independent versioning needed

---

## 11. Security & Privacy

### 11.1 Data Access

**Principle:** petalTongue sees what the user sees.

- petalTongue operates with user's permissions
- No privileged access to primal internals
- Cannot bypass BiomeOS access controls
- All data via BiomeOS API (subject to auth)

### 11.2 Telemetry Privacy

**Sensitive Data Handling:**
- No payload contents displayed (only metadata)
- API call names visible, but not arguments
- Traffic volume shown, but not content
- Health status public, but not reasons

**Configuration:**
```yaml
privacy:
  show_api_payloads: false      # Never show sensitive data
  show_endpoint_urls: true      # URLs are not sensitive
  show_traffic_volume: true     # Volume is not sensitive
  show_latency: true            # Performance is not sensitive
  anonymize_external: false     # External services visible
```

### 11.3 Visual Scraping Protection

When petalTongue becomes independent:
- Require authentication for API access
- Rate limit topology requests
- Watermark exported images
- Audit visualization access

---

## 12. Future Enhancements

### 12.1 3D Visualization

```
┌───────────────────────────────────────────────────────────────────────┐
│                                                                       │
│                        ┌──────────┐                                   │
│                       │ BiomeOS  │ (Front)                            │
│                       └────┬─────┘                                    │
│                            │                                          │
│              ┌─────────────┼─────────────┐                            │
│              │                           │                            │
│         ┌────┴────┐                 ┌────┴────┐                       │
│        │ToadStool│ (Mid)           │Songbird │ (Mid)                  │
│        └────┬────┘                 └────┬────┘                        │
│             │                           │                             │
│        ┌────┴────┐                 ┌────┴────┐                        │
│       │NestGate │ (Back)          │ BearDog │ (Back)                  │
│       └─────────┘                 └─────────┘                         │
│                                                                       │
│   [Rotate: ↻ ↺] [Zoom: ± ] [View: Top/Side/Free]                    │
└───────────────────────────────────────────────────────────────────────┘
```

### 12.2 Geographic View

For distributed deployments:
```
World Map View:
┌───────────────────────────────────────────────────────────────────────┐
│                                                                       │
│  🌍 North America                     🌏 Asia                         │
│     └─ [ToadStool-US] 🟢                └─ [ToadStool-JP] 🟢         │
│     └─ [Songbird-US] 🟢                 └─ [Songbird-SG] 🟡          │
│                                                                       │
│  🌍 Europe                             🌏 Australia                   │
│     └─ [ToadStool-DE] 🟢                └─ [ToadStool-AU] 🟢         │
│     └─ [NestGate-UK] 🟡                                               │
│                                                                       │
│  Latency Overlay: Americas ↔ Asia: 180ms                             │
└───────────────────────────────────────────────────────────────────────┘
```

### 12.3 Capability Matrix

```
Capability Matrix View:
┌────────────────┬─────────────┬──────────┬──────────┬────────┐
│                │  ToadStool  │ Songbird │ NestGate │BearDog │
├────────────────┼─────────────┼──────────┼──────────┼────────┤
│ Compute        │     ●●●     │    ●     │          │        │
│ Orchestration  │             │   ●●●    │          │   ●    │
│ Storage        │             │          │   ●●●    │        │
│ Security       │      ●      │    ●     │    ●     │  ●●●   │
│ Discovery      │      ●      │   ●●●    │    ●     │   ●    │
│ Federation     │             │   ●●●    │          │        │
└────────────────┴─────────────┴──────────┴──────────┴────────┘
  ● = Basic    ●● = Advanced    ●●● = Expert
```

### 12.4 AI-Powered Insights

```
┌─ AI Topology Analysis ──────────────────────────────────────────┐
│                                                                  │
│  🤖 Detected Issues:                                             │
│   • NestGate-1 showing signs of degradation                     │
│   • Unusual traffic spike: ToadStool → Songbird (+300%)         │
│   • BearDog not responding to health checks                     │
│                                                                  │
│  💡 Recommendations:                                             │
│   • Consider adding redundant NestGate instance                 │
│   • Investigate cause of ToadStool traffic increase             │
│   • Restart BearDog service                                     │
│                                                                  │
│  📊 Predicted Load:                                              │
│   • Next hour: +45% traffic expected                            │
│   • Consider pre-scaling ToadStool                              │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 13. Glossary

**petalTongue** - The UI and visualization system for the ecoPrimals ecosystem. Named for its role in "tasting" and "speaking" the state of the ecosystem to users.

**Primal** - A single-purpose organism in the ecoPrimals ecosystem (ToadStool, Songbird, NestGate, BearDog, Squirrel).

**Topology** - The graph structure of primals and their connections.

**Node** - A visual representation of a primal in the graph.

**Edge** - A visual representation of a connection between primals.

**Flow** - The movement of messages/data between primals.

**Telemetry** - Real-time event stream from the ecosystem.

**Layout Algorithm** - Mathematical algorithm for positioning nodes in the graph.

**Force-Directed Layout** - Physics-based layout that treats nodes as particles with repulsion and edges as springs with attraction.

**Hierarchical Layout** - Tree-based layout with levels of hierarchy.

**Capability** - A specific function provided by a primal.

**Health Status** - Current operational state (Healthy, Degraded, Unhealthy, Unknown).

---

## 14. References

### Internal References
- [BiomeOS Architecture](../ARCHITECTURE.md)
- [Cross-Primal API Contracts](./CROSS_PRIMAL_API_CONTRACTS.md)
- [Primal Service Registration Standards](./PRIMAL_SERVICE_REGISTRATION_STANDARDS.md)
- [Service Discovery Specification](./SERVICE_DISCOVERY_SPEC.md)

### External References
- [egui - Immediate Mode GUI](https://github.com/emilk/egui)
- [egui_graphs - Graph visualization for egui](https://github.com/blitzarx1/egui_graphs)
- [petgraph - Graph data structures](https://github.com/petgraph/petgraph)
- [Force-Directed Graph Drawing](https://en.wikipedia.org/wiki/Force-directed_graph_drawing)
- [Graph Visualization Techniques](https://www.graphviz.org/)

---

## 15. Appendix: Migration Path

### From: Embedded in BiomeOS
```
biomeOS/ui/src/
├── views/
│   ├── primals.rs              # Table view
│   ├── dashboard.rs            # Metrics
│   └── ...
```

### To: petalTongue Module
```
biomeOS/ui/src/
├── petaltongue/                # NEW MODULE
│   ├── mod.rs
│   ├── graph/
│   ├── views/
│   ├── animation/
│   └── telemetry/
├── views/
│   ├── primals.rs              # Delegates to petalTongue
│   ├── dashboard.rs
│   └── ...
```

### Future: Independent Primal
```
petalTongue/                    # NEW PRIMAL
├── Cargo.toml
├── src/
│   ├── main.rs                 # Server entry point
│   ├── api/                    # REST/WebSocket API
│   ├── graph/                  # Core visualization
│   └── telemetry/              # Event ingestion
└── ui/                         # Frontend (if needed)
```

**Migration Strategy:**
1. **Phase 1:** Create `petaltongue/` module in BiomeOS UI
2. **Phase 2:** Migrate visualization code into module
3. **Phase 3:** Define internal API boundary
4. **Phase 4:** Extract as library crate
5. **Phase 5:** Create independent primal with API

---

**Specification Status:** Ready for Implementation  
**Next Steps:** Begin Phase 1 implementation  
**Estimated Timeline:** 8 weeks to Phase 4 completion

---

*petalTongue: See the ecosystem, understand the flow, trust the process.*

