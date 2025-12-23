# Demo 01: Basic Topology

**Duration**: 2-3 minutes  
**Target Audience**: First-time users, executives  
**Difficulty**: Beginner

---

## Overview

This demo introduces petalTongue with a simple 5-primal topology. All primals are healthy (green), making it easy to focus on the core UI interactions without the distraction of failures.

**Goal**: Show users what petalTongue is and how to navigate the graph.

---

## Scenario Details

- **Primals**: 5 (BearDog, ToadStool, Songbird, NestGate, Squirrel)
- **Health**: All healthy (green circles)
- **Connections**: 5 edges showing service interactions
- **Layout**: Force-directed (default)

---

## Key Features Demonstrated

1. **Visual Graph**
   - Nodes (circles) represent primals
   - Edges (arrows) show connections
   - Colors indicate health (all green = healthy)

2. **Basic Interactions**
   - **Pan**: Click and drag to move the graph
   - **Zoom**: Scroll wheel to zoom in/out
   - **Select**: Click a node to see details

3. **Layout Algorithms**
   - Switch between Force-Directed, Hierarchical, Circular
   - Watch graph rearrange automatically

4. **Statistics Panel**
   - Node count: 5
   - Edge count: 5
   - Average degree: ~2.0
   - Zoom level (changes as you zoom)

---

## Presenter Script

### Opening (30 seconds)

> "This is **petalTongue** - a universal representation system for monitoring distributed systems. You're looking at a simple ecosystem with 5 primals - services that coordinate to provide functionality."

> "The circles are primals, the arrows show how they connect. All green means everything is healthy."

### Interaction Demo (1 minute)

> "Let me show you the basics. I can **pan** by dragging..." *(drag the graph around)*

> "I can **zoom** in and out..." *(scroll to zoom)*

> "And I can **select** a node to see its details." *(click on BearDog)*

> "See how the right panel updates? It shows the audio description for this primal - how a blind user would 'hear' this node. BearDog is a Security primal, represented by deep bass tones."

### Layout Demo (30 seconds)

> "The graph uses different layout algorithms. This is Force-Directed - nodes push apart naturally. Watch when I switch to Hierarchical..." *(change layout)*

> "Now it's arranged as a tree. And Circular..." *(change layout again)*

> "arranges them in a circle. The same data, different spatial arrangements."

### Statistics (30 seconds)

> "The top-left shows statistics - 5 nodes, 5 edges, average degree around 2. As I zoom..." *(zoom in)* "...you see the zoom level change."

> "There's also a **Reset Camera** button if I get lost."

### Closing (30 seconds)

> "That's the basics of petalTongue. A simple, intuitive way to see your ecosystem at a glance. In the next demo, we'll see what happens when things go wrong."

---

## Setup Instructions

### Automated

```bash
cd showcase/
./scripts/run-demo.sh 01
```

### Manual

1. **Copy scenario**:
   ```bash
   cp sandbox/scenarios/simple.json sandbox/scenarios/demo-active.json
   ```

2. **Configure mock server** (if not already running):
   ```bash
   cd sandbox/mock-biomeos/src/
   # Edit main.rs line 56: use "demo-active.json"
   cd ../
   cargo build --release
   ```

3. **Start mock server**:
   ```bash
   cd sandbox/
   ./scripts/start-mock.sh
   ```

4. **Launch petalTongue**:
   ```bash
   cd ..
   BIOMEOS_URL=http://localhost:3333 cargo run --release -p petal-tongue-ui
   ```

---

## Troubleshooting

### Graph is empty
- Check mock server is running: `curl http://localhost:3333/api/v1/primals`
- Check petalTongue is connected: Look for "Last refresh" timestamp in UI

### Colors are wrong
- This demo should show all green (healthy)
- If you see yellow/red, check you're using `simple.json` scenario

### Layout looks weird
- Try clicking "Reset Camera"
- Try switching layout algorithms
- Default is Force-Directed - may take a few seconds to stabilize

---

## Variations

### Shorter Version (1 minute)
- Skip statistics explanation
- Just show: graph, pan/zoom, one layout switch

### Extended Version (5 minutes)
- Show all three layouts
- Demonstrate selecting each primal
- Read all audio descriptions
- Explain primal types (Security, Compute, Discovery, Storage, AI)

### Accessibility Focus
- Emphasize audio descriptions panel
- Explain that blind users can navigate by sound
- Preview Demo 04 (Audio-Only Experience)

---

## Follow-Up

After Demo 01, transition to:
- **Demo 02** (Degraded System) - "Now let's see what happens when things break"
- **Demo 05** (Production Scale) - "Let's scale this up to 50+ primals"

---

## Notes for Presenters

- This is the **gentlest** introduction - all healthy, small topology
- Perfect for executives or non-technical audiences
- Keep it simple - don't overwhelm with technical details
- Focus on the **visual clarity** and **ease of navigation**
- If asked about audio, say "We'll see that in Demo 04"

---

**Status**: ✅ Scenario ready  
**Requirements**: Mock server, simple.json  
**Tested**: Yes

