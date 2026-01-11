# Audio Sovereignty Evolution - Pure Rust PipeWire

**Date**: January 11, 2026  
**Status**: 🚧 In Progress  
**Goal**: Evolve from direct hardware access to pure Rust PipeWire abstraction

---

## The Evolution

### Current State: Audio Canvas (Direct Hardware)

**Approach**: Direct `/dev/snd/pcmC0D0p` access
- Like: Raw framebuffer `/dev/fb0`
- Requires: `audio` group membership
- Problem: Privilege dependency (not TRUE PRIMAL)

```rust
/dev/snd/pcmC0D0p → Hardware
       ↑
  Requires audio group ❌
```

### Target State: PipeWire Client (Unix Socket)

**Approach**: PipeWire protocol via Unix socket
- Like: Songbird discovery (Unix socket IPC)
- Requires: Nothing! User-level socket
- Solution: Pure Rust, no privileges needed

```rust
/run/user/$UID/pipewire-0 → PipeWire → Hardware
           ↑
    No permissions needed! ✅
```

---

## Architecture

### Discovery Pattern (TRUE PRIMAL)

```rust
Audio Discovery Chain:
1. Search for PipeWire socket: /run/user/$UID/pipewire-0
2. Search for PulseAudio socket: /run/user/$UID/pulse/native
3. Fallback to direct: /dev/snd/pcmC0D0p (if audio group)
4. Graceful degradation: Silent mode (visual-only)
```

### Pattern Alignment

This aligns PERFECTLY with our existing architecture:

```
Songbird Discovery:
  Unix Socket → /run/user/$UID/songbird-nat0.sock

PipeWire Audio:
  Unix Socket → /run/user/$UID/pipewire-0

Same Pattern! Consistent! TRUE PRIMAL! 🌸
```

---

## Implementation Plan

### Phase 1: PipeWire Discovery ✅ (Starting Now)

Create `crates/petal-tongue-ui/src/audio_pipewire_discovery.rs`:

```rust
/// Discover PipeWire/PulseAudio sockets
pub fn discover_audio_sockets() -> Vec<AudioSocket> {
    // Check /run/user/$UID/pipewire-0
    // Check /run/user/$UID/pulse/native
    // Return available sockets
}
```

### Phase 2: PipeWire Protocol Client

Create `crates/petal-tongue-ui/src/audio_pipewire_client.rs`:

```rust
/// Pure Rust PipeWire client (Unix socket)
pub struct PipeWireClient {
    socket: UnixStream,
    // Protocol state
}

impl PipeWireClient {
    pub fn connect(socket_path: &Path) -> Result<Self>;
    pub fn negotiate_stream(&mut self) -> Result<StreamInfo>;
    pub fn write_samples(&mut self, samples: &[f32]) -> Result<()>;
}
```

### Phase 3: Fallback Chain

Update `audio_canvas.rs` to use fallback:

```rust
pub enum AudioBackend {
    PipeWire(PipeWireClient),  // Preferred
    Direct(DirectDevice),       // Fallback
    Silent,                     // Graceful
}

impl AudioBackend {
    pub fn discover() -> Self {
        // Try PipeWire first
        // Fallback to direct
        // Graceful to silent
    }
}
```

---

## Benefits

### TRUE PRIMAL Principles

✅ **Self-Discovery**: Discovers audio at runtime  
✅ **No Hard Dependencies**: Works with or without PipeWire  
✅ **Graceful Degradation**: Falls back gracefully  
✅ **Pure Rust**: No C dependencies  
✅ **No Privileges**: User-level Unix socket  
✅ **Universal**: Works everywhere  

### Pattern Consistency

✅ Same as Songbird discovery (Unix socket)  
✅ Same as IPC architecture (Unix socket)  
✅ Same as biomeOS integration (runtime discovery)  
✅ Modern idiomatic Rust  

---

## Timeline

- **Phase 1**: PipeWire discovery (2-3 hours)
- **Phase 2**: Protocol client (1-2 days)
- **Phase 3**: Integration & testing (1 day)

**Total**: 2-3 days for complete evolution

---

## Success Criteria

✅ Audio plays via PipeWire Unix socket  
✅ No audio group membership required  
✅ Graceful fallback to direct/silent  
✅ 100% Pure Rust  
✅ Works on modern Linux (PipeWire/PulseAudio)  
✅ Maintains backward compatibility  

---

**Status**: 🚧 Phase 1 starting now!

This is deep debt resolution and evolution to modern idiomatic Rust! 🦀✨
