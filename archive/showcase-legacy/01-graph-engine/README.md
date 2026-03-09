# 🎨 Graph Engine Fundamentals

**Master the core graph engine and layout algorithms**

---

## 🎯 What You'll Learn

In **10 minutes**, you'll master:
- The modality-agnostic graph engine
- 4 different layout algorithms
- How layouts work independently of rendering
- Node and edge management

---

## ⏱️ Duration

**10 minutes**

---

## 📋 Prerequisites

- petalTongue built (`cargo build --release`)
- Completed: `00-hello-petaltongue`

---

## 🚀 Run the Demo

```bash
./demo.sh
```

---

## 🎓 Key Concepts

### **1. Modality-Agnostic Architecture**

```
GraphEngine (Core)
├── Nodes (id, type, health, metadata)
├── Edges (source, target, bandwidth)
└── Layout (positions only, no rendering)
         ↓
    ┌────┴────┐
    ↓         ↓
Visual 2D   Audio
Renderer   Renderer
```

**The engine has ZERO knowledge of rendering!**

### **2. Four Layout Algorithms**

#### **Force-Directed** (Physics-based)
- Nodes attract/repel based on connections
- Natural clustering emerges
- Best for: Understanding relationships

#### **Circular** (Ring arrangement)
- Nodes arranged in a circle
- Equal spacing
- Best for: Symmetry, equal importance

#### **Hierarchical** (Tree structure)
- Parent-child relationships
- Top-down or left-right
- Best for: Organizational charts, dependencies

#### **Random** (Scattered)
- Random positions
- Useful for: Testing, debugging
- Best for: Initial exploration

### **3. Graph Components**

**Nodes**:
```rust
pub struct Node {
    id: String,              // Unique identifier
    node_type: PrimalType,   // Discovery, Security, etc.
    health: HealthStatus,    // Healthy, Warning, Critical
    position: Vec2,          // Layout algorithm sets this
    metadata: HashMap<String, String>,
}
```

**Edges**:
```rust
pub struct Edge {
    source: String,          // Source node ID
    target: String,          // Target node ID
    bandwidth: f32,          // Traffic volume
}
```

---

## 👀 What You'll See

### **Layout Comparison**

**Force-Directed**:
```
    Node1
      |
      |
   Node2----Node3
      |
   Node4
```
Natural clustering, connected nodes near each other.

**Circular**:
```
      Node1
   Node4    Node2
      Node3
```
Perfect circle, equal spacing.

**Hierarchical**:
```
       Node1
      /     \
   Node2   Node3
     |
   Node4
```
Tree structure, clear hierarchy.

**Random**:
```
  Node3    Node1
       
   Node4      Node2
```
Scattered positions.

### **Performance Characteristics**

| Layout | Speed | Best For |
|--------|-------|----------|
| Random | Instant | Debug |
| Circular | Fast | Symmetry |
| Hierarchical | Fast | Trees |
| Force-Directed | Iterative | Understanding |

---

## 💡 Try This

### **1. Switch Between Layouts**

```rust
// In the UI, try each layout:
1. Force-Directed → Watch natural clustering
2. Circular → See perfect symmetry
3. Hierarchical → Observe tree structure
4. Random → See chaos
```

### **2. Observe Position Changes**

- Note how **same graph** looks different
- **Data is identical**, only **positions** change
- Renderers get new positions, redraw

### **3. Watch Force-Directed Animate**

- It runs 100 iterations
- Positions converge over time
- Natural structure emerges

---

## 📊 What This Demonstrates

1. ✅ **Separation of Concerns** - Engine knows no rendering
2. ✅ **Layout Flexibility** - Multiple algorithms available
3. ✅ **Data Independence** - Same data, different views
4. ✅ **Performance** - Fast layout computation
5. ✅ **Extensibility** - Easy to add new layouts

---

## 🔍 Deep Dive: Force-Directed Algorithm

### **Physics Simulation**

```rust
// Repulsion (all nodes push each other away)
for each pair of nodes:
    force = repulsion / distance²
    apply force away from each other

// Attraction (connected nodes pull together)
for each edge:
    force = spring_constant × distance
    apply force toward each other

// Update positions
for each node:
    velocity += force / mass
    position += velocity × damping
```

### **Parameters**

- **Repulsion**: 1000.0 (push strength)
- **Spring**: 0.5 (pull strength)
- **Damping**: 0.8 (energy loss)
- **Iterations**: 100 (convergence steps)

---

## 🐛 Troubleshooting

### **Layouts look the same**

Different graphs may converge to similar shapes. Try:
- Adding more nodes
- Changing connectivity
- Observing the process

### **Force-directed doesn't stabilize**

It may need more iterations:
```rust
graph.layout(200); // More iterations
```

---

## 🎯 Success Criteria

You've mastered graph engine when you:
- ✅ Understand the 4 layout types
- ✅ See how layouts are separate from rendering
- ✅ Can explain force-directed algorithm
- ✅ Appreciate modality-agnostic design

---

## ➡️ Next Steps

```bash
cd ../02-visual-2d/
cat README.md
```

**Next**: Learn interactive visual rendering and controls.

---

## 📚 Technical Details

### **Graph Engine API**

```rust
// Create engine
let mut graph = GraphEngine::new();

// Add nodes
graph.add_node(Node::new("node1", PrimalType::Discovery));

// Add edges
graph.add_edge(Edge::new("node1", "node2", 1.0));

// Set layout algorithm
graph.set_layout(LayoutAlgorithm::ForceDirected);

// Compute positions
graph.layout(100); // 100 iterations

// Get positions (for rendering)
let nodes = graph.nodes();
for node in nodes {
    let pos = node.position; // Renderer uses this
}
```

### **Performance**

- **10 nodes**: < 1ms
- **50 nodes**: < 10ms
- **100 nodes**: < 50ms
- **1000 nodes**: < 500ms

All layout algorithms except force-directed are instant.

---

## 🌟 Key Takeaway

**The graph engine is modality-agnostic by design.**

It only knows:
- ✅ Graph structure (nodes, edges)
- ✅ Layout algorithms (positions)
- ❌ How to render visually
- ❌ How to sonify to audio
- ❌ Any presentation details

**This is what makes multi-modal rendering possible!**

---

*"Great architecture enables innovation."* 🌸

