# 🌸 PetalTongue Human Entropy Capture Specification

**Version**: 1.0.0  
**Date**: January 3, 2026  
**Status**: SPECIFICATION - Ready for Implementation  
**Responsibility**: petalTongue (100% - Input Modalities)

---

## 🎯 Overview

PetalTongue is responsible for **capturing rich, multi-modal human entropy** through intuitive, accessible interfaces. This entropy is streamed to BearDog (via biomeOS) for cryptographic mixing into non-fungible, sovereign keys.

### Scope

**In Scope (petalTongue)**:
- ✅ User interface design and implementation
- ✅ Multi-modal input capture (audio, visual, narrative, gesture, video)
- ✅ Real-time quality assessment and feedback
- ✅ User guidance and error handling
- ✅ Accessibility features (screen readers, multi-modal alternatives)
- ✅ Streaming API client (encrypted transmission)

**Out of Scope (Other Systems)**:
- ❌ Cryptographic mixing (BearDog's responsibility)
- ❌ Entropy persistence (Never happens - stream-only)
- ❌ Trust evaluation (BearDog's responsibility)
- ❌ Proxying/routing (biomeOS's responsibility)

---

## 🎨 Input Modalities

### Modality 1: Audio Entropy (Singing/Speaking) 🎵

**Capture Method**: Microphone input (waveform)

**Entropy Sources**:
- Timing between words/notes
- Pitch variations
- Volume dynamics
- Breath patterns
- Natural pauses

**Quality Metrics**:
- `timing_entropy`: Natural rhythm (0.0-1.0)
- `pitch_variance`: Human vocal range (0.0-1.0)
- `amplitude_dynamics`: Volume variations (0.0-1.0)
- `overall_quality`: Weighted average (0.0-1.0)

**UI Requirements**:
- Real-time waveform visualization
- Quality meter (live feedback)
- Duration indicator (30-60s recommended)
- Recording controls (Start, Stop, Retry)
- Accessibility: Screen reader announces quality

**Data Structure**:
```rust
pub struct AudioEntropyCapture {
    pub waveform: Vec<f32>,
    pub sample_rate: u32,
    pub duration_seconds: f32,
    pub quality_metrics: AudioQualityMetrics,
}

pub struct AudioQualityMetrics {
    pub timing_entropy: f64,
    pub pitch_variance: f64,
    pub amplitude_dynamics: f64,
    pub overall_quality: f64,
}
```

---

### Modality 2: Visual Entropy (Drawing/Painting) 🎨

**Capture Method**: Canvas with drawing tools

**Entropy Sources**:
- Stroke patterns (path, pressure)
- Timing between strokes
- Spatial coverage
- Color choices
- Movement dynamics

**Quality Metrics**:
- `movement_entropy`: Natural human motion (0.0-1.0)
- `spatial_entropy`: Coverage & variance (0.0-1.0)
- `timing_entropy`: Natural rhythm (0.0-1.0)
- `overall_quality`: Weighted average (0.0-1.0)

**UI Requirements**:
- Drawing canvas (800x600 or larger)
- Tools: Brush, eraser, color picker
- Stroke count indicator
- Coverage meter (percentage)
- Quality feedback (real-time)
- Accessibility: Audio description of quality

**Data Structure**:
```rust
pub struct VisualEntropyCapture {
    pub strokes: Vec<Stroke>,
    pub canvas_size: (u32, u32),
    pub total_coverage: f64,
    pub quality_metrics: VisualQualityMetrics,
}

pub struct Stroke {
    pub points: Vec<Point2D>,
    pub timestamps: Vec<Duration>,
    pub pressure: Vec<f32>,
    pub color: Color,
}

pub struct VisualQualityMetrics {
    pub movement_entropy: f64,
    pub spatial_entropy: f64,
    pub timing_entropy: f64,
    pub overall_quality: f64,
}
```

---

### Modality 3: Narrative Entropy (Storytelling) 📝

**Capture Method**: Text editor with keystroke dynamics

**Entropy Sources**:
- Keystroke timing (inter-key intervals)
- Typing rhythm (natural vs. copied)
- Backspace patterns (corrections)
- Pause durations (thinking time)
- Text content (story uniqueness)

**Quality Metrics**:
- `keystroke_entropy`: Typing rhythm variance (0.0-1.0)
- `pause_entropy`: Natural thinking patterns (0.0-1.0)
- `correction_entropy`: Human mistakes/edits (0.0-1.0)
- `content_entropy`: Story uniqueness (0.0-1.0)
- `overall_quality`: Weighted average (0.0-1.0)

**UI Requirements**:
- Text editor (multi-line)
- Word count indicator (100-200 recommended)
- Quality meter (live)
- Prompt/suggestions (optional)
- Accessibility: Full screen reader support

**Data Structure**:
```rust
pub struct NarrativeEntropyCapture {
    pub text: String,
    pub keystroke_timings: Vec<Duration>,
    pub backspace_events: Vec<BackspaceEvent>,
    pub pause_durations: Vec<Duration>,
    pub quality_metrics: NarrativeQualityMetrics,
}

pub struct BackspaceEvent {
    pub timestamp: Duration,
    pub position: usize,
    pub deleted_char: char,
}

pub struct NarrativeQualityMetrics {
    pub keystroke_entropy: f64,
    pub pause_entropy: f64,
    pub correction_entropy: f64,
    pub content_entropy: f64,
    pub overall_quality: f64,
}
```

---

### Modality 4: Gesture Entropy (Motion/Touch) 🤲

**Capture Method**: Sensor data (accelerometer, gyroscope, touch)

**Entropy Sources**:
- Device movement (shake, rotate, tilt)
- Touch patterns (pressure, duration, path)
- Gesture timing (natural rhythm)
- Sensor diversity (multiple sources)

**Quality Metrics**:
- `motion_entropy`: Movement variance (0.0-1.0)
- `pattern_uniqueness`: Your gesture signature (0.0-1.0)
- `timing_entropy`: Natural rhythm (0.0-1.0)
- `sensor_diversity`: Multi-source bonus (0.0-1.0)
- `overall_quality`: Weighted average (0.0-1.0)

**UI Requirements**:
- Gesture guide (visual/audio instructions)
- Sensor feedback (real-time)
- Duration indicator (15-30s recommended)
- Quality meter (live)
- Accessibility: Audio guidance

**Data Structure**:
```rust
pub struct GestureEntropyCapture {
    pub accelerometer: Vec<Vec3>,
    pub gyroscope: Vec<Vec3>,
    pub touch_events: Vec<TouchEvent>,
    pub timestamps: Vec<Duration>,
    pub quality_metrics: GestureQualityMetrics,
}

pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct TouchEvent {
    pub position: Point2D,
    pub pressure: f32,
    pub timestamp: Duration,
}

pub struct GestureQualityMetrics {
    pub motion_entropy: f64,
    pub pattern_uniqueness: f64,
    pub timing_entropy: f64,
    pub sensor_diversity: f64,
    pub overall_quality: f64,
}
```

---

### Modality 5: Video Entropy (Motion Patterns) 📹

**Capture Method**: Camera input (motion analysis)

**Entropy Sources**:
- Frame-to-frame motion vectors
- Lighting variations
- Scene complexity
- Movement patterns
- Temporal dynamics

**Quality Metrics**:
- `motion_entropy`: Movement variance (0.0-1.0)
- `scene_entropy`: Visual complexity (0.0-1.0)
- `temporal_entropy`: Time-based patterns (0.0-1.0)
- `overall_quality`: Weighted average (0.0-1.0)

**UI Requirements**:
- Camera preview (live)
- Motion indicator (visual feedback)
- Duration indicator (10-30s recommended)
- Quality meter (live)
- Privacy notice (motion only, not video!)
- Accessibility: Audio motion feedback

**Data Structure**:
```rust
pub struct VideoEntropyCapture {
    pub motion_vectors: Vec<MotionField>,
    pub lighting_samples: Vec<f32>,
    pub scene_complexity: Vec<f32>,
    pub timestamps: Vec<Duration>,
    pub quality_metrics: VideoQualityMetrics,
}

pub struct MotionField {
    pub vectors: Vec<Vec2>,
    pub magnitude: f32,
    pub direction: f32,
}

pub struct VideoQualityMetrics {
    pub motion_entropy: f64,
    pub scene_entropy: f64,
    pub temporal_entropy: f64,
    pub overall_quality: f64,
}
```

---

## 🏗️ Architecture

### Module Structure

```
crates/petal-tongue-entropy/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Main module
│   ├── types.rs                    # Shared types
│   ├── quality.rs                  # Quality assessment
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── capture.rs              # Microphone capture
│   │   ├── analysis.rs             # Timing, pitch analysis
│   │   └── ui.rs                   # Audio UI components
│   ├── visual/
│   │   ├── mod.rs
│   │   ├── canvas.rs               # Drawing canvas
│   │   ├── strokes.rs              # Stroke capture
│   │   └── ui.rs                   # Visual UI components
│   ├── narrative/
│   │   ├── mod.rs
│   │   ├── editor.rs               # Text editor
│   │   ├── keystroke.rs            # Keystroke dynamics
│   │   └── ui.rs                   # Narrative UI components
│   ├── gesture/
│   │   ├── mod.rs
│   │   ├── sensors.rs              # Sensor capture
│   │   ├── patterns.rs             # Pattern recognition
│   │   └── ui.rs                   # Gesture UI components
│   ├── video/
│   │   ├── mod.rs
│   │   ├── camera.rs               # Camera input
│   │   ├── motion.rs               # Motion analysis
│   │   └── ui.rs                   # Video UI components
│   └── stream/
│       ├── mod.rs
│       ├── client.rs               # Streaming API client
│       └── encryption.rs           # Encryption layer
```

### Data Flow

```
1. User Interaction
   ↓
2. Modality-Specific Capture
   ↓
3. Real-Time Quality Assessment
   ↓
4. User Feedback (Live)
   ↓
5. User Confirms "Use This"
   ↓
6. Encrypt Entropy Data
   ↓
7. Stream to biomeOS
   ↓
8. (biomeOS proxies to BearDog)
   ↓
9. (BearDog mixes & creates ephemeral seed)
   ↓
10. Success Confirmation (to User)
```

---

## 🔒 Security & Privacy

### Stream-Only Architecture

**Never Stored**:
- Raw audio/visual/narrative/gesture/video data
- Intermediate processing results
- Quality assessment data

**Encrypted Transmission**:
```rust
pub async fn stream_entropy(
    entropy: EntropyCapture,
    endpoint: &str,
) -> Result<StreamConfirmation> {
    // 1. Serialize entropy data
    let data = serialize_entropy(&entropy)?;
    
    // 2. Encrypt (TLS + application-level)
    let encrypted = encrypt_for_beardog(&data)?;
    
    // 3. Stream (never buffered to disk)
    let confirmation = stream_encrypted(&encrypted, endpoint).await?;
    
    // 4. Zeroize local data
    zeroize(&data);
    zeroize(&encrypted);
    
    Ok(confirmation)
}
```

**Zeroization**:
- All entropy data zeroized after streaming
- Memory cleared securely
- No persistence (RAM or disk)

### Quality Thresholds

**Minimum Acceptable Quality**: 0.3
- Below threshold: User shown error, asked to retry
- Above threshold: User can proceed

**Recommended Quality**: 0.6+
- Good quality: Green indicator
- Medium quality: Yellow indicator (suggest improvements)
- Low quality: Red indicator (require retry)

**BearDog Enforcement**:
- BearDog performs additional validation
- May reject if quality insufficient
- petalTongue shows rejection reason

---

## 🎨 User Experience Guidelines

### Real-Time Feedback

**Quality Meter**:
- Visual: Progress bar with color coding (red/yellow/green)
- Audio: Tone frequency (low = poor, high = good)
- Text: Percentage + qualitative label ("Excellent!", "Good", "Try again")

**Guidance**:
- Tips shown during capture
- Suggestions for improvement
- Encouragement (positive reinforcement)

**Error Handling**:
- Clear error messages
- Specific improvement suggestions
- Multiple retry attempts allowed
- Option to try different modality

### Accessibility

**Visual Impairment**:
- Full screen reader support
- Audio quality feedback (tones, spoken percentages)
- Audio modality preferred (singing/speaking)
- Keyboard navigation (no mouse required)

**Hearing Impairment**:
- Visual modality preferred (drawing)
- Visual quality feedback (meters, colors)
- Vibration feedback (mobile devices)
- Text-based instructions

**Motor Impairment**:
- Narrative modality (keyboard only)
- Gesture modality (simple motions)
- Large touch targets
- Extended time limits

---

## 📊 Quality Assessment Algorithms

### Audio Entropy

**Timing Entropy**:
```rust
fn calculate_timing_entropy(intervals: &[Duration]) -> f64 {
    // Shannon entropy of inter-event intervals
    let histogram = create_histogram(intervals, 10);
    shannon_entropy(&histogram)
}
```

**Pitch Variance**:
```rust
fn calculate_pitch_variance(waveform: &[f32], sample_rate: u32) -> f64 {
    // FFT + peak detection + variance
    let frequencies = fft(waveform, sample_rate);
    let peaks = detect_peaks(&frequencies);
    variance(&peaks)
}
```

### Visual Entropy

**Spatial Entropy**:
```rust
fn calculate_spatial_entropy(strokes: &[Stroke], canvas_size: (u32, u32)) -> f64 {
    // Grid-based coverage + variance
    let grid = rasterize_strokes(strokes, canvas_size, 10, 10);
    let coverage_variance = variance(&grid);
    coverage_variance
}
```

**Movement Entropy**:
```rust
fn calculate_movement_entropy(strokes: &[Stroke]) -> f64 {
    // Direction changes + speed variance
    let directions = calculate_stroke_directions(strokes);
    let speeds = calculate_stroke_speeds(strokes);
    (variance(&directions) + variance(&speeds)) / 2.0
}
```

### Narrative Entropy

**Keystroke Entropy**:
```rust
fn calculate_keystroke_entropy(timings: &[Duration]) -> f64 {
    // Inter-keystroke interval (IKI) variance
    let intervals = calculate_intervals(timings);
    shannon_entropy(&intervals)
}
```

**Content Entropy**:
```rust
fn calculate_content_entropy(text: &str) -> f64 {
    // Character n-gram entropy
    let bigrams = extract_bigrams(text);
    shannon_entropy(&bigrams)
}
```

---

## 🚀 Implementation Phases

### Phase 1: Foundation (Week 1)

**Tasks**:
- [ ] Create `petal-tongue-entropy` crate
- [ ] Define core types (EntropyCapture, QualityMetrics)
- [ ] Implement quality assessment framework
- [ ] Create streaming API client
- [ ] Unit tests for quality algorithms

**Deliverable**: Core infrastructure

---

### Phase 2: Audio Modality (Week 2)

**Tasks**:
- [ ] Implement audio capture (microphone)
- [ ] Audio analysis (timing, pitch, amplitude)
- [ ] Audio UI (waveform, controls, quality meter)
- [ ] Real-time quality feedback
- [ ] Integration tests

**Deliverable**: "Sing a Song" feature

---

### Phase 3: Visual Modality (Week 3)

**Tasks**:
- [ ] Implement drawing canvas
- [ ] Stroke capture & analysis
- [ ] Visual UI (tools, quality meter)
- [ ] Real-time quality feedback
- [ ] Integration tests

**Deliverable**: "Paint a Picture" feature

---

### Phase 4: Narrative Modality (Week 4)

**Tasks**:
- [ ] Implement text editor
- [ ] Keystroke dynamics capture
- [ ] Narrative analysis
- [ ] Narrative UI (editor, quality meter)
- [ ] Integration tests

**Deliverable**: "Tell a Story" feature

---

### Phase 5: Gesture Modality (Week 5)

**Tasks**:
- [ ] Implement sensor capture
- [ ] Gesture pattern analysis
- [ ] Gesture UI (guide, feedback)
- [ ] Real-time quality feedback
- [ ] Integration tests

**Deliverable**: "Gesture Input" feature

---

### Phase 6: Video Modality (Week 6)

**Tasks**:
- [ ] Implement camera capture
- [ ] Motion vector analysis
- [ ] Video UI (preview, privacy notice)
- [ ] Real-time quality feedback
- [ ] Integration tests

**Deliverable**: "Take a Video" feature

---

### Phase 7: Integration & Polish (Week 7-8)

**Tasks**:
- [ ] Multi-modal UI integration
- [ ] End-to-end testing with BearDog
- [ ] Performance optimization
- [ ] Accessibility audit
- [ ] User testing
- [ ] Documentation

**Deliverable**: Production-ready human entropy capture

---

## 📋 Dependencies

### Rust Crates

**Audio**:
- `cpal`: Cross-platform audio I/O
- `hound`: WAV encoding/decoding
- `rustfft`: FFT for frequency analysis

**Visual**:
- `egui`: Drawing canvas (already in use)
- `image`: Image processing (if needed)

**Narrative**:
- `egui`: Text editor (already in use)
- (Minimal dependencies)

**Gesture**:
- Platform-specific sensor APIs
- `serde`: Serialization

**Video**:
- `nokhwa`: Cross-platform camera access
- `opencv` (optional): Motion analysis

**Streaming**:
- `reqwest`: HTTP client (already in use)
- `ring` or `rustls`: Encryption

**Quality**:
- `statrs`: Statistical functions
- (Most algorithms custom-implemented)

---

## 🎯 Success Criteria

### Functional

- [ ] All 5 modalities implemented
- [ ] Real-time quality feedback working
- [ ] Stream-only architecture (zero persistence)
- [ ] Encrypted transmission
- [ ] BearDog integration successful

### Non-Functional

- [ ] Response time: < 100ms for quality updates
- [ ] Quality assessment accuracy: > 90%
- [ ] Accessibility: WCAG 2.1 AA compliance
- [ ] Security: Zeroization verified
- [ ] Performance: 60 FPS UI (no lag)

### User Experience

- [ ] User testing: 8+/10 satisfaction
- [ ] Clarity: Users understand process
- [ ] Confidence: Users trust the system
- [ ] Accessibility: All modalities usable by all users

---

## 📚 References

### External

- BearDog: `ENTROPY_HIERARCHY_PRINCIPLE.md`
- BearDog: `docs/genetics/ENTROPY_HIERARCHY_GUIDE.md`
- BearDog: `crates/beardog-tunnel/src/tunnel/hsm/human_entropy_unified.rs`

### Internal

- petalTongue: `HUMAN_ENTROPY_INTEGRATION_VISION_JAN_3_2026.md`
- petalTongue: `archive/specs-archive/PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md` (archived; see `specs/GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md`)

---

**Status**: ✅ **SPECIFICATION COMPLETE - READY FOR IMPLEMENTATION**

🌸 **petalTongue: Human-centered entropy capture through beautiful, accessible interfaces!** 🌸

