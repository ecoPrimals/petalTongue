# 06 - Songbird Discovery Integration

**Duration**: 20 minutes  
**Purpose**: Visualize Songbird's discovery and multi-tower federation capabilities

---

## 🐦 What This Demo Shows

This demonstrates how petalTongue visualizes **Songbird's discovery mesh** - showing real-time primal discovery, multi-tower federation, and inter-primal communication patterns.

**Songbird Success**: Songbird has 15+ comprehensive scenarios including multi-tower federation across physical machines!

---

## 🎯 What You'll See

### Songbird's Capabilities Visualized

1. **Primal Discovery**
   - Nodes appear as Songbird discovers primals
   - Discovery mesh topology
   - Service capability mapping

2. **Multi-Tower Federation**
   - Multiple Songbird towers (can be on different machines!)
   - Federation connections between towers
   - Tower-to-tower communication

3. **Inter-Primal Routing**
   - How Songbird routes between primals
   - Service discovery patterns
   - Health propagation

4. **Protocol Escalation**
   - Different protocol types (HTTP, gRPC, tarpc)
   - Fallback patterns
   - Performance optimization

---

## 🚀 Quick Start

### Prerequisites

1. **Songbird built and available**:
   ```bash
   cd ../../../../songbird
   cargo build --release
   ```

2. **petalTongue built**:
   ```bash
   cd ../../petalTongue
   cargo build --release
   ```

3. **Songbird showcase available**:
   ```bash
   ls ../../../../songbird/showcase/
   # Should see: 01-isolated/, 02-federation/, etc.
   ```

### Run Demo

```bash
./demo.sh
```

---

## 📋 Demo Scenarios

### Scenario 1: Single Tower Discovery

**Script**: `./demo-single-tower.sh`

**What you'll do**:
1. Launch single Songbird tower
2. Launch 3-5 primals
3. Watch petalTongue show discovery in real-time

**Visual**:
- Songbird node (center)
- Discovered primals appearing around it
- Edges representing discovery connections

**Audio**:
- Songbird = Light Chimes (discovery sound!)
- Other primals with their instrument types
- Spatial positioning shows topology

**Key Learning**: Songbird discovers and maps the ecosystem!

---

### Scenario 2: Multi-Tower Federation

**Script**: `./demo-multi-tower.sh`

**What you'll do**:
1. Launch 2-3 Songbird towers
2. Watch them federate
3. Launch primals across different towers
4. See the complete federated mesh

**Visual**:
- Multiple Songbird nodes
- Federation edges between towers
- Primals connected to their local tower
- Cross-tower routing paths

**Audio**:
- Multiple chime instruments (one per tower)
- Spatial audio shows tower positions
- Harmonic = federation healthy

**Key Learning**: Multi-tower federation creates resilient discovery!

---

### Scenario 3: Real-Time Discovery Events

**Script**: `./demo-discovery-events.sh`

**What you'll do**:
1. Start with empty topology
2. Add primals one-by-one
3. Watch discovery propagate through Songbird
4. See topology grow in real-time

**Visual**:
- Empty graph initially
- Nodes appear as discovered
- Layout adjusts dynamically
- Auto-refresh shows live state

**Audio**:
- Silence → chimes as Songbird starts
- New instruments as primals discovered
- Activity increases with discovery events

**Key Learning**: Discovery is dynamic, not static!

---

### Scenario 4: Service Capability Mapping

**Script**: `./demo-capabilities.sh`

**What you'll do**:
1. Launch diverse primals (security, storage, compute)
2. Watch Songbird discover their capabilities
3. See capability-based routing visualization

**Visual**:
- Node colors indicate capability types
- Edge thickness shows routing preferences
- Labels show discovered capabilities

**Audio**:
- Different instruments = different capabilities
- Volume indicates capability activity
- Harmonic relationships = compatible services

**Key Learning**: Songbird understands what each primal CAN DO!

---

### Scenario 5: Health & Resilience

**Script**: `./demo-resilience.sh`

**What you'll do**:
1. Launch federated Songbird mesh
2. Kill one tower
3. Watch federation heal itself
4. See rerouting in real-time

**Visual**:
- Healthy green nodes
- Failing node turns red → gray
- Edges reroute around failed tower
- Topology adjusts

**Audio**:
- Dissonant tone as tower fails
- Silence as it goes away
- Remaining chimes adjust
- Harmony restored as system heals

**Key Learning**: Federated discovery is resilient!

---

## ✅ Success Criteria

After this demo, you should understand:

- [x] How Songbird discovers primals
- [x] How multi-tower federation works
- [x] How discovery events propagate
- [x] How capability-based routing functions
- [x] How federation provides resilience

---

## 🐦 Songbird Integration Details

### What petalTongue Reads from Songbird

**Discovery API**:
- `/api/v1/primals` - List of discovered primals
- `/api/v1/topology` - Full mesh topology
- `/api/v1/towers` - Federation tower list
- `/api/v1/capabilities` - Service capability map

**Event Stream** (if available):
- Discovery events (primal_discovered, primal_lost)
- Federation events (tower_joined, tower_left)
- Routing events (route_updated)

### How petalTongue Visualizes It

**Nodes**:
- Songbird towers = Light Chimes (discovery instrument)
- Discovered primals = Their native instruments
- Size indicates connection count
- Color indicates health

**Edges**:
- Discovery connections (tower → primal)
- Federation connections (tower ↔ tower)
- Routing paths (primal → tower → primal)
- Thickness indicates traffic/preference

**Layout**:
- Force-directed shows organic relationships
- Hierarchical shows tower hierarchy
- Circular shows federation ring

---

## 📊 Visual Patterns to Observe

### Healthy Federation
```
     🐦 Tower A ←→ 🐦 Tower B
       ↙ ↓ ↘         ↙ ↓ ↘
      🐻 🏠 🍄      🐿️ 🐦 🐻
```
- Multiple towers connected
- Each tower manages some primals
- Federation edges strong

### Discovery Event
```
     🐦 Songbird
       ↓ (discovers)
      🏠 NestGate (new!)
```
- New node appears
- Edge forms to discovering tower
- Layout adjusts

### Tower Failure
```
     🐦 Tower A     ⚫ Tower B (failed!)
       ↙ ↓ ↘         
      🐻 🏠 🍄    → reroute →  🐦 Tower A
                              ↙ ↓ ↘ ↘ ↓ ↘
                            🐻 🏠 🍄 🐿️ 🐦 🐻
```
- Failed tower grays out
- Primals reconnect to surviving towers
- System heals

---

## 🎧 Audio Patterns to Listen For

### Discovery Event
- **Before**: Silence or existing instruments
- **During**: New instrument appears (e.g., bass for BearDog)
- **After**: Harmonic integration with other instruments

### Federation
- **Single Tower**: Single chime instrument
- **Multi-Tower**: Multiple chimes in harmony
- **Healthy Federation**: Consonant chords
- **Degraded Federation**: Dissonance increases

### Primal Loss
- **Before**: Full soundscape
- **During**: Instrument fades out
- **After**: Remaining instruments adjust

---

## 🔬 Advanced Exploration

### Try Different Federation Patterns

1. **Hub and Spoke**:
   - Central tower, peripheral towers
   - Hierarchical layout shows this well

2. **Ring**:
   - Towers form a circle
   - Circular layout visualizes perfectly

3. **Full Mesh**:
   - Every tower connected to every other
   - Force-directed shows density

### Test Failure Scenarios

1. **Single Tower Failure**: System continues
2. **Network Partition**: Split brain visualization
3. **Cascade Failure**: Watch propagation
4. **Recovery**: See healing in real-time

### Compare with Songbird's Own Showcase

```bash
# Run Songbird's federation demo
cd ../../../../songbird/showcase/02-federation/
./QUICK_START.sh

# In parallel, run petalTongue
cd ../../petalTongue/showcase/local/06-songbird-discovery/
./demo-multi-tower.sh
```

**Observation**: petalTongue provides VISUAL + AUDIO representation of what Songbird's logs show textually!

---

## 🌱 Fermentation Notes

### What Works Well
- Real-time discovery visualization
- Federation mesh is intuitive
- Audio provides "ambient awareness"
- Both modalities show same information

### Potential Improvements
- Edge animation for active discovery
- Color coding for different protocols
- Time-series of discovery events
- Capability filters (show only storage, etc.)

### Gaps to Document
- If discovery is too fast, hard to see individual events
- Very large meshes (100+ nodes) may be cluttered
- Federation events might need throttling for clarity

---

## 📚 Learning from Songbird

**Songbird's Showcase Excellence**:
- 15+ comprehensive scenarios
- Real multi-machine federation
- Protocol escalation demos
- Student onboarding guides
- Extensive documentation

**What We're Adopting**:
- Real deployments, not mocks
- Progressive complexity
- Clear success criteria
- Troubleshooting guides

**What We're Adding**:
- Visual + audio representation
- Real-time topology visualization
- Accessibility-first approach
- Multi-modal understanding

---

## ⏭️ Next Steps

Once you've visualized Songbird's discovery:

```bash
cd ../07-nestgate-storage/
cat README.md
```

This will show you how to visualize **NestGate's storage mesh**!

---

**Status**: 🌱 Ready to build  
**Complexity**: High (requires Songbird integration)  
**Dependencies**: Songbird built and available, Phase 1 complete

---

*"Discovery is invisible... until petalTongue makes it visible AND audible!"* 🌸

