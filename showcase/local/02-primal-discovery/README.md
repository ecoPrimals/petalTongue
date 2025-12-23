# 02 - Primal Discovery

**Duration**: 10-15 minutes  
**Purpose**: Watch real-time primal discovery as services start and stop

---

## 🎯 What This Demo Does

1. **Starts with empty graph** (0 nodes)
2. **Launches primals one by one** while watching petalTongue
3. **Observes auto-refresh** as new nodes appear
4. **Removes primals dynamically** and watches them disappear
5. **Tests discovery timing** and reliability

**Goal**: Validate that petalTongue correctly displays the ecosystem as it changes in real-time.

---

## 🚀 Quick Start

```bash
./demo.sh
```

This will orchestrate the full discovery demo automatically.

---

## 📋 Prerequisites

- Completed `00-setup` and `01-single-primal`
- BiomeOS running
- petalTongue UI open
- No primals running initially (clean slate)

---

## 🎬 Demo Flow

### Phase 1: Sequential Discovery (Adding Primals)

**Start**: Empty graph (0 nodes, 0 edges)

#### Step 1: Add BearDog

```bash
./add-primal.sh beardog
```

**Observe**:
- Wait 5 seconds (auto-refresh interval)
- Graph updates: **1 node, 0 edges**
- New node appears: BearDog Security (green)

**Timing**: Discovery should happen within 5-10 seconds

#### Step 2: Add NestGate

```bash
./add-primal.sh nestgate
```

**Observe**:
- Wait for auto-refresh
- Graph updates: **2 nodes, ? edges**
- New node appears: NestGate Storage (green)
- May see edges if primals connect

**Note**: With 2 nodes, layout algorithms start to matter

#### Step 3: Add More Primals

```bash
./add-primal.sh songbird
./add-primal.sh toadstool
```

**Observe**:
- Each addition triggers discovery
- Graph grows organically
- Layout algorithm spreads nodes
- Edges may appear between related primals

**Expected**: **4-5 nodes, 2-4 edges** (depending on primal coordination)

### Phase 2: Primal Disappearance (Removing Primals)

#### Step 4: Remove BearDog

```bash
./remove-primal.sh beardog
```

**Observe**:
- Wait for auto-refresh
- Node disappears
- Connected edges disappear
- Remaining nodes stay visible
- Layout may re-adjust

**Validation**: Graph should reflect current state accurately

#### Step 5: Remove All

```bash
./remove-all-primals.sh
```

**Observe**:
- All nodes disappear
- Graph returns to empty state
- No errors or crashes

**Final State**: **0 nodes, 0 edges**

---

## ✅ Success Criteria

After this demo, you should have validated:

- [x] New primals appear within 5-10 seconds
- [x] Auto-refresh works reliably
- [x] Removed primals disappear correctly
- [x] No stale nodes (removed primals don't linger)
- [x] Layout adapts to changing topology
- [x] petalTongue handles 0 → N → 0 gracefully

---

## 🔧 Troubleshooting

### Primal doesn't appear

**Problem**: Added primal but graph doesn't update  
**Solutions**:
1. Wait full 5 seconds for auto-refresh
2. Click "Refresh Now" to force update
3. Check BiomeOS discovered it: `curl http://localhost:3000/api/v1/primals | jq`
4. Check primal is actually running: `ps aux | grep <primal>`

### Primal won't disappear

**Problem**: Removed primal but node stays visible  
**Solutions**:
1. Wait for auto-refresh (5s)
2. Force refresh
3. Verify primal is actually stopped: `ps aux | grep <primal>`
4. Check BiomeOS no longer lists it: `curl http://localhost:3000/api/v1/primals | jq`

### Edges don't appear

**Problem**: Multiple primals but no connections shown  
**Solutions**:
1. This might be correct - not all primals connect immediately
2. Check BiomeOS topology: `curl http://localhost:3000/api/v1/topology | jq`
3. Wait longer - connections may take time to establish
4. Some primal combinations don't create edges

---

## 🌱 Fermentation Notes

### Gaps to Watch For

As you run this demo, look for:

- **Discovery Timing**:
  - Is 5s auto-refresh too slow?
  - Should there be visual feedback during discovery?
  - Does manual refresh work reliably?

- **Visual Feedback**:
  - Any indication that refresh is happening?
  - Does graph "jump" during layout updates?
  - Are transitions smooth or jarring?

- **Edge Cases**:
  - What if primal crashes mid-discovery?
  - What if BiomeOS loses connection to primal?
  - What if multiple primals start simultaneously?

- **Layout Behavior**:
  - Does layout thrash with rapid adds/removes?
  - Does camera position stay sensible?
  - Do nodes spread out nicely as more are added?

**Document any gaps in**: `../GAPS.md`

---

## 📊 Expected Timeline

| Action | Wait Time | Expected Result |
|--------|-----------|-----------------|
| Start BiomeOS | ~5s | Ready to discover |
| Launch primal | ~2s | Primal starts |
| BiomeOS discovers | ~3s | Added to registry |
| petalTongue refreshes | 0-5s | Node appears |
| **Total** | **10-15s** | **Primal visible** |

**Key Insight**: Discovery is not instant. There's a pipeline:
1. Primal starts
2. BiomeOS discovers (via Songbird or other mechanism)
3. petalTongue polls BiomeOS
4. Graph updates

---

## 🎓 Learning Points

### Discovery Pipeline

```
Primal Starts
     ↓
BiomeOS Discovery Service (Songbird)
     ↓
BiomeOS Registry
     ↓
petalTongue Polls (every 5s)
     ↓
Graph Engine Update
     ↓
Visual Renderer
     ↓
User Sees Node
```

**Total Latency**: 5-15 seconds (depending on timing)

### Why Auto-Refresh?

**Polling** (current):
- Simple to implement
- Predictable timing
- Low complexity

**Push** (future):
- Lower latency
- More complex
- Requires WebSocket/SSE

**Trade-off**: For monitoring, 5s latency is acceptable. For real-time ops, push would be better.

---

## 💡 Real-World Analogies

### Kubernetes Discovery

Similar to watching `kubectl get pods` as deployments scale:
```bash
# Start: 0 pods
kubectl scale deployment/app --replicas=5
# Watch pods appear one by one
kubectl get pods -w
```

### Docker Compose

Similar to `docker-compose up` with `depends_on`:
```bash
# Services start in order
# petalTongue would show them appearing sequentially
```

### Service Mesh Discovery

Similar to Istio/Linkerd service discovery:
- Services register with control plane
- Sidecars discover each other
- Topology graph builds over time

---

## ⏭️ Next Steps

Once comfortable with discovery, proceed to:

```bash
cd ../03-topology-visualization/
cat README.md
```

This will show you **full ecosystem topology** with all primals running together.

---

## 🎮 Advanced Experiments

### Rapid Add/Remove

Test discovery robustness:
```bash
for i in {1..10}; do
  ./add-primal.sh beardog
  sleep 2
  ./remove-primal.sh beardog
  sleep 2
done
```

**Observe**: Does petalTongue handle rapid churn? Any issues?

### Simultaneous Starts

Test concurrent discovery:
```bash
./add-primal.sh beardog &
./add-primal.sh nestgate &
./add-primal.sh songbird &
wait
```

**Observe**: Do all primals appear? Any race conditions?

### Failure Injection

Test resilience:
```bash
./add-primal.sh beardog
sleep 3
kill -9 <beardog-pid>  # Crash it
# Wait for discovery to notice
```

**Observe**: Does it disappear? How long does it take?

---

**Status**: 🌱 Ready to build  
**Complexity**: Medium  
**Dependencies**: 00-setup, 01-single-primal  
**Learning Value**: High (real-world scenario)

---

*Discovery is where the ecosystem comes alive!* 🌸

