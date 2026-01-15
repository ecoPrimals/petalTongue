# Audio System Migration Plan

**Date**: January 13, 2026  
**Goal**: Migrate from old AudioSystem to new substrate-agnostic AudioManager  
**Status**: Planning → Implementation

---

## 🎯 Current State

### Old System (audio_providers.rs)
```rust
pub struct AudioSystem {
    providers: Vec<Box<dyn AudioProvider>>,
    current_provider: usize,
}

impl AudioSystem {
    pub fn new() -> Self { /* ... */ }
    pub fn play_tone(&self, /* ... */) { /* ... */ }
    pub fn play_file(&self, path: &Path) { /* ... */ }
}
```

**Issues**:
- Still uses AudioCanvas directly (hardcoded)
- Not truly substrate-agnostic
- Synchronous API (not async)

### New System (audio/manager.rs)
```rust
pub struct AudioManager {
    backends: Vec<Box<dyn AudioBackend>>,
    active_backend_idx: Option<usize>,
}

impl AudioManager {
    pub async fn init() -> Result<Self> { /* ... */ }
    pub async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> { /* ... */ }
}
```

**Benefits**:
- TRUE substrate agnosticism
- Runtime discovery
- Async API
- Graceful degradation

---

## 🔄 Migration Strategy

### Phase 1: Compatibility Layer (TODAY)
Create adapter that makes AudioManager compatible with existing code.

```rust
// New wrapper for backward compatibility
pub struct AudioSystemV2 {
    manager: Arc<Mutex<AudioManager>>,
    runtime: tokio::runtime::Handle,
}

impl AudioSystemV2 {
    pub fn new() -> Self {
        // Initialize AudioManager with tokio runtime
    }
    
    pub fn play_tone(&self, /* ... */) {
        // Convert to samples, call manager.play_samples()
    }
    
    pub fn play_file(&self, path: &Path) {
        // Load file, call manager.play_samples()
    }
}
```

### Phase 2: Update App (TODAY)
Replace `AudioSystem` with `AudioSystemV2` in app.rs.

```rust
// Old
audio_system: AudioSystem,

// New
audio_system: AudioSystemV2,
```

### Phase 3: Deprecate Old System (NEXT)
Mark old AudioSystem as deprecated, point to new one.

### Phase 4: Remove Old System (FUTURE)
After testing, remove audio_providers.rs, audio_canvas.rs, etc.

---

## 📋 Implementation Checklist

### Today (Phase 1-2)

- [ ] Create `AudioSystemV2` compatibility wrapper
- [ ] Implement synchronous API over async AudioManager
- [ ] Update `PetalTongueApp` to use `AudioSystemV2`
- [ ] Update `startup_audio.rs` to use new system
- [ ] Test compilation
- [ ] Test runtime behavior

### Next Session (Phase 3)

- [ ] Mark old `AudioSystem` as deprecated
- [ ] Add migration warnings
- [ ] Update all callsites
- [ ] Integration testing

### Future (Phase 4)

- [ ] Remove `audio_providers.rs`
- [ ] Remove `audio_canvas.rs` (keep as reference)
- [ ] Remove `audio_discovery.rs` (keep as reference)
- [ ] Clean up dependencies

---

## 🎯 Success Criteria

- ✅ App compiles and runs
- ✅ Startup audio works
- ✅ Audio system discovers backends correctly
- ✅ Graceful degradation works (silent mode)
- ✅ No behavior changes from user perspective

---

Let's proceed!

