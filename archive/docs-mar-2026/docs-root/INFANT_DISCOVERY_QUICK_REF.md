# Infant Discovery Pattern - Quick Reference
## Zero Knowledge, Runtime Discovery

**Philosophy:** "Code starts with ZERO knowledge, discovers like an infant"

---

## 🚀 Quick Start

```rust
use petal_tongue_ui::universal_discovery::UniversalDiscovery;

// Create discovery (zero configuration needed)
let discovery = UniversalDiscovery::new();

// Discover by capability (not by name!)
let services = discovery.discover_capability("gpu-rendering").await?;

// Use whatever we found
if let Some(service) = services.first() {
    println!("Found: {} at {}", service.id, service.endpoint);
}
```

---

## 🎯 Key Principles

### ❌ NEVER Do This

```rust
// ❌ Hardcoded primal names
let songbird = connect_to_songbird().await?;
let toadstool = songbird.find_toadstool().await?;

// ❌ Hardcoded ports
let endpoint = "http://localhost:8080";

// ❌ Hardcoded vendor names
if is_k8s_environment() { ... }
```

### ✅ ALWAYS Do This

```rust
// ✅ Discover by capability
let discovery = UniversalDiscovery::new();
let services = discovery.discover_capability("gpu-rendering").await?;

// ✅ Use environment variables
let endpoint = std::env::var("GPU_RENDERING_ENDPOINT")?;

// ✅ Auto-detect environment
let meshes = discovery.discover_capability("discovery").await?;
```

---

## 🔍 Discovery Methods

### 1. Environment Variables (Fastest)

```bash
# Capability-specific
export GPU_RENDERING_ENDPOINT=tarpc://localhost:9001

# Generic service mesh
export SERVICE_MESH_ENDPOINT=unix:///tmp/mesh.sock

# Generic discovery
export DISCOVERY_SERVICE_ENDPOINT=http://localhost:8080
```

### 2. Unix Socket Probing (Port-Free)

Automatically probes:
- `/tmp/*.sock`
- `/var/run/*.sock`
- `~/.local/share/*/sockets/*`

### 3. mDNS Discovery (Zero-Config)

Queries:
- `_gpu-rendering._tcp.local`
- `_discovery._tcp.local`
- `_storage._tcp.local`

### 4. HTTP Probing (Fallback)

Probes:
- `localhost:8080-8090`
- Endpoints: `/capabilities`, `/health`

---

## 📋 Common Capabilities

| Capability | Description | Example Provider |
|------------|-------------|------------------|
| `gpu-rendering` | GPU-accelerated rendering | Toadstool, GPU-Prime |
| `discovery` | Service mesh/discovery | Songbird, Consul, etcd |
| `storage` | Data storage | NestGate, MinIO, S3 |
| `compute` | General compute | Cloud services |
| `database` | Database access | PostgreSQL, etc. |

---

## 🏗️ Deployment Patterns

### Simple (Direct Connection)

```bash
export GPU_RENDERING_ENDPOINT=tarpc://gpu-server:9001
petal-tongue &
```

### Complex (Service Mesh)

```bash
export SERVICE_MESH_ENDPOINT=unix:///tmp/mesh.sock
petal-tongue &
```

### Zero-Config (mDNS)

```bash
petal-tongue &
# Auto-discovers via mDNS!
```

---

## 💡 Examples

### Discover GPU Rendering

```rust
let discovery = UniversalDiscovery::new();
let renderers = discovery.discover_capability("gpu-rendering").await?;

for renderer in renderers {
    println!("Renderer: {} at {}", renderer.id, renderer.endpoint);
    // Could be ANYTHING that provides gpu-rendering!
}
```

### Discover Service Mesh

```rust
let discovery = UniversalDiscovery::new();
let meshes = discovery.discover_capability("discovery").await?;

if let Some(mesh) = meshes.first() {
    // Use mesh to discover other services
    let storage = query_mesh(&mesh, "storage").await?;
}
```

### Graceful Fallback

```rust
let discovery = UniversalDiscovery::new();
let renderers = discovery.discover_capability("gpu-rendering").await?;

if renderers.is_empty() {
    // No GPU renderer found, use pure Rust fallback
    use_tiny_skia_rendering()?;
} else {
    // Use discovered GPU renderer
    use_gpu_rendering(&renderers[0])?;
}
```

---

## 🎓 Best Practices

### DO ✅

- **Discover by capability**, not by name
- **Use generic env vars** (`SERVICE_MESH_ENDPOINT`, not `SONGBIRD_ENDPOINT`)
- **Provide fallbacks** for when services aren't found
- **Document capabilities** your service provides
- **Advertise via mDNS** for zero-config discovery

### DON'T ❌

- **Hardcode primal names** (Songbird, Toadstool, etc.)
- **Hardcode vendor names** (k8s, consul, etc.)
- **Hardcode port numbers**
- **Assume services exist**
- **Depend on specific implementations**

---

## 🔧 Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_discovery() {
    unsafe {
        std::env::set_var("GPU_RENDERING_ENDPOINT", "mock://test");
    }
    
    let discovery = UniversalDiscovery::new();
    let services = discovery.discover_capability("gpu-rendering").await.unwrap();
    
    assert!(!services.is_empty());
    assert_eq!(services[0].endpoint, "mock://test");
    
    unsafe {
        std::env::remove_var("GPU_RENDERING_ENDPOINT");
    }
}
```

### Integration Tests

```bash
# Start mock service
export GPU_RENDERING_ENDPOINT=tarpc://localhost:9999
mock-gpu-renderer &

# Run tests
cargo test --test integration_tests

# Cleanup
killall mock-gpu-renderer
```

---

## 🐛 Troubleshooting

### No Services Found

```rust
let services = discovery.discover_capability("gpu-rendering").await?;
if services.is_empty() {
    // Check environment variables
    println!("GPU_RENDERING_ENDPOINT: {:?}", std::env::var("GPU_RENDERING_ENDPOINT"));
    println!("SERVICE_MESH_ENDPOINT: {:?}", std::env::var("SERVICE_MESH_ENDPOINT"));
    
    // Use fallback
    return Ok(use_fallback_rendering());
}
```

### Discovery Too Slow

```rust
// Reduce discovery methods
let mut discovery = UniversalDiscovery::new();
discovery.discovery_methods = vec![
    DiscoveryMethod::Environment,  // Fastest
    // Skip slower methods
];
```

### Wrong Service Discovered

```rust
// Filter by metadata
let services = discovery.discover_capability("gpu-rendering").await?;
let service = services.iter()
    .find(|s| s.metadata.get("version") == Some(&"2.0".to_string()))
    .unwrap();
```

---

## 📚 Further Reading

- `docs/architecture/INFANT_DISCOVERY_PATTERN.md` - Complete guide
- `HARDCODING_ELIMINATION_JAN_6_2026.md` - Evolution process
- `INFANT_DISCOVERY_COMPLETE_JAN_6_2026.md` - Final report
- `crates/petal-tongue-ui/src/universal_discovery.rs` - Implementation

---

## 🌸 Philosophy

**"Each primal is an infant, discovering the world at runtime."**

- Start with **ZERO** knowledge
- Discover **EVERYTHING** at runtime
- Work with **ANY** provider
- **NO** hardcoded assumptions

This is the TRUE PRIMAL way. 🌸👶✨

---

**Quick Reference Version:** 1.0.0  
**Last Updated:** January 6, 2026  
**Status:** Complete and Ready for Use ✅

