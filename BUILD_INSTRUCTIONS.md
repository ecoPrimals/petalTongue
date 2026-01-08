# Build Instructions for petalTongue

## Quick Start

### Standard Build (No System Dependencies Required)
```bash
cargo build --workspace --no-default-features
cargo run --bin petal-tongue --no-default-features
```

### With Audio Support (Requires System Dependencies)

**On Linux (Ubuntu/Debian)**:
```bash
# Install ALSA development libraries
sudo apt-get install -y libasound2-dev pkg-config

# Build with audio features
cargo build --workspace --features audio
cargo run --bin petal-tongue --features audio
```

**On macOS**:
```bash
# No additional dependencies needed
cargo build --workspace --features audio
```

**On Windows**:
```bash
# No additional dependencies needed
cargo build --workspace --features audio
```

## Feature Flags

### Core Features
- `default` - Minimal build, no audio, no video (works everywhere)
- `external-display` - Enable native window support (eframe/egui)

### Optional Features
- `audio` - Enable audio entropy capture (requires ALSA on Linux)
- `video` - Enable video entropy capture (requires camera access)
- `software-rendering` - Pure software pixel rendering
- `framebuffer-direct` - Direct framebuffer access (embedded/kiosk)
- `toadstool-wasm` - ToadStool GPU acceleration via WASM

## Testing

### Run Tests (No Audio)
```bash
cargo test --workspace --no-default-features
```

### Run Tests (With Audio)
```bash
# Requires ALSA on Linux
cargo test --workspace --features audio
```

### Run Coverage
```bash
cargo llvm-cov --workspace --no-default-features --html
```

## Building for Different Environments

### Embedded/IoT (No System Dependencies)
```bash
cargo build --release --no-default-features
```

### Headless Server (No GUI)
```bash
cargo build --release --bin petal-tongue-headless --no-default-features
```

### Full Desktop (All Features)
```bash
# Requires ALSA on Linux
cargo build --release --all-features
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

