# 05 - Accessibility Validation

**Duration**: 20-30 minutes  
**Purpose**: Validate audio-only mode, screen reader compatibility, and universal accessibility

---

## 🎯 What This Demo Does

1. **Tests audio-only mode** (visual disabled or eyes closed)
2. **Validates AI narration** accuracy
3. **Checks screen reader** compatibility
4. **Tests keyboard-only** navigation
5. **Evaluates blind, deaf, neurodiverse** user experiences

**Goal**: Ensure petalTongue is truly accessible to ALL users, not just sighted users.

---

## 🚀 Quick Start

```bash
./demo.sh
```

This will guide you through comprehensive accessibility testing.

---

## 📋 Prerequisites

- Completed scenarios 00-04
- BiomeOS running
- petalTongue UI open
- Headphones recommended (for audio testing)
- Screen reader installed (for SR testing): `orca` on Linux, VoiceOver on Mac, NVDA/JAWS on Windows

---

## 🎬 Demo Flow

### Phase 1: Audio-Only Mode (Blind User Simulation)

**Objective**: Can a blind user understand the ecosystem using ONLY audio?

#### Step 1: Close Your Eyes

```bash
./launch-ecosystem-audio-only.sh
```

**Instructions**:
1. **Close your eyes** (or turn off monitor)
2. **Listen to the audio output**
3. **Try to answer**:
   - How many primals are running?
   - Which primal types are present?
   - What are their health states?
   - Are there any connections?

**Expected Audio**: AI narration describing the ecosystem soundscape

#### Step 2: Health Degradation (Audio Only)

```bash
./trigger-warning-audio.sh beardog
```

**Listen for**:
- Change in pitch (dissonance)
- Change in rhythm
- AI narration of warning
- Stereo positioning shift?

**Question**: Could you identify WHICH primal degraded without looking?

#### Step 3: AI Narration Test

petalTongue should provide AI-generated descriptions like:

```
"The ecosystem currently has 5 active primals:
- BearDog Security at center-left, healthy, connected to 2 neighbors
- NestGate Storage at bottom, healthy, hub with 3 connections
- Songbird Discovery at top, WARNING state, high activity
..."
```

**Validation**:
- Is narration accurate?
- Is it timely?
- Is it helpful?

### Phase 2: Screen Reader Compatibility

**Objective**: Can a screen reader user navigate petalTongue?

#### Step 1: Launch Screen Reader

**Linux**: `orca`  
**Mac**: VoiceOver (`Cmd+F5`)  
**Windows**: NVDA/JAWS

#### Step 2: Navigate UI with Screen Reader

**Test**:
- Tab through UI elements
- Activate buttons (Refresh, Reset Camera, Layout dropdown)
- Select nodes
- Read stats panel

**Questions**:
- Are all interactive elements announced?
- Can you operate petalTongue without a mouse?
- Is information hierarchy clear?

#### Step 3: ARIA Labels Validation

Check that:
- Nodes have `aria-label="BearDog Security, Healthy, 2 connections"`
- Buttons have descriptive labels
- Stats are in semantic HTML (not just visual)
- Graph is `role="img"` with `aria-describedby`

### Phase 3: Keyboard-Only Navigation

**Objective**: Can a user with motor disabilities navigate without a mouse?

#### Keyboard Map

| Key | Action |
|-----|--------|
| `Tab` | Next element |
| `Shift+Tab` | Previous element |
| `Enter` / `Space` | Activate button |
| `Arrow Keys` | Pan graph |
| `+` / `-` | Zoom in/out |
| `R` | Reset camera |
| `L` | Cycle layouts |
| `1-9` | Select node by index |

**Test**:
- Navigate entire UI using only keyboard
- Select a node
- Change layout
- Refresh graph
- Read node details

**Questions**:
- Is keyboard navigation smooth?
- Are there focus indicators?
- Is tab order logical?

### Phase 4: Colorblind Mode

**Objective**: Can users with color vision deficiency distinguish health states?

#### Color Blindness Types

**Deuteranopia** (red-green): 5% of males  
**Protanopia** (red-green): 1% of males  
**Tritanopia** (blue-yellow): <1%  
**Achromatopsia** (no color): <0.1%

#### Test

**Question**: If you couldn't see color, could you still identify health states?

**Current**: Green/Yellow/Red nodes  
**Problem**: Indistinguishable for some users

**Solutions to validate**:
- Shape differences (circle, triangle, square)
- Pattern fills (solid, striped, dotted)
- Size differences (small = healthy, large = critical)
- Text labels always visible

### Phase 5: Cognitive Accessibility

**Objective**: Is petalTongue usable for neurodiverse users?

#### Considerations

**ADHD**: Overwhelming visual complexity?  
**Dyslexia**: Text readable?  
**Autism**: Sensory overload from audio?  
**Anxiety**: Stressful red colors?

#### Test

- Can you simplify the UI?
- Can you disable audio?
- Can you slow down animations?
- Are there too many simultaneous updates?

---

## ✅ Success Criteria

After this demo, you should have validated:

- [x] Audio-only mode provides complete information
- [x] AI narration is accurate and timely
- [x] Screen reader can navigate entire UI
- [x] Keyboard-only navigation is possible
- [x] Colorblind users can distinguish health states
- [x] Cognitive load is manageable

---

## 🌱 Fermentation Notes

### Gaps to Watch For

- **Audio-Only**:
  - Can you truly "see" the ecosystem?
  - Is sonification informative or gimmicky?
  - Does AI narration provide enough context?

- **Screen Reader**:
  - Are all elements accessible?
  - Is tab order logical?
  - Are dynamic updates announced?

- **Keyboard**:
  - Are all actions possible?
  - Are focus indicators visible?
  - Are shortcuts discoverable?

- **Color Vision**:
  - Health states distinguishable without color?
  - Shape/pattern/size as alternatives?
  - Is color redundant?

- **Cognitive**:
  - Is UI overwhelming?
  - Can you reduce stimulation?
  - Is information prioritized?

**Document ALL gaps in**: `../GAPS.md`

---

## 💡 Accessibility Best Practices

### WCAG 2.1 AAA Compliance

petalTongue aims for **Level AAA** (highest accessibility standard):

**Perceivable**:
- Text alternatives for all non-text content
- Captions and audio descriptions
- Content adaptable to different formats

**Operable**:
- All functionality via keyboard
- Enough time to read/use content
- No seizure-inducing flashing

**Understandable**:
- Readable and predictable
- Input assistance for errors
- Clear navigation

**Robust**:
- Compatible with assistive technologies
- Works on all devices/browsers

### Universal Design

**Goal**: ONE interface that works for EVERYONE

**Not**: "Regular UI" + "Accessible UI"  
**But**: Single UI that's natively accessible

**Example**:
- Nodes have color AND shape
- Stats have visual AND audio
- Navigation has mouse AND keyboard
- Information has text AND speech

---

## 🎓 Learning Points

### Accessibility = Better UX for Everyone

**Curb cuts**: Designed for wheelchairs, used by:
- Parents with strollers
- Delivery workers
- Cyclists
- Travelers with luggage

**Captions**: Designed for deaf, used by:
- Non-native speakers
- Noisy environments
- Quiet environments
- Better comprehension

**petalTongue Audio**: Designed for blind, useful for:
- Eyes-busy tasks (driving?)
- Multi-monitoring (listening while coding)
- Ambient awareness
- Pattern detection (dissonance = problem)

### Accessibility as Innovation Driver

Constraints force creativity:

**Constraint**: Must work audio-only  
**Innovation**: Sonification engine (map primals to instruments)

**Constraint**: Must work without color  
**Innovation**: Shape-based health indicators

**Constraint**: Must work keyboard-only  
**Innovation**: Efficient shortcuts, logical navigation

### Legal & Ethical Obligations

**Legal**: ADA (US), Section 508, WCAG 2.1  
**Ethical**: Digital sovereignty includes accessibility  
**Practical**: 15% of population has disabilities

**Not optional. Not "nice to have". Required.**

---

## ⏭️ Next Steps

Once accessibility is validated, proceed to:

```bash
cd ../06-performance-benchmarking/
cat README.md
```

This will test **performance at scale** (100s of nodes).

---

**Status**: 🌱 Ready to build  
**Complexity**: High (requires assistive tech testing)  
**Dependencies**: 00-04 complete  
**Learning Value**: Very High (ethical imperative)

---

*True universality means designing for the margins first.  
When you design for the blind, everyone benefits.* 🌸

