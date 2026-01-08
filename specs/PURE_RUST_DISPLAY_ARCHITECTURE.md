# Pure Rust Display Architecture Specification

**Version**: 0.3.0  
**Date**: January 8, 2026  
**Status**: 🚧 In Progress

## Overview

Complete the Pure Rust GUI evolution by implementing display capabilities that work **without** traditional display servers (X11/Wayland). The awakening experience and full GUI functionality should work everywhere.

## The Gap

### Current State
```
EguiGUI → eframe → OpenGL → X11/Wayland ❌ (requires display server)
```

### Goal State
```
EguiGUI → Pure Rust Display → Multiple Backends ✅ (sovereign)
```

## Four-Tier Display Strategy

### Tier 1: Toadstool WASM Rendering (Network Effect)
**Priority**: Highest (leverages primal ecosystem)

```
petalTongue → Capability Discovery → Toadstool
                                    ↓
                                 WASM Rendering
                                    ↓
                              Pixel Buffer → petalTongue
                                    ↓
                              Display Locally
```

**Benefits**:
- ✅ Leverages Toadstool's GPU capabilities
- ✅ Network effect (primal collaboration)
- ✅ WASM = portable, sandboxed, fast
- ✅ Toadstool already supports WASM
- ✅ No display server needed on petalTongue host

**Implementation**:
1. Discover Toadstool via capability query (`gpu-rendering`, `wasm`)
2. Send egui rendering commands to Toadstool
3. Toadstool renders to WASM pixel buffer
4. petalTongue receives pixel buffer
5. petalTongue displays via software renderer

### Tier 2: Local Software Rendering (Pure Rust)
**Priority**: High (full sovereignty)

```
EguiGUI → egui rendering → Pixel Buffer (Pure Rust)
                              ↓
                        softbuffer/pixels
                              ↓
                    Display via multiple backends:
                    - VNC server
                    - HTTP/WebSocket stream
                    - Framebuffer (/dev/fb0)
                    - Window (if available)
```

**Libraries**:
- `softbuffer` - Software-rendered pixel buffer
- `pixels` - Alternative pixel buffer
- `tiny-skia` - 2D rendering (already used for PNG)
- `winit` - Window creation (optional)

**Benefits**:
- ✅ No GPU needed
- ✅ No display server needed
- ✅ Pure Rust implementation
- ✅ Works everywhere Rust compiles

### Tier 3: Framebuffer Direct (Linux Console)
**Priority**: Medium (embedded/server use case)

```
EguiGUI → Software Renderer → /dev/fb0 (Linux framebuffer)
```

**Implementation**:
- Write directly to `/dev/fb0` device
- No X11, no Wayland, just raw framebuffer
- Perfect for embedded systems, kiosks, servers
- Requires root or framebuffer permissions

**Crates**:
- `framebuffer` - Direct framebuffer access
- `linux-video` - Video device control

### Tier 4: External Display Fallback (Benchmark)
**Priority**: Low (backwards compatibility)

```
Check for DISPLAY/WAYLAND_DISPLAY
    ↓
If available → Use traditional eframe/OpenGL
    ↓
If not → Prompt user for sudo to start display
    ↓
If declined → Fall back to Tier 1/2/3
```

**Benefits**:
- ✅ Benchmark against native performance
- ✅ User can opt-in to traditional GUI
- ✅ Graceful degradation
- ✅ Educational (shows the difference)

## Architecture

### Display Backend Trait

```rust
/// Pure Rust display backend
#[async_trait]
pub trait DisplayBackend: Send + Sync {
    /// Initialize the display
    async fn init(&mut self) -> Result<()>;
    
    /// Get display dimensions
    fn dimensions(&self) -> (u32, u32);
    
    /// Present a frame (RGBA8 pixel buffer)
    async fn present(&mut self, buffer: &[u8]) -> Result<()>;
    
    /// Check if backend is available
    fn is_available() -> bool;
    
    /// Backend name
    fn name(&self) -> &str;
    
    /// Performance characteristics
    fn capabilities(&self) -> DisplayCapabilities;
}

/// Display capabilities
#[derive(Debug, Clone)]
pub struct DisplayCapabilities {
    pub requires_network: bool,      // Toadstool = true
    pub requires_gpu: bool,           // OpenGL = true
    pub requires_root: bool,          // Framebuffer = true
    pub supports_resize: bool,
    pub max_fps: u32,
    pub latency_ms: u32,
}
```

### Backend Implementations

```rust
/// Tier 1: Toadstool WASM rendering
pub struct ToadstoolDisplay {
    toadstool_endpoint: String,
    wasm_module: Option<Vec<u8>>,
    buffer: Vec<u8>,
}

/// Tier 2: Software rendering
pub struct SoftwareDisplay {
    backend: SoftwareBackend,
    buffer: Vec<u8>,
}

enum SoftwareBackend {
    VNC(VncServer),           // VNC server
    WebSocket(WsServer),      // HTTP/WebSocket stream
    Window(Window),           // Window (if available)
}

/// Tier 3: Framebuffer direct
pub struct FramebufferDisplay {
    fb_device: File,          // /dev/fb0
    fb_info: FixedScreenInfo,
    buffer: Vec<u8>,
}

/// Tier 4: External display server
pub struct ExternalDisplay {
    display_type: ExternalDisplayType,
    window: Option<Window>,
}

enum ExternalDisplayType {
    X11,
    Wayland,
    Windows,
    MacOS,
}
```

### Display Manager

```rust
/// Manages display backend selection and fallback
pub struct DisplayManager {
    backends: Vec<Box<dyn DisplayBackend>>,
    active_backend: Option<usize>,
}

impl DisplayManager {
    /// Discover and initialize best available backend
    pub async fn init() -> Result<Self> {
        let mut backends: Vec<Box<dyn DisplayBackend>> = vec![];
        
        // Tier 1: Try Toadstool first (network effect!)
        if let Ok(toadstool) = ToadstoolDisplay::discover().await {
            info!("🌸 Using Toadstool WASM rendering (primal collaboration)");
            backends.push(Box::new(toadstool));
        }
        
        // Tier 2: Try software rendering
        if SoftwareDisplay::is_available() {
            info!("🎨 Software rendering available (Pure Rust)");
            backends.push(Box::new(SoftwareDisplay::new()));
        }
        
        // Tier 3: Try framebuffer
        if FramebufferDisplay::is_available() {
            info!("🖥️  Framebuffer direct available (Linux console)");
            backends.push(Box::new(FramebufferDisplay::new()?));
        }
        
        // Tier 4: Try external display
        if ExternalDisplay::is_available() {
            info!("🪟 External display server available");
            backends.push(Box::new(ExternalDisplay::new()));
        } else {
            // Prompt user for sudo to start display server
            if prompt_for_display_server()? {
                // User will start display server manually
                info!("⏳ Waiting for display server...");
            }
        }
        
        // Initialize first available backend
        for (idx, backend) in backends.iter_mut().enumerate() {
            if backend.init().await.is_ok() {
                info!("✅ Active display: {}", backend.name());
                return Ok(Self {
                    backends,
                    active_backend: Some(idx),
                });
            }
        }
        
        Err(anyhow!("No display backend available"))
    }
    
    /// Render frame to active backend
    pub async fn present(&mut self, buffer: &[u8]) -> Result<()> {
        if let Some(idx) = self.active_backend {
            self.backends[idx].present(buffer).await
        } else {
            Err(anyhow!("No active display backend"))
        }
    }
    
    /// Fallback to next backend on error
    pub async fn fallback(&mut self) -> Result<()> {
        // Try next backend...
        todo!()
    }
}
```

## Integration with Egui

### Current Integration
```rust
// Current: Direct eframe integration
fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "petalTongue",
        options,
        Box::new(|cc| Ok(Box::new(PetalTongueApp::new(cc)))),
    )
}
```

### New Integration
```rust
// New: Abstracted display backend
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize display manager
    let mut display = DisplayManager::init().await?;
    
    // Create egui context
    let ctx = egui::Context::default();
    
    // Create app
    let mut app = PetalTongueApp::new(&ctx);
    
    // Main loop
    loop {
        // Update egui
        let output = ctx.run(Default::default(), |ctx| {
            app.update(ctx);
        });
        
        // Render to pixel buffer
        let pixels = render_egui_to_buffer(&ctx, &output)?;
        
        // Present via active backend
        display.present(&pixels).await?;
        
        // Handle input, timing, etc.
        tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
    }
}

fn render_egui_to_buffer(ctx: &egui::Context, output: &egui::FullOutput) -> Result<Vec<u8>> {
    // Use egui_glow or custom renderer to convert to pixels
    // This is where we decouple from OpenGL!
    todo!()
}
```

## Toadstool WASM Integration

### Discovery Flow

```rust
/// Discover Toadstool rendering capability
pub async fn discover_toadstool_rendering() -> Result<ToadstoolDisplay> {
    use petal_tongue_ui::universal_discovery::discover_capability;
    
    // Discover via infant discovery pattern
    let endpoints = discover_capability("wasm-rendering").await?;
    
    if endpoints.is_empty() {
        return Err(anyhow!("No Toadstool WASM renderer found"));
    }
    
    // Connect to first available
    let endpoint = &endpoints[0];
    info!("🌸 Found Toadstool WASM renderer at {}", endpoint);
    
    Ok(ToadstoolDisplay::connect(endpoint).await?)
}
```

### Rendering Protocol

```rust
/// Send rendering request to Toadstool
pub async fn render_via_toadstool(
    endpoint: &str,
    width: u32,
    height: u32,
    egui_commands: &[EguiCommand],
) -> Result<Vec<u8>> {
    // 1. Serialize egui rendering commands
    let commands = serde_json::to_vec(egui_commands)?;
    
    // 2. Send to Toadstool via JSON-RPC or TARPC
    let response: RenderResponse = jsonrpc_request(
        endpoint,
        "render_egui_wasm",
        json!({
            "width": width,
            "height": height,
            "commands": commands,
        }),
    ).await?;
    
    // 3. Receive pixel buffer (RGBA8)
    Ok(response.pixels)
}
```

## External Display Prompt

### Sudo Prompt Flow

```rust
/// Prompt user to start display server with sudo
pub fn prompt_for_display_server() -> Result<bool> {
    // Check if display is already available
    if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
        return Ok(false); // Already have display
    }
    
    println!("\n════════════════════════════════════════════════════════════");
    println!("   🪟 No Display Server Detected");
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!("petalTongue can run in multiple display modes:");
    println!();
    println!("  1. ✅ Pure Rust (recommended)");
    println!("     - Software rendering (no GPU needed)");
    println!("     - Works everywhere");
    println!("     - Continues automatically");
    println!();
    println!("  2. 🪟 Traditional GUI (benchmark)");
    println!("     - Requires X11 or Wayland");
    println!("     - Better performance (if available)");
    println!("     - You can start manually with:");
    println!();
    println!("       sudo systemctl start display-manager");
    println!("       # or");
    println!("       startx");
    println!();
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!("Press Enter to continue with Pure Rust rendering...");
    println!("(or start a display server in another terminal)");
    println!();
    
    // Wait for user to press Enter
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    // Give user 5 seconds to start display server
    info!("⏳ Checking for display server (5 seconds)...");
    for i in 0..5 {
        std::thread::sleep(Duration::from_secs(1));
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            info!("✅ Display server detected!");
            return Ok(true);
        }
        info!("⏳ {}...", 5 - i);
    }
    
    info!("📦 No display server found. Using Pure Rust rendering.");
    Ok(false)
}
```

## Performance Benchmarking

### Benchmark All Backends

```rust
/// Benchmark display backend performance
pub async fn benchmark_display_backends() -> Result<BenchmarkReport> {
    let mut report = BenchmarkReport::default();
    
    // Test Toadstool WASM
    if let Ok(mut display) = ToadstoolDisplay::discover().await {
        report.toadstool = Some(benchmark_backend(&mut display).await?);
    }
    
    // Test Software Rendering
    if SoftwareDisplay::is_available() {
        let mut display = SoftwareDisplay::new();
        report.software = Some(benchmark_backend(&mut display).await?);
    }
    
    // Test Framebuffer
    if FramebufferDisplay::is_available() {
        let mut display = FramebufferDisplay::new()?;
        report.framebuffer = Some(benchmark_backend(&mut display).await?);
    }
    
    // Test External Display
    if ExternalDisplay::is_available() {
        let mut display = ExternalDisplay::new();
        report.external = Some(benchmark_backend(&mut display).await?);
    }
    
    Ok(report)
}

async fn benchmark_backend(backend: &mut dyn DisplayBackend) -> Result<BackendBenchmark> {
    backend.init().await?;
    
    let mut fps_samples = vec![];
    let mut latency_samples = vec![];
    
    // Render 100 frames
    for _ in 0..100 {
        let start = Instant::now();
        
        // Create test frame
        let buffer = create_test_frame(backend.dimensions());
        
        // Present
        backend.present(&buffer).await?;
        
        let elapsed = start.elapsed();
        fps_samples.push(1.0 / elapsed.as_secs_f64());
        latency_samples.push(elapsed.as_millis() as u32);
    }
    
    Ok(BackendBenchmark {
        name: backend.name().to_string(),
        avg_fps: fps_samples.iter().sum::<f64>() / fps_samples.len() as f64,
        avg_latency_ms: latency_samples.iter().sum::<u32>() / latency_samples.len() as u32,
        capabilities: backend.capabilities(),
    })
}
```

## Implementation Roadmap

### Phase 1: Toadstool WASM Integration (Week 1)
- [ ] Define rendering protocol (JSON-RPC)
- [ ] Implement ToadstoolDisplay backend
- [ ] Test with Toadstool WASM module
- [ ] Benchmark performance

### Phase 2: Software Rendering (Week 2)
- [ ] Integrate `softbuffer` or `pixels`
- [ ] Implement SoftwareDisplay backend
- [ ] Add VNC server support
- [ ] Add WebSocket streaming
- [ ] Test on various platforms

### Phase 3: Framebuffer Direct (Week 3)
- [ ] Integrate `framebuffer` crate
- [ ] Implement FramebufferDisplay backend
- [ ] Test on Linux console
- [ ] Handle permissions properly

### Phase 4: Integration & Polish (Week 4)
- [ ] Implement DisplayManager
- [ ] Add external display prompt
- [ ] Wire awakening experience
- [ ] Comprehensive benchmarks
- [ ] Documentation

## Success Criteria

- ✅ Awakening experience works without X11/Wayland
- ✅ All backends available as fallbacks
- ✅ Toadstool integration leverages primal network
- ✅ Performance benchmarks document tradeoffs
- ✅ User can opt-in to external display
- ✅ Complete Pure Rust GUI sovereignty

## Benefits

### For Users
- 🌸 GUI works everywhere (SSH, console, servers, embedded)
- 🚀 Leverages primal network for GPU acceleration
- 📊 Multiple backends for different use cases
- 🎯 Graceful degradation and fallback

### For Ecosystem
- 🌿 Demonstrates primal collaboration (Toadstool + petalTongue)
- 🎨 Network effect (more primals = more capabilities)
- 📖 Educational (shows Pure Rust vs traditional)
- 🔧 Reusable by other primals

## Related Specifications

- `PRIMAL_MULTIMODAL_RENDERING_SPECIFICATION.md` - Overall modality system
- `PETALTONGUE_AWAKENING_EXPERIENCE.md` - Awakening sequence
- `INTER_PRIMAL_COMMUNICATION.md` - Toadstool communication
- `INFANT_DISCOVERY_PATTERN.md` - Capability discovery

---

**Status**: 🚧 Ready to implement  
**Priority**: High  
**Estimated Effort**: 4 weeks  
**Dependencies**: Toadstool WASM support (available)

