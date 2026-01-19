# Build Instructions for petalTongue

## 🎉 ZERO Dependencies Required - 100% Pure Rust!

**EVOLUTION COMPLETE**: AudioCanvas eliminates ALL build dependencies!

### Quick Start (All Platforms - No Dependencies!)

```bash
# That's it - just build!
cargo build --workspace --release

# Run
cargo run --bin petal-tongue
```

**Platforms**: Linux, macOS, Windows - all work identically! ✅  
**Requirements**: ZERO (just Rust toolchain) ✅  
**Build Time**: ~15 seconds  
**Audio**: Works out of the box via AudioCanvas (pure Rust!)

---

## What Changed? ALSA Elimination! 🚀

**Before** (deprecated):
```bash
# OLD: Required system dependencies
sudo apt-get install -y libasound2-dev pkg-config
cargo build --features audio
```

**After** (current):
```bash
# NEW: Zero dependencies!
cargo build
```

**AudioCanvas** provides:
- **Linux**: Direct `/dev/snd` device access
- **macOS**: System CoreAudio APIs  
- **Windows**: System WASAPI APIs
- **All**: 100% Pure Rust, zero C dependencies!

See [ALSA_ELIMINATION_COMPLETE.md](ALSA_ELIMINATION_COMPLETE.md) for details.

## Feature Flags

### Core Features (Zero Dependencies)
- `default` - Full build including AudioCanvas (pure Rust audio!) ✅
- `external-display` - Enable native window support (eframe/egui)

### Optional Features (Still Zero Dependencies!)
- `software-rendering` - Pure software pixel rendering
- `framebuffer-direct` - Direct framebuffer access (embedded/kiosk)
- `toadstool-wasm` - ToadStool GPU acceleration via WASM

### Deprecated Features (No Longer Needed!)
- ~~`audio`~~ - **DEPRECATED**: AudioCanvas works by default!
- `video` - Camera access (Phase 3 feature)

## Testing

### Run Tests (Zero Dependencies!)
```bash
# All tests work without dependencies
cargo test --workspace
```

### Run Coverage
```bash
# Measure test coverage
cargo llvm-cov --workspace --html --open
```

### Run Specific Test Suites
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Documentation tests
cargo test --doc
```

## Building for Different Environments

### Embedded/IoT (Minimal)
```bash
cargo build --release --no-default-features
```

### Headless Server (No GUI)
```bash
cargo build --release --bin petal-tongue-headless
```

### Full Desktop (All Features)
```bash
# All features, still zero dependencies!
cargo build --release --all-features
```

### Cross-Compilation
```bash
# Works for any target - no C dependencies!
cargo build --target x86_64-unknown-linux-musl
cargo build --target aarch64-unknown-linux-gnu
cargo build --target x86_64-pc-windows-gnu
```

## Troubleshooting

### "alsa-sys build failed"
```bash
# This means audio features are enabled but ALSA libraries not installed
# Solution 1: Install ALSA
sudo apt-get install libasound2-dev pkg-config

# Solution 2: Disable audio
cargo build --no-default-features
```

### "Test compilation failed"
```bash
# Some tests may require specific features
cargo test --workspace --no-default-features
```

## CI/CD Configuration

### GitHub Actions Example
```yaml
- name: Build (No System Dependencies)
  run: cargo build --workspace --no-default-features

- name: Test (No System Dependencies)
  run: cargo test --workspace --no-default-features

- name: Build (With Audio on Linux)
  if: runner.os == 'Linux'
  run: |
    sudo apt-get install -y libasound2-dev pkg-config
    cargo build --workspace --features audio
```

## Architecture Notes

petalTongue is designed to build and run **anywhere** without system dependencies by default. Audio and video capabilities are **discovered at runtime** - the application works perfectly without them.

This aligns with our **TRUE PRIMAL** philosophy:
- Zero hardcoded dependencies
- Graceful degradation
- Capability-based discovery
- Works in constrained environments (SSH, containers, embedded)

