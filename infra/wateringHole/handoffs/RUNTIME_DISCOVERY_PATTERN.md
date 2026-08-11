# Runtime Discovery Pattern — G72 Self-Knowledge Evolution

**Wave**: 157g | **Date**: August 10, 2026 | **For**: All primal teams

---

## Summary

petalTongue eliminated all hardcoded peer primal knowledge. Health probes,
gossip injection, and socket discovery now resolve endpoints at runtime via
filesystem scanning + environment variable resolution.

**Principle**: A primal only knows itself. It discovers peers through biomeOS
socket directories and capability advertisements — never through hardcoded lists.

---

## Pattern: Dynamic Primal Discovery

### Socket Directory Resolution

```rust
fn socket_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::with_capacity(3);
    if let Ok(dir) = std::env::var("BIOMEOS_RUNTIME_DIR") {
        dirs.push(PathBuf::from(dir));
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        dirs.push(PathBuf::from(xdg).join("biomeos"));
    }
    dirs.push(PathBuf::from("/run/membrane"));
    dirs
}
```

### Primal Name Extraction from Socket Path

```rust
// /run/membrane/sweetgrass.sock → "sweetgrass"
// /run/membrane/petaltongue-e8b62b6e.sock → "petaltongue"
// /run/membrane/beardog-default.sock → "beardog"
// /run/membrane/songbird-desktop-nucleus.sock → "songbird"
```

Strip known suffixes: `-default`, `-desktop-nucleus`, 8-char hex family hashes.

### Filtering

- Accept: `*.sock` files
- Reject: `*.tarpc.sock` (binary RPC, not health-checkable via JSON-RPC)
- Reject: `*.negotiate.sock` (G65 protocol negotiation, not health)
- Prefer: `-default.sock` over main socket (avoids BTSP handshake for health checks)

---

## Anti-Pattern (ELIMINATED)

```rust
// BAD: hardcoded peer knowledge
const PRIMAL_ENDPOINTS: &[(&str, &str)] = &[
    ("sweetgrass", "/run/membrane/sweetgrass.sock"),
    ("loamspine", "/run/membrane/loamspine.sock"),
    // ... 13 hardcoded entries
];
```

This violates the primal self-knowledge principle. New primals added to the mesh
would be invisible. Socket paths change across gates.

---

## Adoption Checklist

- [ ] Replace any hardcoded primal endpoint lists with filesystem scanning
- [ ] Use `BIOMEOS_RUNTIME_DIR` / `XDG_RUNTIME_DIR` for socket directory resolution
- [ ] Never embed `/run/user/1000/` — resolve UID dynamically
- [ ] Filter out `.tarpc.sock` and `.negotiate.sock` from health probes
- [ ] Prefer `-default.sock` when available (plaintext health check path)
