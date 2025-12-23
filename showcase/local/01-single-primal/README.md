# 01 - Single Primal Visualization

**Duration**: 5-10 minutes  
**Purpose**: Learn to visualize individual primals before complex topologies

---

## 🎯 What This Demo Does

1. **Launches one primal** at a time (beardog, nestgate, songbird, etc.)
2. **Shows petalTongue** displaying a single node
3. **Demonstrates health states** (healthy by default)
4. **Displays audio descriptions** for that primal
5. **Validates basic rendering** works correctly

**Goal**: Understand petalTongue with the simplest possible topology (1 node, 0 edges).

---

## 🚀 Quick Start

### Run All Single-Primal Demos

```bash
./demo.sh
```

This will show each primal type one at a time.

### Run Individual Primal Demos

```bash
./beardog-only.sh       # Security primal
./nestgate-only.sh      # Storage primal
./songbird-only.sh      # Discovery primal (if available)
./toadstool-only.sh     # Compute primal (if available)
```

---

## 📋 Prerequisites

- Completed `00-setup` scenario
- BiomeOS running
- petalTongue UI open
- At least one primal binary available in `biomeOS/bin/primals/`

---

## 🎬 Demo Flow

### Step 1: Launch BearDog (Security Primal)

```bash
./beardog-only.sh
```

**What happens**:
1. Script launches beardog primal
2. BiomeOS discovers it (via Songbird)
3. petalTongue auto-refreshes (within 5s)
4. You see: **1 node, 0 edges**

**Expected UI**:
- **Center**: Single green circle labeled "BearDog Security"
- **Right Panel**: Audio description
  - Instrument: Deep Bass
  - Pitch: Healthy (harmonic)
  - Position: Center (no spatial data yet)
  - Volume: Moderate
- **Statistics**: "1 node, 0 edges"

**What to observe**:
- Node renders correctly
- Health color is green
- Audio description makes sense
- No errors in console

### Step 2: Stop BearDog

```bash
./stop-all-primals.sh
```

**What happens**:
1. Script stops beardog
2. BiomeOS notices it's gone
3. petalTongue auto-refreshes
4. You see: **0 nodes, 0 edges**

**Verify**: Graph is empty again

### Step 3: Launch NestGate (Storage Primal)

```bash
./nestgate-only.sh
```

**Expected UI**:
- **Center**: Single green circle labeled "NestGate Storage"
- **Right Panel**: Audio description
  - Instrument: Sustained Strings
  - Pitch: Healthy (harmonic)
  - Position: Center
  - Volume: Moderate
- **Statistics**: "1 node, 0 edges"

### Step 4: Compare Primal Types

Try each primal script and observe differences:

| Primal | Type | Audio Instrument | Visual Color | Purpose |
|--------|------|------------------|--------------|---------|
| BearDog | Security | Deep Bass | Green | Identity & crypto |
| NestGate | Storage | Sustained Strings | Green | Data storage |
| Songbird | Discovery | Light Chimes | Green | Service discovery |
| ToadStool | Compute | Rhythmic Drums | Green | Workload execution |
| Squirrel | AI | High Synth | Green | AI coordination |

**Key insight**: Same data (1 healthy node), different audio mapping, same visual representation.

---

## ✅ Success Criteria

After this demo, you should understand:

- [x] How petalTongue displays a single primal
- [x] What audio descriptions look like
- [x] How layout algorithms handle 1 node (trivial - just center it)
- [x] How auto-refresh works
- [x] How to start/stop primals for testing

---

## 🔧 Troubleshooting

### Primal won't start

**Problem**: `./beardog-only.sh` fails  
**Solutions**:
1. Check if binary exists: `ls ../../../biomeOS/bin/primals/beardog`
2. Check if port is available: `lsof -i :8001`
3. Check logs: `tail -f logs/beardog.log`

### petalTongue doesn't show primal

**Problem**: Launched primal but graph is empty  
**Solutions**:
1. Wait for auto-refresh (5s)
2. Click "Refresh Now" button
3. Check BiomeOS discovered it: `curl http://localhost:3000/api/v1/primals`
4. Check primal is running: `ps aux | grep beardog`

### Node is red/yellow instead of green

**Problem**: Primal shows as warning/critical  
**Solutions**:
1. This might be correct - check primal health endpoint
2. Check primal logs for errors
3. Restart primal: `./stop-all-primals.sh && ./beardog-only.sh`

---

## 🌱 Fermentation Notes

### Gaps to Watch For

As you run this demo, look for:

- **Visual Issues**:
  - Node doesn't render?
  - Color is wrong?
  - Label is unclear?
  
- **Audio Issues**:
  - Description doesn't make sense?
  - Instrument mapping seems off?
  - Missing information?

- **UX Issues**:
  - Hard to see single node?
  - Camera zoomed too far in/out?
  - Layout algorithm choice matters for 1 node?

- **Performance Issues**:
  - Slow to refresh?
  - High CPU with just 1 node?
  - Memory usage unexpected?

**Document any gaps in**: `../GAPS.md`

---

## 📊 What You Should See

### BearDog (Security)

**Visual**:
```
         ●
    BearDog-1
     Security
```
- Green circle
- Centered in view
- Label below

**Audio Panel**:
```
🎵 BearDog Security
   Instrument: Deep Bass
   Pitch: Healthy (harmonic, on-key)
   Position: Center (0, 0)
   Volume: Moderate (activity level)
   
   A blind user would hear: A low, smooth bass tone
   at moderate volume, centered in both speakers,
   indicating a healthy security service.
```

**Statistics**:
```
📊 Graph Statistics
   Nodes: 1
   Edges: 0
   Average Degree: 0.0
   Zoom: 1.0x
```

---

## 💡 Key Learnings

### Why Start with Single Primal?

1. **Simplest case**: Can't get simpler than 1 node
2. **Isolates variables**: Any issue is definitely node-rendering
3. **Tests basics**: Node shape, color, label, audio
4. **Builds foundation**: Complex topologies are just many single nodes

### Differences from Mock Server

**Mock server** (sandbox): Uses JSON files, no real primals  
**This demo**: Uses **actual primal binaries**, real discovery, live health

**Why this matters**: Real primals can fail, have network issues, report actual health. Mocks always behave perfectly.

---

## ⏭️ Next Steps

Once comfortable with single primals, proceed to:

```bash
cd ../02-primal-discovery/
cat README.md
```

This will show you how petalTongue handles **real-time discovery** as primals come and go.

---

## 🎓 Advanced Exploration

### Try Different Health States

1. Launch primal
2. Manually inject warning state (if primal supports it)
3. Observe color change: green → yellow
4. Observe audio description change: harmonic → off-key

### Try Different Layouts

With just 1 node, layouts shouldn't matter. But try switching:
- Force-Directed → should just center it
- Hierarchical → should just center it
- Circular → should just center it

**Observation**: For 1 node, layout is irrelevant. Good to know!

### Try Camera Controls

- Pan: Should be able to move view around
- Zoom: Should be able to zoom in/out
- Reset: Should return to centered view

**Validation**: Basic camera controls work even with minimal data.

---

**Status**: 🌱 Ready to build  
**Complexity**: Low  
**Dependencies**: 00-setup complete  
**Learning Value**: High (foundation for everything else)

---

*Single nodes are simple, but they teach us the fundamentals!* 🌸

