# Axum 0.7→0.8 Migration — Tier 2 Pattern

**Wave**: 157i | **Date**: August 11, 2026 | **For**: All primals using axum

---

## Summary

petalTongue migrated axum 0.7→0.8.9. The upgrade is straightforward but
has several mechanical breaking changes. This handoff documents the exact
migration pattern for other primals in the fleet.

---

## Version Matrix

| Dependency | Before | After | Notes |
|-----------|--------|-------|-------|
| axum | 0.7 | 0.8 | Breaking changes |
| tower-http | 0.5 | 0.6 | Required by axum 0.8 |
| tokio-tungstenite | 0.24 | 0.29 | Aligned with axum's transitive |

## Breaking Changes and Fixes

### 1. Route Parameter Syntax

```rust
// Before (0.7):
.route("/api/primals/:id", get(handler))

// After (0.8):
.route("/api/primals/{id}", get(handler))
```

Old syntax panics at runtime in 0.8 to prevent silent behavior changes.

### 2. WebSocket Message::Text

`Message::Text` now wraps `Utf8Bytes` instead of `String`:

```rust
// Before (0.7):
socket.send(Message::Text(response)).await

// After (0.8) — sending:
socket.send(Message::Text(response.into())).await

// Receiving — Utf8Bytes derefs to &str, so pattern matching works:
Message::Text(text) => {
    serde_json::from_str::<T>(&text)  // works, Utf8Bytes: Deref<Target=str>
}
```

For `&str` literals or `to_owned()` patterns:
```rust
// Before:
ws.send(Message::Text(req.to_owned()))

// After:
ws.send(Message::Text(req.into()))
```

### 3. serve() API

`axum::serve` is now generic over listener/IO types. Standard `TcpListener`
usage is unchanged:

```rust
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

### 4. Handlers Must Be Sync

All handlers/services added to Router must implement `Sync`. This is usually
already the case for async handlers. Only affects custom `Service` impls.

### 5. WebSocket::close Removed

Send explicit close frames instead:
```rust
socket.send(Message::Close(None)).await.ok();
```

## Anti-Patterns

- Do NOT use `tungstenite::Message::Text(String)` — it's now `Utf8Bytes`
- Do NOT use `/:param` syntax — it panics at runtime
- Do NOT pin `tokio-tungstenite` lower than axum's transitive (causes duplicate)

## Validation

After migration:
```bash
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
cargo doc --workspace --all-features --no-deps
```

All should pass with 0 errors and 0 warnings.
