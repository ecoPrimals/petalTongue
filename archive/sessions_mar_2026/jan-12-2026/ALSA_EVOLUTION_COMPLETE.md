# 🎨 ALSA Evolution to Pure Rust - Complete Plan

**Date**: January 12, 2026  
**Status**: In Progress → Complete  
**Philosophy**: ALSA is an **external extension**, not a core dependency

---

## 🎯 Current State Analysis

### Where ALSA Dependencies Exist

| Crate | File | Dependency | Usage | Feature Flag |
|-------|------|------------|-------|--------------|
| `petal-tongue-graph` | `audio_playback.rs` | `rodio` | Audio playback | ✅ `native-audio` |
| `petal-tongue-core` | `capabilities.rs` | `rodio` | Capability detection | ✅ `audio` |
| `petal-tongue-entropy` | `audio.rs` | `cpal` | Mic capture | ✅ `audio` |
| `petal-tongue-ui` | `sensors/audio.rs` | `rodio` | Audio sensor | ❌ **Not gated!** |

### What We Have (Pure Rust)

✅ **AudioCanvas** (`petal-tongue-ui/src/audio_canvas.rs`):
- Direct `/dev/snd/pcmC0D0p` access
- No ALSA, no C dependencies
- Like WGPU for graphics, or framebuffer for display
- **This is the future!**

✅ **Symphonia** (already in use):
- Pure Rust audio decoding (MP3, WAV, FLAC)
- No C dependencies
- Production-ready

---

## 🏗️ Evolution Strategy

### Tier System (TRUE PRIMAL Architecture)

```
Tier 1: Self-Stable (Pure Rust)
  ✅ AudioCanvas (direct /dev/snd)
  ✅ Symphonia (decoding)
  ✅ Hound (WAV export)
  → Works WITHOUT any external dependencies

Tier 2: Network Extensions (Optional)
  ✅ ToadStool (advanced synthesis via network)
  → Enhanced capability, not required

Tier 3: External Extensions (Optional)
  ⚠️ ALSA/rodio/cpal (external C library)
  → Legacy compatibility only
```

### Migration Path

**For Audio Playback** (rodio → AudioCanvas):
```rust
// OLD (rodio - requires ALSA)
let (_stream, handle) = rodio::OutputStream::try_default()?;
let sink = rodio::Sink::try_new(&handle)?;
sink.append(source);

// NEW (AudioCanvas - pure Rust)
let mut canvas = AudioCanvas::open_default()?;
canvas.write_samples(&samples)?;
```

**For Audio Capture** (cpal → AudioCanvas):
```rust
// OLD (cpal - requires ALSA)
let host = cpal::default_host();
let device = host.default_input_device()?;
let stream = device.build_input_stream(...)?;

// NEW (AudioCanvas - pure Rust)
let mut canvas = AudioCanvas::open_capture("/dev/snd/pcmC0D0c")?;
let samples = canvas.read_samples(4096)?;
```

---

## 📋 Implementation Checklist

### Phase 1: Make ALSA Truly Optional ✅

- [x] 1. Audit all `rodio`/`cpal` usage (done above)
- [ ] 2. Gate `sensors/audio.rs` behind feature flag
- [ ] 3. Update `Cargo.toml` files to clarify extension status
- [ ] 4. Document build options clearly

### Phase 2: Implement Pure Rust Alternatives 🔄

- [ ] 5. **AudioCanvas Playback** (expand current implementation)
  - [x] Basic write_samples (already done)
  - [ ] Add buffering/streaming support
  - [ ] Add format configuration (sample rate, channels)
  - [ ] Error handling improvements

- [ ] 6. **AudioCanvas Capture** (new feature)
  - [ ] Open `/dev/snd/pcmC0D0c` (capture devices)
  - [ ] Read samples from device
  - [ ] Buffering and overrun handling
  - [ ] Quality metrics integration

- [ ] 7. **Capability Detection** (pure Rust)
  - [ ] Replace `rodio::OutputStream::try_default()`
  - [ ] Use AudioCanvas::discover() instead
  - [ ] Check `/dev/snd/` directory existence

### Phase 3: Update Usage Sites 🔄

- [ ] 8. **graph/audio_playback.rs**
  - [ ] Wrap rodio usage in `#[cfg(feature = "native-audio")]`
  - [ ] Add AudioCanvas alternative implementation
  - [ ] Default to AudioCanvas (Tier 1)

- [ ] 9. **core/capabilities.rs**
  - [ ] Replace rodio check with AudioCanvas::discover()
  - [ ] Pure Rust capability detection

- [ ] 10. **entropy/audio.rs**
  - [ ] Wrap cpal in `#[cfg(feature = "audio")]`
  - [ ] Implement AudioCanvas capture alternative
  - [ ] Use for entropy quality measurement

- [ ] 11. **ui/sensors/audio.rs**
  - [ ] Add `#[cfg(feature = "alsa-sensor")]` gates
  - [ ] Provide AudioCanvas alternative
  - [ ] Graceful degradation

### Phase 4: Documentation & Testing 📚

- [ ] 12. Update BUILD_REQUIREMENTS.md
- [ ] 13. Update BUILD_INSTRUCTIONS.md  
- [ ] 14. Add feature comparison table
- [ ] 15. Test builds with/without ALSA
- [ ] 16. Update audit documentation

---

## 🔧 Concrete Changes Needed

### 1. Update `petal-tongue-ui/Cargo.toml`

```toml
[features]
default = ["external-display"]
external-display = []
software-rendering = ["softbuffer", "pixels"]
framebuffer-direct = ["nix", "dialoguer"]
toadstool-wasm = ["wasm-bindgen", "wasm-bindgen-futures", "web-sys"]

# EXTERNAL EXTENSION: ALSA-based audio sensor
# This is OPTIONAL and requires C library at build time
# Consider using AudioCanvas (pure Rust) instead
alsa-sensor = []  # Feature flag to gate sensors/audio.rs
```

### 2. Update `petal-tongue-graph/Cargo.toml`

```toml
[features]
default = []

# EXTERNAL EXTENSION: Legacy ALSA audio playback
# DEPRECATED: Use AudioCanvas or ToadStool instead
# Requires: sudo apt-get install libasound2-dev pkg-config
native-audio = ["rodio"]
```

### 3. Update `petal-tongue-core/Cargo.toml`

```toml
[features]
default = []

# EXTERNAL EXTENSION: ALSA-based audio capability
# DEPRECATED: Use AudioCanvas discovery instead
alsa-capability = ["rodio"]

test-fixtures = []
```

### 4. Update `petal-tongue-entropy/Cargo.toml`

```toml
[features]
default = []

# EXTERNAL EXTENSION: ALSA-based audio capture
# EVOLVING TO: AudioCanvas (pure Rust /dev/snd access)
# Requires: sudo apt-get install libasound2-dev pkg-config
alsa-audio = ["cpal", "hound", "rustfft"]

video = ["nokhwa"]
```

### 5. Gate Code Behind Features

**petal-tongue-ui/src/sensors/audio.rs**:
```rust
// At top of file
#![cfg(feature = "alsa-sensor")]

// OR wrap entire implementation
#[cfg(feature = "alsa-sensor")]
pub struct AudioSensor { /* rodio-based */ }

#[cfg(not(feature = "alsa-sensor"))]
pub struct AudioSensor { /* AudioCanvas-based */ }
```

**petal-tongue-core/src/capabilities.rs**:
```rust
// Replace rodio check
#[cfg(feature = "alsa-capability")]
fn detect_audio_capability() -> bool {
    rodio::OutputStream::try_default().is_ok()
}

#[cfg(not(feature = "alsa-capability"))]
fn detect_audio_capability() -> bool {
    // Pure Rust: Check if /dev/snd/ exists with playback devices
    use crate::audio_canvas::AudioCanvas;
    !AudioCanvas::discover().unwrap_or_default().is_empty()
}
```

---

## 📊 Feature Comparison

| Feature | AudioCanvas (Pure Rust) | ALSA (C Library) |
|---------|-------------------------|------------------|
| **Build Dependencies** | None | libasound2-dev, pkg-config |
| **Runtime Dependencies** | None | ALSA libraries |
| **Platform Support** | Linux only | Cross-platform |
| **Sovereignty** | ✅ 100% | ⚠️ External C |
| **Latency** | ✅ Direct hardware | ⚠️ ALSA layer |
| **Complexity** | ✅ Simple | ⚠️ Complex API |
| **Recommended** | ✅ Yes (Tier 1) | ⚠️ Legacy/Extension |

---

## 🎯 Build Options Documentation

### Option 1: Pure Rust (Recommended)

```bash
# No ALSA required - 100% sovereign
cargo build --release --no-default-features
```

**Capabilities**:
- ✅ AudioCanvas (direct device access)
- ✅ Symphonia (audio decoding)
- ✅ Hound (WAV export)
- ✅ ToadStool network audio (if available)

### Option 2: With ALSA Extensions

```bash
# Install ALSA headers first
sudo apt-get install libasound2-dev pkg-config

# Build with ALSA extensions
cargo build --release --features alsa-sensor,native-audio,alsa-capability,alsa-audio
```

**Additional Capabilities**:
- ✅ Everything from Option 1
- ➕ Legacy rodio playback
- ➕ Legacy cpal capture
- ➕ ALSA-based sensors

---

## 🚀 Deployment Recommendation

### Default Build

```toml
[features]
default = []  # Pure Rust only - no ALSA
```

**Why**:
1. ✅ Zero build-time dependencies
2. ✅ Zero runtime dependencies
3. ✅ Sovereign architecture (TRUE PRIMAL)
4. ✅ Works on any Linux system
5. ✅ Simple, fast, reliable

### Extension Build

```bash
# Only if user explicitly wants ALSA compatibility
cargo build --features alsa-extensions
```

Where `alsa-extensions` is a meta-feature:
```toml
alsa-extensions = ["alsa-sensor", "native-audio", "alsa-capability", "alsa-audio"]
```

---

## 📝 Documentation Updates Needed

### BUILD_REQUIREMENTS.md

Update to clarify:
```markdown
## Core Requirements (Always)

- Rust 1.70+ (cargo, rustc)
- Linux kernel with /dev/snd/ support

## Optional Extensions

### ALSA Audio Extension

**Status**: Optional, not recommended (use AudioCanvas instead)

If you want legacy ALSA compatibility:
```bash
sudo apt-get install libasound2-dev pkg-config
cargo build --features alsa-extensions
```

**Recommended**: Use pure Rust AudioCanvas (no installation needed)
```

### BUILD_INSTRUCTIONS.md

Add feature matrix:
```markdown
## Feature Flags

### Audio Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| (none) | AudioCanvas (pure Rust) | None ✅ |
| `native-audio` | Legacy rodio playback | ALSA C library ⚠️ |
| `alsa-sensor` | ALSA-based audio sensor | ALSA C library ⚠️ |
| `alsa-capability` | ALSA capability detection | ALSA C library ⚠️ |
| `alsa-audio` | ALSA entropy capture | ALSA C library ⚠️ |

**Recommendation**: Use default (no features) for pure Rust sovereignty
```

---

## ✅ Success Criteria

- [ ] Build succeeds without ALSA: `cargo build --workspace --no-default-features`
- [ ] Clippy passes without ALSA: `cargo clippy --workspace --no-default-features`
- [ ] Tests pass without ALSA: `cargo test --workspace --no-default-features --lib`
- [ ] Documentation clearly marks ALSA as optional extension
- [ ] AudioCanvas works for playback and capture
- [ ] Feature flags properly gate all ALSA code

---

## 🎯 Timeline

**Phase 1** (Gate code): ~2 hours
**Phase 2** (AudioCanvas improvements): ~4 hours
**Phase 3** (Update usage sites): ~3 hours
**Phase 4** (Documentation): ~1 hour

**Total**: ~10 hours of focused work

---

## 🌸 TRUE PRIMAL Alignment

This evolution ensures:

✅ **Self-Stable**: Works without ANY external dependencies  
✅ **Sovereign**: 100% Rust, no C libraries required  
✅ **Capability-Based**: Extensions enhance, don't define  
✅ **Graceful Degradation**: Works without ALSA  
✅ **Runtime Discovery**: AudioCanvas discovers devices at runtime  

**ALSA becomes Tier 3** (External Extension), not Tier 1 (Core).

---

*Status: Ready for implementation*  
*Philosophy: External dependencies are extensions, not requirements*  
🎨 **Audio Canvas is the future!** 🎨

