# 🎨 Interactive Paint Mode - biomeOS Modeling

**Date**: January 15, 2026  
**Goal**: Add interactive node/edge creation to paint mode for biomeOS prototyping  
**Status**: 🚧 **IN PROGRESS**

---

## 🎯 The Problem

Current paint mode is **display-only**:
- ❌ Can't add new nodes
- ❌ Can't connect nodes
- ❌ Can't edit node properties
- ❌ Can't delete nodes
- ❌ Static scenario display only

**User feedback**: "should i be able to do things in the paint ui? it was fairly static, other than layout switching."

---

## ✨ The Solution: Interactive Canvas

Transform paint mode into an **interactive modeling tool** for biomeOS:

### Phase 1: Node Creation ✅
- **Double-click canvas** → Create new node at cursor position
- **Node types**: Model after biomeOS primals
  - NUCLEUS (coordination)
  - BearDog (security)
  - Songbird (discovery)
  - Toadstool (compute)
  - NestGate (storage)
  - Custom (user-defined)

### Phase 2: Edge Creation
- **Drag from node to node** → Create connection
- **Edge types**: Model after biomeOS capabilities
  - Security Provider/Consumer
  - Discovery Provider/Consumer
  - Compute Request/Response
  - Data Storage/Retrieval
  - Coordination Link

### Phase 3: Node Editing
- **Right-click node** → Context menu
  - Edit name
  - Change type
  - Set capabilities
  - Adjust properties (CPU, memory, etc.)
  - Delete node

### Phase 4: Selection & Deletion
- **Click to select** node (already works)
- **Delete key** → Remove selected node(s)
- **Ctrl+Click** → Multi-select
- **Drag-select** → Box selection

### Phase 5: Tool Palette
```
┌─────────────────────┐
│ 🖱️ Select           │ ← Default
│ ➕ Add Node         │ ← Create mode
│ 🔗 Connect          │ ← Edge creation
│ ✏️ Edit             │ ← Property editing
│ 🗑️ Delete           │ ← Remove mode
├─────────────────────┤
│ Node Types:         │
│ 🎯 NUCLEUS          │
│ 🐻 BearDog          │
│ 🐦 Songbird         │
│ 🍄 Toadstool        │
│ 🏠 NestGate         │
│ ⚙️ Custom           │
└─────────────────────┘
```

---

## 🧬 Modeling for biomeOS

### Node Properties (Match PrimalInfo):
```rust
struct InteractiveNode {
    id: String,           // Auto-generated or user-set
    name: String,         // User-editable
    primal_type: String,  // NUCLEUS, BearDog, etc.
    position: (f32, f32), // Canvas coordinates
    capabilities: Vec<String>,  // What it provides/requires
    health: u8,           // 0-100 (default: 100)
    properties: HashMap<String, Value>,  // Extensible
}
```

### Edge Properties (Match Capabilities):
```rust
struct InteractiveEdge {
    from: String,         // Source node ID
    to: String,           // Target node ID
    edge_type: EdgeType,  // Security, Discovery, Compute, Data
    bidirectional: bool,  // Can data flow both ways?
    required: bool,       // Is this connection mandatory?
}

enum EdgeType {
    Security,      // BearDog ← → others
    Discovery,     // Songbird ← → others
    Compute,       // Toadstool ← → others
    Storage,       // NestGate ← → others
    Coordination,  // NUCLEUS ← → others
    Custom(String),
}
```

### Validation Rules (biomeOS Compatible):
1. **NUCLEUS** must exist (coordination center)
2. **Security** should connect to all primals (BearDog)
3. **Discovery** should connect to NUCLEUS (Songbird)
4. **Capabilities** must match (provider → consumer)
5. **No cycles** in dependency graph (optional enforcement)

---

## 🎮 User Interactions

### Current (v2.2.0):
- **Pan**: Drag canvas with mouse
- **Zoom**: Scroll wheel
- **Select**: Click node
- **Layout**: Switch via top menu

### Added (v2.3.0):
- **Create Node**: Double-click empty space
- **Create Edge**: Drag from node → node
- **Edit Node**: Right-click → menu
- **Delete**: Select + Delete key
- **Multi-select**: Ctrl+Click or drag-box
- **Undo/Redo**: Ctrl+Z / Ctrl+Shift+Z

---

## 🛠️ Implementation Plan

### Step 1: Input Handling
```rust
// In Visual2DRenderer::handle_input()
if response.double_clicked() {
    // Create node at mouse position
    let world_pos = self.screen_to_world(mouse_pos, screen_center);
    self.create_node_at(world_pos);
}

if response.drag_started() && hovering_node {
    // Start edge creation
    self.drawing_edge = Some(EdgeDraft {
        from: hovered_node_id,
        current_pos: mouse_pos,
    });
}

if response.drag_released() && drawing_edge {
    if let Some(target_node) = self.hovered_node {
        // Complete edge connection
        self.create_edge(edge_draft.from, target_node);
    }
    self.drawing_edge = None;
}
```

### Step 2: Node Creation
```rust
fn create_node_at(&mut self, world_pos: Position) {
    let mut graph = self.graph.write().unwrap();
    
    let new_node = PrimalInfo {
        id: format!("node-{}", uuid::Uuid::new_v4()),
        name: format!("New Node {}", graph.node_count() + 1),
        primal_type: self.current_node_type.clone(), // From tool palette
        position: world_pos,
        capabilities: vec![],
        health: PrimalHealthStatus::Healthy,
        properties: Properties::new(),
        // ... other fields
    };
    
    graph.add_node(new_node);
}
```

### Step 3: Edge Creation
```rust
fn create_edge(&mut self, from: String, to: String) {
    let mut graph = self.graph.write().unwrap();
    
    graph.add_edge(from, to, EdgeType::Coordination);
    
    // Optional: Validate capability matching
    self.validate_edge_capabilities(&from, &to);
}
```

### Step 4: Tool Palette
```rust
enum PaintTool {
    Select,
    AddNode(NodeType),
    Connect,
    Edit,
    Delete,
}

struct PaintToolPalette {
    active_tool: PaintTool,
    node_type: String,  // Current node type to create
}

impl PaintToolPalette {
    fn render(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui.button("🖱️ Select").clicked() {
                self.active_tool = PaintTool::Select;
            }
            if ui.button("➕ Add Node").clicked() {
                self.active_tool = PaintTool::AddNode(self.node_type.clone());
            }
            if ui.button("🔗 Connect").clicked() {
                self.active_tool = PaintTool::Connect;
            }
            // ... etc
        });
    }
}
```

---

## 🎨 Paint Mode Controls

### Keyboard Shortcuts:
- **Double-Click**: Create node
- **Delete**: Remove selected
- **Ctrl+Z**: Undo
- **Ctrl+Shift+Z**: Redo
- **Ctrl+S**: Save scenario to JSON
- **Ctrl+O**: Load scenario from JSON
- **N**: Switch to NUCLEUS node type
- **B**: Switch to BearDog node type
- **S**: Switch to Songbird node type
- **T**: Switch to Toadstool node type

### Mouse Actions:
- **Left-click**: Select node
- **Double-click**: Create node
- **Right-click**: Context menu
- **Drag node**: Move node
- **Drag empty space**: Pan canvas
- **Drag node-to-node**: Create edge
- **Scroll**: Zoom

---

## 🧪 Testing Strategy

### Unit Tests:
```rust
#[test]
fn test_node_creation_at_position() {
    let renderer = setup_paint_renderer();
    renderer.create_node_at(Position::new_2d(100.0, 200.0));
    
    let graph = renderer.graph.read().unwrap();
    assert_eq!(graph.node_count(), 1);
    
    let node = graph.nodes().next().unwrap();
    assert_eq!(node.position.x, 100.0);
    assert_eq!(node.position.y, 200.0);
}

#[test]
fn test_edge_creation_between_nodes() {
    let renderer = setup_paint_renderer_with_nodes();
    renderer.create_edge("node-1".to_string(), "node-2".to_string());
    
    let graph = renderer.graph.read().unwrap();
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn test_capability_validation() {
    // BearDog (security provider) → NUCLEUS (security consumer)
    assert!(validate_connection("beardog-1", "nucleus-1"));
    
    // Random node → BearDog (invalid: BearDog provides, doesn't consume)
    assert!(!validate_connection("custom-1", "beardog-1"));
}
```

### E2E Tests:
```rust
#[test]
fn test_paint_mode_workflow() {
    // 1. Open paint mode
    let app = launch_paint_mode();
    
    // 2. Create NUCLEUS
    app.double_click(400.0, 300.0);
    app.select_node_type("NUCLEUS");
    
    // 3. Create BearDog
    app.double_click(200.0, 300.0);
    app.select_node_type("BearDog");
    
    // 4. Connect them
    app.drag_from_to("beardog-1", "nucleus-1");
    
    // 5. Verify
    assert_eq!(app.node_count(), 2);
    assert_eq!(app.edge_count(), 1);
    
    // 6. Save scenario
    app.save_scenario("test-ecosystem.json");
}
```

---

## 🚀 Benefits

### 1. Rapid Prototyping
- Quickly model biomeOS ecosystems
- Test different primal topologies
- Validate capability requirements

### 2. Co-Evolution with biomeOS
- Same data structures (PrimalInfo, Properties)
- Same capability model
- Export to JSON → load in biomeOS

### 3. Visual Debugging
- See primal connections
- Identify missing capabilities
- Debug coordination flows

### 4. Documentation
- Create diagrams for whitepapers
- Export ecosystem visualizations
- Share primal architectures

---

## 📊 Example Workflow

### Creating a Basic Ecosystem:

1. **Open paint mode**:
   ```bash
   cargo run --bin petal-tongue -- --scenario sandbox/scenarios/paint-simple.json
   ```

2. **Create NUCLEUS** (double-click center):
   - Sets up coordination center
   - Auto-assigns coordination capability

3. **Create BearDog** (double-click left):
   - Drag from BearDog → NUCLEUS
   - Creates "security-provider" edge

4. **Create Songbird** (double-click right):
   - Drag from Songbird → NUCLEUS
   - Creates "discovery-provider" edge

5. **Create Toadstool** (double-click bottom):
   - Drag from NUCLEUS → Toadstool
   - Creates "compute-consumer" edge

6. **Save ecosystem**:
   - File → Save As → `my-ecosystem.json`
   - Load in biomeOS for testing

---

## 🎯 Success Criteria

- [ ] Can create nodes with double-click
- [ ] Can drag to create edges
- [ ] Can edit node properties
- [ ] Can delete selected nodes
- [ ] Tool palette shows available node types
- [ ] Saved scenarios are biomeOS-compatible
- [ ] Undo/redo works correctly
- [ ] Keyboard shortcuts functional
- [ ] Unit tests passing
- [ ] E2E tests passing

---

## 🔄 Integration with biomeOS

### Export Format (Compatible):
```json
{
  "name": "Custom Ecosystem",
  "version": "2.0.0",
  "mode": "live-ecosystem",
  "ecosystem": {
    "primals": [
      {
        "id": "nucleus-1",
        "name": "My NUCLEUS",
        "type": "coordination",
        "capabilities": ["coordinate", "graph-exec"],
        "position": { "x": 400, "y": 300 }
      },
      {
        "id": "beardog-1",
        "name": "Security Layer",
        "type": "security",
        "capabilities": ["auth", "authz"],
        "position": { "x": 200, "y": 300 }
      }
    ],
    "connections": [
      {
        "from": "beardog-1",
        "to": "nucleus-1",
        "type": "security-provider"
      }
    ]
  }
}
```

### Import to biomeOS:
```bash
# 1. Create in petalTongue paint mode
# 2. Save as my-ecosystem.json
# 3. Load in biomeOS

cd biomeOS
./nucleus bootstrap --scenario ../petalTongue/my-ecosystem.json
```

---

## 📝 Next Steps

1. ✅ Implement double-click node creation
2. ⏳ Implement drag-to-connect edges
3. ⏳ Add tool palette UI
4. ⏳ Add node property editor
5. ⏳ Add undo/redo system
6. ⏳ Add save/load functionality
7. ⏳ Write comprehensive tests
8. ⏳ Document keyboard shortcuts
9. ⏳ Create tutorial scenario

---

**Status**: 🚧 Starting implementation  
**ETA**: 2-3 hours for basic interactivity  
**Priority**: 🔥 **HIGH** - Enables co-evolution with biomeOS

---

🎨 **Vision**: Paint mode becomes the **visual IDE** for designing biomeOS ecosystems!

