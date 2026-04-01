# Collaborative Intelligence Integration Specification

**Version**: 1.0  
**Date**: January 11, 2026  
**Status**: Planning  
**Priority**: High (Critical Path)  
**Timeline**: 4 weeks  

---

## 🎯 Vision

Enable **Human-AI Collaboration as Equals** through interactive graph editing and real-time AI reasoning display.

### Goals

1. **View**: Live graph execution visualization
2. **Understand**: AI reasoning transparency (why decisions are made)
3. **Modify**: Real-time graph editing (active collaboration)
4. **Configure**: Pre-deployment niche configuration (bootstrap systems)
5. **Learn**: Mutual learning (AI learns from user, user learns from AI)

### Impact

- **Before**: 2-4 weeks to deploy new niche (AI learning curve)
- **After**: 2-4 days to deploy new niche (user bootstraps, AI learns)
- **Result**: 10x faster deployments, better decisions through collaboration

---

## 📋 Requirements

### 1. Interactive Graph Editor

#### 1.1 Graph Manipulation UI

**Capability**: User can visually create and modify graphs

**Components**:
- **Canvas**: Interactive graph canvas (egui-based)
- **Node Palette**: Draggable node types
- **Edge Editor**: Visual connection creation
- **Properties Panel**: Node configuration UI

**User Stories**:
- As a user, I can drag nodes onto the canvas
- As a user, I can connect nodes with edges (dependencies)
- As a user, I can configure node properties
- As a user, I can delete nodes and edges
- As a user, I can undo/redo changes

**Technical Requirements**:
```rust
// Graph data model
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub properties: serde_json::Value,
    pub position: (f32, f32),
}

pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: DependencyType,
}

pub struct Graph {
    pub id: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
}
```

#### 1.2 Drag-and-Drop System

**Capability**: Intuitive drag-and-drop interface

**Requirements**:
- Drag nodes from palette to canvas
- Drag nodes to reposition
- Drag edge endpoints to connect nodes
- Snap-to-grid for alignment
- Visual feedback during drag

**Technical Implementation**:
- egui drag-and-drop primitives
- Canvas coordinate system
- Hit detection for nodes/edges
- Visual hover states

#### 1.3 Live Execution Visualization

**Capability**: Real-time graph execution display

**Requirements**:
- Node status indicators (running, completed, failed, pending)
- Progress indicators (percentage, time elapsed)
- Resource usage display (CPU, memory, network)
- Execution flow animation (highlight active paths)
- AI reasoning display (why this node next?)

**Visual Design**:
```
Node States:
  ⚪ Pending   - Gray, waiting
  🔵 Running   - Blue, animated pulse
  ✅ Completed - Green, checkmark
  ❌ Failed    - Red, error icon
  ⏸️ Paused    - Yellow, pause icon
```

---

### 2. JSON-RPC Methods

#### 2.1 Method Specifications

**Protocol**: tarpc PRIMARY for inter-primal RPC; JSON-RPC 2.0 universal fallback (line-delimited over UDS); HTTP for external/browser fallback

##### `ui.graph.editor_open`

Open graph editor for a specific graph.

```rust
// Request
{
  "jsonrpc": "2.0",
  "method": "ui.graph.editor_open",
  "params": {
    "graph_id": "string"
  },
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "graph": Graph,
    "template_info": TemplateInfo | null
  },
  "id": 1
}
```

##### `ui.graph.add_node`

Add a node to the graph.

```rust
// Request
{
  "jsonrpc": "2.0",
  "method": "ui.graph.add_node",
  "params": {
    "graph_id": "string",
    "node": GraphNode
  },
  "id": 2
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "node_id": "string",
    "validation": ValidationResult
  },
  "id": 2
}
```

##### `ui.graph.modify_node`

Modify an existing node.

```rust
// Request
{
  "jsonrpc": "2.0",
  "method": "ui.graph.modify_node",
  "params": {
    "graph_id": "string",
    "node_id": "string",
    "changes": serde_json::Value
  },
  "id": 3
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "success": bool,
    "validation": ValidationResult
  },
  "id": 3
}
```

##### `ui.graph.remove_node`

Remove a node from the graph.

```rust
// Request
{
  "jsonrpc": "2.0",
  "method": "ui.graph.remove_node",
  "params": {
    "graph_id": "string",
    "node_id": "string"
  },
  "id": 4
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "success": bool,
    "affected_edges": Vec<String>
  },
  "id": 4
}
```

##### `ui.graph.add_edge`

Add an edge (dependency) between nodes.

```rust
// Request
{
  "jsonrpc": "2.0",
  "method": "ui.graph.add_edge",
  "params": {
    "graph_id": "string",
    "from": "string",
    "to": "string",
    "edge_type": "DependencyType"
  },
  "id": 5
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "edge_id": "string",
    "validation": ValidationResult
  },
  "id": 5
}
```

##### `ui.graph.save_template`

Save graph as a reusable template.

```rust
// Request
{
  "jsonrpc": "2.0",
  "method": "ui.graph.save_template",
  "params": {
    "graph_id": "string",
    "name": "string",
    "description": "string",
    "tags": Vec<String>
  },
  "id": 6
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "template_id": "string",
    "saved_at": "timestamp"
  },
  "id": 6
}
```

##### `ui.graph.apply_template`

Load a template into the graph editor.

```rust
// Request
{
  "jsonrpc": "2.0",
  "method": "ui.graph.apply_template",
  "params": {
    "template_id": "string",
    "merge": bool  // true = merge with existing, false = replace
  },
  "id": 7
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "graph": Graph,
    "nodes_added": u32,
    "edges_added": u32
  },
  "id": 7
}
```

##### `ui.graph.get_preview`

Preview execution plan for a graph.

```rust
// Request
{
  "jsonrpc": "2.0",
  "method": "ui.graph.get_preview",
  "params": {
    "graph": Graph
  },
  "id": 8
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "execution_order": Vec<String>,
    "estimated_duration": "duration",
    "resource_requirements": ResourceEstimate,
    "validation_warnings": Vec<Warning>
  },
  "id": 8
}
```

---

### 3. Real-Time Streaming

#### 3.1 WebSocket Support

**Capability**: Bi-directional real-time communication

**Protocol**: WebSocket over existing Unix socket connection

**Implementation**:
- Use `tokio-tungstenite` for WebSocket support (pure Rust)
- Upgrade tarpc connection to WebSocket when streaming needed
- Fallback to HTTP polling if WebSocket unavailable

**Message Format**:
```rust
pub enum StreamMessage {
    // Graph execution updates
    NodeStatusUpdate {
        graph_id: String,
        node_id: String,
        status: NodeStatus,
        timestamp: SystemTime,
    },
    
    // Progress updates
    ProgressUpdate {
        graph_id: String,
        node_id: String,
        progress: f32,  // 0.0 - 1.0
        message: String,
    },
    
    // AI reasoning
    ReasoningUpdate {
        graph_id: String,
        reasoning: AIReasoning,
    },
    
    // Resource usage
    ResourceUpdate {
        graph_id: String,
        node_id: String,
        resources: ResourceUsage,
    },
    
    // Errors
    ErrorUpdate {
        graph_id: String,
        node_id: String,
        error: ErrorInfo,
    },
}
```

#### 3.2 Node Status Display

**Capability**: Real-time node status visualization

**Requirements**:
- Status badge on each node
- Progress bar for running nodes
- Error details on hover
- Completion time display
- Resource usage indicators

#### 3.3 AI Reasoning Display

**Capability**: Transparent AI decision explanation

**Requirements**:
- "Why this node?" explanation
- Confidence scores
- Alternative options considered
- Data sources used
- Historical pattern references

**UI Design**:
```
┌─────────────────────────────────────┐
│ 🤖 AI Reasoning                     │
├─────────────────────────────────────┤
│ Decision: Execute Node A next       │
│ Confidence: 87%                     │
│                                     │
│ Why:                                │
│ • Highest priority (user pattern)  │
│ • Resources available (CPU: 40%)   │
│ • Dependencies satisfied            │
│                                     │
│ Alternatives Considered:            │
│ • Node B (73%) - lower priority    │
│ • Node C (45%) - missing deps      │
│                                     │
│ Data Sources:                       │
│ • User history (last 10 graphs)    │
│ • Community patterns (500 graphs)  │
│ • Resource availability (live)     │
└─────────────────────────────────────┘
```

#### 3.4 Conflict Resolution

**Capability**: Handle concurrent modifications gracefully

**Scenarios**:
1. **User vs AI modification**: User always wins, AI learns
2. **User vs User**: Last write wins, show conflict warning
3. **Execution vs Modification**: Allow safe modifications, validate before applying

**Conflict UI**:
```
⚠️ Conflict Detected
─────────────────────
Your change: Modified Node A timeout
AI suggestion: Modified Node A priority

Action:
  [ Keep My Change ]  [ Use AI Suggestion ]  [ Merge Both ]
```

---

## 🏗️ Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                       petalTongue                           │
│                                                             │
│  ┌────────────────┐         ┌─────────────────┐           │
│  │  Graph Editor  │────────▶│  JSON-RPC API   │           │
│  │    (egui)      │         │   (8 methods)   │           │
│  └────────────────┘         └─────────────────┘           │
│         │                            │                      │
│         │                            │                      │
│  ┌────────────────┐         ┌─────────────────┐           │
│  │   WebSocket    │◀────────│  Stream Handler │           │
│  │   Renderer     │         │   (live data)   │           │
│  └────────────────┘         └─────────────────┘           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
         │                            ▲
         │ RPC                        │ WebSocket
         ▼                            │
┌─────────────────────────────────────────────────────────────┐
│                        biomeOS                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Squirrel  │  │  NestGate   │  │  Songbird   │        │
│  │    (AI)     │  │  (Storage)  │  │ (Discovery) │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

```
1. User opens editor:
   petalTongue → ui.graph.editor_open() → biomeOS
   biomeOS → returns Graph + AI suggestions
   petalTongue → renders interactive canvas

2. User modifies graph:
   User drags node → petalTongue validates locally
   petalTongue → ui.graph.add_node() → biomeOS
   biomeOS → validates with Songbird, BearDog
   biomeOS → Squirrel learns from modification
   biomeOS → returns validation result
   petalTongue → updates UI

3. User deploys graph:
   petalTongue → deploys to biomeOS
   biomeOS → starts execution
   biomeOS → WebSocket stream (status updates)
   petalTongue → renders live execution
   petalTongue → shows AI reasoning

4. User saves template:
   petalTongue → ui.graph.save_template() → biomeOS
   biomeOS → stores in NestGate
   biomeOS → Squirrel learns pattern
   biomeOS → returns template_id
```

---

## 🎯 Implementation Plan

### Week 1: Foundation

**Goals**: Interactive canvas + data model

**Tasks**:
1. Design and implement graph data structures
2. Create egui-based graph canvas widget
3. Implement drag-and-drop system
4. Basic node add/remove functionality
5. Unit tests for data model

**Deliverables**:
- Graph data model (Node, Edge, Graph)
- Interactive canvas widget
- Drag-and-drop working
- Unit tests passing

### Week 2: RPC Methods

**Goals**: All 8 JSON-RPC methods implemented

**Tasks**:
1. Implement `ui.graph.editor_open`
2. Implement `ui.graph.add_node`
3. Implement `ui.graph.modify_node`
4. Implement `ui.graph.remove_node`
5. Implement `ui.graph.add_edge`
6. Implement `ui.graph.save_template`
7. Implement `ui.graph.apply_template`
8. Implement `ui.graph.get_preview`
9. Integration tests for all methods

**Deliverables**:
- 8 RPC methods implemented
- RPC client/server tests
- Integration with existing IPC layer

### Week 3: Real-Time Streaming

**Goals**: WebSocket + live updates

**Tasks**:
1. Add `tokio-tungstenite` dependency
2. Implement WebSocket upgrade mechanism
3. Implement stream message types
4. Create status display UI
5. Create AI reasoning display UI
6. Implement conflict resolution UI
7. Integration testing with biomeOS

**Deliverables**:
- WebSocket streaming working
- Live execution visualization
- AI reasoning display
- Conflict resolution UI

### Week 4: Polish & Integration

**Goals**: Production-ready

**Tasks**:
1. End-to-end testing with biomeOS
2. Performance optimization
3. Error handling and edge cases
4. UI/UX polish
5. Documentation
6. Example graphs and templates

**Deliverables**:
- Production-ready integration
- Full test coverage
- Documentation complete
- Example templates

---

## 🧪 Testing Strategy

### Unit Tests

- Graph data model operations
- Drag-and-drop coordinate system
- Validation logic
- RPC method handlers

### Integration Tests

- RPC method round-trips
- WebSocket connection establishment
- Stream message handling
- Template save/load cycle

### End-to-End Tests

- Complete user workflow:
  1. Open editor
  2. Create graph
  3. Deploy graph
  4. Watch execution
  5. Modify during execution
  6. Save as template

### Performance Tests

- Large graph rendering (1000+ nodes)
- WebSocket message throughput
- UI responsiveness during streaming
- Memory usage with live updates

---

## 📊 Success Criteria

### MVP (Minimum Viable Product)

1. ✅ User can view live graph execution
2. ✅ User can modify graph before deployment
3. ✅ AI suggests improvements
4. ✅ User can save templates
5. ✅ User can load templates

### Full Feature Set

6. ✅ User can modify graph during execution
7. ✅ AI learns from user modifications
8. ✅ AI provides reasoning for suggestions
9. ✅ Community template sharing
10. ✅ Resource planning and optimization
11. ✅ Security validation
12. ✅ Coordination validation

---

## 🌸 TRUE PRIMAL Alignment

### Principles Applied

✅ **Human and AI as Equals**: Neither subservient, both contribute  
✅ **Transparent Reasoning**: Every AI suggestion includes "why"  
✅ **User Always in Control**: User can override any AI decision  
✅ **Learn Together**: AI learns from user, user learns from AI  
✅ **Bootstrap Fast**: User expertise + AI learning = 10x faster  

### Self-Discovery

- Graph editor discovers available node types at runtime
- Template system discovers community patterns
- AI reasoning adapts to user preferences

### Graceful Degradation

- Works without WebSocket (polling fallback)
- Works without AI suggestions (manual mode)
- Works without templates (blank canvas)

### No Hard Dependencies

- WebSocket is optional (HTTP fallback)
- AI suggestions are optional (manual mode)
- Templates are optional (create from scratch)

---

## 📚 References

- **biomeOS Handoff**: Collaborative Intelligence spec
- **Existing**: `specs/BIDIRECTIONAL_UUI_ARCHITECTURE.md` (SAME DAVE)
- **Existing**: `specs/DISCOVERY_INFRASTRUCTURE_EVOLUTION_SPECIFICATION.md`
- **Tracking**: `COLLABORATIVE_INTELLIGENCE_TRACKING.md` (root)

---

**Status**: ✅ Specification complete  
**Next**: Implementation (4 weeks)  
**Alignment**: TRUE PRIMAL principles applied  
**Impact**: 10x faster deployments through human-AI collaboration

