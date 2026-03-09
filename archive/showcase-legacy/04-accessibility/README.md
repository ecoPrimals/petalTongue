# 🌸 Accessibility Showcase - Multi-Modal Interface Demonstrations

**Status**: Live Demonstrations with Real Data  
**Principles**: Any Input → Any Output | No Mocks | Universal Access

---

## 🎯 Core Concept

**petalTongue is designed for EVERY human, regardless of ability:**

- **Blind users**: Audio sonification of topology + narrative descriptions
- **Deaf users**: Visual graph + text descriptions
- **Non-verbal**: Audio input (singing), visual input (drawing)
- **Illiterate**: Audio-only interface
- **Motor disabilities**: Single-switch scanning, voice control
- **Multiple disabilities**: Combine modalities as needed

**Every demonstration uses LIVE data** - no mocks, no fake streams.

---

## 📊 Live Data Sources (All Real)

### 1. System Resources (sysinfo)
- Real-time CPU usage
- Memory consumption
- Disk I/O
- Process list

### 2. Network Topology (mDNS/biomeOS)
- Live primal discovery
- Real connection status
- Actual health metrics

### 3. Human Input (Real Sensors)
- Microphone (audio entropy)
- Keyboard (keystroke dynamics)
- Mouse/touchpad (gesture tracking)
- Camera (optional, video entropy)

### 4. Process Metrics
- Running processes
- Resource usage per process
- Network connections

---

## 🎨 Demonstration Scenarios

### Scenario 1: Blind User - Audio-Only Interface

**User Profile**: Totally blind software engineer

**Input Methods**:
- Voice commands (future)
- Keyboard shortcuts (now)
- Audio entropy capture ("Sing a Song")

**Output Methods**:
- Audio sonification of network topology
- Screen reader compatible text
- Narrative audio descriptions
- Audio feedback for interactions

**Live Demo**:
```bash
cd showcase/04-accessibility/01-blind-user
./demo.sh
```

**What Happens** (All Live Data):
1. Launch petalTongue with audio-only mode
2. Sonify REAL system CPU/memory as audio tones
3. Discover primals via mDNS → audio announcement
4. Graph edges → audio patterns (pitch = distance, volume = bandwidth)
5. User sings a song → real microphone capture
6. Quality metrics → audio feedback (high pitch = good quality)
7. Stream to BearDog → success tone

**Key Features**:
- ✅ Zero visual dependency
- ✅ All information conveyed through sound
- ✅ Real microphone, real system data, real discovery
- ✅ Fully functional workflow

---

### Scenario 2: Deaf User - Visual-Only Interface

**User Profile**: Deaf artist who communicates via drawing

**Input Methods**:
- Drawing/painting (visual entropy)
- Keyboard/text
- Mouse/touchpad

**Output Methods**:
- High-contrast visual graph
- Text descriptions (no audio)
- Visual alerts (color flashes, animations)
- Waveform visualization

**Live Demo**:
```bash
cd showcase/04-accessibility/02-deaf-user
./demo.sh
```

**What Happens** (All Live Data):
1. Launch petalTongue visual-only mode
2. Real-time graph of discovered primals (mDNS)
3. CPU/memory metrics visualized as node colors
4. User draws a picture → canvas stroke capture
5. Stroke timing/pressure analyzed → quality display
6. Network activity → animated particles
7. Alerts via screen flashing (not audio)

**Key Features**:
- ✅ Zero audio dependency
- ✅ All information conveyed visually
- ✅ Real drawing capture, real metrics
- ✅ Fully accessible to deaf users

---

### Scenario 3: Non-Verbal User - Alternative Input

**User Profile**: Non-verbal user with cerebral palsy

**Input Methods**:
- Eye tracking (via accessibility tools)
- Single-switch scanning
- Head movements
- Pre-recorded audio (humming, sounds)

**Output Methods**:
- Large visual targets
- Audio confirmation
- Simplified interface
- Clear feedback

**Live Demo**:
```bash
cd showcase/04-accessibility/03-nonverbal-user
./demo.sh
```

**What Happens** (All Live Data):
1. Launch with simplified large-button UI
2. User hums a tune → real microphone captures
3. System accepts ANY audio (not just speech)
4. Quality assessed from timing/pitch variation
5. Visual feedback shows entropy quality
6. Audio tone confirms submission
7. Real topology visualized with large nodes

**Key Features**:
- ✅ No speech required
- ✅ Accepts ANY audio input
- ✅ Real audio processing
- ✅ Large visual targets

---

### Scenario 4: Illiterate User - Audio + Visual

**User Profile**: Illiterate farmer using ecosystem for first time

**Input Methods**:
- Voice (singing, speaking in any language)
- Simple button presses
- Touch/gestures

**Output Methods**:
- Audio narration (in user's language)
- Icons (not text)
- Visual symbols
- Audio feedback

**Live Demo**:
```bash
cd showcase/04-accessibility/04-illiterate-user
./demo.sh
```

**What Happens** (All Live Data):
1. Launch with icon-based UI (no text)
2. Audio greeting in user's language
3. User sings traditional song → captured
4. Quality shown as colored indicator (green/yellow/red)
5. System topology shown as farm diagram (barn = server, etc.)
6. Real metrics mapped to familiar concepts
7. Audio narrates discoveries

**Key Features**:
- ✅ Zero text dependency
- ✅ Cultural adaptation (farm metaphors)
- ✅ Real audio, real metrics
- ✅ Intuitive interface

---

### Scenario 5: Motor Disability - Single Switch

**User Profile**: User with ALS, single-switch access only

**Input Methods**:
- Single switch (space bar, button, breath sensor)
- Scanning interface
- Timed selections

**Output Methods**:
- Visual highlighting of scan position
- Audio confirmation of selections
- Simplified choices
- Undo-friendly

**Live Demo**:
```bash
cd showcase/04-accessibility/05-motor-disability
./demo.sh
```

**What Happens** (All Live Data):
1. Launch scanning interface
2. Options highlight sequentially (live scan)
3. User hits switch at right time
4. "Sing a Song" option → records 5 seconds
5. User's breathing/humming captured → real audio
6. System accepts any sound as entropy
7. Visual progress bar, audio feedback
8. Topology shown with auto-cycling info

**Key Features**:
- ✅ Single input method
- ✅ Time-based selection
- ✅ Forgiving (undo available)
- ✅ Real audio capture

---

### Scenario 6: Deaf-Blind User - Tactile + Text

**User Profile**: Deaf-blind user with braille display

**Input Methods**:
- Braille keyboard
- Tactile feedback devices
- Keyboard shortcuts

**Output Methods**:
- Braille display output
- Tactile vibration patterns
- Screen reader (text-to-braille)

**Live Demo**:
```bash
cd showcase/04-accessibility/06-deaf-blind-user
./demo.sh
```

**What Happens** (All Live Data):
1. Launch text-only mode
2. Status updates → braille display
3. User types narrative (story/poem)
4. Keystroke dynamics captured → real timing
5. Quality metrics → braille output ("GOOD", "FAIR")
6. Primal discovery → text descriptions
7. Topology → text tree structure

**Key Features**:
- ✅ No visual dependency
- ✅ No audio dependency
- ✅ Pure text/braille interface
- ✅ Full functionality

---

## 🔄 Combo Modalities

### Any Input → Any Output

**Key Principle**: Input and output are INDEPENDENT

| User Type | Input | Output | Demo |
|-----------|-------|--------|------|
| Blind | Keyboard | Audio | `01-blind-user/` |
| Deaf | Voice | Visual | `02-deaf-user/` |
| Deaf-Blind | Keyboard | Braille | `06-deaf-blind/` |
| Motor | Switch | Visual+Audio | `05-motor-disability/` |
| Low Vision | Voice | Large Visual | `07-low-vision/` |
| Cognitive | Icons | Simplified Audio | `08-cognitive/` |

---

## 📈 Live System Metrics (All Real)

### CPU Sonification
```bash
cd showcase/04-accessibility/demos/cpu-sonification
./demo.sh
```

**What Happens**:
- Real CPU usage from `sysinfo`
- High CPU → high pitch tone
- Low CPU → low pitch tone
- Multiple cores → chord (harmony)
- Updates every 100ms (live)

### Memory Visualization
```bash
cd showcase/04-accessibility/demos/memory-visualization
./demo.sh
```

**What Happens**:
- Real memory usage from `sysinfo`
- Fill bar animates with actual RAM usage
- Color changes: green (low), yellow (medium), red (high)
- Audio tone pitch varies with memory pressure
- Both visual AND audio (accessible to all)

### Network Discovery Audio
```bash
cd showcase/04-accessibility/demos/network-audio
./demo.sh
```

**What Happens**:
- Real mDNS discovery every 5 seconds
- New primal discovered → "ding" sound + narration
- Primal disappeared → "bong" sound + narration
- Active connections → continuous tone
- No primals → "searching..." audio

### Process List with Audio Navigation
```bash
cd showcase/04-accessibility/demos/process-audio-nav
./demo.sh
```

**What Happens**:
- Real process list from `sysinfo`
- Arrow keys navigate (live processes)
- Selected process → audio narration
  - "Chrome, 45% CPU, 2.3GB memory"
- Enter key → detailed narration
- Real data, updates every second

---

## 🎵 Audio Entropy Demos (Real Microphone)

### Demo 1: Sing a Song (Any Song)
```bash
cd showcase/04-accessibility/demos/audio-entropy-basic
./demo.sh
```

**Live Features**:
- Real microphone via `cpal`
- Records user singing ANY song
- FFT analysis → spectral entropy (real-time)
- Quality display: visual + audio feedback
- AES-256-GCM encryption
- Stream to biomeOS/BearDog

### Demo 2: Humming/Breathing (Non-Verbal)
```bash
cd showcase/04-accessibility/demos/audio-entropy-nonverbal
./demo.sh
```

**Live Features**:
- Accepts ANY sound (not just singing)
- Humming, breathing, whistling all valid
- Real-time quality assessment
- Encouragement audio if quality low
- Success audio when quality good

### Demo 3: Environmental Sounds (No Voice Needed)
```bash
cd showcase/04-accessibility/demos/audio-entropy-environmental
./demo.sh
```

**Live Features**:
- Tap rhythms on desk
- Shake keys
- Rustle paper
- ANY audio pattern works
- Real entropy from timing variation

---

## 🎨 Visual Entropy Demos (Real Drawing)

### Demo 1: Mouse/Touchpad Drawing
```bash
cd showcase/04-accessibility/demos/visual-entropy-mouse
./demo.sh
```

**Live Features**:
- Canvas for mouse drawing
- Stroke timing captured (real ms precision)
- Pressure simulated from speed
- Quality from stroke variation
- Visual feedback: color changes

### Demo 2: Keyboard "Art" (Text Patterns)
```bash
cd showcase/04-accessibility/demos/visual-entropy-keyboard
./demo.sh
```

**Live Features**:
- User types text/symbols to create pattern
- Keystroke dynamics captured
- Character variation → entropy
- Timing between keys → additional entropy
- Accessible to motor disabilities

---

## 📝 Narrative Entropy Demos (Real Typing)

### Demo 1: Tell a Story
```bash
cd showcase/04-accessibility/demos/narrative-entropy-story
./demo.sh
```

**Live Features**:
- Text box for story
- Real keystroke timing captured
- Backspace patterns tracked
- Pauses (thinking time) analyzed
- Quality from natural variation

### Demo 2: Describe Your Day
```bash
cd showcase/04-accessibility/demos/narrative-entropy-journal
./demo.sh
```

**Live Features**:
- Journal prompt
- No minimum length
- ANY text accepted
- Timing makes it unique
- Real keystroke dynamics

---

## 🚀 Run Complete Accessibility Suite

### Quick Start (All Scenarios)
```bash
cd showcase/04-accessibility
./RUN_ALL_SCENARIOS.sh
```

**What It Does**:
1. Runs each scenario sequentially
2. Demonstrates every input/output combo
3. Uses only LIVE data (CPU, memory, mDNS, microphone)
4. Shows accessibility features
5. Generates report with metrics

### Individual Scenario
```bash
cd showcase/04-accessibility/01-blind-user
./demo.sh
```

---

## 📊 Technical Architecture (All Live)

### Live Data Flow
```
Real System Data → petalTongue → Multi-Modal Output
      ↓                             ↓
  - CPU (sysinfo)            - Visual Graph
  - Memory (sysinfo)         - Audio Tones
  - Processes (sysinfo)      - Braille Text
  - Network (mDNS)           - Haptic Feedback
  - Microphone (cpal)        - Narrator Voice
```

### Zero Mocks Guarantee
- ✅ All system metrics: `sysinfo` crate (real data)
- ✅ All network discovery: mDNS (real packets)
- ✅ All audio capture: `cpal` (real microphone)
- ✅ All process info: `/proc` via `sysinfo`
- ✅ All entropy quality: real FFT analysis

---

## 🎯 Key Achievements

**Universal Access**:
- ✅ Blind users can use 100% of features
- ✅ Deaf users can use 100% of features
- ✅ Deaf-blind users can use 100% of features
- ✅ Motor disabilities accommodated
- ✅ Cognitive disabilities simplified
- ✅ Illiterate users supported

**Live Data**:
- ✅ Zero mocks in showcase
- ✅ Real system metrics
- ✅ Real network discovery
- ✅ Real audio capture
- ✅ Real entropy analysis

**Production Quality**:
- ✅ AES-256-GCM encryption
- ✅ Secure zeroization
- ✅ Privacy-first (stream-only)
- ✅ No data persistence

---

🌸 **petalTongue: Accessible to EVERYONE, powered by REAL data** 🌸

See individual demo READMEs for detailed instructions!

