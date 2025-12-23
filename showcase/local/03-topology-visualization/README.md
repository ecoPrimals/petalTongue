# 03 - Topology Visualization

**Duration**: 15-20 minutes  
**Purpose**: Visualize a complete ecosystem topology with all primals and their connections

---

## 🎯 What This Demo Does

1. **Launches a full ecosystem** (5+ primals simultaneously)
2. **Displays the topology graph** showing all connections
3. **Compares layout algorithms** (Force-Directed, Hierarchical, Circular)
4. **Shows common topology patterns** (hub-and-spoke, mesh, layered)
5. **Tests graph navigation** (pan, zoom, select)

**Goal**: Validate that petalTongue can effectively visualize complex, multi-primal ecosystems.

---

## 🚀 Quick Start

```bash
./demo.sh
```

This will launch a full ecosystem and guide you through topology exploration.

---

## 📋 Prerequisites

- Completed `00-setup`, `01-single-primal`, `02-primal-discovery`
- BiomeOS running
- petalTongue UI open
- No primals running initially (clean slate)

---

## 🎬 Demo Flow

### Phase 1: Full Ecosystem Launch

**Start**: Empty graph (0 nodes, 0 edges)

#### Step 1: Launch All Primals

```bash
./launch-ecosystem.sh
```

**Launches**:
- BearDog (Security)
- NestGate (Storage)
- Songbird (Discovery)
- ToadStool (Compute)
- Squirrel (AI Coordination)

**Wait**: ~15-20 seconds for all primals to be discovered

**Observe**:
- 5+ nodes appear
- Connections form between primals
- Graph displays full ecosystem topology

**Expected Final State**: **5 nodes, 4-8 edges** (depending on ecosystem configuration)

### Phase 2: Layout Algorithm Comparison

Now that we have a full topology, let's explore different layout algorithms.

#### Force-Directed Layout (Default)

**How it works**:
- Nodes repel each other (like magnets)
- Edges act as springs
- System reaches equilibrium

**Best for**: General-purpose, natural clustering

**UI Action**: Select "Force-Directed" from layout dropdown

**Observe**:
- Nodes spread evenly
- Connected nodes stay close
- Natural clustering emerges

#### Hierarchical Layout

**How it works**:
- Identifies "root" nodes (few incoming edges)
- Arranges in layers
- Top-down flow

**Best for**: Service dependencies, data pipelines

**UI Action**: Select "Hierarchical" from layout dropdown

**Observe**:
- Layers become visible
- Dependencies flow top → bottom
- Clear hierarchy

**Example**:
```
     BiomeOS (Root)
        ↓   ↓
    BearDog  Songbird
        ↓     ↓
     NestGate ToadStool
```

#### Circular Layout

**How it works**:
- Places nodes on a circle
- Minimizes edge crossings

**Best for**: Small ecosystems, demos, symmetry

**UI Action**: Select "Circular" from layout dropdown

**Observe**:
- Nodes arranged in circle
- Symmetrical appearance
- Easy to see all connections

#### Random Layout

**How it works**:
- Randomly positions nodes
- Useful for testing

**Best for**: Debugging, stress testing

**UI Action**: Select "Random" from layout dropdown

**Observe**:
- Chaos (intentional)
- Useful to see how other algorithms improve clarity

### Phase 3: Graph Navigation

Now let's explore the graph interactively.

#### Pan (Move Camera)

**Action**: Click and drag on empty space

**Observe**:
- Entire graph moves
- Useful for large ecosystems

#### Zoom

**Action**: Mouse wheel or pinch gesture

**Observe**:
- Graph scales in/out
- Maintains aspect ratio

#### Select Node

**Action**: Click on a node

**Observe**:
- Right panel shows node details:
  - Name
  - Type
  - Capabilities
  - Health status
  - Endpoint

#### Reset View

**Action**: Click "Reset Camera" button

**Observe**:
- Camera returns to origin
- Zoom resets to 1.0

### Phase 4: Topology Patterns

Let's identify common topology patterns in our ecosystem.

#### Hub-and-Spoke

**Pattern**: One central node, many connected peripherals

**Example**:
```
    BearDog ← Songbird → ToadStool
                ↑
            NestGate
```

**Use Case**: Service discovery hub, API gateway

**Observe**: Songbird likely acts as hub

#### Mesh

**Pattern**: Many nodes, many connections

**Example**:
```
A ↔ B
↕   ↕
C ↔ D
```

**Use Case**: Distributed systems, peer-to-peer

**Observe**: Full or partial mesh topology

#### Layered

**Pattern**: Clear separation into layers

**Example**:
```
Layer 1: Entry (BiomeOS)
Layer 2: Security (BearDog)
Layer 3: Core Services (NestGate, ToadStool)
Layer 4: Extensions (Squirrel)
```

**Use Case**: Traditional 3-tier architecture, microservices

**Observe**: Hierarchical layout makes this clear

---

## ✅ Success Criteria

After this demo, you should have validated:

- [x] Multiple primals (5+) can be visualized simultaneously
- [x] Edges (connections) are displayed correctly
- [x] All 4 layout algorithms work
- [x] Graph navigation (pan, zoom, select) is smooth
- [x] Can identify topology patterns
- [x] Performance is acceptable (no lag with 5 nodes)

---

## 🔧 Troubleshooting

### Not all primals appear

**Problem**: Launched 5 primals but only 3 visible  
**Solutions**:
1. Wait longer (20-30s for all to be discovered)
2. Check BiomeOS: `curl http://localhost:3000/api/v1/primals | jq`
3. Check primal logs: `tail -f logs/*.log`
4. Verify all primal binaries exist: `ls -la ../../../biomeOS/bin/primals/`

### No edges shown

**Problem**: Multiple primals but 0 edges  
**Solutions**:
1. This might be correct - edges represent active connections
2. Check BiomeOS topology: `curl http://localhost:3000/api/v1/topology | jq`
3. Some ecosystems have sparse connectivity
4. Edges may form over time as primals communicate

### Layout looks messy

**Problem**: Nodes overlap, edges cross badly  
**Solutions**:
1. Try different layout algorithm (Hierarchical often helps)
2. Increase iteration count (edit in code)
3. Zoom out to see full picture
4. Use "Reset Camera" to recenter

### Performance issues

**Problem**: Graph feels sluggish, low FPS  
**Solutions**:
1. This is useful feedback! Document in GAPS.md
2. Check system resources: `htop`
3. Close other applications
4. Try with fewer nodes first

---

## 🌱 Fermentation Notes

### Gaps to Watch For

As you run this demo, look for:

- **Layout Quality**:
  - Are nodes well-distributed?
  - Do edges cross minimally?
  - Is any layout clearly superior?

- **Performance**:
  - Any lag with 5 nodes?
  - What about 10? 20? 50?
  - FPS drop during layout changes?

- **Navigation**:
  - Pan/zoom smooth?
  - Select node responsive?
  - Camera controls intuitive?

- **Topology Representation**:
  - Are edges directional? Should they be?
  - Edge labels visible?
  - Easy to understand data flow?

- **Visual Clarity**:
  - Color coding effective?
  - Node size appropriate?
  - Text readable at all zoom levels?

**Document any gaps in**: `../GAPS.md`

---

## 📊 Expected Topology (Example)

For a standard ecoPrimals ecosystem:

```
Nodes:
  • beardog-1 (Security)
  • nestgate-1 (Storage)
  • songbird-1 (Discovery)
  • toadstool-1 (Compute)
  • squirrel-1 (AI Coordination)

Edges (Example):
  beardog-1 → nestgate-1 (authenticates)
  songbird-1 → all (discovers)
  toadstool-1 → nestgate-1 (stores results)
  squirrel-1 → toadstool-1 (executes models)
```

**Note**: Actual topology depends on primal behavior and BiomeOS configuration.

---

## 🎓 Learning Points

### Why Multiple Layout Algorithms?

Different layouts reveal different insights:

- **Force-Directed**: Natural clustering, general-purpose
- **Hierarchical**: Service dependencies, data flow
- **Circular**: Symmetry, small ecosystems
- **Random**: Baseline, stress testing

**No single layout is perfect.** The best one depends on:
- Ecosystem size
- Topology structure
- What you're trying to understand

### Graph Navigation Best Practices

1. **Start zoomed out** - See the big picture
2. **Zoom in on details** - Inspect specific nodes
3. **Switch layouts** - Gain different perspectives
4. **Select nodes** - Understand individual services

### Topology as Communication

The graph IS the message. A healthy topology:
- Clear structure
- Minimal bottlenecks
- Appropriate connectivity

A problematic topology:
- Single points of failure
- Overloaded hubs
- Disconnected components

**petalTongue makes topology problems visible.**

---

## 💡 Real-World Analogies

### City Map

Think of the graph as a city map:
- **Nodes** = Buildings
- **Edges** = Roads
- **Layout** = Map projection (Mercator vs. Peters)
- **Zoom** = Scale (city view vs. country view)

Different layouts = different map projections. Each reveals something different.

### Circuit Diagram

Or think of it as a circuit:
- **Nodes** = Components (resistors, capacitors)
- **Edges** = Wires
- **Layout** = Schematic style
- **Current** = Data flow

A good circuit diagram makes the design obvious. So does a good topology graph.

---

## ⏭️ Next Steps

Once comfortable with topology visualization, proceed to:

```bash
cd ../04-health-monitoring/
cat README.md
```

This will show you **dynamic health status** as primals degrade and recover.

---

## 🎮 Advanced Experiments

### Compare Layouts Side-by-Side

Take screenshots of each layout:
```bash
# Force-Directed
# [Screenshot]

# Hierarchical
# [Screenshot]

# Circular
# [Screenshot]
```

**Analysis**: Which layout makes the architecture clearest?

### Measure Layout Performance

Time each layout algorithm:
```bash
time ./launch-ecosystem.sh
# Switch layouts in UI, note performance
```

**Document**: Which is fastest? Which uses most CPU?

### Stress Test (Large Ecosystem)

If you have access to more primals:
```bash
# Launch 10, 20, 50 primals
# Observe performance degradation
```

**Gap Discovery**: At what size does performance become unacceptable?

### Edge Directionality Test

Manually verify edges:
```bash
curl http://localhost:3000/api/v1/topology | jq '.[] | {from, to, type}'
```

**Compare**: Does petalTongue accurately represent direction?

---

## 📐 Topology Metrics

### Complexity Measures

For a graph with N nodes and E edges:

**Density** = E / (N * (N-1))
- Sparse: < 0.1
- Medium: 0.1 - 0.3
- Dense: > 0.3

**Average Degree** = 2E / N
- How many connections per node on average

**Clustering Coefficient**
- How "clumped" the graph is

**Diameter**
- Longest shortest path

**petalTongue currently shows**: Nodes, Edges
**Future**: Could calculate and display these metrics

---

**Status**: 🌱 Ready to build  
**Complexity**: High  
**Dependencies**: 00-setup, 01-single-primal, 02-primal-discovery  
**Learning Value**: Very High (core use case)

---

*Complex topology made simple. That's the power of visualization.* 🌸

