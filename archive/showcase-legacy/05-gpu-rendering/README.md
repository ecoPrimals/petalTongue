# GPU Rendering Discovery Showcase
## petalTongue discovers Toadstool for GPU rendering

**Purpose:** Demonstrate capability-based discovery for GPU rendering via inter-primal collaboration

---

## 🎯 What This Demonstrates

**TRUE PRIMAL Architecture:**
- ✅ petalTongue knows ONLY itself
- ✅ Toadstool discovered via Songbird
- ✅ Capability-based routing ("GPU rendering" not "Toadstool")
- ✅ Graceful fallback to self-contained rendering

**Inter-Primal Flow:**
```
petalTongue → Songbird → Toadstool
    ↓            ↓           ↓
"Need GPU"  "Who has?"  "I do!"
```

---

## 📋 Prerequisites

### Running Primals

1. **Songbird** (Discovery)
```bash
cd ../../../../phase1/songbird
cargo run --release
# Listens on port 8081 (self-knowledge)
```

2. **Toadstool** (GPU Compute)
```bash
cd ../../../../phase1/toadstool
cargo run --bin toadstool-daemon
# Listens on port 8084 (self-knowledge)
# Registers GPU capability with Songbird
```

3. **petalTongue** (Visualization)
```bash
cd ../..
cargo run --bin petal-tongue
# Discovers GPU rendering via Songbird
# Falls back to pure Rust if not found
```

---

## 🚀 Running the Showcase

### Option 1: Automated (Recommended)

```bash
./demo.sh
```

This will:
1. Start Songbird (discovery primal)
2. Start Toadstool (registers GPU capability)
3. Start petalTongue (discovers GPU rendering)
4. Show inter-primal communication
5. Render topology via discovered GPU

### Option 2: Manual

```bash
# Terminal 1: Songbird
cd ../../../../phase1/songbird
RUST_LOG=info cargo run --release

# Terminal 2: Toadstool
cd ../../../../phase1/toadstool
RUST_LOG=info cargo run --bin toadstool-daemon

# Terminal 3: petalTongue (wait 2 seconds for registration)
cd ../../phase2/petalTongue
SHOWCASE_GPU_RENDERING=1 cargo run --bin petal-tongue

# Terminal 4: Watch discovery
curl http://localhost:8081/primals/capabilities/gpu-rendering
```

---

## 🔍 What to Observe

### 1. Zero Hardcoded Knowledge

**petalTongue logs:**
```
🌸 petalTongue starting...
🔍 Discovering rendering capabilities...
📡 Querying Songbird for 'gpu-rendering' capability...
✅ Found provider: <discovered-primal-id>
🎨 Connecting to discovered GPU renderer...
✅ GPU rendering enabled!
```

**Note:** petalTongue never says "Toadstool"!

### 2. Songbird Routing

**Songbird logs:**
```
📡 Received capability query: gpu-rendering
🔍 Found 1 provider(s)
📤 Returning provider info: toadstool-<uuid>
```

### 3. Toadstool Execution

**Toadstool logs:**
```
🍄 Toadstool daemon started
📡 Registering capabilities with Songbird...
✅ Registered capability: gpu-rendering
🎨 Received rendering request from petalTongue
🖥️  GPU backend: WebGPU (or Vulkan/OpenCL/CPU)
✅ Rendered 47 nodes, 89 edges in 12ms
```

### 4. Graceful Fallback

If Songbird or Toadstool unavailable:

```
🌸 petalTongue starting...
🔍 Discovering rendering capabilities...
⚠️  Songbird not available (connection refused)
ℹ️  Falling back to self-contained rendering
🎨 Using pure Rust canvas renderer (tiny-skia)
✅ Rendering complete (software mode)
```

---

## 📊 Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                     SHOWCASE FLOW                       │
└─────────────────────────────────────────────────────────┘

Step 1: Toadstool Registration
┌──────────┐         ┌──────────┐
│Toadstool │ ------> │ Songbird │
└──────────┘         └──────────┘
   "I can do         "Registered:
    GPU rendering"    Toadstool = GPU"

Step 2: petalTongue Discovery
┌──────────┐         ┌──────────┐
│petalTongue -----> │ Songbird │
└──────────┘         └──────────┘
   "Who can do       "Toadstool can!"
    GPU rendering?"   (returns address)

Step 3: Direct Communication
┌──────────┐         ┌──────────┐
│petalTongue <-----> │Toadstool │
└──────────┘         └──────────┘
   "Render graph"    "Here's pixels"
   (sends topology)   (returns render)

Step 4: Fallback (if needed)
┌──────────┐         ┌──────────┐
│petalTongue         │ Canvas   │
└──────────┘         └──────────┘
   No GPU found?     Uses tiny-skia
   → Self-render     (pure Rust)
```

---

## 🧪 Testing Scenarios

### Scenario 1: Full Stack Available ✅

**Setup:** Songbird + Toadstool running  
**Expected:** petalTongue uses discovered GPU rendering  
**Verify:**
```bash
curl http://localhost:8081/primals | jq '.[] | select(.capabilities | contains(["gpu-rendering"]))'
```

### Scenario 2: Songbird Down ⚠️

**Setup:** Stop Songbird, keep Toadstool  
**Expected:** petalTongue falls back to self-rendering  
**Verify:** petalTongue logs show "Falling back to self-contained"

### Scenario 3: Toadstool Down ⚠️

**Setup:** Songbird running, Toadstool stopped  
**Expected:** Songbird returns empty, petalTongue falls back  
**Verify:**
```bash
curl http://localhost:8081/primals/capabilities/gpu-rendering
# Returns: []
```

### Scenario 4: All Down (Air-Gapped) ✅

**Setup:** No network, no other primals  
**Expected:** petalTongue works perfectly (pure Rust)  
**Verify:** Renders using tiny-skia, no network calls

---

## 📝 Code Example

### petalTongue: Discovery Code

```rust
// src/rendering/discovery.rs

/// Discover GPU rendering capability (no hardcoded primals!)
pub async fn discover_gpu_renderer() -> Result<Option<DiscoveredRenderer>> {
    // Step 1: Find Songbird via mDNS/env
    let songbird = SongbirdClient::discover().await?;
    
    // Step 2: Query for capability (NOT a specific primal)
    let providers = songbird
        .find_capability("gpu-rendering")
        .await?;
    
    if let Some(provider) = providers.first() {
        tracing::info!(
            "🎨 Discovered GPU rendering provider: {}",
            provider.id  // Could be Toadstool, could be anything!
        );
        
        // Step 3: Connect to discovered provider
        let renderer = DiscoveredRenderer::connect(provider).await?;
        return Ok(Some(renderer));
    }
    
    tracing::info!("ℹ️  No GPU rendering providers found");
    Ok(None)
}

/// Rendering with graceful fallback
pub async fn create_renderer() -> Result<Box<dyn UniversalUI>> {
    // Try to discover GPU renderer
    if let Some(gpu_renderer) = discover_gpu_renderer().await? {
        tracing::info!("✅ Using discovered GPU renderer");
        return Ok(Box::new(gpu_renderer));
    }
    
    // Fallback to self-contained
    tracing::info!("✅ Using self-contained renderer");
    Ok(Box::new(CanvasUI::new(1920, 1080)))
}
```

### Toadstool: Capability Registration

```rust
// src/capabilities.rs

/// Register GPU rendering capability with Songbird
pub async fn register_capabilities() -> Result<()> {
    // Discover Songbird (no hardcoding!)
    let songbird = SongbirdClient::discover().await?;
    
    // Register what WE can do (self-knowledge)
    songbird.register_capability(CapabilityInfo {
        name: "gpu-rendering".to_string(),
        provider_id: self_id(),  // Our own ID
        endpoint: self_endpoint(),  // Our own address
        metadata: json!({
            "backends": ["webgpu", "vulkan", "opencl", "cpu"],
            "max_resolution": "4K",
            "formats": ["rgba", "bgra"],
        }),
    }).await?;
    
    tracing::info!("✅ Registered GPU rendering capability");
    Ok(())
}
```

---

## 🎓 Key Lessons

### 1. Self-Knowledge Only ✅

**Each primal knows ONLY itself:**
- petalTongue: "I visualize, I need rendering"
- Toadstool: "I compute, I can render"
- Songbird: "I route, I connect capabilities"

### 2. Capability-Based Discovery ✅

**Query by capability, not by primal:**
```rust
// ❌ BAD
find_primal("toadstool")

// ✅ GOOD
find_capability("gpu-rendering")
```

### 3. Graceful Degradation ✅

**Always have a fallback:**
```
Discovered GPU → Pure Rust GPU → Software → Terminal
   (best)         (fallback 1)    (fallback 2)  (always)
```

### 4. Dev Knowledge vs Runtime ✅

**Showcase contains dev knowledge:**
- We KNOW Toadstool provides GPU rendering
- We KNOW to start both for the demo
- This is SHOWCASE knowledge, not production code

**Production has zero knowledge:**
- petalTongue discovers what's available
- Works alone or with discovered primals
- No assumptions about ecosystem

---

## 🎯 Success Criteria

### Must Demonstrate

- [ ] petalTongue starts without Toadstool knowledge
- [ ] Discovery happens via Songbird query
- [ ] GPU rendering works when discovered
- [ ] Graceful fallback when not available
- [ ] Zero hardcoded primal references in code

### Should Show

- [ ] Render performance comparison (GPU vs software)
- [ ] Network traffic (Songbird → Toadstool routing)
- [ ] Logs showing discovery flow
- [ ] Capability metadata exchange

### Nice to Have

- [ ] Multiple GPU providers (competition)
- [ ] Provider failure and retry
- [ ] Cache invalidation testing
- [ ] Load balancing across providers

---

## 📚 Related Documentation

- [PETALTONGUE_PURE_RUST_GUI_EVOLUTION.md](../../PETALTONGUE_PURE_RUST_GUI_EVOLUTION.md)
- [TRUE PRIMAL Architecture](../../../../wateringHole/TRUE_PRIMAL_ARCHITECTURE.md)
- [Capability-Based Discovery](../../../../phase1/songbird/docs/CAPABILITY_DISCOVERY.md)
- [Toadstool GPU Capabilities](../../../../phase1/toadstool/docs/gpu-compute/GPU_CAPABILITIES.md)

---

## 🌸 Philosophy

**"Knowledge of other primals is dev knowledge."**

This showcase demonstrates inter-primal collaboration for development and education. In production:

- Each primal runs independently
- Discovery happens at runtime
- No primal assumes others exist
- Everything works alone (Tier 1)
- Enhancements discovered dynamically (Tier 3)

**TRUE PRIMAL: Self-knowledge + Runtime discovery = Universal adaptability** 🌸🍄

---

**Ready to run?**

```bash
./demo.sh
```

*Watch capability-based discovery in action!* ✨

