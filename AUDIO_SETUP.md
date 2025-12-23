# 🎵 Audio Setup Guide

## Prerequisites

petalTongue uses `rodio` for audio synthesis, which requires system audio libraries.

### Linux (Debian/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y libasound2-dev pkg-config
```

### Linux (Fedora/RHEL)

```bash
sudo dnf install alsa-lib-devel pkg-config
```

### macOS

No additional setup required (uses CoreAudio).

### Windows

No additional setup required (uses WASAPI).

## Building with Audio

Once prerequisites are installed:

```bash
cd petalTongue/
cargo build --release -p petal-tongue-ui
```

## Running Without Audio (Fallback)

If you cannot install audio libraries, petalTongue will still work with:
- Visual representation (fully functional)
- Audio descriptions (text-based)
- No actual sound output

The audio synthesis code is optional and gracefully degrades.

## Troubleshooting

### Error: "failed to run custom build command for `alsa-sys`"

**Solution**: Install ALSA development libraries (see above).

### Error: "Failed to initialize audio output"

**Solution**: Check that your system has audio output devices available.

### No Sound

1. Check master volume in petalTongue UI
2. Check "Audio Enabled" toggle
3. Check system volume/mute
4. Verify audio device is working (test with other apps)

## Architecture

petalTongue's audio is designed to be optional:
- `AudioPlaybackEngine` handles actual sound generation
- Falls back gracefully if audio unavailable
- Visual representation always works
- Text descriptions always available

This ensures accessibility even without speakers!

