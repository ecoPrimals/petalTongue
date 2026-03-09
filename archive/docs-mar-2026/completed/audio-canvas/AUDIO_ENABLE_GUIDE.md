# Enable Audio - Quick Guide

**Goal**: Enable audio playback immediately using existing Audio Canvas (Pure Rust!)

---

## Immediate Solution (5 minutes)

### Add User to Audio Group

The Audio Canvas requires access to `/dev/snd/pcmC0D0p` which needs the `audio` group:

```bash
# Add your user to the audio group
sudo usermod -aG audio $USER

# Verify it was added
groups $USER | grep audio

# Logout and login (or reboot) for changes to take effect
# Then test:
./target/release/petal-tongue
```

### Why This Works

Audio Canvas uses **100% Pure Rust** direct hardware access:
- No C dependencies
- No external commands (mpv, aplay, etc.)
- Direct `/dev/snd/pcmC0D0p` device write
- Like framebuffer for graphics!

The only requirement is permission to access the device file.

---

## What You'll Experience

Once enabled:

1. **Signature Tone** (0.5s):
   - C-E-G chord (ascending)
   - Harmonic overtones
   - Distinctive petalTongue sound

2. **Startup Music**:
   - "Welcome Home Morning Star - Godking.mp3"
   - 11MB embedded at compile time
   - Decoded with `symphonia` (pure Rust!)
   - Played via Audio Canvas (direct hardware!)

3. **GUI with Proprioception**:
   - Visual output confirmed
   - Bidirectional feedback loop
   - SAME DAVE (neuroanatomy model)
   - Complete self-awareness!

---

## Evolution Path (Future)

### Current State: Audio Canvas ✅
- 100% Pure Rust
- Direct hardware access
- Requires: audio group (one-time setup)
- Status: PRODUCTION READY

### Future Evolution: PipeWire Client 🚧
- 100% Pure Rust protocol implementation
- Unix socket communication
- Requires: Nothing! User-level
- Timeline: 2-4 weeks focused work
- Status: DOCUMENTED, ready for evolution

### Architecture:

```
Current (Audio Canvas):
  petalTongue → /dev/snd/pcmC0D0p → Hardware
                      ↑
                Requires audio group (one-time)

Future (PipeWire):
  petalTongue → /run/user/$UID/pipewire-0 → PipeWire → Hardware
                            ↑
                      No permissions needed!
```

---

## TRUE PRIMAL Philosophy

Following the principle:
> "Primals are self-stable, then network, then externals.  
> Externals should have pure Rust mirrors we evolve."

**Our Approach**:
1. ✅ **Self-Stable**: Audio Canvas (pure Rust, works standalone)
2. 🚧 **Network**: PipeWire client (pure Rust, to be evolved)
3. 📋 **Graceful**: Silent mode (visual-only fallback)

We ship with Audio Canvas (self-stable) and evolve PipeWire client over time.

---

## Testing After Enable

```bash
# Build release
cargo build --release

# Run petalTongue
./target/release/petal-tongue

# You should see:
# - GUI window opens
# - Signature tone plays (C-E-G chord)
# - Startup music plays (Welcome Home Morning Star)
# - Bidirectional proprioception active
```

---

## Troubleshooting

### Audio doesn't play after adding group:
- **Solution**: Logout and login (or reboot)
- Group changes require new login session

### Permission denied on /dev/snd/pcmC0D0p:
- **Check**: `groups | grep audio`
- **If missing**: Run usermod command again
- **If present**: Logout/login required

### No sound but no errors:
- **Check**: PulseAudio/PipeWire running
  ```bash
  ps aux | grep -E "pulseaudio|pipewire"
  ```
- **Check**: System volume not muted
- **Check**: Correct output device selected

---

## Next Steps

1. **Immediate**: Add audio group and test
2. **Short-term**: Use and enjoy petalTongue with audio!
3. **Long-term**: We'll evolve pure Rust PipeWire client (2-4 weeks)

The Audio Canvas is production-ready **TODAY**! 🎵✨

---

**Status**: ✅ Ready to enable  
**Time**: 5 minutes  
**Pure Rust**: 100% ✅  
**Evolution**: Documented and planned 🚧

