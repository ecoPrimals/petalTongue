# Demo 02: Degraded System

**Duration**: 3-5 minutes  
**Target Audience**: SREs, operations teams, monitoring specialists  
**Difficulty**: Beginner

---

## Overview

This demo shows how petalTongue visualizes system health states. Starting from a scenario with mixed health (healthy, warning, critical), it demonstrates how color coding and audio cues make problems immediately apparent.

**Goal**: Show that petalTongue makes degraded states obvious at a glance.

---

## Scenario Details

- **Primals**: 10 (multiple instances of each type)
- **Health Mix**:
  - 5 healthy (green)
  - 3 warning (yellow)
  - 2 critical (red)
- **Connections**: 9 edges
- **Layout**: Force-directed (nodes push apart, critical ones stand out)

---

## Key Features Demonstrated

1. **Health State Colors**
   - **Green**: Healthy, operating normally
   - **Yellow**: Warning, degraded but functional
   - **Red**: Critical, immediate attention required

2. **Audio Health Mapping**
   - Healthy → Harmonic, on-key tones
   - Warning → Off-key, unstable tones
   - Critical → Dissonant, harsh tones

3. **Real-Time Updates**
   - Auto-refresh (every 5s)
   - Manual refresh ("Refresh Now" button)
   - Health changes reflected immediately

---

## Presenter Script

### Opening (30 seconds)

> "In the last demo, everything was healthy - all green. Real systems aren't like that. Let me show you what petalTongue looks like when things break."

### Show Mixed Health (1 minute)

> "Now I'm looking at a more realistic scenario. 10 primals, and not all of them are happy."

> "See the color distribution:
- **Green nodes** are healthy - BearDog, Songbird, NestGate 1, 2, and 3
- **Yellow nodes** are warning - ToadStool 1, 2, and 3 are degraded
- **Red nodes** are critical - Squirrel 1 and 2 need immediate attention"

> "At a glance, I know:  
- 50% of my system is healthy
- 30% is degraded but functional
- 20% is critical"

### Health Details (1 minute)

> "If I click on a healthy node..." *(click BearDog)*

> "...it says 'healthy pitch (harmonic)'. A blind user would hear a smooth, on-key tone."

> "If I click on a warning node..." *(click ToadStool)*

> "...'warning pitch (off-key, unstable)'. A blind user would hear something's not quite right - it's out of tune."

> "And if I click on a critical node..." *(click Squirrel)*

> "...'critical pitch (dissonant, harsh)'. A blind user would immediately hear the alarm - it sounds BAD."

### Real-Time Monitoring (30 seconds)

> "This updates in real-time. The mock server simulates a live ecosystem. Every 5 seconds, petalTongue refreshes and reflects the current state."

> "In production, you'd point this at your real BiomeOS, and it would show your actual systems."

### Multi-Modal (30 seconds)

> "Notice that the **same health information** is encoded in:
- **Color** (visual)
- **Pitch** (audio)
- **Text descriptions** (screen readers)

All three modalities get the same data. Sighted users see colors. Blind users hear pitch. Screen reader users get text. Everyone is informed."

### Closing (30 seconds)

> "This is petalTongue in action - real-time health monitoring with multi-modal representation. Whether you're looking at a screen, listening to audio, or using a screen reader, you know exactly what's healthy and what's not."

---

## Setup Instructions

### Automated

```bash
cd showcase/
./scripts/run-demo.sh 02
```

### Manual

1. **Copy scenario**:
   ```bash
   cp sandbox/scenarios/unhealthy.json sandbox/scenarios/demo-active.json
   ```

2. **Configure mock server**:
   ```bash
   cd sandbox/mock-biomeos/
   # Edit src/main.rs line 56: use "demo-active.json"
   cargo build --release
   ```

3. **Restart mock server**:
   ```bash
   cd ../
   ./scripts/start-mock.sh
   ```

4. **Launch petalTongue**:
   ```bash
   cd ..
   BIOMEOS_URL=http://localhost:3333 cargo run --release -p petal-tongue-ui
   ```

5. **Wait for refresh** (5s) or click "Refresh Now"

---

## Variations

### Quick Version (2 min)
- Skip audio details
- Just show: colors, mixed health, click one critical node

### Extended Version (7 min)
- Show all three health states in detail
- Demonstrate auto-refresh by editing scenario file
- Show statistics panel (healthy vs warning vs critical count)

### Accessibility Focus
- Emphasize audio pitch mapping
- Compare healthy vs critical audio descriptions side-by-side
- Preview Demo 04 (full audio experience)

---

## Troubleshooting

### All nodes are green
- Wrong scenario loaded
- Check mock server is using `unhealthy.json`
- Click "Refresh Now"

### Colors don't match health status
- Restart petalTongue (may have cached old data)
- Check scenario file has correct "health" values

---

## Follow-Up

After Demo 02, transition to:
- **Demo 03** (Scaling) - "Let's add more complexity"
- **Demo 04** (Audio-Only) - "Now let me close my eyes..."
- **Demo 05** (Production Scale) - "Let's stress test with 50+ nodes"

---

**Status**: ✅ Scenario ready  
**Requirements**: Mock server, unhealthy.json  
**Tested**: Yes

