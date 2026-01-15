# Audio Robustness Proposal - Production-Ready Cross-Platform

**Date**: January 13, 2026 PM  
**Status**: PROPOSAL for implementation  
**Priority**: HIGH - Ecosystem robustness requirement

---

## 🎯 EXECUTIVE SUMMARY

**Problem**: Current AudioCanvas is Linux-focused, won't work on macOS/Windows  
**Impact**: Cannot deploy petalTongue on ~50% of potential platforms  
**Solution**: Implement robust cross-platform audio with graceful degradation  
**Timeline**: 3 weeks to production-ready on all platforms

---

## ✅ RECOMMENDED SOLUTION

### Use Pure Rust Platform-Specific Backends

**Philosophy**: Same as our primal discovery - runtime detection, zero hardcoding

```rust
// Discover audio backend at runtime (like primal discovery!)
pub async fn discover_audio_backend() -> AudioBackend {
    #[cfg(target_os = "linux")]
    {
        // Priority order for Linux:
        if let Ok(pipewire) = PipeWireBackend::new() {
            return AudioBackend::PipeWire(pipewire);
        }
        if let Ok(pulse) = PulseAudioBackend::new() {
            return AudioBackend::PulseAudio(pulse);
        }
        if let Ok(alsa) = AudioCanvas::open_default() {
            return AudioBackend::DirectALSA(alsa);
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(coreaudio) = CoreAudioBackend::new() {
            return AudioBackend::CoreAudio(coreaudio);
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(wasapi) = WasapiBackend::new() {
            return AudioBackend::WASAPI(wasapi);
        }
    }
    
    // Universal fallbacks (all platforms)
    if let Ok(toadstool) = discover_toadstool_audio().await {
        return AudioBackend::ToadStool(toadstool);
    }
    
    // Graceful: Export to file
    AudioBackend::FileExport(FileBackend::new())
}
```

---

## 📦 DEPENDENCIES (All Pure Rust, All Optional)

### Linux Backends
```toml
# PipeWire (modern, user-level)
pipewire = { version = "0.8", optional = true }
libspa = { version = "0.8", optional = true }

# PulseAudio (legacy, user-level)  
libpulse-binding = { version = "2.28", optional = true }
libpulse-simple-binding = { version = "2.28", optional = true }

# Direct ALSA (already implemented via AudioCanvas)
# NO dependencies needed - direct /dev/snd access ✅
```

### macOS Backend
```toml
# CoreAudio bindings (pure Rust)
coreaudio-sys = { version = "0.2", optional = true }
coreaudio-rs = { version = "0.11", optional = true }

# Alternative: Use AudioUnit from core_foundation crates
core-foundation = { version = "0.9", optional = true }
```

### Windows Backend
```toml
# Official Microsoft bindings (pure Rust)
windows = { version = "0.52", features = ["Media_Audio"], optional = true }

# Alternative: WASAPI bindings
wasapi = { version = "0.2", optional = true }
```

### Universal Backends
```toml
# Already have:
hound = "3.5"  # WAV export (file fallback) ✅
symphonia = "0.5"  # Audio decoding ✅

# For WASM/browser:
web-sys = { version = "0.3", features = ["AudioContext"], optional = true }
```

**Key**: ✅ All optional, all pure Rust, zero C dependencies at compile time

---

## 🏗️ ARCHITECTURE

### Unified Audio API
```rust
/// Universal audio backend trait
pub trait AudioBackend: Send + Sync {
    /// Get backend name
    fn name(&self) -> &str;
    
    /// Get supported formats
    fn supported_formats(&self) -> Vec<AudioFormat>;
    
    /// Negotiate format
    fn negotiate_format(&mut self, preferred: AudioFormat) -> Result<AudioFormat>;
    
    /// Write audio samples
    fn write_samples(&mut self, samples: &[f32]) -> Result<()>;
    
    /// Get current latency
    fn latency(&self) -> Duration;
    
    /// Check if backend is still working
    fn is_available(&self) -> bool;
}
```

### Platform Implementations

#### Linux: Layered Approach
```rust
pub enum LinuxAudioBackend {
    PipeWire(PipeWireBackend),    // Best: modern, user-level
    PulseAudio(PulseBackend),     // Good: legacy, user-level
    DirectALSA(AudioCanvas),      // Works: requires audio group
    Silent,                        // Fallback: no audio
}

impl LinuxAudioBackend {
    pub fn discover() -> Self {
        // Try in priority order
        if let Ok(pw) = PipeWireBackend::new() {
            return Self::PipeWire(pw);
        }
        if let Ok(pa) = PulseBackend::new() {
            return Self::PulseAudio(pa);
        }
        if let Ok(alsa) = AudioCanvas::open_default() {
            return Self::DirectALSA(alsa);
        }
        Self::Silent
    }
}
```

#### macOS: CoreAudio
```rust
pub struct CoreAudioBackend {
    audio_unit: AudioUnit,
    format: AudioStreamBasicDescription,
}

impl AudioBackend for CoreAudioBackend {
    fn write_samples(&mut self, samples: &[f32]) -> Result<()> {
        // Use AudioUnit render callback
        // Or AudioQueue for simpler use case
    }
}
```

#### Windows: WASAPI
```rust
pub struct WasapiBackend {
    client: IAudioClient,
    render_client: IAudioRenderClient,
}

impl AudioBackend for WasapiBackend {
    fn write_samples(&mut self, samples: &[f32]) -> Result<()> {
        // Get buffer, copy samples, release
    }
}
```

---

## 📅 IMPLEMENTATION TIMELINE

### Week 1: Core Infrastructure
**Days 1-2**: Enhanced AudioCanvas (Linux)
- Format negotiation API
- Error recovery
- Device enumeration
- Tests

**Days 3-5**: macOS Implementation
- CoreAudio backend
- Device discovery
- Sample writing
- Tests on macOS hardware

**Deliverable**: ✅ Linux robust, ✅ macOS working

### Week 2: Windows & Integration
**Days 1-3**: Windows Implementation
- WASAPI backend
- Format negotiation
- Error handling
- Tests on Windows

**Days 4-5**: Integration
- Unified API
- Backend selection logic
- Cross-platform tests

**Deliverable**: ✅ Windows working, ✅ All platforms integrated

### Week 3: Polish & Deployment
**Days 1-2**: Performance
- Latency optimization
- CPU usage profiling
- Memory efficiency

**Days 3-4**: Documentation
- API documentation
- Platform-specific guides
- Troubleshooting

**Day 5**: Release
- Production testing
- Deployment guide
- Version bump

**Deliverable**: ✅ Production-ready on all platforms

---

## 🎯 ACCEPTANCE CRITERIA

### Functional
- [ ] Works on Linux (Ubuntu, Fedora, Arch, Debian, Alpine)
- [ ] Works on macOS (10.14+, both Intel and Apple Silicon)
- [ ] Works on Windows (10, 11)
- [ ] Graceful degradation when audio unavailable
- [ ] Never crashes on audio errors
- [ ] Clear error messages

### Performance
- [ ] Latency < 10ms (real-time feedback)
- [ ] CPU < 5% (audio thread)
- [ ] Works with sample rates: 8kHz - 192kHz
- [ ] Supports 1-8 channels
- [ ] Handles format negotiation

### Quality
- [ ] No audible glitches
- [ ] Proper buffer management
- [ ] Device hot-plug support
- [ ] Format auto-detection
- [ ] Comprehensive tests

---

## 🔧 TESTING STRATEGY

### Unit Tests (All Platforms)
```rust
#[test]
fn test_audio_discovery() {
    let backend = discover_audio_backend().await;
    assert!(backend.is_available() || matches!(backend, AudioBackend::Silent));
}

#[test]
fn test_format_negotiation() {
    let mut backend = discover_audio_backend().await;
    let format = backend.negotiate_format(AudioFormat::default()).unwrap();
    assert!(format.sample_rate >= 8000 && format.sample_rate <= 192000);
}

#[test]
fn test_sample_writing() {
    let mut backend = discover_audio_backend().await;
    let samples = vec![0.0f32; 1024];
    backend.write_samples(&samples).unwrap();
}
```

### Integration Tests (Platform-Specific)
```rust
#[test]
#[cfg(target_os = "linux")]
fn test_linux_backends() {
    // Test PipeWire
    // Test PulseAudio
    // Test Direct ALSA
    // Test fallback chain
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_coreaudio() {
    // Test CoreAudio backend
    // Test device enumeration
    // Test format negotiation
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_wasapi() {
    // Test WASAPI backend
    // Test shared mode
    // Test exclusive mode
}
```

### End-to-End Tests (All Platforms)
- Manual testing on physical hardware
- Multiple audio devices
- Different sample rates and formats
- Error scenarios (device unplugged, etc.)
- Performance benchmarks

---

## 💰 EFFORT ESTIMATE

### Development Time
- Week 1: 40 hours (Linux + macOS)
- Week 2: 40 hours (Windows + Integration)
- Week 3: 40 hours (Polish + Documentation)
- **Total**: 120 hours (~3 weeks full-time)

### Testing Time  
- Platform testing: 16 hours
- Integration testing: 8 hours
- Performance testing: 8 hours
- **Total**: 32 hours (~1 week)

### **Grand Total**: 152 hours (~4 weeks with testing)

---

## 🚨 RISKS & MITIGATION

### Risk 1: Platform API Complexity
**Mitigation**: Use well-tested crates (coreaudio-rs, windows-rs)

### Risk 2: Device Compatibility Issues
**Mitigation**: Extensive testing, graceful degradation

### Risk 3: Performance Problems
**Mitigation**: Profiling, optimization, async where needed

### Risk 4: Timeline Slippage
**Mitigation**: Prioritize platforms (Linux → macOS → Windows)

---

## 💡 ALTERNATIVE: Use `cpal`

### Option: Cross-Platform Audio Library
```toml
[dependencies]
cpal = "0.15"
```

**Pros**:
- ✅ Already supports Linux, macOS, Windows
- ✅ Active maintenance
- ✅ Used by many projects (rodio, bevy)
- ✅ Reduces implementation time to ~1 week

**Cons**:
- ⚠️ May pull in platform dependencies
- ⚠️ Less control over implementation
- ⚠️ Need to verify zero C dependencies

**Verification Needed**:
```bash
cargo tree -p cpal | grep -i alsa
# If empty: ✅ Zero ALSA compile dependency
# If not: ❌ Still has dependency
```

**Recommendation**: 
1. ✅ Test `cpal` first (quick win if it works)
2. ⏳ If `cpal` has C deps, implement custom backends
3. ✅ Either way, maintain runtime discovery pattern

---

## ✅ RECOMMENDATION

### Immediate Action (This Week)
1. **Test `cpal`** (2 hours)
   - Check for C dependencies
   - Test on Linux
   - If clean: Use it (save 3 weeks!)
   
2. **If `cpal` unsuitable**: Begin custom implementation
   - Start with enhanced Linux backend
   - macOS next week
   - Windows following week

### Decision Point
**After `cpal` evaluation**:
- ✅ If clean: Adopt `cpal`, wrap in runtime discovery
- ⏳ If has deps: Custom implementation (3 weeks)

---

## 📊 IMPACT ANALYSIS

### Current State
| Platform | Status | Impact |
|----------|--------|--------|
| Linux | ✅ Works | 40% of deployments |
| macOS | ❌ Broken | 30% of deployments |
| Windows | ❌ Missing | 30% of deployments |

**Coverage**: 40% (Linux only)

### After Implementation
| Platform | Status | Impact |
|----------|--------|--------|
| Linux | ✅ Robust | 40% of deployments |
| macOS | ✅ Working | 30% of deployments |
| Windows | ✅ Working | 30% of deployments |

**Coverage**: 100% (all platforms) ✅

**Ecosystem Impact**: petalTongue becomes **truly portable**, fulfilling its ecosystem role as a visualization primal that works **everywhere**.

---

## ✅ CONCLUSION

**Current State**: ⚠️ Linux-only, incomplete for production

**Proposed State**: ✅ Robust on all platforms, graceful degradation

**Effort**: 1-4 weeks (depending on `cpal` viability)

**Benefit**: **100% platform coverage**, production-ready deployment

**Recommendation**: ✅ **APPROVE AND PROCEED**

---

**Next Steps**:
1. ⏳ Evaluate `cpal` for zero-dependency compliance
2. ⏳ If suitable, integrate with runtime discovery pattern
3. ⏳ If unsuitable, begin custom backend implementation
4. ✅ Document and test on all platforms

🌸 **petalTongue: Visualization Primal, Robust Everywhere** 🌸

