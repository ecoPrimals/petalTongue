# 🎵 Songbird Discovery - Inter-Primal Showcase

**Duration**: 15 minutes  
**Complexity**: Medium  
**Prerequisites**: Songbird running (local or remote)

---

## 🎯 What This Demonstrates

This showcase demonstrates petalTongue's ability to:
1. **Discover** songbird towers via mDNS/HTTP
2. **Visualize** multi-tower federation topology
3. **Monitor** routing decisions in real-time
4. **Sonify** protocol escalation (HTTP → tarpc → TLS)
5. **Show** trust relationships between towers

---

## 🚀 Quick Start

```bash
# 1. Start songbird (if not running)
cd /path/to/songBird
cargo run --release &

# 2. Wait for startup (10 seconds)
sleep 10

# 3. Run demo
cd /path/to/petalTongue/showcase/03-inter-primal/01-songbird-discovery
./demo.sh
```

---

## 📋 Prerequisites

### **Minimal Setup** (Single Tower)
```bash
# Just songbird running locally
cd /path/to/songBird
cargo run --release
```

**Expected**: Single node visualization

---

### **Federation Setup** (Multi-Tower) - OPTIONAL
```bash
# Start tower A
cd /path/to/songBird/showcase/02-federation
./scripts/start-tower-a.sh &

# Start tower B
./scripts/start-tower-b.sh &

# Verify federation
./scripts/check-federation.sh
```

**Expected**: Multi-tower graph with federation links

---

## 🎨 What You'll See

### **Visual Representation**
- **Nodes**: Each songbird tower
- **Edges**: Federation connections
- **Colors**: 
  - 🟢 Green = Healthy, trusted
  - 🟡 Yellow = Elevated trust
  - 🔵 Blue = Limited trust
  - ⚫ Gray = Unknown/new
- **Size**: Node size based on number of known primals
- **Animation**: Flow particles showing routing activity

### **Audio Representation**
- **Piano**: Tower discovery (higher pitch = more trusted)
- **Strings**: Federation establishment
- **Brass**: Protocol escalation (HTTP → tarpc)
- **Synth**: TLS handshake
- **Spatial**: Left/right based on trust level

---

## 📊 Expected Output

### **Console Output**
```
🌸 petalTongue Showcase: Songbird Discovery
===========================================

[00:00] Starting discovery...
[00:02] ✅ Discovered: songbird-tower-1 (http://localhost:3000)
[00:02]    - Status: Healthy
[00:03]    - Capabilities: [discovery, routing, federation]
[00:04]    - Known primals: 5

[00:05] 🎵 Generating visual representation...
[00:05]    - Layout: ForceDirected
[00:05]    - Nodes: 1
[00:05]    - Edges: 0

[00:06] 🎵 Generating audio sonification...
[00:06]    - Instrument: Piano (discovery)
[00:06]    - Pitch: A4 (healthy status)
[00:06]    - Panning: Center

[00:10] Press Ctrl+C to stop demo
```

### **UI Window**
- **Main panel**: Graph visualization
- **Top-left**: FPS counter, zoom level
- **Top-right**: Discovered primals count
- **Bottom**: Audio controls (play/pause, instrument selection)
- **Side panel**: Primal details (name, health, capabilities)

---

## 🎓 What You're Learning

### **Concept 1: Discovery Protocol**
petalTongue uses:
1. **mDNS** for local network discovery
2. **HTTP** for primal metadata
3. **Properties API** for ecosystem-specific data

**Watch for**: How petalTongue discovers without hardcoding

### **Concept 2: Multi-Tower Federation**
Songbird's strength:
- Towers federate automatically
- Trust escalates progressively
- Routes optimize based on capabilities

**Watch for**: Federation links appearing in graph

### **Concept 3: Protocol Escalation**
petalTongue visualizes:
- HTTP discovery (first contact)
- tarpc for efficient RPC
- TLS for secure channels

**Watch for**: Edge colors changing (protocol upgrade)

### **Concept 4: Multi-Modal Rendering**
Both visual AND audio:
- Blind users hear topology
- Sighted users see graph
- Both together = complete picture

**Watch for**: Audio changing with graph updates

---

## 🛠️ Troubleshooting

### **Problem**: "No primals discovered"
**Solution**:
```bash
# Check songbird is running
curl http://localhost:3000/health

# If not running:
cd /path/to/songBird
cargo run --release &
sleep 10

# Try demo again
./demo.sh
```

---

### **Problem**: "Connection refused"
**Solution**:
```bash
# Check port (default 3000)
ss -tlnp | grep 3000

# If port different:
export SONGBIRD_URL=http://localhost:YOUR_PORT
./demo.sh
```

---

### **Problem**: "Graph is empty"
**Solution**:
```bash
# Discovery takes 5-10 seconds, wait
# Or manually refresh in UI (press R)

# Check logs:
RUST_LOG=debug ./demo.sh
```

---

### **Problem**: "No audio"
**Solution**:
```bash
# Audio system may need initialization
# Check capabilities panel (press C)
# Verify "Audio: Enabled"

# If disabled:
# Check ALSA/PulseAudio running
pactl info

# May need to restart with audio explicit:
AUDIO_ENABLED=1 ./demo.sh
```

---

## 🧪 Experiments to Try

### **Experiment 1: Add More Towers**
```bash
# Start tower B
cd /path/to/songBird/showcase/02-federation
./scripts/start-tower-b.sh &

# Watch graph update
# Listen for federation sound (strings)
```

**Expected**: New node appears, federation edge connects

---

### **Experiment 2: Kill a Tower**
```bash
# Find tower PID
ps aux | grep songbird

# Kill it
kill PID

# Watch graph update
# Listen for degradation sound (lower pitch)
```

**Expected**: Node color changes to gray, edges disappear

---

### **Experiment 3: Protocol Escalation**
```bash
# Start with HTTP only
PROTOCOLS=http ./demo.sh

# Then enable tarpc
# (Restart with full protocols)
./demo.sh

# Watch edge color change
# Listen for escalation sound (brass)
```

**Expected**: Edge color brightens (protocol upgrade)

---

### **Experiment 4: Change Layout**
```bash
# In UI, press L to cycle layouts:
# - ForceDirected (organic)
# - Circular (clean)
# - Hierarchical (top-down)
# - Grid (structured)

# Which layout best shows federation?
```

**Expected**: Same data, different visual structure

---

## 📚 Related Demos

### **Before This**:
- **Phase 2**: `02-primal-discovery` (general discovery)
- **Phase 2**: `03-topology-viz` (basic topology)

### **After This**:
- **03-inter-primal**: `02-beardog-security` (key visualization)
- **03-inter-primal**: `04-toadstool-compute` (compute mesh)

### **Advanced**:
- **03-inter-primal**: `07-full-ecosystem` (all primals)

---

## 🌟 Key Takeaways

1. **Discovery is automatic** - No config needed
2. **Federation is visual** - Can see tower relationships
3. **Multi-modal works** - Audio + visual together
4. **Real-time updates** - Graph reflects reality
5. **Accessibility first** - Can experience without sight

---

## 📊 Success Criteria

After this demo, you should be able to:
- ✅ Discover songbird via mDNS/HTTP
- ✅ Visualize federation topology
- ✅ Hear protocol escalation
- ✅ Understand trust relationships
- ✅ Monitor routing in real-time

---

## 🚀 What's Next?

### **Immediate**:
Try: `../02-beardog-security/` (key lineage)

### **Build On This**:
Try: `../07-full-ecosystem/` (add more primals)

### **Deep Dive**:
Read: `/path/to/songBird/showcase/02-federation/README.md`

---

*Demo Ready: January 2026*  
*Status: 📋 Planned (Week 2)*  
*Integration: songbird + petalTongue*

🌸🎵 **Visualize the federation!** 🚀

