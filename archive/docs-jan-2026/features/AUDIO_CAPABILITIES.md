# Audio Capabilities - petalTongue

## Overview

petalTongue has **dual audio capabilities**:

1. **Self-Sufficient**: Can generate all sounds programmatically
2. **User-Extensible**: Can load and play user-provided audio files

As you said: "people can bring their own songs sounds and libraries of colors, and petalTongue can adapt, but it also can do it all by itself as needed. albeit maybe not a symphony"

**Exactly.** We're not trying to be a DAW or symphony orchestra - we're providing **functional, generated audio for data sonification**.

---

## 1. Programmatic Sound Generation (Current)

### What We Have

Located in `crates/petal-tongue-graph/src/audio_playback.rs`:

```rust
/// Generate sounds programmatically from data
pub struct AudioPlaybackEngine {
    // Can generate 5 types of waveforms
    // Maps data → audio attributes → actual sound
}
```

### Waveforms We Can Generate

| Waveform | Characteristics | Use Case |
|----------|----------------|----------|
| **Sine** | Pure, smooth tones | Bass, melody, fundamental |
| **Square** | Harsh, electronic | Synth, electronic sounds |
| **Triangle** | Bright but smooth | Chimes, bells |
| **Sawtooth** | Rich in harmonics | Strings, complex tones |
| **Noise/Percussion** | White noise + decay | Drums, impacts |

### How It Works

```rust
// Data → Audio Attributes
let attrs = AudioAttributes {
    instrument: Instrument::DeepBass,  // → Sine wave
    pitch: 0.5,                        // → 450 Hz
    volume: 0.7,                       // → 70% amplitude
    pan: -0.3,                         // → Slightly left
};

// Generate and play
engine.play_tone(&attrs)?;
```

### Current Mapping (Graph Sonification)

```
Primal Type    → Instrument → Waveform
─────────────────────────────────────
Security       → Bass       → Sine (low, 100-200Hz)
Compute        → Drums      → Noise burst
Discovery      → Chimes     → Triangle (high)
Storage        → Strings    → Sawtooth (rich)
AI             → Synth      → Square (electronic)

Health Status  → Pitch
─────────────────────────
Healthy        → 0.7-0.8 (harmonic)
Warning        → 0.5-0.6 (off-key)
Critical       → 0.2-0.3 (dissonant)

Activity       → Volume
Position       → Stereo Pan
```

### Example: BingoCube Sonification

```rust
// Cell position and color → Audio
let audio = renderer.generate_cell_audio(row, col, color);

// Maps to:
// - Color → Instrument (0-4 = 5 instruments)
// - Color → Pitch offset (0-11 semitones)
// - Row → Octave
// - Position → Pan (left to right)
// - Distance from center → Volume
```

---

## 2. User-Provided Audio (Via rodio)

### What rodio Supports

The `rodio` library (which we already use) can load:

- ✅ **WAV** - Uncompressed audio
- ✅ **MP3** - Compressed audio (via symphonia)
- ✅ **FLAC** - Lossless compression
- ✅ **Ogg Vorbis** - Open format

### How to Add User Audio Support

#### Option A: Simple File Loading

```rust
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;

impl AudioPlaybackEngine {
    /// Play a user-provided audio file
    pub fn play_file(&self, path: &str) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;
        
        let sink = Sink::try_new(&self.stream_handle)?;
        sink.set_volume(self.master_volume);
        sink.append(source);
        sink.detach();
        
        Ok(())
    }
}
```

#### Option B: Audio Library System

```rust
/// User-provided audio library
pub struct AudioLibrary {
    sounds: HashMap<String, PathBuf>,
}

impl AudioLibrary {
    /// Register a sound file
    pub fn register(&mut self, name: &str, path: PathBuf) {
        self.sounds.insert(name.to_string(), path);
    }
    
    /// Play a registered sound
    pub fn play(&self, engine: &AudioPlaybackEngine, name: &str) -> anyhow::Result<()> {
        if let Some(path) = self.sounds.get(name) {
            engine.play_file(path.to_str().unwrap())?;
        }
        Ok(())
    }
}
```

#### Option C: Hybrid System

```rust
pub enum AudioSource {
    /// Generate programmatically
    Generated(AudioAttributes),
    /// Load from file
    File(PathBuf),
    /// Use registered sample
    Sample(String),
}

impl AudioPlaybackEngine {
    pub fn play(&self, source: AudioSource) -> anyhow::Result<()> {
        match source {
            AudioSource::Generated(attrs) => self.play_tone(&attrs),
            AudioSource::File(path) => self.play_file(path.to_str().unwrap()),
            AudioSource::Sample(name) => self.library.play(self, &name),
        }
    }
}
```

---

## 3. Proposed Architecture: Adaptive Audio System

### User Configuration

```toml
# petalTongue.toml

[audio]
mode = "adaptive"  # "generated" | "user" | "adaptive"

[audio.library]
# User-provided sounds (optional)
bass_sound = "/path/to/deep_bass.wav"
drum_sound = "/path/to/kick.mp3"
chime_sound = "/path/to/bell.flac"
string_sound = "/path/to/violin.wav"
synth_sound = "/path/to/synth.ogg"

[audio.colors]
# User-provided color → sound mappings
color_0 = "C4"  # Middle C
color_1 = "D4"
color_2 = "E4"
# ... or ...
color_palette = "/path/to/custom_samples/"
```

### Adaptive Logic

```rust
impl AudioPlaybackEngine {
    /// Play sound using best available method
    pub fn play_adaptive(&self, attrs: &AudioAttributes) -> anyhow::Result<()> {
        // 1. Try user-provided sound for this instrument
        if let Some(user_sound) = self.library.get_instrument_sound(&attrs.instrument) {
            return self.play_file_with_pitch(user_sound, attrs.pitch);
        }
        
        // 2. Fall back to programmatic generation
        self.play_tone(attrs)
    }
}
```

---

## 4. What We Can Do NOW vs FUTURE

### Current Capabilities ✅

- [x] Generate sine, square, triangle, sawtooth, noise waveforms
- [x] Map data to audio attributes (pitch, volume, pan, instrument)
- [x] Play tones through speakers (when audio feature enabled)
- [x] Honest capability detection (knows if audio works)
- [x] Graceful degradation (works without audio)

### Easy Additions 🟡

- [ ] Load user WAV/MP3/FLAC/OGG files (rodio supports this)
- [ ] Audio library system (register sounds by name)
- [ ] Pitch shifting for user samples
- [ ] Audio file browser in UI
- [ ] Export soundscape to audio file

### Future Enhancements 🔵

- [ ] ADSR envelopes (attack, decay, sustain, release)
- [ ] Low-pass / high-pass filters
- [ ] Reverb / delay effects
- [ ] Multi-track mixing
- [ ] Real-time audio parameter modulation
- [ ] MIDI support
- [ ] Sample libraries (built-in sound packs)

---

## 5. Philosophy: Self-Sufficient yet Extensible

### Core Principle

> **petalTongue must work out of the box, but welcome user customization.**

### Why This Matters

1. **Accessibility First**
   - Blind users need audio immediately
   - Can't require users to provide sounds
   - Must have sensible defaults

2. **Sovereignty**
   - Users own their data
   - Users own their audio aesthetic
   - Users can bring their own cultural sounds

3. **Graceful Degradation**
   - Works with generated sounds (always)
   - Works better with user sounds (optional)
   - Works best with user + generated hybrid (adaptive)

### Example Use Cases

#### Use Case 1: Out of the Box

```rust
// User opens petalTongue for the first time
// No setup needed - uses generated waveforms
let engine = AudioPlaybackEngine::new()?;
engine.play_tone(&attrs)?;  // Works immediately
```

#### Use Case 2: Cultural Adaptation

```rust
// User from different culture wants traditional instruments
library.register("bass", "~/sounds/taiko_drum.wav");
library.register("chimes", "~/sounds/singing_bowl.wav");
library.register("strings", "~/sounds/sitar.wav");

// petalTongue uses their sounds
engine.play_adaptive(&attrs)?;  // Uses taiko, not synth bass
```

#### Use Case 3: Wartime Field Use

```rust
// Doctor in conflict zone, limited bandwidth
// No time to upload custom sounds
// Uses generated audio - works offline, no dependencies

let engine = AudioPlaybackEngine::new()?;
// Just works - no files needed
```

---

## 6. Comparison: Generated vs User Audio

### Generated Audio (What We Have)

**Pros:**
- ✅ Always available
- ✅ No file dependencies
- ✅ Predictable
- ✅ Small binary size
- ✅ Fast generation
- ✅ Works offline
- ✅ Deterministic (same data → same sound)

**Cons:**
- ❌ Basic waveforms (not "musical")
- ❌ No cultural variety
- ❌ Limited timbral quality
- ❌ Can sound "synthetic"

### User Audio (What We Can Add)

**Pros:**
- ✅ High quality samples
- ✅ Cultural diversity
- ✅ Musical instruments
- ✅ User preference
- ✅ Professional sound design

**Cons:**
- ❌ Requires user setup
- ❌ File dependencies
- ❌ Larger storage
- ❌ May not exist
- ❌ Licensing concerns

### Hybrid (Best of Both)

- ✅ Works immediately (generated default)
- ✅ Better with user samples (optional upgrade)
- ✅ Fallback when files missing (robust)
- ✅ User sovereignty (bring your own)

---

## 7. Implementation Roadmap

### Phase 1: Current ✅

- [x] Waveform generators (sine, square, triangle, sawtooth, noise)
- [x] Audio attribute mapping (data → sound parameters)
- [x] Capability detection (honest about audio availability)
- [x] Graceful degradation (visual-only mode)

### Phase 2: Basic User Audio (Next)

- [ ] Add `play_file()` method to `AudioPlaybackEngine`
- [ ] Create `AudioLibrary` for registered sounds
- [ ] UI file picker for loading audio files
- [ ] Config file support for audio paths
- [ ] Documentation for user audio setup

### Phase 3: Adaptive System

- [ ] Hybrid `play_adaptive()` method
- [ ] Fallback logic (user → generated)
- [ ] Pitch shifting for user samples
- [ ] Volume normalization
- [ ] Audio preview in UI

### Phase 4: Advanced Features

- [ ] ADSR envelopes
- [ ] Audio effects (reverb, delay, filters)
- [ ] Multi-track mixing
- [ ] Export to audio file
- [ ] Built-in sample packs (optional download)

---

## 8. Technical Notes

### Why rodio?

- **Pure Rust** - No C dependencies (except platform audio APIs)
- **Cross-platform** - Works on Linux (ALSA), macOS (CoreAudio), Windows (WASAPI)
- **Format support** - WAV, MP3, FLAC, Ogg via `symphonia`
- **Simple API** - Easy to use
- **Active development** - Well maintained

### Sample Rate

- Default: **48 kHz** (professional quality)
- Configurable if needed
- Matches most modern audio hardware

### Latency

- Current: ~10-50ms (acceptable for data sonification)
- Not for real-time music performance
- Good enough for accessibility and data feedback

### Memory

- Generated audio: **Minimal** (calculated on-the-fly)
- User audio: **Per file** (loaded into memory)
- Can stream large files if needed

---

## 9. Example: Complete Adaptive System

```rust
// Complete example of self-sufficient + user-extensible audio

use std::path::PathBuf;
use std::collections::HashMap;

pub struct AdaptiveAudioSystem {
    engine: AudioPlaybackEngine,
    library: AudioLibrary,
    mode: AudioMode,
}

pub enum AudioMode {
    GeneratedOnly,
    UserOnly,
    Adaptive,  // Try user, fall back to generated
}

impl AdaptiveAudioSystem {
    pub fn new() -> Self {
        Self {
            engine: AudioPlaybackEngine::new().unwrap_or_default(),
            library: AudioLibrary::new(),
            mode: AudioMode::Adaptive,
        }
    }
    
    /// Play sound using configured mode
    pub fn play(&self, attrs: &AudioAttributes) -> anyhow::Result<()> {
        match self.mode {
            AudioMode::GeneratedOnly => {
                self.engine.play_tone(attrs)
            }
            AudioMode::UserOnly => {
                if let Some(sample) = self.library.get_for_instrument(&attrs.instrument) {
                    self.engine.play_file_with_params(sample, attrs)
                } else {
                    anyhow::bail!("User audio mode but no sample for {:?}", attrs.instrument)
                }
            }
            AudioMode::Adaptive => {
                // Try user sample first
                if let Some(sample) = self.library.get_for_instrument(&attrs.instrument) {
                    self.engine.play_file_with_params(sample, attrs)?;
                } else {
                    // Fall back to generated
                    self.engine.play_tone(attrs)?;
                }
                Ok(())
            }
        }
    }
    
    /// Register user audio files
    pub fn register_instrument(&mut self, instrument: Instrument, path: PathBuf) {
        self.library.register(instrument, path);
    }
}
```

---

## 10. Conclusion

**YES, we can generate sounds in Rust.**

✅ **Self-sufficient**: Works immediately with generated waveforms  
✅ **User-extensible**: Can load WAV/MP3/FLAC/OGG files  
✅ **Adaptive**: Uses user sounds when available, generates when not  
✅ **Sovereign**: Users bring their own audio culture  
✅ **Accessible**: Works out of the box for blind users  
✅ **Robust**: Degrades gracefully when audio unavailable  

**Not a symphony, but functional and honest.**

The system knows what it can do, does it well, and welcomes user enhancement.

---

*"petalTongue: Self-sufficient yet extensible. Works alone, works better together."*

