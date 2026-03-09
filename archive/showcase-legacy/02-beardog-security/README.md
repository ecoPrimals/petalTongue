# 🐻 BearDog Security - Inter-Primal Showcase

**Duration**: 10 minutes  
**Complexity**: Medium  
**Prerequisites**: BearDog running, BiomeOS discovering it

---

## 🎯 What This Demonstrates

This showcase demonstrates petalTongue's ability to:
1. **Visualize** BearDog security primal via BiomeOS
2. **Show** trust relationships (trust_level, family_id)
3. **Display** security capabilities (btsp, birdsong, lineage)
4. **Sonify** trust levels through audio
5. **Monitor** security primal health in real-time

---

## 🚀 Quick Start

```bash
# 1. Ensure BearDog is running (check via BiomeOS)
curl http://localhost:3000/api/v1/primals | jq '.primals[] | select(.primal_type == "security")'

# 2. Run demo
cd /path/to/petalTongue/showcase/03-inter-primal/02-beardog-security
./demo.sh
```

---

## 📋 Prerequisites

### **Minimal Setup**
```bash
# BearDog should be discovered via BiomeOS
curl http://localhost:3000/api/v1/primals
```

**Expected**: BearDog listed with `primal_type: "security"`

---

## 🎨 What You'll See

### **Visual Representation**
- **Node**: BearDog security primal
- **Colors**: 
  - 🟢 Green (trust_level 3 = Full) 
  - 🟠 Orange (trust_level 2 = Elevated)
  - 🟡 Yellow (trust_level 1 = Limited)
  - ⚫ Gray (trust_level 0 = None)
- **Badge**: Trust level indicator
- **Ring**: Family ID color-coded
- **Size**: Based on capability count

### **Audio Representation**
- **Piano**: Discovery (higher pitch = higher trust)
- **Strings**: Trust establishment
- **Brass**: Capability invocation
- **Chime**: Full trust (trust_level 3)
- **Spatial**: Center (single primal view)

### **Trust Levels Explained**:
- **Level 0 (None)**: No trust, capabilities blocked
- **Level 1 (Limited)**: Basic capabilities only
- **Level 2 (Elevated)**: Most capabilities allowed
- **Level 3 (Full)**: All capabilities, full access

---

## 📊 Expected Output

### **Console Output**
```
🌸 petalTongue Showcase: BearDog Security
==========================================

[00:00] Checking prerequisites...
[00:01] Checking if BiomeOS is running...
✅ BiomeOS running at http://localhost:3000

[00:02] Checking BiomeOS health...
{
  "status": "healthy",
  "version": "0.1.0",
  "mode": "live"
}

[00:03] Looking for security primals...
✅ Found security primal: BearDog
{
  "id": "beardog-local",
  "name": "BearDog",
  "primal_type": "security",
  "health": "healthy",
  "capabilities": ["btsp", "birdsong", "lineage"],
  "trust_level": 3,
  "family_id": "iidn"
}

[00:04] Launching petalTongue...
```

### **UI Window**
- **Main panel**: Graph with BearDog node
- **Trust badge**: "🟢" (full trust)
- **Family ring**: Colored ring around node
- **Capabilities**: Listed in side panel
- **Audio**: Chime sound (full trust)

---

## 🎓 What You're Learning

### **Concept 1: Trust-Based Visualization**
petalTongue shows security posture visually:
- **Color** = trust level (intuitive)
- **Badge** = trust indicator (quick reference)
- **Ring** = family grouping (relationships)

**Watch for**: Node color matching trust level

### **Concept 2: Capability Discovery**
BearDog's capabilities discovered at runtime:
- `btsp` - Birdsong Trusted Secure Protocol
- `birdsong` - Discovery protocol support
- `lineage` - Key lineage tracking

**Watch for**: Capabilities listed in UI panel

### **Concept 3: Family Grouping**
Trust families group related primals:
- Same `family_id` = same visual ring color
- Helps identify primal relationships
- Useful for multi-instance security

**Watch for**: Family ring color (if multiple primals)

### **Concept 4: Trust Audio Feedback**
Different trust levels have different sounds:
- Level 0: Low bass note
- Level 1: Mid-range note
- Level 2: Higher note
- Level 3: Bright chime

**Watch for**: Audio correlating with visual

---

## 🛠️ Troubleshooting

### **Problem**: "No security primals found"
**Solution**:
```bash
# Check if BearDog is running
ps aux | grep beardog

# Check if BiomeOS discovered it
curl http://localhost:3000/api/v1/primals | jq '.primals'

# If not running:
cd /path/to/ecoPrimals/primalBins
./beardog &
sleep 5

# BiomeOS should discover automatically
```

---

### **Problem**: "Trust level shows as 0 or null"
**Solution**:
```bash
# BearDog may not have trust evaluation yet
# This is expected on first discovery
# Trust can be elevated via BiomeOS API

# Check current trust:
curl http://localhost:3000/api/v1/primals | jq '.primals[] | select(.primal_type == "security") | {name, trust_level}'
```

---

### **Problem**: "No capabilities shown"
**Solution**:
```bash
# BearDog should report capabilities in health endpoint
curl http://localhost:9000/health | jq '.capabilities'

# If empty, BearDog version may not report them
# Upgrade to latest BearDog (v0.15.0+)
```

---

## 🧪 Experiments to Try

### **Experiment 1: Trust Visualization**
```bash
# View current trust level
curl http://localhost:3000/api/v1/primals | jq '.primals[] | select(.primal_type == "security") | .trust_level'

# In petalTongue:
# - Note the node color
# - Note the trust badge
# - Listen to audio pitch
```

**Expected**: Visual + audio match trust level

---

### **Experiment 2: Multiple Security Primals** (if available)
```bash
# If you have multiple BearDog instances:
BEARDOG_PORT=9001 beardog &

# Wait for BiomeOS discovery
sleep 10

# Run petalTongue
./demo.sh

# Watch for:
# - Two beardog nodes
# - Same family_id = same ring color
# - Different trust levels possible
```

**Expected**: Multiple nodes, family grouping visible

---

### **Experiment 3: Capability-Based Routing**
```bash
# In petalTongue UI:
# 1. Select BearDog node
# 2. View capabilities panel
# 3. See: btsp, birdsong, lineage

# This data enables:
# - Capability-based routing
# - Feature discovery
# - API compatibility checking
```

**Expected**: Clear capability list in UI

---

### **Experiment 4: Trust Evolution** (future)
```bash
# NOTE: Trust elevation via petalTongue UI is Phase 4 feature
# For now, trust is read-only visualization

# Future capability:
# - Right-click node
# - "Elevate Trust" option
# - BiomeOS API call to elevate
# - Visual/audio feedback
```

**Expected**: (Phase 4) Interactive trust management

---

## 📚 Related Demos

### **Before This**:
- **Phase 2**: `04-health-monitoring` (health visualization)
- **03-inter-primal**: `01-songbird-discovery` (ecosystem overview)

### **After This**:
- **03-inter-primal**: `04-toadstool-compute` (compute visualization)
- **03-inter-primal**: `07-full-ecosystem` (all primals together)

### **Advanced**:
- **BearDog Showcases**: `/phase1/beardog/showcase/` (BearDog-specific features)

---

## 🌟 Key Takeaways

1. **Trust is visual** - Color-coded security posture
2. **Capabilities discoverable** - Runtime feature detection
3. **Family grouping works** - Related primals identifiable
4. **Audio enhances understanding** - Trust levels audible
5. **BiomeOS aggregates security** - Single view of all security primals

---

## 📊 Success Criteria

After this demo, you should be able to:
- ✅ Visualize BearDog via BiomeOS
- ✅ Understand trust level colors
- ✅ See capability arrays
- ✅ Hear trust level audio
- ✅ Identify family relationships

---

## 🚀 What's Next?

### **Immediate**:
Try: `../04-toadstool-compute/` (compute mesh visualization)

### **Build On This**:
Try: `../07-full-ecosystem/` (security + orchestration + compute)

### **Deep Dive**:
Read: `/phase1/beardog/docs/TRUST_FRAMEWORK.md`

---

*Demo Ready: January 2026*  
*Status: ✅ Complete (with live BearDog)*  
*Integration: BiomeOS aggregator*

🌸🐻 **Visualize security trust!** 🚀

