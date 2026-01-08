# Inter-Primal Communication Architecture
## TRUE PRIMAL Pattern - January 6, 2026

---

## 🎯 Core Principle

**"Primals communicate directly. Songbird is an enhancement for complex deployments."**

---

## ❌ What We Got Wrong Initially

### Incorrect Architecture (Songbird Dependency)

```
petalTongue → Songbird (REQUIRED) → Toadstool
     ↑                                    ↑
  Helpless!                          Can't talk!
```

**Problem:** petalTongue can't communicate without Songbird!

### What We Accidentally Created

```rust
// ❌ BAD: Requires Songbird to communicate
async fn discover_renderer() -> Result<Option<DiscoveredRenderer>> {
    let songbird = discover_songbird().await?; // BLOCKS if Songbird down!
    // ...
}
```

**Issue:** If Songbird is unavailable, petalTongue can't connect to Toadstool **even if we know where Toadstool is!**

---

## ✅ Correct Architecture (TRUE PRIMAL)

### Three-Tier Communication

```
┌─────────────────────────────────────────────┐
│ Tier 3: Songbird-Enhanced Discovery        │
│ • Complex deployments                       │
│ • Multiple providers                        │
│ • Load balancing                            │
│ • Service mesh                              │
└─────────────────────────────────────────────┘
              ↓ enhances
┌─────────────────────────────────────────────┐
│ Tier 2: Direct Primal-to-Primal (DEFAULT) │
│ • petalTongue has tarpc/jsonrpc client      │
│ • Connects directly to Toadstool            │
│ • Via env vars or config                    │
│ • NO Songbird required                      │
└─────────────────────────────────────────────┘
              ↓ fallback
┌─────────────────────────────────────────────┐
│ Tier 1: Pure Rust Self-Contained           │
│ • No external communication needed          │
│ • tiny-skia rendering                       │
│ • ALWAYS works                              │
└─────────────────────────────────────────────┘
```

---

## 🏗️ Corrected Implementation

### Self-Knowledge: petalTongue's Communication Capabilities

**petalTongue knows:**
- ✅ "I can speak tarpc"
- ✅ "I can speak JSON-RPC"
- ✅ "I can speak HTTP"
- ✅ "I can discover via Songbird (optional)"
- ✅ "I can render myself (fallback)"

**petalTongue does NOT know:**
- ❌ "Toadstool exists" (discovered at runtime)
- ❌ "Toadstool is at port X" (configured or discovered)

### Corrected Discovery Flow

```rust
/// Discover GPU renderer with graceful degradation
pub async fn discover_renderer() -> Result<Option<DiscoveredRenderer>> {
    // TIER 2 (DEFAULT): Direct connection via env/config
    if let Some(direct) = try_direct_connection().await? {
        info!("✅ Using direct GPU connection");
        return Ok(Some(direct));
    }
    
    // TIER 3 (ENHANCEMENT): Songbird discovery
    if let Some(discovered) = try_songbird_discovery().await? {
        info!("✅ Using Songbird-discovered GPU provider");
        return Ok(Some(discovered));
    }
    
    // TIER 1 (FALLBACK): Self-contained
    info!("ℹ️  No GPU provider found, using self-contained rendering");
    Ok(None)
}

/// Try direct connection to known GPU provider
async fn try_direct_connection() -> Result<Option<DiscoveredRenderer>> {
    // Check env vars for direct connection
    if let Ok(endpoint) = std::env::var("GPU_RENDERER_ENDPOINT") {
        debug!("Found GPU_RENDERER_ENDPOINT: {}", endpoint);
        
        // Verify it's reachable
        if ping_endpoint(&endpoint).await.is_ok() {
            return Ok(Some(DiscoveredRenderer {
                provider_id: "direct-connection".to_string(),
                endpoint,
                capabilities: vec!["gpu-rendering".to_string()],
                protocol: detect_protocol(&endpoint),
                metadata: None,
            }));
        }
    }
    
    // Check config file
    // if let Some(config_endpoint) = load_gpu_config()? { ... }
    
    Ok(None)
}

/// Try Songbird discovery (OPTIONAL enhancement)
async fn try_songbird_discovery() -> Result<Option<DiscoveredRenderer>> {
    // Find Songbird (non-blocking)
    let songbird_endpoint = discover_songbird().await.ok()?;
    
    // Query for GPU capability
    query_rendering_capability(&songbird_endpoint).await
        .ok()
        .and_then(|providers| providers.into_iter().next())
        .map(Ok)
        .transpose()
}
```

---

## 📊 Communication Patterns

### Pattern 1: Simple Deployment (2 Primals)

```
petalTongue                    Toadstool
    ↓                              ↑
    └──── tarpc://localhost:9001 ──┘
    
    (Direct connection via GPU_RENDERER_ENDPOINT)
```

**No Songbird needed!** ✅

### Pattern 2: Complex Deployment (Many Primals)

```
             Songbird
           /    |    \
          /     |     \
petalTongue  Toadstool1  Toadstool2
                         Toadstool3
    
    (Songbird provides discovery & load balancing)
```

**Songbird enhances, doesn't block!** ✅

### Pattern 3: Air-Gapped / Offline

```
petalTongue (standalone)
    ↓
  Pure Rust
  (tiny-skia)
  
  (No external communication)
```

**Always works!** ✅

---

## 🔧 Implementation Changes Needed

### 1. Add Direct Communication Client

```rust
// NEW: Direct tarpc client (no Songbird)
pub struct DirectRenderClient {
    endpoint: String,
    client: TarpcClient,  // Our own tarpc client!
}

impl DirectRenderClient {
    pub async fn connect(endpoint: &str) -> Result<Self> {
        // Connect directly to GPU provider
        let client = TarpcClient::new(endpoint)?;
        
        // Verify it supports rendering
        // (but don't ask Songbird!)
        
        Ok(Self {
            endpoint: endpoint.to_string(),
            client,
        })
    }
    
    pub async fn render(&self, graph: &GraphEngine) -> Result<Vec<u8>> {
        // Send graph data, receive pixels
        // Direct primal-to-primal RPC
        todo!()
    }
}
```

### 2. Update Discovery Hierarchy

```rust
/// Discovery with proper graceful degradation
pub async fn discover_renderer() -> Result<RenderingBackend> {
    // 1. Try direct connection (fastest, most reliable)
    if let Some(direct) = try_direct_connection().await? {
        return Ok(RenderingBackend::Direct(direct));
    }
    
    // 2. Try Songbird discovery (for complex deployments)
    if let Some(discovered) = try_songbird_discovery().await? {
        return Ok(RenderingBackend::Discovered(discovered));
    }
    
    // 3. Use self-contained (always available)
    Ok(RenderingBackend::SelfContained(CanvasUI::new()))
}

pub enum RenderingBackend {
    Direct(DirectRenderClient),          // Tier 2
    Discovered(DiscoveredRenderClient),  // Tier 3
    SelfContained(CanvasUI),             // Tier 1
}
```

### 3. Configuration

```toml
# petalTongue.toml (optional config)

[rendering]
# Direct connection (Tier 2)
gpu_endpoint = "tarpc://localhost:9001"  # Direct to Toadstool

# Songbird discovery (Tier 3)
use_songbird = true
songbird_endpoint = "unix:///tmp/songbird.sock"

# Fallback (Tier 1)
software_rendering = true
```

```bash
# Or via environment variables
export GPU_RENDERER_ENDPOINT=tarpc://localhost:9001
export SONGBIRD_ENDPOINT=unix:///tmp/songbird.sock  # optional!
```

---

## 🎓 Key Principles

### 1. Self-Sufficiency First ✅

**Each primal is self-sufficient:**
- petalTongue can render (pure Rust)
- petalTongue can communicate (tarpc/jsonrpc)
- Songbird enhances, doesn't enable

### 2. Direct Communication Default ✅

**Primal-to-primal is the default:**
```
Simple deployment: petalTongue ←→ Toadstool
                   (direct tarpc)
```

**Songbird is for complexity:**
```
Complex deployment: petalTongue → Songbird → Toadstool(s)
                                 (discovery & routing)
```

### 3. Graceful Degradation ✅

**Always have a path that works:**
1. Direct connection (if configured)
2. Songbird discovery (if available)
3. Self-contained (always)

---

## 📝 Inter-Primal Interactions Are Dev Knowledge

### Showcase (Dev Knowledge) ✅

```bash
# showcase/05-gpu-rendering/demo.sh

# WE KNOW (dev knowledge):
# - petalTongue visualizes
# - Toadstool provides GPU
# - Songbird discovers

# So we start them together:
songbird-orchestrator &
toadstool-daemon &
petal-tongue &
```

**This is fine!** It's a **demo** showing the ecosystem.

### Production (Zero Knowledge) ✅

```bash
# Production deployment

# Option A: Simple (2 primals, direct)
export GPU_RENDERER_ENDPOINT=tarpc://toadstool:9001
petal-tongue &

# Option B: Complex (many primals, discovery)
export SONGBIRD_ENDPOINT=unix:///tmp/songbird.sock
petal-tongue &

# Option C: Standalone (air-gapped)
petal-tongue &  # Just works!
```

**Zero hardcoding in code!** Configuration is external.

---

## 🎯 Summary

### What We Fixed

**Before (Songbird Dependency):**
```rust
// ❌ Can't communicate without Songbird
let songbird = discover_songbird().await?;  // BLOCKS!
let renderer = songbird.find_gpu().await?;
```

**After (Direct Communication + Optional Discovery):**
```rust
// ✅ Direct connection first
if let Some(direct) = try_direct_connection().await? {
    return Ok(direct);  // No Songbird needed!
}

// ✅ Songbird enhancement second
if let Some(discovered) = try_songbird_discovery().await? {
    return Ok(discovered);  // Songbird helps!
}

// ✅ Self-contained fallback
Ok(self_contained())  // Always works!
```

### Architecture

```
Tier 3: Songbird Discovery (ENHANCEMENT for complex deployments)
  ↓ enhances
Tier 2: Direct Primal-to-Primal (DEFAULT for simple deployments)
  ↓ fallback
Tier 1: Pure Rust Self-Contained (ALWAYS available)
```

**Result:** TRUE PRIMAL architecture! 🌸

---

**"Primals communicate directly. Songbird enhances, doesn't enable."** 🌸🍄

