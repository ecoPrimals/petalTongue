# Demo 03: Scaling Event

**Duration**: 5-7 minutes  
**Target Audience**: DevOps, platform engineers, SREs  
**Difficulty**: Intermediate

---

## Overview

This demo shows how petalTongue handles dynamic topology changes. Starting with 5 primals, we scale up to 20, demonstrating real-time updates, layout adaptation, and the system's ability to visualize complex production-like topologies.

**Goal**: Prove petalTongue scales from simple to complex ecosystems gracefully.

---

## Scenario Details

- **Start**: 5 primals (simple.json)
- **End**: 20 primals (complex.json)
- **Scaling Factor**: 4x increase
- **Health Mix**: Mostly healthy, some warnings
- **Topology**: Production-like (replication, delegation, discovery)

---

## Key Features Demonstrated

1. **Dynamic Scaling**
   - 5 → 20 primals in real-time
   - Layout adapts automatically
   - No performance degradation

2. **Layout Algorithms**
   - Force-directed (organic clustering)
   - Hierarchical (tree structure)
   - Circular (equal distribution)

3. **Real-Time Updates**
   - Auto-refresh picks up changes
   - Hot-reload from mock server
   - Smooth transitions

4. **Complex Topologies**
   - Multiple instances per primal type
   - Replication edges (NestGate → replicas)
   - Delegation edges (ToadStool → workers)
   - Discovery edges (Songbird → all)

---

## Presenter Script

### Opening (30 seconds)

> "We've seen small topologies - 5, 10 nodes. Real systems are bigger. Let me show you how petalTongue scales."

### Initial State (1 minute)

> "We're starting with 5 primals - simple topology, all healthy. This is what we saw in Demo 01."

*(Show simple topology, pan around, select a node)*

> "Now watch what happens when the ecosystem grows."

### The Scaling (2 minutes)

*(Switch to complex.json scenario - edit mock server config, rebuild, restart)*

> "I've just told the mock server to load a production-like scenario. petalTongue will pick this up on the next auto-refresh..."

*(Wait 5s for auto-refresh OR click "Refresh Now")*

> "There. We went from 5 to 20 primals. Four times the size."

*(Let the force-directed layout stabilize - nodes push apart)*

> "See how the layout algorithm handles it? Nodes push apart naturally, avoiding overlap. The graph stays readable."

### Explore Complexity (2 minutes)

*(Zoom in on a cluster)*

> "Let me zoom in on the NestGate cluster. Here's the primary storage node..." *(click nestgate-1)* "...and here are its replicas." *(click nestgate-2, nestgate-3)*

> "The edges show replication - data flowing from primary to replicas."

*(Zoom out, show ToadStool cluster)*

> "Over here, ToadStool's main parser delegates work to multiple workers. This is production architecture - load balancing, redundancy."

*(Show Songbird discovering services)*

> "And Songbird, the discovery service, has connections to everyone - it's the hub that lets other primals find each other."

### Layout Switching (1-2 minutes)

> "With this many nodes, different layouts show different insights."

*(Switch to Hierarchical layout)*

> "Hierarchical layout shows the tree structure clearly. BiomeOS at the top, coordinating everything below."

*(Switch to Circular layout)*

> "Circular layout spreads them equally - good for seeing all nodes at once without bias."

*(Switch back to Force-directed)*

> "Force-directed clusters related services together naturally."

### Performance (30 seconds)

> "Even with 20 nodes, it's smooth. Check the stats..." *(point to statistics panel)* "...60 FPS, no lag. Pan, zoom, select - everything's responsive."

### Closing (30 seconds)

> "This is a production-like topology. Multiple instances, replication, delegation, discovery - everything you'd see in a real deployment. And petalTongue handles it gracefully."

> "We've tested up to 50 nodes - still smooth. With optimizations, it could handle hundreds."

---

## Setup Instructions

### Automated

```bash
cd showcase/
./scripts/run-demo.sh 03
```

### Manual

**Part 1: Start with simple.json**

1. Copy simple scenario:
   ```bash
   cp sandbox/scenarios/simple.json sandbox/scenarios/demo-active.json
   ```

2. Start mock server (if not running):
   ```bash
   cd sandbox/
   ./scripts/start-mock.sh
   ```

3. Launch petalTongue:
   ```bash
   BIOMEOS_URL=http://localhost:3333 cargo run --release -p petal-tongue-ui
   ```

**Part 2: Scale up to complex.json**

4. **During demo**, switch scenario:
   ```bash
   # In another terminal
   cp sandbox/scenarios/complex.json sandbox/scenarios/demo-active.json
   ```

5. Mock server will auto-reload (watch its logs)

6. petalTongue will auto-refresh (wait 5s or click "Refresh Now")

7. Watch graph transition from 5 → 20 nodes!

---

## Variations

### Quick Version (3 min)
- Skip detailed exploration
- Just show: 5 → 20 scaling, one layout switch

### Extended Version (10 min)
- Explore each primal type in detail
- Show all three layouts
- Demonstrate manual refresh vs auto-refresh
- Click on multiple nodes to show relationships

### Performance Focus
- Emphasize FPS, responsiveness
- Show statistics panel
- Compare layout algorithm performance
- Preview Demo 05 (50+ nodes)

---

## Troubleshooting

### Graph doesn't update
- Check mock server logs: "Reloaded scenario"
- Click "Refresh Now" button manually
- Verify scenario file was copied correctly

### Too many nodes (overwhelming)
- Use Hierarchical layout (clearer structure)
- Zoom out to see full graph
- Click "Reset Camera"

### Performance is slow
- Check CPU usage (close other apps)
- Try simpler layout (Circular is fastest)
- Reduce window size

---

## Follow-Up

After Demo 03, transition to:
- **Demo 04** (Audio-Only) - "Now let me close my eyes..."
- **Demo 05** (Production Scale) - "Let's push it further - 50+ nodes"

---

**Status**: ✅ Scenarios ready (simple.json + complex.json)  
**Requirements**: Mock server, hot-reload working  
**Tested**: Yes

