# Audio Substrate-Agnostic Architecture - TRUE PRIMAL Evolution

**Date**: January 13, 2026  
**Status**: 🎯 Architecture Design  
**Principle**: Evolve from hardcoded OS APIs to true substrate agnosticism

---

## 🎯 The Problem

**Current Approach (First Iteration)**:
```rust
// Still hardcoding external systems!
match platform {
    Linux => use_pipewire_or_pulseaudio_or_alsa(),  // ❌ Hardcoded Linux APIs
    macOS => use_coreaudio(),                        // ❌ Hardcoded macOS API
    Windows => use_wasapi(),                         // ❌ Hardcoded Windows API
}
```

**Issues**:
- ❌ Hardcodes PipeWire, PulseAudio, ALSA, CoreAudio, WASAPI
- ❌ These are **external systems outside our control**
- ❌ They can change, break, or disappear (like ALSA → PipeWire evolution)
- ❌ Not truly substrate-agnostic (assumes specific OS audio stacks)
- ❌ Violates TRUE PRIMAL principle: "discover at runtime, don't hardcode"

---

## ✅ The Solution: Learn from Display Backend Pattern

**petalTongue already solved this for DISPLAYS!**

### Display Manager Pattern (Proven Success)

Located in `crates/petal-tongue-ui/src/display/manager.rs`:

```rust
pub struct DisplayManager {
    backends: Vec<Box<dyn DisplayBackend>>,
    active_backend_idx: Option<usize>,
}

impl DisplayManager {
    pub async fn init() -> Result<Self> {
        let mut backends = Vec::new();
        
        // Tier 1: Network (Toadstool WASM)
        if let Ok(toadstool) = ToadstoolDisplay::discover().await {
            backends.push(Box::new(toadstool));
        }
        
        // Tier 2: Software (Pure Rust, always works)
        if SoftwareDisplay::is_available() {
            backends.push(Box::new(SoftwareDisplay::new()));
        }
        
        // Tier 3: Framebuffer (Direct /dev/fb0)
        if FramebufferDisplay::is_available() {
            backends.push(Box::new(FramebufferDisplay::new()));
        }
        
        // Tier 4: External (X11/Wayland/Windows/macOS)
        if ExternalDisplay::is_available() {
            backends.push(Box::new(ExternalDisplay::new()));
        }
        
        // Graceful degradation: Pick best available
        backends.sort_by_key(|entry| entry.priority);
        Ok(Self { backends, ... })
    }
}
```

**Why This Works**:
- ✅ No hardcoding of specific backends
- ✅ Runtime discovery
- ✅ Graceful fallback
- ✅ Extensible (new backends = new impl)
- ✅ Substrate-agnostic (works on ANY platform)

---

## 🏗️ Proposed: Audio Manager Pattern

**Mirror the Display Manager pattern for audio!**

### Core Architecture

```rust
/// Audio Manager - Substrate-agnostic audio coordination
pub struct AudioManager {
    backends: Vec<Box<dyn AudioBackend>>,
    active_backend_idx: Option<usize>,
}

impl AudioManager {
    /// Discover ALL available audio backends at runtime
    pub async fn init() -> Result<Self> {
        let mut backends = Vec::new();
        
        // Tier 1: Network Audio (ToadStool primal)
        // Like how we network to ToadStool for GPU rendering!
        if let Ok(toadstool) = ToadstoolAudioBackend::discover().await {
            info!("✅ ToadStool audio synthesis available");
            backends.push(Box::new(toadstool));
        }
        
        // Tier 2: Pure Rust Software Synthesis (always available)
        // Generate tones programmatically (like we do now!)
        info!("✅ Pure Rust software synthesis available");
        backends.push(Box::new(SoftwareSynthesisBackend::new()));
        
        // Tier 3: Socket-Based Audio Servers (runtime discovery)
        // Discover whatever socket-based audio is available
        for socket in discover_audio_sockets().await {
            info!("✅ Socket audio server: {}", socket.name);
            backends.push(Box::new(SocketAudioBackend::new(socket)));
        }
        
        // Tier 4: Direct Device Access (runtime discovery)
        // Discover whatever direct devices exist
        for device in discover_audio_devices() {
            info!("✅ Direct audio device: {}", device.name);
            backends.push(Box::new(DirectAudioBackend::new(device)));
        }
        
        // Tier 5: Silent Mode (graceful degradation)
        info!("✅ Silent mode available (visual-only)");
        backends.push(Box::new(SilentBackend::new()));
        
        // Sort by priority and capabilities
        backends.sort_by_key(|b| b.priority());
        
        Ok(Self { backends, active_backend_idx: None })
    }
}
```

---

## 🔌 Audio Backend Trait (Universal Interface)

**Define WHAT we need, not HOW to do it!**

```rust
/// Universal audio backend trait
/// 
/// Any audio provider implements this - we don't care HOW!
#[async_trait]
pub trait AudioBackend: Send + Sync {
    /// Get backend metadata (for display only!)
    fn metadata(&self) -> BackendMetadata;
    
    /// Get priority (lower = preferred)
    fn priority(&self) -> u8;
    
    /// Check if backend is currently available
    async fn is_available(&self) -> bool;
    
    /// Initialize backend (prepare for playback)
    async fn initialize(&mut self) -> Result<()>;
    
    /// Play audio samples (async, non-blocking)
    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()>;
    
    /// Get capabilities (what can this backend do?)
    fn capabilities(&self) -> AudioCapabilities;
}

/// Backend metadata (display only, never for logic!)
pub struct BackendMetadata {
    pub name: String,           // "ToadStool Audio Synthesis"
    pub backend_type: String,   // "network", "software", "socket", "direct"
    pub description: String,    // Human-readable description
}

/// Audio capabilities (what can this backend do?)
pub struct AudioCapabilities {
    pub can_play: bool,
    pub can_record: bool,
    pub max_sample_rate: u32,
    pub max_channels: u8,
    pub latency_estimate_ms: u32,
}
```

---

## 🌐 Backend Implementations

### 1. ToadStool Audio Backend (Network - Tier 1)

**Like how we network to ToadStool for GPU rendering!**

```rust
pub struct ToadstoolAudioBackend {
    client: ToadstoolClient,
}

impl ToadstoolAudioBackend {
    /// Discover ToadStool via capability-based discovery
    pub async fn discover() -> Result<Self> {
        // Use existing primal discovery (like Songbird!)
        let client = ToadstoolClient::discover_by_capability("audio.synthesis").await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl AudioBackend for ToadstoolAudioBackend {
    fn priority(&self) -> u8 { 10 } // Highest priority - GPU acceleration!
    
    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        // Send audio generation workload to ToadStool
        let request = AudioGenerationRequest {
            samples: samples.to_vec(),
            sample_rate,
            format: AudioFormat::F32,
        };
        
        self.client.submit_workload(Workload::AudioPlayback(request)).await?;
        Ok(())
    }
    
    fn capabilities(&self) -> AudioCapabilities {
        AudioCapabilities {
            can_play: true,
            can_record: false,
            max_sample_rate: 192000,  // High-end!
            max_channels: 8,          // Surround sound!
            latency_estimate_ms: 50,  // Network latency
        }
    }
}
```

---

### 2. Pure Rust Software Synthesis (Tier 2 - Always Available)

**100% Pure Rust, works EVERYWHERE!**

```rust
pub struct SoftwareSynthesisBackend {
    sample_buffer: Vec<f32>,
}

impl SoftwareSynthesisBackend {
    pub fn new() -> Self {
        Self { sample_buffer: Vec::new() }
    }
}

#[async_trait]
impl AudioBackend for SoftwareSynthesisBackend {
    fn priority(&self) -> u8 { 50 } // Middle priority
    
    async fn is_available(&self) -> bool {
        true // ALWAYS available (Pure Rust!)
    }
    
    async fn play_samples(&mut self, samples: &[f32], _sample_rate: u32) -> Result<()> {
        // Generate audio programmatically (like we do now!)
        self.sample_buffer.extend_from_slice(samples);
        
        // Try to play via next backend in chain
        // (This is a synthesizer, not a speaker!)
        Ok(())
    }
    
    fn capabilities(&self) -> AudioCapabilities {
        AudioCapabilities {
            can_play: true,
            can_record: false,
            max_sample_rate: 48000,
            max_channels: 2,
            latency_estimate_ms: 10,
        }
    }
}
```

---

### 3. Socket Audio Backend (Tier 3 - Runtime Discovery)

**Discovers socket-based audio servers at runtime!**

```rust
pub struct SocketAudioBackend {
    socket: DiscoveredSocket,
    connection: Option<UnixStream>,
}

impl SocketAudioBackend {
    /// Create from discovered socket
    pub fn new(socket: DiscoveredSocket) -> Self {
        Self { socket, connection: None }
    }
    
    /// Discover ALL socket-based audio servers
    /// (PipeWire, PulseAudio, or future unknowns!)
    pub async fn discover_all() -> Vec<DiscoveredSocket> {
        let mut sockets = Vec::new();
        
        // Pattern 1: XDG_RUNTIME_DIR sockets
        if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
            // Check for common patterns (but don't hardcode!)
            for entry in fs::read_dir(&runtime_dir).ok().into_iter().flatten() {
                let path = entry.ok()?.path();
                
                // Look for anything that looks like an audio socket
                if is_audio_socket(&path) {
                    sockets.push(DiscoveredSocket {
                        path,
                        detected_type: detect_socket_type(&path),
                    });
                }
            }
        }
        
        // Pattern 2: Well-known locations (discovery, not hardcoding!)
        for candidate in AUDIO_SOCKET_CANDIDATES {
            if candidate.exists() && is_audio_socket(candidate) {
                sockets.push(DiscoveredSocket {
                    path: candidate.to_path_buf(),
                    detected_type: detect_socket_type(candidate),
                });
            }
        }
        
        sockets
    }
}

#[async_trait]
impl AudioBackend for SocketAudioBackend {
    fn priority(&self) -> u8 { 30 } // Higher priority than direct
    
    async fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<()> {
        // Connect to socket if not connected
        if self.connection.is_none() {
            self.connection = Some(UnixStream::connect(&self.socket.path).await?);
        }
        
        // Send audio data via socket protocol
        // (Protocol negotiation happens at runtime!)
        self.send_audio_data(samples, sample_rate).await?;
        Ok(())
    }
}

/// Helper: Detect if path is an audio socket (runtime heuristics)
fn is_audio_socket(path: &Path) -> bool {
    // Check metadata, name patterns, etc.
    // This is discovery, not hardcoding!
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    
    // Common patterns (but we adapt to new ones!)
    name.contains("audio") || 
    name.contains("sound") || 
    name.contains("pipewire") || 
    name.contains("pulse") ||
    // ... extensible pattern matching
}
```

---

### 4. Direct Device Backend (Tier 4 - Runtime Discovery)

**Discovers direct hardware access at runtime!**

```rust
pub struct DirectAudioBackend {
    device: DiscoveredDevice,
    file: Option<File>,
}

impl DirectAudioBackend {
    /// Discover ALL direct audio devices
    pub fn discover_all() -> Vec<DiscoveredDevice> {
        let mut devices = Vec::new();
        
        // Pattern 1: Linux /dev/snd
        if let Ok(entries) = fs::read_dir("/dev/snd") {
            for entry in entries.flatten() {
                let path = entry.path();
                if is_playback_device(&path) {
                    devices.push(DiscoveredDevice {
                        path,
                        device_type: DeviceType::Pcm,
                    });
                }
            }
        }
        
        // Pattern 2: macOS /dev/audio
        if Path::new("/dev/audio").exists() {
            devices.push(DiscoveredDevice {
                path: PathBuf::from("/dev/audio"),
                device_type: DeviceType::Audio,
            });
        }
        
        // Pattern 3: FreeBSD /dev/dsp
        if Path::new("/dev/dsp").exists() {
            devices.push(DiscoveredDevice {
                path: PathBuf::from("/dev/dsp"),
                device_type: DeviceType::Dsp,
            });
        }
        
        // Extensible: New platforms add new patterns!
        
        devices
    }
}

#[async_trait]
impl AudioBackend for DirectAudioBackend {
    fn priority(&self) -> u8 { 40 } // Lower priority than sockets
    
    async fn play_samples(&mut self, samples: &[f32], _sample_rate: u32) -> Result<()> {
        // Open device if not open
        if self.file.is_none() {
            self.file = Some(File::create(&self.device.path)?);
        }
        
        // Write samples directly (like AudioCanvas!)
        write_samples_to_device(self.file.as_mut().unwrap(), samples)?;
        Ok(())
    }
}
```

---

### 5. Silent Backend (Tier 5 - Graceful Degradation)

**Always available, never fails!**

```rust
pub struct SilentBackend;

#[async_trait]
impl AudioBackend for SilentBackend {
    fn priority(&self) -> u8 { 255 } // Lowest priority (fallback)
    
    async fn is_available(&self) -> bool {
        true // ALWAYS available!
    }
    
    async fn play_samples(&mut self, _samples: &[f32], _sample_rate: u32) -> Result<()> {
        // Do nothing (silent mode)
        debug!("🔇 Silent mode - audio playback skipped");
        Ok(())
    }
    
    fn capabilities(&self) -> AudioCapabilities {
        AudioCapabilities {
            can_play: false,
            can_record: false,
            max_sample_rate: 0,
            max_channels: 0,
            latency_estimate_ms: 0,
        }
    }
}
```

---

## 🔄 Usage Pattern (Like Display Manager!)

```rust
// Initialize audio system (discovers ALL backends)
let mut audio_manager = AudioManager::init().await?;

// Play audio (manager picks best available backend)
let samples = generate_tone(440.0, 1.0); // 440 Hz, 1 second
audio_manager.play_samples(&samples, 44100).await?;

// Backend selection is automatic:
// 1. Try ToadStool (network, GPU-accelerated)
// 2. Try Socket (PipeWire/PulseAudio/etc.)
// 3. Try Direct (/dev/snd, /dev/audio, etc.)
// 4. Fall back to Silent (visual-only)
```

---

## 🌟 Benefits of This Architecture

### 1. True Substrate Agnosticism
- ✅ No hardcoded OS APIs (PipeWire, CoreAudio, WASAPI)
- ✅ Discovers whatever is available at runtime
- ✅ Works on Linux, macOS, Windows, FreeBSD, future platforms
- ✅ Adapts to new audio systems without code changes

### 2. TRUE PRIMAL Compliance
- ✅ Runtime discovery (not build-time dependencies)
- ✅ Capability-based (not name-based)
- ✅ Graceful degradation (always works)
- ✅ Zero hardcoding of external systems

### 3. Extensibility
- ✅ New backends = new `impl AudioBackend`
- ✅ No changes to core logic
- ✅ Users can add custom backends
- ✅ Future-proof architecture

### 4. Alignment with biomeOS Needs
- ✅ **Installer UI**: Silent mode for headless installs
- ✅ **Multi-Modal**: Audio + visual for accessibility
- ✅ **LiveSpore**: Works on USB boot (no system audio)
- ✅ **Federation**: ToadStool audio over network

---

## 📋 Implementation Plan

### Phase 1: Core Abstraction (Week 1)
1. ✅ Define `AudioBackend` trait
2. ✅ Create `AudioManager` (mirrors `DisplayManager`)
3. ✅ Implement `SilentBackend` (always works)
4. ✅ Implement `SoftwareSynthesisBackend` (pure Rust)

### Phase 2: Discovery Layer (Week 2)
1. ✅ Socket discovery (generic, not hardcoded to PipeWire/Pulse)
2. ✅ Device discovery (generic, not hardcoded to /dev/snd)
3. ✅ Priority-based backend selection
4. ✅ Integration tests (all platforms)

### Phase 3: Network Integration (Week 3)
1. ✅ ToadStool audio backend
2. ✅ Primal discovery integration
3. ✅ Capability negotiation
4. ✅ E2E tests with biomeOS

### Phase 4: Platform Testing (Week 4)
1. ✅ Test on Linux (PipeWire, PulseAudio, ALSA, none)
2. ✅ Test on macOS (CoreAudio, /dev/audio, none)
3. ✅ Test on Windows (WASAPI, none)
4. ✅ Test on FreeBSD (/dev/dsp)

---

## 🎯 Success Criteria

- ✅ **Zero hardcoded OS APIs** in production code
- ✅ **Runtime discovery** of all audio backends
- ✅ **Works on Linux, macOS, Windows** without platform-specific builds
- ✅ **Graceful degradation** to silent mode if no audio
- ✅ **ToadStool integration** for network audio
- ✅ **biomeOS installer** works (silent mode + visual)
- ✅ **100% pure Rust** (no C dependencies)

---

## 🔮 Future Evolution

### Substrate Evolution
As new platforms emerge (Redox, Fuchsia, WebAssembly, embedded), they just need to:
1. Provide sockets OR devices
2. Audio manager discovers them at runtime
3. Everything just works ✅

### Primal Evolution
- **BearDog**: Encrypted audio tunnels
- **Songbird**: Federated audio (multi-node playback)
- **NestGate**: Audio recording/storage
- **Squirrel**: AI audio analysis

---

## 🌸 Bottom Line

**We don't hardcode external systems. We discover capabilities at runtime.**

Just like how:
- We don't hardcode "Songbird" - we discover `service.registry` capability
- We don't hardcode "X11" - we discover display backends
- We don't hardcode "BearDog" - we discover `encryption` capability

**Audio should be the same!**

```rust
// ❌ DON'T: Hardcode external systems
if cfg!(target_os = "linux") {
    use_pipewire(); // External system!
}

// ✅ DO: Discover capabilities at runtime
let backends = AudioManager::discover_all().await;
// Might find: ToadStool, socket, device, or silent
// We adapt to WHATEVER exists!
```

---

**Status**: 🎯 Architecture defined, ready to implement  
**Timeline**: 4 weeks (mirrors display backend evolution)  
**Confidence**: 🌟🌟🌟🌟🌟 (proven pattern from display backends)

---

🌸 **petalTongue: True Substrate Agnosticism, Every Platform, Every Workload** 🌸

