# Audio Setup for petalTongue

## The Problem

petalTongue was **claiming audio capability but not actually playing audio**. This is a **critical accessibility and reliability issue**.

## Why This Matters

1. **Accessibility**: Blind users rely on audio to navigate
2. **Critical Use Cases**: AR in wartime, disaster response, emergency situations
3. **Trust**: Users must be able to trust capability claims
4. **Sovereignty**: petalTongue must be self-aware about what it can actually do

## The Solution

petalTongue now has a **capability detection system** that:

- ✅ **Tests** audio output devices (doesn't just assume)
- ✅ **Reports honestly** when audio is unavailable
- ✅ **Degrades gracefully** when hardware/drivers are missing
- ✅ **Never lies** about what it can do

## Enabling Audio

### Linux (Ubuntu/Debian/Pop!_OS)

Install ALSA development libraries:

```bash
sudo apt-get install -y libasound2-dev pkg-config
```

Then build with audio support:

```bash
cargo build --features audio
cargo test --features audio
```

### macOS

No additional setup needed - CoreAudio is built-in:

```bash
cargo build --features audio
```

### Windows

No additional setup needed - WASAPI is built-in:

```bash
cargo build --features audio
```

## Checking Audio Status

Run petalTongue and check the capability report:

```rust
use petal_tongue_core::CapabilityDetector;

let detector = CapabilityDetector::default();
println!("{}", detector.capability_report());
```

Output example (audio unavailable):

```
🔍 petalTongue Modality Capabilities

✅ Visual2D: Available (tested)
   Reason: egui window rendering available

❌ Audio: Unavailable (tested)
   Reason: Audio feature not compiled (requires libasound2-dev on Linux)

✅ Animation: Available (not tested)
   Reason: Animation system available

✅ TextDescription: Available (tested)
   Reason: Text rendering available

❌ Haptic: Unavailable (not tested)
   Reason: Haptic feedback not yet implemented

❌ VR3D: Unavailable (not tested)
   Reason: VR/AR rendering not yet implemented
```

Output example (audio available):

```
🔍 petalTongue Modality Capabilities

✅ Visual2D: Available (tested)
   Reason: egui window rendering available

✅ Audio: Available (tested)
   Reason: Audio output device initialized successfully

✅ Animation: Available (not tested)
   Reason: Animation system available

✅ TextDescription: Available (tested)
   Reason: Text rendering available

❌ Haptic: Unavailable (not tested)
   Reason: Haptic feedback not yet implemented

❌ VR3D: Unavailable (not tested)
   Reason: VR/AR rendering not yet implemented
```

## Architecture

### Capability Detection (`petal-tongue-core/src/capabilities.rs`)

- `CapabilityDetector`: Tests and reports modality availability
- `Modality`: Enum of available modalities (Visual2D, Audio, Haptic, Animation, VR3D, TextDescription)
- `ModalityStatus`: Available, NotInitialized, Unavailable, Disabled
- `ModalityCapability`: Detection result with reason and test status

### Audio Playback (`petal-tongue-graph/src/audio_playback.rs`)

- `AudioPlaybackEngine`: Plays actual sounds through speakers using rodio
- Supports multiple instruments (bass, drums, chimes, strings, synth)
- Handles volume, pitch, panning, duration

### Audio Sonification (`petal-tongue-graph/src/audio_sonification.rs`)

- `AudioSonificationRenderer`: Maps graph topology to audio attributes
- Primal type → Instrument
- Health status → Pitch
- Activity → Volume
- Position → Stereo panning

### BingoCube Audio (`bingoCube/adapters/src/audio.rs`)

- `BingoCubeAudioRenderer`: Sonifies BingoCube patterns
- Cell color → Instrument and pitch
- Position → Pan and volume
- Progressive reveal → Dynamic soundscape

## Testing Audio

### Unit Tests

```bash
cargo test --features audio capabilities::tests
```

### Integration Tests

```bash
cargo test --features audio audio_playback::tests
```

### Manual Testing

1. Build with audio feature:
   ```bash
   cargo run --features audio -p petal-tongue-ui
   ```

2. Click "🎲 BingoCube Tool" in the UI

3. Click "🎵 Audio" button - you should see:
   - Soundscape description
   - Instrument counts
   - If audio is actually available, you'll hear tones

## Roadmap

- [x] Capability detection system
- [x] Honest audio status reporting
- [ ] Integrate `AudioPlaybackEngine` into UI
- [ ] Connect BingoCube audio to playback
- [ ] Add "Play" button for audio preview
- [ ] Real-time audio updates as graph changes
- [ ] Audio recording/export capabilities

## Philosophy

> **Never claim a capability you don't have.**
>
> In critical situations, false capability claims are dangerous.
> petalTongue must be self-aware and honest about what it can do.

This is not just about audio - it's about **system integrity** and **trust**.

