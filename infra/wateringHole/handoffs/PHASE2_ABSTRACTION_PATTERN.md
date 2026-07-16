# Phase 2 Abstraction Pattern — Handoff from petalTongue

**Wave**: 142b | **Date**: July 16, 2026 | **For**: All primal teams

---

## Context

Phase 1 (Silicon Atheism): `#[cfg(unix)]` / `#[cfg(windows)]` gating. All 14
primals compile for all 4 depot architectures. **DONE.**

Phase 2: **Abstraction over gating**. Replace `#[cfg]` with trait backends.
Every platform is first-class, not conditionally compiled. petalTongue
`petal-tongue-platform` (`1af1a98`) is the reference implementation.

---

## The Pattern

### 1. Define a trait for the platform-specific behavior

```rust
pub trait MeshTopologySource: Send + Sync {
    fn nodes(&self) -> Vec<&'static MeshNode>;
    fn links(&self) -> Vec<&'static MeshLink>;

    // Default impls for derived queries
    fn count_by_enrollment(&self, status: GateEnrollment) -> usize {
        self.nodes().iter().filter(|n| n.enrollment == status).count()
    }
}
```

### 2. Gate static/fixture data behind a feature

```toml
# Cargo.toml
[features]
default = ["offline-topology"]
offline-topology = []
```

```rust
#[cfg(feature = "offline-topology")]
pub struct StaticMeshTopology;

#[cfg(feature = "offline-topology")]
impl MeshTopologySource for StaticMeshTopology {
    fn nodes(&self) -> Vec<&'static MeshNode> { ... }
    fn links(&self) -> Vec<&'static MeshLink> { ... }
}
```

### 3. Consumers accept the trait, not the concrete type

```rust
fn render_topology(source: &dyn MeshTopologySource) -> SceneGraph {
    let nodes = source.nodes();
    // ...
}
```

### 4. Live implementations query capabilities at runtime

```rust
pub struct LiveMeshTopology { /* songBird IPC client */ }

impl MeshTopologySource for LiveMeshTopology {
    fn nodes(&self) -> Vec<&'static MeshNode> {
        // Query songBird mesh.peers capability
    }
}
```

---

## Applicable Domains Per Primal

| Primal | Phase 2 Target | Trait Pattern |
|--------|---------------|---------------|
| petalTongue | Gate mesh topology | `MeshTopologySource` — **SHIPPED** |
| petalTongue | Platform metrics | `PlatformMetrics` — **SHIPPED** |
| bearDog | HSM/keystore | `KeystoreBackend` (Android Keystore, Windows DPAPI, file) |
| squirrel | Credential store | `CredentialStore` (Android Keystore, Windows Credential Mgr, file) |
| toadStool | GPU discovery | `DeviceDiscovery` (Vulkan, CUDA, Metal) |
| toadStool | Compute dispatch | `SwapExecutor` (Vulkan, CUDA, software) |
| All primals | Transport | `TransportEndpoint` dispatch (UDS, NamedPipe, TCP) — Phase 1 done |

---

## Key Principles

1. **Types always available** — platform-agnostic data types compile everywhere
2. **Trait defines the contract** — implementations are swappable
3. **Feature-gated fixtures** — static/offline data behind features, not in production by default
4. **Default enables feature** — no breaking change; lean builds opt out
5. **No `#[cfg]` in consumer code** — consumers see only the trait interface

---

## Verification Checklist

For each Phase 2 adoption:
- [ ] Trait defined with `Send + Sync` bounds
- [ ] Static/fixture data behind a feature flag
- [ ] `cargo check --no-default-features` compiles
- [ ] `cargo check` (with defaults) compiles
- [ ] Tests exercise both trait impls
- [ ] Consumers updated to accept trait (or `impl` not concrete)
- [ ] Zero clippy warnings

---

*From petalTongue team on eastGate. songBird is Phase 1 reference. petalTongue is Phase 2 reference.*
