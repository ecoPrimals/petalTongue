# BTSP ClientHello — Authenticated Outbound Connections

**Wave**: 151c | **Date**: July 26, 2026 | **For**: All primal teams + overwatch
**Status**: **SHIPPED** (petalTongue is 12/13 for BTSP strict mode)

---

## What Shipped

petalTongue now performs the BTSP 4-step ClientHello handshake on all outbound
IPC connections when `BTSP_FAMILY_SEED` or `FAMILY_SEED` is set in the environment.

### Protocol Flow

```text
1. Client → Server: ClientHello { protocol: "btsp", version: 1, client_ephemeral_pub }
2. Server → Client: ServerHello { version: 1, server_ephemeral_pub, challenge }
3. Client → Server: ChallengeResponse { response: HMAC-SHA256(family_seed, challenge), preferred_cipher }
4. Server → Client: HandshakeComplete { status: "ok", session_id, cipher }
```

### Wired Connections

| Connection | Target | Purpose |
|------------|--------|---------|
| `primal.announce` | Neural API (biomeOS) | Capability routing registration |
| `content.resolve` | nestGate CAS | Content-addressed content retrieval |

### Graceful Degradation

- **No family seed** → handshake skipped (development mode)
- **Handshake failure** → logged at debug, connection proceeds (dev)
- **Production** (with seed) → authenticated before payload transmission

---

## API for Other Primals

```rust
use petal_tongue_ipc::{BtspClientConfig, perform_client_handshake};

// Resolve config from env (BTSP_FAMILY_SEED or FAMILY_SEED)
if let Some(config) = BtspClientConfig::from_env() {
    perform_client_handshake(&mut stream, &config).await?;
}
```

### `BtspClientConfig`

| Field | Type | Source |
|-------|------|--------|
| `family_seed` | `Vec<u8>` | `BTSP_FAMILY_SEED` or `FAMILY_SEED` env var |
| `preferred_cipher` | `String` | Default: `"chacha20-poly1305"` |

---

## Upstream Dependencies

| Team | Need | Status |
|------|------|--------|
| **bearDog** | Server-side BTSP responder for client handshakes | SHIPPED (reference) |
| **nestGate** | BTSP ClientHello (Nest Atomic blocker) | **PENDING** — last remaining primal |
| **sporeGate** | BTSP strict mode on remaining gates | Active |

## Downstream Consumers

| Team | Integration |
|------|-------------|
| **footPrint** | petalTongue's `/ws` bridge already BTSP-authenticated server-side |
| **biomeOS** | Receives authenticated `primal.announce` from petalTongue |

---

## BTSP Ecosystem Status

| Count | Status |
|-------|--------|
| 12/13 | ClientHello SHIPPED |
| 1/13 | nestGate PENDING (Nest Atomic blocker) |

---

*Wave 151c: petalTongue BTSP ClientHello complete. Content signing and
authenticated serving enabled. Graceful degradation for development mode.*
