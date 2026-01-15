# 🧠 Neural API Integration Specification
**Version**: 2.0  
**Date**: January 15, 2026  
**Status**: ✅ Integration Complete - Evolution In Progress  
**Authors**: PetalTongue Team + BiomeOS Core Team

---

## 📋 DOCUMENT PURPOSE

This specification defines the integration between PetalTongue and BiomeOS Neural API, including:
1. Current integration architecture
2. Available data and endpoints
3. Planned evolution features
4. Implementation requirements
5. Testing and validation

---

## 🏗️ ARCHITECTURE OVERVIEW

### Before Neural API (v1.x)
```
petalTongue
    ↓
  ┌─┴─────────────────────┐
  ↓                       ↓
Songbird              BiomeOS (HTTP)
  ↓                       ↓
individual primals    fragmented data
```

**Issues**:
- Multiple data sources
- Inconsistent topology
- No central coordination
- Fragmented primal state

---

### After Neural API (v2.0)
```
petalTongue
    ↓
Neural API (central coordinator)
    ↓
  ┌─┴──────────┬──────────┐
  ↓            ↓          ↓
BearDog    Songbird   ToadStool
```

**Benefits**:
- ✅ Single source of truth
- ✅ Consistent topology
- ✅ Aggregated metrics
- ✅ Built-in proprioception
- ✅ Central coordination

---

## 🔌 NEURAL API ENDPOINTS

### 1. Health Check
**Method**: `neural_api.health`  
**Purpose**: Verify Neural API availability

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.health",
  "params": {},
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "healthy",
    "version": "2.0.0"
  },
  "id": 1
}
```

---

### 2. Get Primals
**Method**: `neural_api.get_primals`  
**Purpose**: List all discovered primals

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.get_primals",
  "params": {},
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "primals": [
      {
        "id": "beardog-1",
        "name": "BearDog",
        "primal_type": "security",
        "endpoint": "unix:///tmp/beardog.sock",
        "capabilities": ["crypto", "keys", "lineage"],
        "health": "healthy",
        "trust_level": 3,
        "family_id": "nat0"
      }
    ]
  },
  "id": 2
}
```

---

### 3. Get Proprioception (SAME DAVE)
**Method**: `neural_api.get_proprioception`  
**Purpose**: System self-awareness data

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.get_proprioception",
  "params": {},
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "timestamp": "2026-01-15T01:44:00Z",
    "family_id": "nat0",
    "health": {
      "percentage": 100.0,
      "status": "healthy"
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
  },
  "id": 3
}
```

**SAME DAVE Components**:
- **S**ensory: Socket detection, active connections
- **A**wareness: Knowledge about ecosystem
- **M**otor: Ability to act (deploy, execute)
- **E**valuative: Health assessment, confidence

---

### 4. Get Metrics
**Method**: `neural_api.get_metrics`  
**Purpose**: Aggregated system metrics

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.get_metrics",
  "params": {},
  "id": 4
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
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
  },
  "id": 4
}
```

---

### 5. Get Topology (Enhanced)
**Method**: `neural_api.get_topology`  
**Purpose**: Graph structure with enriched data

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.get_topology",
  "params": {},
  "id": 5
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "nodes": [
      {
        "id": "beardog-1",
        "name": "BearDog",
        "type": "security",
        "status": "healthy",
        "health_percentage": 100.0,
        "trust_level": 3,
        "family_id": "nat0",
        "capabilities": ["crypto", "keys"]
      }
    ],
    "edges": [
      {
        "source": "beardog-1",
        "target": "songbird-1",
        "edge_type": "encryption",
        "weight": 1.0
      }
    ]
  },
  "id": 5
}
```

---

### 6. Save Graph (For Graph Builder)
**Method**: `neural_api.save_graph`  
**Purpose**: Save user-created graph

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.save_graph",
  "params": {
    "name": "my-deployment",
    "graph": {
      "nodes": [
        {
          "id": "node-1",
          "type": "primal_start",
          "primal": "beardog",
          "params": {
            "family_id": "nat0"
          }
        }
      ],
      "edges": []
    }
  },
  "id": 6
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "graph_id": "uuid-here",
    "saved": true
  },
  "id": 6
}
```

---

### 7. Execute Graph (For Graph Builder)
**Method**: `neural_api.execute_graph`  
**Purpose**: Execute saved graph

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.execute_graph",
  "params": {
    "graph_id": "uuid-here"
  },
  "id": 7
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "execution_id": "exec-uuid",
    "status": "running"
  },
  "id": 7
}
```

---

## 🔍 DISCOVERY PROTOCOL

### Socket Search Order

1. **XDG_RUNTIME_DIR** (highest priority)
   - `$XDG_RUNTIME_DIR/biomeos-neural-api-{family_id}.sock`

2. **User Runtime Directory**
   - `/run/user/{uid}/biomeos-neural-api-{family_id}.sock`

3. **System Temp** (development)
   - `/tmp/biomeos-neural-api-{family_id}.sock`

### Discovery Implementation

```rust
use petal_tongue_discovery::NeuralApiProvider;

// Automatic discovery
let provider = NeuralApiProvider::discover(None).await?;

// With specific family
let provider = NeuralApiProvider::discover(Some("staging")).await?;

// Check health
provider.health_check().await?;
```

---

## 🎨 PLANNED FEATURES

### Phase 1: Proprioception Visualization (Weeks 1-2)

**Feature**: Display SAME DAVE self-awareness

**UI Components**:
1. **Health Indicator**
   - Color-coded circle (green/yellow/red)
   - Percentage display
   - Status text

2. **Confidence Meter**
   - Progress bar or gauge
   - Numeric percentage

3. **SAME DAVE Panel**
   - Sensory: Active sockets count
   - Awareness: Known primals count
   - Motor: Capability checkboxes
   - Evaluative: Health status

4. **Timestamp**
   - Relative time ("2s ago")
   - Auto-refresh every 5s

**Data Structures**:
```rust
// crates/petal-tongue-core/src/proprioception.rs

pub struct ProprioceptionData {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub family_id: String,
    pub health: HealthData,
    pub confidence: f32,
    pub self_awareness: SelfAwarenessData,
    pub motor: MotorData,
    pub sensory: SensoryData,
}

pub struct HealthData {
    pub percentage: f32,
    pub status: HealthStatus, // Healthy | Degraded | Critical
}

pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}
```

**Implementation**:
```rust
// crates/petal-tongue-ui/src/proprioception_panel.rs

pub struct ProprioceptionPanel {
    data: Option<ProprioceptionData>,
    last_update: Instant,
}

impl ProprioceptionPanel {
    pub fn update(&mut self, provider: &NeuralApiProvider) {
        if self.last_update.elapsed() > Duration::from_secs(5) {
            if let Ok(data) = provider.get_proprioception().await {
                self.data = Some(data);
                self.last_update = Instant::now();
            }
        }
    }
    
    pub fn render(&self, ui: &mut egui::Ui) {
        if let Some(data) = &self.data {
            // Render health indicator
            // Render confidence meter
            // Render SAME DAVE components
        } else {
            ui.label("No proprioception data available");
        }
    }
}
```

---

### Phase 2: Metrics Dashboard (Week 3)

**Feature**: Real-time system metrics

**UI Components**:
1. **CPU Usage**
   - Progress bar with percentage
   - Sparkline (last 30 points)
   - Color-coded thresholds

2. **Memory Usage**
   - Progress bar with percentage
   - Used/Total display
   - Color-coded thresholds

3. **System Info**
   - Uptime (formatted: "1d 2h 34m")
   - Active primals count
   - Available graphs count
   - Running executions count

**Data Structures**:
```rust
// crates/petal-tongue-core/src/metrics.rs

pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub system: SystemResourceMetrics,
    pub neural_api: NeuralApiMetrics,
}

pub struct SystemResourceMetrics {
    pub cpu_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_percent: f32,
    pub uptime_seconds: u64,
}

pub struct CpuHistory {
    values: VecDeque<f32>, // Ring buffer, max 30 points
}
```

---

### Phase 3: Enhanced Topology (Week 4)

**Feature**: Health-aware topology visualization

**Enhancements**:
1. **Node Colors by Health**
   - Green: 100% healthy
   - Yellow: 50-99% degraded
   - Red: <50% critical

2. **Capability Badges**
   - 🔒 Security
   - 🎵 Discovery
   - ⚡ Compute
   - 📊 Metrics

3. **Edge Labels**
   - "security-provider"
   - "discovery"
   - "coordination"
   - "data-flow"

4. **Hover Tooltips**
   - Detailed primal info
   - Health percentage
   - Trust level
   - Capabilities list

**Implementation**:
```rust
// Modify: crates/petal-tongue-graph/src/visual_2d.rs

fn render_node(&self, node: &Node, ctx: &RenderContext) {
    // Determine color based on health
    let color = match node.health_percentage {
        100.0 => Color32::GREEN,
        h if h >= 50.0 => Color32::YELLOW,
        _ => Color32::RED,
    };
    
    // Draw node with health color
    ctx.painter.circle_filled(node.pos, radius, color);
    
    // Add capability badges
    for (i, cap) in node.capabilities.iter().enumerate() {
        let icon = capability_to_icon(cap);
        ctx.painter.text(
            node.pos + Vec2::new(20.0, i * 15.0),
            Align2::LEFT_CENTER,
            icon,
            font_id,
            Color32::WHITE,
        );
    }
}
```

---

### Phase 4: Visual Graph Builder (Weeks 5-7)

**Feature**: Drag-and-drop graph creation and execution

**Components**:

1. **Canvas**
   - Drag-and-drop area
   - Grid snapping
   - Zoom and pan
   - Node selection

2. **Node Palette**
   - Available node types
   - Draggable items
   - Categories:
     - Start nodes (primal_start)
     - Control nodes (wait_for, conditional)
     - Verification nodes

3. **Property Inspector**
   - Parameter forms for selected node
   - Validation
   - Help text

4. **Toolbar**
   - Save graph
   - Load graph
   - Execute
   - Clear canvas

**Data Structures**:
```rust
// crates/petal-tongue-ui/src/graph_builder/types.rs

pub struct GraphBuilderState {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub selected_node: Option<usize>,
    pub canvas_offset: Vec2,
    pub zoom: f32,
}

pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub position: Vec2,
    pub params: HashMap<String, Value>,
}

pub enum NodeType {
    PrimalStart { primal: String },
    Verification,
    WaitFor,
    Conditional,
}
```

**Graph Serialization**:
```rust
impl GraphBuilderState {
    pub fn to_neural_api_format(&self) -> NeuralApiGraph {
        NeuralApiGraph {
            nodes: self.nodes.iter().map(|n| NeuralApiNode {
                id: n.id.clone(),
                type_name: n.node_type.to_string(),
                params: n.params.clone(),
            }).collect(),
            edges: self.edges.iter().map(|e| NeuralApiEdge {
                source: e.source.clone(),
                target: e.target.clone(),
            }).collect(),
        }
    }
}
```

---

## 🧪 TESTING REQUIREMENTS

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_neural_api_discovery() {
        let provider = NeuralApiProvider::discover(Some("nat0")).await;
        assert!(provider.is_ok());
    }
    
    #[tokio::test]
    async fn test_get_proprioception() {
        let provider = NeuralApiProvider::discover(None).await.unwrap();
        let proprio = provider.get_proprioception().await;
        assert!(proprio.is_ok());
    }
    
    #[tokio::test]
    async fn test_get_metrics() {
        let provider = NeuralApiProvider::discover(None).await.unwrap();
        let metrics = provider.get_metrics().await;
        assert!(metrics.is_ok());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_proprioception_panel_rendering() {
    let mut panel = ProprioceptionPanel::new();
    let provider = NeuralApiProvider::discover(None).await.unwrap();
    
    panel.update(&provider).await;
    
    assert!(panel.data.is_some());
    assert_eq!(panel.data.unwrap().health.status, HealthStatus::Healthy);
}
```

### Manual Testing

1. **Start Neural API**:
   ```bash
   cd biomeOS
   target/release/nucleus serve --family nat0
   ```

2. **Start Primals**:
   ```bash
   plasmidBin/primals/beardog-server &
   plasmidBin/primals/songbird-orchestrator &
   ```

3. **Run PetalTongue**:
   ```bash
   cargo run --bin petal-tongue
   ```

4. **Verify**:
   - Proprioception panel shows data
   - Metrics dashboard updates
   - Topology shows health colors
   - Graph builder works

---

## 📊 SUCCESS CRITERIA

### Phase 1 Complete When:
- [ ] Proprioception panel renders correctly
- [ ] Health indicator shows color-coded status
- [ ] Confidence meter displays percentage
- [ ] SAME DAVE components all visible
- [ ] Auto-refresh works (every 5s)
- [ ] Graceful handling if Neural API unavailable

### Phase 2 Complete When:
- [ ] Metrics dashboard renders correctly
- [ ] CPU sparkline shows history
- [ ] Memory bar shows used/total
- [ ] Uptime formatted correctly
- [ ] Color-coded thresholds work
- [ ] Auto-refresh works (every 5s)

### Phase 3 Complete When:
- [ ] Nodes colored by health
- [ ] Capability badges visible
- [ ] Edge labels show connection types
- [ ] Hover tooltips work
- [ ] Layout is clean and readable

### Phase 4 Complete When:
- [ ] Drag-and-drop nodes works
- [ ] Can connect nodes with edges
- [ ] Parameter forms validate input
- [ ] Can save graphs to Neural API
- [ ] Can load graphs from Neural API
- [ ] Can execute graphs
- [ ] Execution status monitored

---

## 🔒 SECURITY CONSIDERATIONS

### Socket Security

- ✅ Unix sockets with 0600 permissions
- ✅ User-level isolation (/run/user/{uid}/)
- ✅ No network exposure by default

### Data Validation

- ✅ JSON-RPC request validation
- ✅ Response schema validation
- ✅ Timeout handling (30s)
- ✅ Error message sanitization

### Graph Execution Safety

- ⚠️ Graph validation before execution
- ⚠️ Resource limits (TBD with BiomeOS)
- ⚠️ Execution permissions (TBD)
- ⚠️ Audit logging (TBD)

---

## 📚 REFERENCES

### PetalTongue Documentation
- `NEURAL_API_EVOLUTION_ROADMAP.md` - Evolution plan
- `TECHNICAL_DEBT_NEURAL_API.md` - Debt tracking
- `BUILD_INSTRUCTIONS.md` - Build guide

### BiomeOS Documentation
- `NEURAL_API_EVOLUTION_JAN_15_2026.md` - Neural API architecture
- `PETALTONGUE_NEURAL_INTEGRATION_JAN_15_2026.md` - Integration details

### Cross-Primal
- `wateringHole/petaltongue/NEURAL_API_INTEGRATION_RESPONSE.md` - Response
- `wateringHole/INTER_PRIMAL_INTERACTIONS.md` - Coordination

---

## 🔄 CHANGE LOG

### Version 2.0 (January 15, 2026)
- ✅ Neural API integration complete
- ✅ Added proprioception endpoint
- ✅ Added metrics endpoint
- ✅ Enhanced topology with health data
- ✅ Planned graph builder features

### Version 1.0 (January 13, 2026)
- ✅ Initial BiomeOS HTTP integration
- ✅ Basic primal discovery
- ✅ Simple topology visualization

---

**Specification Version**: 2.0  
**Last Updated**: January 15, 2026  
**Status**: ✅ **Active Development**

🌸 **Neural API integration enables revolutionary features!** 🧠✨

