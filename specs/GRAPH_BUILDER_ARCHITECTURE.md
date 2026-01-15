# 🎨 Graph Builder Architecture - Phase 4

**Version**: 1.0.0  
**Date**: January 15, 2026  
**Status**: Design Phase  
**Priority**: High (Completes Neural API Integration)

---

## 🎯 **Vision**

Enable users to **visually construct Neural API graphs** through an intuitive drag-and-drop interface, eliminating the need to manually write JSON graph definitions.

**User Story**:
> "As a system operator, I want to visually design deployment graphs by dragging nodes and connecting them, so I can orchestrate complex primal deployments without writing code."

---

## 🏗️ **Architecture Overview**

```
┌─────────────────────────────────────────────────────────────┐
│                    Graph Builder UI                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Graph Canvas (egui)                    │   │
│  │  ┌─────────┐      ┌─────────┐      ┌─────────┐   │   │
│  │  │ Node A  │─────▶│ Node B  │─────▶│ Node C  │   │   │
│  │  └─────────┘      └─────────┘      └─────────┘   │   │
│  │                                                     │   │
│  │  Drag, drop, connect, configure nodes             │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│  ┌─────────────────────┐ │ ┌─────────────────────────┐    │
│  │   Node Palette      │ │ │  Property Panel         │    │
│  │  ┌──────────────┐  │ │ │  ┌───────────────────┐  │    │
│  │  │ primal_start │  │ │ │  │ Name: [_______]   │  │    │
│  │  │ verification │  │ │ │  │ Type: primal_start│  │    │
│  │  │ wait_for     │  │ │ │  │ Primal: [____]    │  │    │
│  │  │ conditional  │  │ │ │  │ Family: [____]    │  │    │
│  │  └──────────────┘  │ │ │  └───────────────────┘  │    │
│  └─────────────────────┘ │ └─────────────────────────┘    │
│                          ▼                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Graph Validation Engine                    │  │
│  │  • Check for cycles                                  │  │
│  │  • Validate dependencies                             │  │
│  │  • Verify required parameters                        │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │                                  │
│                          ▼                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        Neural API Integration                        │  │
│  │  • save_graph(name, definition)                      │  │
│  │  • get_graphs() → list available graphs              │  │
│  │  • execute_graph(name) → start execution             │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │                                  │
└──────────────────────────┼──────────────────────────────────┘
                           │
               ┌───────────▼────────────┐
               │   biomeOS Neural API   │
               │  (Graph Persistence)   │
               └────────────────────────┘
```

---

## 📦 **Core Components**

### **1. Graph Data Structure**

```rust
/// A visual graph representation for Neural API graphs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualGraph {
    /// Unique graph ID
    pub id: String,
    
    /// Human-readable graph name
    pub name: String,
    
    /// Graph description
    pub description: Option<String>,
    
    /// All nodes in the graph
    pub nodes: Vec<GraphNode>,
    
    /// Edges connecting nodes
    pub edges: Vec<GraphEdge>,
    
    /// Layout metadata (positions, zoom, etc.)
    pub layout: GraphLayout,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
}

/// A node in the visual graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node ID (within this graph)
    pub id: String,
    
    /// Node type (primal_start, verification, wait_for, conditional)
    pub node_type: NodeType,
    
    /// Display position on canvas
    pub position: Vec2,
    
    /// Node-specific parameters
    pub parameters: serde_json::Value,
    
    /// Visual state (selected, error, etc.)
    #[serde(skip)]
    pub visual_state: NodeVisualState,
}

/// Node types supported by Neural API
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    /// Start a primal
    PrimalStart,
    
    /// Verify primal health
    Verification,
    
    /// Wait for condition
    WaitFor,
    
    /// Conditional branch
    Conditional,
}

/// Edge connecting two nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,
    
    /// Target node ID
    pub to: String,
    
    /// Edge type (dependency, data flow, etc.)
    pub edge_type: EdgeType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    /// Node B depends on Node A (execution order)
    Dependency,
    
    /// Data flows from A to B
    DataFlow,
}
```

### **2. Graph Canvas Widget**

```rust
/// Interactive graph canvas for building Neural API graphs
pub struct GraphCanvas {
    /// Current graph being edited
    graph: VisualGraph,
    
    /// Camera position and zoom
    camera: Camera2D,
    
    /// Currently selected nodes
    selected_nodes: HashSet<String>,
    
    /// Drag state for nodes
    drag_state: Option<DragState>,
    
    /// Edge being drawn (if any)
    drawing_edge: Option<EdgeDrawState>,
    
    /// Grid settings
    grid_enabled: bool,
    grid_size: f32,
    
    /// Snap to grid
    snap_to_grid: bool,
}

impl GraphCanvas {
    /// Render the canvas
    pub fn render(&mut self, ui: &mut egui::Ui);
    
    /// Handle mouse input
    fn handle_mouse_input(&mut self, ui: &mut egui::Ui);
    
    /// Handle keyboard shortcuts
    fn handle_keyboard(&mut self, ui: &mut egui::Ui);
    
    /// Add node at position
    pub fn add_node(&mut self, node_type: NodeType, position: Vec2) -> String;
    
    /// Remove selected nodes
    pub fn delete_selected(&mut self);
    
    /// Connect two nodes
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<(), String>;
    
    /// Validate graph structure
    pub fn validate(&self) -> Result<(), Vec<String>>;
}

/// Camera for panning and zooming
#[derive(Clone, Debug)]
pub struct Camera2D {
    /// Camera center position
    pub position: Vec2,
    
    /// Zoom level (1.0 = 100%)
    pub zoom: f32,
}
```

### **3. Node Palette**

```rust
/// Node palette for selecting node types to add
pub struct NodePalette {
    /// Available node types
    node_types: Vec<NodeTypeInfo>,
    
    /// Search filter
    search: String,
    
    /// Dragging state
    dragging: Option<NodeType>,
}

#[derive(Clone, Debug)]
pub struct NodeTypeInfo {
    /// Node type
    pub node_type: NodeType,
    
    /// Display name
    pub name: &'static str,
    
    /// Icon
    pub icon: &'static str,
    
    /// Description
    pub description: &'static str,
    
    /// Required parameters
    pub required_params: Vec<&'static str>,
}

impl NodePalette {
    /// Render the palette
    pub fn render(&mut self, ui: &mut egui::Ui);
    
    /// Get available node types
    pub fn get_node_types() -> Vec<NodeTypeInfo> {
        vec![
            NodeTypeInfo {
                node_type: NodeType::PrimalStart,
                name: "Start Primal",
                icon: "🚀",
                description: "Start a primal service",
                required_params: vec!["primal_name", "family_id"],
            },
            NodeTypeInfo {
                node_type: NodeType::Verification,
                name: "Verify Health",
                icon: "✅",
                description: "Verify primal health status",
                required_params: vec!["primal_name", "timeout"],
            },
            NodeTypeInfo {
                node_type: NodeType::WaitFor,
                name: "Wait For",
                icon: "⏳",
                description: "Wait for condition to be met",
                required_params: vec!["condition", "timeout"],
            },
            NodeTypeInfo {
                node_type: NodeType::Conditional,
                name: "Conditional",
                icon: "❓",
                description: "Branch based on condition",
                required_params: vec!["condition"],
            },
        ]
    }
}
```

### **4. Property Panel**

```rust
/// Property editor for selected node
pub struct PropertyPanel {
    /// Currently editing node
    editing_node: Option<String>,
    
    /// Parameter forms
    parameter_forms: HashMap<String, String>,
}

impl PropertyPanel {
    /// Render property panel for selected node
    pub fn render(&mut self, ui: &mut egui::Ui, graph: &mut VisualGraph);
    
    /// Render parameter form based on node type
    fn render_parameters(&mut self, ui: &mut egui::Ui, node: &mut GraphNode);
}
```

### **5. Graph Validation Engine**

```rust
/// Validates graph structure and parameters
pub struct GraphValidator;

impl GraphValidator {
    /// Validate entire graph
    pub fn validate(graph: &VisualGraph) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // Check for cycles
        if let Err(e) = Self::check_cycles(graph) {
            errors.push(e);
        }
        
        // Validate each node
        for node in &graph.nodes {
            if let Err(e) = Self::validate_node(node) {
                errors.push(e);
            }
        }
        
        // Validate edges
        for edge in &graph.edges {
            if let Err(e) = Self::validate_edge(edge, graph) {
                errors.push(e);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Check for cycles in the graph
    fn check_cycles(graph: &VisualGraph) -> Result<(), ValidationError>;
    
    /// Validate a single node
    fn validate_node(node: &GraphNode) -> Result<(), ValidationError>;
    
    /// Validate an edge
    fn validate_edge(edge: &GraphEdge, graph: &VisualGraph) -> Result<(), ValidationError>;
}

#[derive(Clone, Debug)]
pub struct ValidationError {
    pub error_type: ValidationErrorType,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ValidationErrorType {
    Cycle,
    MissingParameter,
    InvalidParameter,
    DanglingEdge,
    DuplicateNodeId,
}
```

### **6. Neural API Integration**

```rust
/// Graph persistence and execution via Neural API
pub struct GraphPersistence {
    provider: Arc<NeuralApiProvider>,
}

impl GraphPersistence {
    /// Save graph to Neural API
    pub async fn save_graph(&self, graph: &VisualGraph) -> Result<()> {
        // Convert VisualGraph to Neural API format
        let neural_graph = self.to_neural_format(graph)?;
        
        // Save via Neural API
        self.provider
            .call_method(
                "neural_api.save_graph",
                Some(serde_json::json!({
                    "name": graph.name,
                    "definition": neural_graph,
                })),
            )
            .await?;
        
        Ok(())
    }
    
    /// Load all available graphs
    pub async fn load_graphs(&self) -> Result<Vec<String>> {
        let result = self.provider
            .call_method("neural_api.get_graphs", None)
            .await?;
        
        let graphs: Vec<String> = serde_json::from_value(result)?;
        Ok(graphs)
    }
    
    /// Execute a graph
    pub async fn execute_graph(&self, name: &str) -> Result<String> {
        let result = self.provider
            .call_method(
                "neural_api.execute_graph",
                Some(serde_json::json!({ "name": name })),
            )
            .await?;
        
        let execution_id: String = serde_json::from_value(result)?;
        Ok(execution_id)
    }
    
    /// Convert VisualGraph to Neural API format
    fn to_neural_format(&self, graph: &VisualGraph) -> Result<serde_json::Value> {
        // Transform our visual representation to Neural API JSON format
        // ...
        todo!("Implement graph format conversion")
    }
}
```

---

## 🎨 **User Interface Design**

### **Layout**

```
┌──────────────────────────────────────────────────────────────┐
│ 🌸 petalTongue - Graph Builder                    [Save] [▶] │
├──────────┬───────────────────────────────────────────────────┤
│          │                                                    │
│  Node    │           Graph Canvas                            │
│  Palette │                                                    │
│          │  ┌─────────┐                                      │
│ 🚀 Start │  │ Start   │                                      │
│ ✅ Verify│  │ BearDog │───┐                                  │
│ ⏳ Wait  │  └─────────┘   │                                  │
│ ❓ Branch│                 ▼                                  │
│          │           ┌─────────┐                             │
│          │           │ Verify  │                             │
│ [Search] │           │ BearDog │                             │
│          │           └─────────┘                             │
│          │                                                    │
├──────────┴───────────────────────────────────────────────────┤
│ Properties (Selected Node)                                   │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Name: start_beardog                                     │ │
│ │ Type: primal_start                                      │ │
│ │ Primal Name: [beardog-server          ]                │ │
│ │ Family ID:   [nat0                    ]                │ │
│ │ Socket Path: [/tmp/beardog.sock       ]                │ │
│ └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### **Interactions**

1. **Drag from Palette** → Creates new node on canvas
2. **Click Node** → Select node, show properties
3. **Drag Node** → Move node position
4. **Ctrl+Drag from Node** → Start drawing edge
5. **Drop on Target Node** → Create edge
6. **Delete Key** → Delete selected nodes
7. **Ctrl+S** → Save graph
8. **Click Execute** → Run graph via Neural API

### **Visual Feedback**

- **Node Colors**:
  - Default: Blue (#4A90E2)
  - Selected: Orange (#F5A623)
  - Error: Red (#D0021B)
  - Valid: Green (#7ED321)
- **Edge Styles**:
  - Dependency: Solid arrow
  - Data Flow: Dashed arrow
  - Drawing: Dotted line following mouse
- **Grid**: Subtle dots (optional, toggleable)

---

## 🔧 **Implementation Plan**

### **Phase 4.1: Core Data Structures** (8 hours)
- [ ] Create `VisualGraph`, `GraphNode`, `GraphEdge` types
- [ ] Implement serialization/deserialization
- [ ] Add basic graph operations (add node, add edge, remove node)
- [ ] Unit tests for data structures

### **Phase 4.2: Canvas Rendering** (16 hours)
- [ ] Implement `GraphCanvas` widget
- [ ] Node rendering (boxes with labels)
- [ ] Edge rendering (arrows between nodes)
- [ ] Camera controls (pan with middle-mouse, zoom with scroll)
- [ ] Grid rendering
- [ ] Selection visualization

### **Phase 4.3: Node Interaction** (12 hours)
- [ ] Node dragging
- [ ] Multi-select (Ctrl+Click, drag select box)
- [ ] Edge creation (Ctrl+Drag from node to node)
- [ ] Delete nodes/edges
- [ ] Undo/redo stack
- [ ] Snap to grid

### **Phase 4.4: Node Palette** (8 hours)
- [ ] Create `NodePalette` widget
- [ ] List all node types with icons
- [ ] Search/filter functionality
- [ ] Drag from palette to canvas
- [ ] Tooltips with descriptions

### **Phase 4.5: Property Panel** (12 hours)
- [ ] Create `PropertyPanel` widget
- [ ] Dynamic form generation based on node type
- [ ] Parameter validation (required fields, types)
- [ ] Real-time validation feedback
- [ ] Help text for each parameter

### **Phase 4.6: Graph Validation** (8 hours)
- [ ] Cycle detection algorithm
- [ ] Required parameter checking
- [ ] Edge validation (no dangling edges)
- [ ] Visual error indicators on nodes
- [ ] Validation error list panel

### **Phase 4.7: Neural API Integration** (12 hours)
- [ ] Implement `GraphPersistence`
- [ ] Convert `VisualGraph` to Neural API JSON format
- [ ] Save graph functionality
- [ ] Load graphs list
- [ ] Execute graph functionality
- [ ] Execution status monitoring

### **Phase 4.8: Polish & Testing** (16 hours)
- [ ] Keyboard shortcuts (Ctrl+S save, Delete, Undo/Redo)
- [ ] Auto-layout for new nodes
- [ ] Graph templates (common patterns)
- [ ] Export/import graphs as JSON
- [ ] Comprehensive testing
- [ ] User documentation

**Total Estimated Time**: 92 hours

---

## 🎯 **Success Criteria**

- [ ] User can drag nodes from palette onto canvas
- [ ] User can connect nodes with edges
- [ ] User can configure node parameters
- [ ] Graph validation prevents invalid graphs
- [ ] User can save graph to Neural API
- [ ] User can execute saved graphs
- [ ] UI is responsive (60 FPS maintained)
- [ ] Graceful error handling for Neural API failures
- [ ] Comprehensive documentation

---

## 📚 **References**

- **Similar Tools**: Node-RED, n8n, Prefect, Apache Airflow
- **Neural API Spec**: `specs/NEURAL_API_INTEGRATION_SPECIFICATION.md`
- **egui Canvas Examples**: https://docs.rs/egui/latest/egui/
- **Graph Algorithms**: Tarjan's for cycle detection

---

**Next Steps**: Begin with Phase 4.1 (Core Data Structures)

---

**Version**: 1.0.0  
**Last Updated**: January 15, 2026  
**Status**: Ready for Implementation

