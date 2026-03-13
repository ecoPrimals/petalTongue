# Infant Discovery Pattern
## Zero Knowledge, Universal Adaptation

**Date:** January 6, 2026  
**Philosophy:** "Code starts with ZERO knowledge, discovers like an infant"

---

## 🎯 Core Principle

**NO HARDCODED KNOWLEDGE**

The code should start knowing NOTHING and discover everything at runtime:
- ❌ No service names (Songbird, Toadstool, k8s, consul)
- ❌ No port numbers (8080, 9001, etc.)
- ❌ No protocols (tarpc, http, etc.) 
- ❌ No vendor names
- ❌ No primal names

Instead, it discovers:
- ✅ "What capabilities exist?"
- ✅ "Who provides what?"
- ✅ "How do I connect?"

---

## ❌ What We Were Doing Wrong

### Hardcoded Primal Names

```rust
// ❌ BAD: Knows about "Songbird"
async fn discover_songbird() -> Result<String> {
    // Hardcoded knowledge!
}

async fn try_songbird_discovery() -> Result<Service> {
    // Assumes Songbird exists!
}
```

### Hardcoded Vendor Names

```rust
// ❌ BAD: Knows about k8s, consul
if is_k8s_environment() {
    discover_via_k8s()
} else if is_consul_environment() {
    discover_via_consul()
}
```

### Hardcoded Ports

```rust
// ❌ BAD: Hardcoded port numbers
let endpoints = vec![
    "http://localhost:8080",  // Songbird?
    "http://localhost:9001",  // Toadstool?
];
```

---

## ✅ Infant Discovery Pattern

### Universal Capability Discovery

```rust
// ✅ GOOD: No assumptions, just capabilities
let discovery = UniversalDiscovery::new();

// Discover rendering (don't care who provides it)
let renderers = discovery.discover_capability("gpu-rendering").await?;

for renderer in renderers {
    // Could be Toadstool, GPU-Prime, CloudRender, anything!
    println!("Found: {} at {}", renderer.id, renderer.endpoint);
}
```

### Environment-Based Configuration

```rust
// ✅ GOOD: Generic environment variables
export GPU_RENDERING_ENDPOINT=tarpc://localhost:9001
export SERVICE_MESH_ENDPOINT=unix:///tmp/discovery.sock
export DISCOVERY_SERVICE_ENDPOINT=http://localhost:8080

// Code discovers these WITHOUT knowing what they are!
```

### Auto-Detection

```rust
// ✅ GOOD: Probe and discover
async fn discover_capability(capability: &str) -> Vec<Service> {
    // Try environment variables
    // Try Unix socket probing
    // Try mDNS discovery
    // Try HTTP probing
    // NO hardcoded names!
}
```

---

## 🔍 Discovery Methods (In Order)

### 1. Environment Variables (FASTEST)

```bash
# Capability-specific
export GPU_RENDERING_ENDPOINT=tarpc://localhost:9001

# Generic service mesh
export SERVICE_MESH_ENDPOINT=unix:///tmp/mesh.sock

# Generic discovery service
export DISCOVERY_SERVICE_ENDPOINT=http://localhost:8080
```

**Pattern:**
- `{CAPABILITY}_ENDPOINT` - Direct connection
- `SERVICE_MESH_ENDPOINT` - Query for capability
- `DISCOVERY_SERVICE_ENDPOINT` - Query for capability

### 2. Unix Socket Probing (PORT-FREE)

```rust
// Probe common locations
/tmp/*.sock
/var/run/*.sock
~/.local/share/*/sockets/*

// Query each socket generically
```

### 3. mDNS Discovery (ZERO-CONFIG)

```rust
// Query for service types
_discovery._tcp.local
_gpu-rendering._tcp.local
_compute._tcp.local

// No hardcoded service names!
```

### 4. HTTP Probing (FALLBACK)

```rust
// Probe common ports
localhost:8080-8090

// Check generic endpoints
/capabilities
/health  
/api/v1/capabilities

// NO assumptions about what's running!
```

---

## 🎓 Examples

### Discovering GPU Rendering

```rust
// INFANT MODE: Zero assumptions
let discovery = UniversalDiscovery::new();
let renderers = discovery.discover_capability("gpu-rendering").await?;

// Could find:
// - Toadstool (if running)
// - GPU-Prime (if installed)
// - CloudRender (if configured)
// - Custom GPU service
// - ANYTHING that provides "gpu-rendering"!
```

### Discovering Service Mesh

```rust
// INFANT MODE: Don't assume "Songbird"
let discovery = UniversalDiscovery::new();
let meshes = discovery.discover_capability("discovery").await?;

// Could find:
// - Songbird (our implementation)
// - Consul
// - etcd
// - k8s service discovery
// - Custom mesh
// - ANYTHING that provides "discovery"!
```

### Discovering Storage

```rust
// INFANT MODE: Don't assume "NestGate"
let discovery = UniversalDiscovery::new();
let storage = discovery.discover_capability("storage").await?;

// Could find:
// - NestGate (our implementation)
// - MinIO
// - S3
// - Local filesystem
// - ANYTHING that provides "storage"!
```

---

## 🏗️ Universal Adapter Pattern

### Each Primal Uses Same Pattern

```rust
// petalTongue discovers rendering
let discovery = UniversalDiscovery::new();
let renderer = discovery.discover_capability("gpu-rendering").await?;

// Songbird discovers services (uses SAME pattern!)
let discovery = UniversalDiscovery::new();
let services = discovery.discover_capability("storage").await?;

// Toadstool discovers compute resources (SAME pattern!)
let discovery = UniversalDiscovery::new();
let compute = discovery.discover_capability("gpu-compute").await?;
```

**Result:** Universal pattern, NO hardcoding anywhere!

---

## 📊 Comparison

### Before (Hardcoded)

```rust
// ❌ Knows about Songbird
let songbird = connect_to_songbird().await?;

// ❌ Knows about Toadstool  
let toadstool = songbird.find_toadstool().await?;

// ❌ Knows about port
let renderer = ToadstoolClient::connect("localhost:9001")?;
```

### After (Infant Discovery)

```rust
// ✅ Zero knowledge
let discovery = UniversalDiscovery::new();

// ✅ Discover by capability
let renderers = discovery.discover_capability("gpu-rendering").await?;

// ✅ Connect to whatever we found
if let Some(renderer) = renderers.first() {
    let client = connect_to_service(renderer).await?;
    // Don't know or care what it is!
}
```

---

## 🎯 Benefits

### 1. True Sovereignty ✅

- No dependency on specific services
- No vendor lock-in
- Works with ANY provider

### 2. Future-Proof ✅

- New providers work automatically
- No code changes needed
- Backwards compatible

### 3. Deployment Flexibility ✅

- Simple deployments (direct)
- Complex deployments (mesh)
- Custom deployments (anything!)

### 4. Zero Configuration ✅

- mDNS auto-discovery
- Socket probing
- Works out of the box

---

## 🚀 Migration Path

### Step 1: Remove Hardcoded Names

```diff
- async fn discover_songbird() -> Result<String>
+ async fn discover_service_mesh() -> Result<String>

- async fn try_songbird_discovery()
+ async fn try_mesh_discovery()

- let songbird = SongbirdClient::new()
+ let mesh = ServiceMeshClient::new()
```

### Step 2: Remove Hardcoded Ports

```diff
- let endpoint = "http://localhost:8080";
+ let endpoint = std::env::var("SERVICE_MESH_ENDPOINT")?;

- let ports = vec![8080, 9001];
+ let ports = discover_active_ports()?;
```

### Step 3: Add Universal Discovery

```diff
- let renderer = find_toadstool().await?;
+ let renderers = discovery.discover_capability("gpu-rendering").await?;
```

---

## 📝 Configuration Examples

### Simple Deployment

```bash
# Direct connection, no mesh
export GPU_RENDERING_ENDPOINT=tarpc://gpu-server:9001
petal-tongue &
```

### Complex Deployment

```bash
# Use service mesh for discovery
export SERVICE_MESH_ENDPOINT=unix:///tmp/mesh.sock
petal-tongue &
```

### Zero Configuration

```bash
# mDNS auto-discovery
petal-tongue &
# Discovers everything automatically!
```

---

## 🎊 Result

**"Code starts with ZERO knowledge, discovers like an infant."**

- ✅ No hardcoded primal names
- ✅ No hardcoded vendor names
- ✅ No hardcoded ports
- ✅ No hardcoded protocols
- ✅ Universal adapter pattern
- ✅ Works with ANY provider
- ✅ TRUE sovereignty

**This is the TRUE PRIMAL way.** 🌸
