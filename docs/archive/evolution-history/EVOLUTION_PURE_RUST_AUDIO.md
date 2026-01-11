# 🔧 Evolution: Pure Rust MP3 Playback (No External Dependencies)

**Issue**: Current code uses external commands (`mpv`, `ffplay`, `paplay`) - violates TRUE PRIMAL sovereignty

**Solution**: Use `rodio` (already in dependencies) for pure Rust MP3 playback

---

## Current State (EXTERNAL DEPENDENCIES ❌)

```rust
// startup_audio.rs:224-237
// Uses external system commands
let players = vec!["mpv", "ffplay", "paplay", "aplay"];
for player in players {
    if Command::new(player).arg(&path).spawn().is_ok() {
        info!("🎵 Startup music playing with {}...", player);
        break;
    }
}
```

**Problems**:
- ❌ Requires external tools installed
- ❌ Not self-contained
- ❌ Violates sovereignty
- ❌ No playback control
- ❌ Silent failures

---

## Evolution: Pure Rust with `rodio` ✅

### 1. Add `rodio` + `symphonia` (MP3 decoder)

**File**: `crates/petal-tongue-ui/Cargo.toml`

```toml
[dependencies]
# Pure Rust audio (no external dependencies!)
rodio = { version = "0.19", default-features = false, features = ["symphonia-mp3"] }
```

### 2. Evolve `startup_audio.rs` to Pure Rust

```rust
//! Startup Audio System - PURE RUST (No External Dependencies)
//!
//! Architecture:
//! 1. Signature Tone: Pure Rust generation (always works)
//! 2. Startup Music: rodio MP3 playback (self-contained)
//! 3. No external commands needed!

use std::io::Cursor;
use tracing::{info, warn};

/// Embedded startup music (11MB MP3)
const EMBEDDED_STARTUP_MUSIC: &[u8] = include_bytes!("../assets/startup_music.mp3");

/// Play startup audio sequence
pub fn play(&self, _audio_system: &AudioSystem) {
    let play_signature = self.play_signature;
    let play_music = self.play_music;
    let use_embedded = self.use_embedded;

    // Spawn background thread for audio playback
    std::thread::spawn(move || {
        // 1. Play signature tone (pure Rust WAV)
        if play_signature {
            if let Err(e) = Self::play_signature_pure_rust() {
                warn!("Signature tone failed: {}", e);
            }
        }

        // 2. Play startup music (pure Rust MP3 via rodio)
        if play_music && use_embedded {
            if let Err(e) = Self::play_embedded_mp3_pure_rust() {
                warn!("Startup music failed: {}", e);
                info!("💡 Tip: rodio may need ALSA/PulseAudio for audio output");
            }
        }

        info!("✨ Startup audio sequence complete");
    });
}

/// Play signature tone using pure Rust
fn play_signature_pure_rust() -> Result<(), Box<dyn std::error::Error>> {
    use rodio::{OutputStream, Sink, Source};
    
    info!("🎵 Playing signature tone (pure Rust)...");
    
    // Get default output device
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    
    // Generate signature tone samples
    let signature = Self::generate_signature_tone();
    let sample_rate = crate::audio_pure_rust::SAMPLE_RATE;
    
    // Create rodio source from samples
    let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, signature);
    
    sink.append(source);
    sink.sleep_until_end();
    
    info!("✓ Signature tone complete");
    Ok(())
}

/// Play embedded MP3 using pure Rust (rodio + symphonia)
fn play_embedded_mp3_pure_rust() -> Result<(), Box<dyn std::error::Error>> {
    use rodio::{Decoder, OutputStream, Sink};
    
    info!("🎵 Playing embedded MP3 (pure Rust via rodio + symphonia)...");
    info!("   Size: {} bytes", EMBEDDED_STARTUP_MUSIC.len());
    
    // Get default output device
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    
    // Decode MP3 from embedded bytes
    let cursor = Cursor::new(EMBEDDED_STARTUP_MUSIC);
    let source = Decoder::new(cursor)?;
    
    info!("✓ MP3 decoded successfully");
    
    // Play (non-blocking - music continues in background)
    sink.append(source);
    
    // Don't wait for music to finish - it plays in background
    // The sink and stream will be dropped when thread ends
    info!("✓ Startup music playing (non-blocking)...");
    
    // Keep thread alive for music duration
    sink.sleep_until_end();
    
    Ok(())
}
```

---

## Benefits of Pure Rust Evolution ✅

### Before (External Dependencies):
- ❌ Requires `mpv` or `ffplay` installed
- ❌ Silent failures if player missing
- ❌ No playback control
- ❌ Platform-specific commands
- ❌ Violates sovereignty

### After (Pure Rust):
- ✅ **Self-contained** - No external tools needed
- ✅ **Sovereign** - Embedded MP3 + Rust decoder
- ✅ **Cross-platform** - Works on Linux/Mac/Windows
- ✅ **Playback control** - Can stop/pause/volume
- ✅ **Error handling** - Proper Result types
- ✅ **Performance** - No process spawning overhead

---

## Implementation Plan

### Phase 1: Add Dependencies (5 min)
```toml
# Cargo.toml
rodio = { version = "0.19", default-features = false, features = ["symphonia-mp3"] }
```

### Phase 2: Evolve startup_audio.rs (30 min)
1. Remove `std::process::Command` usage
2. Add `rodio::Decoder` for MP3
3. Use `rodio::Sink` for playback
4. Add proper error handling

### Phase 3: Test (10 min)
```bash
cargo build --release
./target/release/petal-tongue
# Should hear signature tone + music!
```

### Phase 4: Evolve Other Audio (30 min)
- `audio_providers.rs:play_samples()` - Use rodio
- `audio_providers.rs:UserSoundProvider::play()` - Use rodio
- Remove all `Command::new()` audio calls

---

## Dependencies to Add

```toml
[dependencies]
# Pure Rust audio - NO external dependencies!
rodio = { version = "0.19", default-features = false, features = ["symphonia-mp3", "symphonia-wav"] }

# Note: rodio uses cpal for cross-platform audio output
# cpal is pure Rust and works with:
# - Linux: ALSA, PulseAudio, JACK
# - macOS: CoreAudio
# - Windows: WASAPI
# No external commands needed!
```

---

## Architecture: TRUE PRIMAL Audio Stack

```
petalTongue (Sovereign)
  ↓
rodio (Pure Rust Audio Library)
  ├─ Decoder (MP3/WAV/FLAC/OGG via symphonia)
  ├─ Source (Sample generation)
  └─ Sink (Playback control)
  ↓
cpal (Pure Rust Cross-Platform Audio I/O)
  ├─ Linux: ALSA/PulseAudio (via libc FFI)
  ├─ macOS: CoreAudio (via frameworks)
  └─ Windows: WASAPI (via Windows API)
  ↓
Hardware Audio Output
```

**Key**: All Rust code, no external process spawning!

---

## Migration Strategy

### Step 1: Enable rodio Feature
```bash
# Make sure rodio is available
cargo tree | grep rodio
```

### Step 2: Update startup_audio.rs
- Replace `Command::new("mpv")` with `rodio::Decoder`
- Test signature tone
- Test MP3 playback

### Step 3: Update audio_providers.rs
- Evolve `play_samples()` to use rodio
- Evolve `UserSoundProvider` to use rodio
- Remove all external command calls

### Step 4: Test Everything
```bash
# Test startup audio
RUST_LOG=debug ./target/release/petal-tongue

# Test pure Rust tones
# (trigger success sound, error sound, etc in GUI)

# Test user sounds (if PETALTONGUE_SOUNDS_DIR set)
```

---

## Fallback Strategy

If rodio fails (rare - no audio hardware):
1. Log clear error: "No audio output device detected"
2. Continue without audio (visual UI still works)
3. Show notification in GUI: "Audio disabled (no output device)"

This is TRUE PRIMAL graceful degradation!

---

## Testing Checklist

- [ ] Signature tone plays (pure Rust sine waves)
- [ ] Embedded MP3 plays (rodio decoder)
- [ ] No external commands spawned
- [ ] Works without `mpv`/`ffplay` installed
- [ ] Proper error messages if audio unavailable
- [ ] Non-blocking playback (UI stays responsive)
- [ ] Cross-platform (test on Linux/Mac/Windows)

---

## Code Location

**Files to modify**:
1. `crates/petal-tongue-ui/Cargo.toml` - Add rodio dependency
2. `crates/petal-tongue-ui/src/startup_audio.rs` - Pure Rust playback
3. `crates/petal-tongue-ui/src/audio_providers.rs` - Pure Rust playback

**Time Estimate**: 1-2 hours for complete evolution

---

## Benefits Recap

### TRUE PRIMAL Principles ✅
- ✅ **Sovereignty**: No external dependencies
- ✅ **Self-contained**: Everything in binary
- ✅ **Capability-based**: Audio is a capability, not requirement
- ✅ **Graceful degradation**: Works without audio hardware

### Technical Benefits ✅
- ✅ **Performance**: No process spawning
- ✅ **Control**: Pause/stop/volume control
- ✅ **Reliability**: No "player not found" errors
- ✅ **Cross-platform**: Same code everywhere

---

**Status**: Ready to implement  
**Priority**: HIGH (fixes current audio issue properly)  
**Complexity**: Low (rodio already available)  
**Time**: 1-2 hours  
**Impact**: Complete audio sovereignty ✨

