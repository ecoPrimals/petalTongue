# Startup Audio - petalTongue

## Overview

petalTongue now plays a welcoming audio sequence when launched, consisting of:
1. **Signature Tone**: Pure Rust-generated harmonic chord (C-E-G major)
2. **Startup Music**: "Welcome Home Morning Star" by Godking (from reagents/)

## Architecture

### Two-Tier System
1. **Tier 1 - Signature Tone** (Always Available)
   - Pure Rust audio generation (zero dependencies)
   - C major ascending chord (C4-E4-G4) with harmonic overtones
   - Generates welcoming, flourishing sound
   - Falls back gracefully if playback fails

2. **Tier 2 - Startup Music** (Optional)
   - User-provided MP3 file from reagents/
   - Plays after signature tone
   - Graceful degradation if file not found

### Implementation

Located in `crates/petal-tongue-ui/src/startup_audio.rs`

```rust
// Signature tone: C major chord with staggered notes
let notes = [
    261.63, // C4
    329.63, // E4  
    392.00, // G4
];

// + harmonic overtone at C5 for sparkle
```

## Configuration

### Environment Variables

```bash
# Custom startup music path (optional)
export PETALTONGUE_STARTUP_MUSIC="/path/to/your/music.mp3"
```

### File Locations (Auto-Discovery)

petalTongue searches these locations in order:
1. `../../../reagents/Welcome Home Morning Star - Godking.mp3` (relative)
2. `/home/eastgate/Development/ecoPrimals/reagents/Welcome Home Morning Star - Godking.mp3` (absolute)
3. `$PETALTONGUE_STARTUP_MUSIC` environment variable

## Features

### Signature Tone Design
- **Harmonic Structure**: C major chord (tonic triad)
- **Staggered Attack**: Notes enter sequentially (0.1s apart)
- **Duration**: ~0.7 seconds total
- **Overtones**: Additional C5 triangle wave for brightness
- **Normalization**: Prevents clipping, leaves headroom

### Non-Blocking Playback
- Audio plays in background thread
- UI remains responsive
- No impact on application startup time

### Graceful Degradation
- Signature tone uses multiple player fallbacks (aplay, paplay, ffplay, mpv)
- Startup music optional - continues without it
- Logs inform user of audio status
- Works in environments without audio hardware

## System Requirements

### Audio Players (Priority Order)

**For WAV (Signature Tone):**
1. `aplay` (ALSA, most common on Linux)
2. `paplay` (PulseAudio)
3. `ffplay` (FFmpeg)
4. `mpv` (Universal player)

**For MP3 (Startup Music):**
1. `mpv` (Recommended, supports --really-quiet)
2. `ffplay` (FFmpeg)
3. `paplay` (PulseAudio)
4. `aplay` (ALSA, may not support MP3)

### Installation

```bash
# Ubuntu/Debian
sudo apt install alsa-utils mpv

# Fedora/RHEL
sudo dnf install alsa-utils mpv

# Arch
sudo pacman -S alsa-utils mpv
```

## Usage

### Default Behavior

When petalTongue starts:
1. 🎵 Signature tone plays (~0.7s)
2. 🎵 Brief pause (~0.2s)
3. 🎵 Startup music begins (non-blocking)

### Logs

```
🎵 Initializing startup audio...
🎵 Found startup music: /path/to/reagents/Welcome Home Morning Star - Godking.mp3
🎵 Generating petalTongue signature tone...
✨ Signature tone generated: 30870 samples
🎵 Signature tone exported to /tmp/petaltongue_signature.wav
🎵 Signature tone playing with aplay...
🎵 Playing startup music: /path/to/reagents/Welcome Home Morning Star - Godking.mp3
🎵 Startup music playing with mpv (non-blocking)...
✨ Startup audio sequence complete
```

## Development

### Future Enhancements (TODO)

1. **Distinctive Signature Sound**
   - Currently uses simple C major chord
   - Design unique petalTongue audio identity
   - Consider adding envelope shaping
   - Explore unique timbres (FM synthesis, additive synthesis)

2. **User Configuration**
   - UI panel for startup audio preferences
   - Enable/disable signature tone
   - Enable/disable startup music
   - Volume control
   - Custom signature tone selection

3. **Extended Audio Library**
   - Multiple startup themes
   - User-created signatures
   - Seasonal variations
   - Event-specific sounds

4. **Cross-Platform Audio**
   - Native Windows support (wasapi)
   - Native macOS support (CoreAudio)
   - WebAssembly support (Web Audio API)

### Testing

```bash
# Test signature tone generation
cargo test -p petal-tongue-ui startup_audio

# Test with custom music
export PETALTONGUE_STARTUP_MUSIC="/path/to/your/music.mp3"
cargo run

# Test without music (signature only)
mv reagents/Welcome*.mp3 ~/
cargo run
```

## Philosophy

### Self-Sovereignty
- **Pure Rust signature**: No dependencies for core audio identity
- **User control**: Environment variables for customization
- **Graceful degradation**: Works without optimal setup
- **Local-first**: No network required

### Accessibility
- **Multi-modal**: Audio feedback for application start
- **Configurable**: Can be disabled via future UI controls
- **Non-intrusive**: Non-blocking, doesn't delay startup
- **Informative**: Logs provide visibility

### User Experience
- **Welcoming**: Pleasant harmonic greeting
- **Professional**: Custom music sets tone
- **Responsive**: No impact on UI responsiveness
- **Reliable**: Falls back gracefully

## Technical Details

### Audio Generation

**Sample Rate**: 44.1 kHz (CD quality)
**Bit Depth**: 32-bit float (internal), 16-bit PCM (WAV export)
**Channels**: Mono
**Format**: WAV (signature), MP3 (music)

### Waveform Synthesis

```rust
// Sine wave generation
fn generate_sine(frequency: f32, time: f32) -> f32 {
    (2.0 * PI * frequency * time).sin()
}

// With envelope and volume
let sample = waveform * volume * envelope;
```

### Normalization

```rust
// Prevent clipping
let max_amplitude = samples.iter().fold(0.0_f32, |max, &s| max.max(s.abs()));
for sample in &mut samples {
    *sample /= max_amplitude;
    *sample *= 0.7; // Leave headroom
}
```

## Related

- [PURE_RUST_AUDIO_SYSTEM.md](PURE_RUST_AUDIO_SYSTEM.md) - Core audio architecture
- [PURE_RUST_AUDIO.md](PURE_RUST_AUDIO.md) - Audio philosophy

## Credits

**Startup Music**: "Welcome Home Morning Star" by Godking  
**Implementation**: petalTongue team (January 6, 2026)  
**Architecture**: Three-tier pure Rust audio system

---

**Status**: ✅ Implemented  
**Version**: 0.1.0  
**Date**: January 6, 2026

