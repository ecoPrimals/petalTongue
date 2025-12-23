# 04 - Health Monitoring

**Duration**: 15-20 minutes  
**Purpose**: Observe real-time health status changes and system degradation

---

## 🎯 What This Demo Does

1. **Launches a healthy ecosystem** (all primals green)
2. **Triggers health degradation** (warning → critical states)
3. **Monitors visual feedback** (color changes, audio cues)
4. **Demonstrates recovery** (critical → healthy)
5. **Tests alerting mechanisms** (how quickly issues are noticed)

**Goal**: Validate that petalTongue effectively communicates health status changes.

---

## 🚀 Quick Start

```bash
./demo.sh
```

This will walk you through health state transitions.

---

## 📋 Prerequisites

- Completed `00-setup`, `01-single-primal`, `02-primal-discovery`, `03-topology-visualization`
- BiomeOS running
- petalTongue UI open
- Clean slate (no primals running)

---

## 🎬 Demo Flow

### Phase 1: Healthy Baseline

**Start**: Launch a healthy ecosystem

```bash
./launch-healthy.sh
```

**Expected State**:
- All nodes: **GREEN** (healthy)
- No warnings or alerts
- Normal operation

**Observe**:
- Visual: All nodes are bright green
- Audio: Harmonious tones (if audio enabled)
- Stats: 100% healthy

### Phase 2: Warning State (Degraded)

**Trigger**: Simulate a warning condition

```bash
./trigger-warning.sh beardog
```

**What happens**:
- BearDog node turns **YELLOW**
- Warning indicator appears
- Audio: Slightly off-pitch tone
- Stats update: Health drops to ~70-80%

**Expected Timeline**: Warning detected within 5-10 seconds

**Observe**:
- How quickly do you notice the issue?
- Is the yellow color distinct enough?
- Does audio help with awareness?

### Phase 3: Critical State (Failed)

**Trigger**: Escalate to critical

```bash
./trigger-critical.sh beardog
```

**What happens**:
- BearDog node turns **RED**
- Critical alert appears
- Audio: Dissonant tone (if enabled)
- Stats update: Health drops to ~20-30%

**Expected Timeline**: Critical detected within 5-10 seconds

**Observe**:
- Is the visual feedback urgent enough?
- Would you notice this immediately in production?
- How does the rest of the ecosystem react?

### Phase 4: Multiple Failures

**Trigger**: Create a cascading failure

```bash
./trigger-critical.sh nestgate
sleep 5
./trigger-warning.sh songbird
```

**What happens**:
- Multiple nodes degrade simultaneously
- System-wide health drops
- Visual chaos (red + yellow nodes)
- Audio: Complex dissonance

**Observe**:
- Can you still understand the topology?
- Which failures are most critical?
- Does the UI become overwhelming?

### Phase 5: Recovery

**Trigger**: Restore health

```bash
./restore-health.sh beardog
sleep 5
./restore-health.sh nestgate
sleep 5
./restore-health.sh songbird
```

**What happens**:
- Nodes transition back to green
- Alerts clear
- Audio returns to harmony
- Stats improve

**Observe**:
- Recovery is as visible as degradation?
- Smooth transitions or jarring?

---

## ✅ Success Criteria

After this demo, you should have validated:

- [x] Health states visually distinct (Green, Yellow, Red)
- [x] Transitions detected within 5-10 seconds
- [x] Audio feedback complements visual (if enabled)
- [x] Multiple failures can be understood simultaneously
- [x] Recovery is clearly communicated
- [x] Stats panel accurately reflects ecosystem health

---

## 🎨 Health Color Coding

### Healthy (Green)

**RGB**: `#00FF00` (bright green)

**Meaning**:
- All systems operational
- No issues detected
- Normal operation

**Audio**: Harmonic, pleasant tone

### Warning (Yellow)

**RGB**: `#FFFF00` (bright yellow)

**Meaning**:
- Minor issues detected
- Degraded performance
- Requires attention soon

**Audio**: Slightly off-pitch, noticeable dissonance

### Critical (Red)

**RGB**: `#FF0000` (bright red)

**Meaning**:
- Major failure
- Non-functional or severely degraded
- Immediate action required

**Audio**: Strong dissonance, urgent tone

### Unknown (Gray)

**RGB**: `#808080` (gray)

**Meaning**:
- Health status not reported
- Connection lost
- Discovery in progress

**Audio**: Silence or very quiet tone

---

## 🔧 Troubleshooting

### Health changes not visible

**Problem**: Triggered warning but node stays green  
**Solutions**:
1. Wait for auto-refresh (5s)
2. Force refresh
3. Check BiomeOS reports the health change: `curl http://localhost:3000/api/v1/primals | jq '.[] | {name, health}'`
4. Verify script actually changed primal state

### Can't tell colors apart

**Problem**: Green/Yellow/Red not distinct enough  
**Solutions**:
1. This is critical accessibility feedback - document in GAPS.md
2. Try adjusting monitor brightness
3. Consider colorblind mode (future feature)
4. Rely on audio feedback instead

### Audio not helping

**Problem**: Can't hear health differences  
**Solutions**:
1. Increase volume
2. Check audio is enabled in petalTongue
3. Try headphones for better distinction
4. Document if audio feedback is unclear

---

## 🌱 Fermentation Notes

### Gaps to Watch For

As you run this demo, look for:

- **Visual Feedback**:
  - Colors distinct enough?
  - Colorblind accessible?
  - Transitions smooth or jarring?
  - Urgent states feel urgent?

- **Audio Feedback**:
  - Dissonance noticeable?
  - Too subtle or too harsh?
  - Helpful or distracting?
  - Blind-accessible?

- **Detection Speed**:
  - 5-10s acceptable?
  - Need push notifications?
  - Real-time critical?

- **Multiple Failures**:
  - Can you prioritize issues?
  - Does UI get overwhelming?
  - Which node to fix first?

- **Statistics**:
  - Overall health % useful?
  - Per-node details sufficient?
  - Trend over time? (future)

**Document any gaps in**: `../GAPS.md`

---

## 📊 Health Monitoring Best Practices

### Visual Hierarchy

1. **Critical** (Red) - Highest priority
2. **Warning** (Yellow) - Medium priority
3. **Healthy** (Green) - No action needed
4. **Unknown** (Gray) - Investigation needed

**Action**: Always address Critical first, then Warning

### Alert Fatigue

**Problem**: Too many warnings → ignore them all

**Solutions**:
- Tune thresholds (what triggers warning/critical?)
- Aggregate alerts (5 warnings → 1 system warning)
- Auto-recovery (don't alert on transient issues)

**petalTongue Goal**: Clear signal, minimal noise

### Accessibility

**Visual**: Color + shape + position + size  
**Audio**: Pitch + rhythm + timbre + stereo  
**AI**: Text narration

**No single modality sufficient alone.**

---

## 💡 Real-World Scenarios

### Production Outage

**Timeline**:
1. Load spike
2. Toadstool goes Warning (high CPU)
3. Requests queue up
4. Toadstool goes Critical (OOM)
5. BearDog goes Warning (auth timeouts)
6. Cascading failure

**petalTongue Shows**:
- Toadstool turns yellow, then red
- BearDog turns yellow
- Edge traffic increases
- System health drops

**Ops Response**:
1. See Toadstool is root cause (red first)
2. Scale Toadstool (more resources)
3. Watch BearDog recover automatically
4. All return to green

### Network Partition

**Timeline**:
1. Network split
2. Songbird can't reach NestGate
3. Both go Unknown (gray)
4. Other primals still healthy

**petalTongue Shows**:
- Songbird and NestGate turn gray
- Edge between them disappears
- Partial topology visible

**Ops Response**:
1. See network issue (not primal issue)
2. Check network logs
3. Restore connectivity
4. Nodes return to green

### Planned Maintenance

**Timeline**:
1. Drain BearDog traffic
2. Shutdown BearDog
3. Upgrade BearDog
4. Restart BearDog
5. Resume traffic

**petalTongue Shows**:
- BearDog goes Warning (drain)
- BearDog disappears (shutdown)
- BearDog reappears (restart)
- BearDog goes Green (healthy)

**Expected**: Planned, controlled, no surprise

---

## 🎓 Learning Points

### Health ≠ Availability

**Healthy** = Internal metrics good  
**Available** = Responding to requests

A primal can be:
- Healthy but unavailable (network issue)
- Available but unhealthy (degraded, slow)

**petalTongue Goal**: Show both (health + connectivity)

### Health as Spectrum

Not just on/off:

```
100% ████████████████████ Healthy (Green)
 80% █████████████░░░░░░░ Warning (Yellow)
 50% ██████░░░░░░░░░░░░░░ Warning (Yellow)
 20% ███░░░░░░░░░░░░░░░░░ Critical (Red)
  0% ░░░░░░░░░░░░░░░░░░░░ Critical (Red)
```

**Thresholds** determine color:
- 80-100%: Green
- 30-79%: Yellow
- 0-29%: Red

**Question**: Are these the right thresholds?

### Health Aggregation

For topology with N nodes:

**System Health** = Average of node healths?  
**Or**: Weighted by importance?  
**Or**: Worst node defines system?

**Different models**:
- **Average**: `sum(health) / N`
- **Weighted**: `sum(health * weight) / sum(weight)`
- **Min**: `min(health)` (pessimistic)
- **Critical-aware**: If any Critical → System Critical

**petalTongue**: Currently uses average. Is that right?

---

## ⏭️ Next Steps

Once comfortable with health monitoring, proceed to:

```bash
cd ../05-accessibility-validation/
cat README.md
```

This will test **audio-only mode** and screen reader compatibility.

---

## 🎮 Advanced Experiments

### Flapping Health

Test rapid state changes:
```bash
for i in {1..5}; do
  ./trigger-warning.sh beardog
  sleep 3
  ./restore-health.sh beardog
  sleep 3
done
```

**Observe**: Does UI handle rapid transitions? Annoying or useful?

### Gradual Degradation

Simulate slow failure:
```bash
./trigger-warning.sh beardog
sleep 30
./trigger-critical.sh beardog
```

**Observe**: Is the transition noticeable? Should intermediate states exist?

### Recovery Timing

Measure recovery detection:
```bash
time (./restore-health.sh beardog && ./wait-for-green.sh beardog)
```

**Observe**: How long from recovery to green in UI?

---

**Status**: 🌱 Ready to build  
**Complexity**: Medium  
**Dependencies**: 03-topology-visualization  
**Learning Value**: Very High (operational awareness)

---

*Health monitoring isn't just about knowing problems exist.  
It's about knowing problems exist **right now**, **which ones matter**, and **what to do**.* 🌸

