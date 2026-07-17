# footPrint WebSocket Bridge — Integration Handoff

**Wave**: 145b | **Date**: July 16, 2026 | **For**: footPrint team
**Status**: petalTongue bridge **SHIPPED and OPERATIONAL**. Awaiting footPrint client wiring.

---

## What Shipped

`petal-tongue-platform` exposes a WebSocket JSON-RPC bridge at
`PETALTONGUE_WS_PORT` (default: 8765). This enables footPrint compositions
to communicate with embedded petalTongue instances over the network.

```
footPrint / browser ──► WebSocket ──► ws_bridge ──► EmbeddedRuntime.ipc_request()
```

**Commits**: `470d7b5` (bridge), `337e1d0` (docs), `f99c6a9` (feature-gate + composition).

---

## Configuration

| Env Var | Default | Purpose |
|---------|---------|---------|
| `PETALTONGUE_WS_PORT` | `8765` | WebSocket listener port |
| `PETALTONGUE_WS_BIND_HOST` | `127.0.0.1` | Bind address (set `0.0.0.0` for remote/Docker) |

---

## Protocol

Standard JSON-RPC 2.0 over WebSocket text frames. Same methods as the UDS IPC
server: `visualization.render.*`, `health.check`, `capabilities.list`, etc.

### Available Methods

| Method | Purpose |
|--------|---------|
| `visualization.render.grammar` | Render a Grammar of Graphics expression → SVG |
| `visualization.render.graph` | Render topology/entity graph → SVG |
| `health.check` | Heartbeat (returns `{"status":"ok"}`) |
| `capabilities.list` | List available visualization capabilities |

### Example

```json
// Client → petalTongue
{"jsonrpc":"2.0","id":1,"method":"visualization.render.grammar","params":{"grammar":"..."}}

// petalTongue → Client
{"jsonrpc":"2.0","id":1,"result":{"svg":"<svg>...</svg>","format":"svg"}}
```

---

## Integration Steps (footPrint ACTION REQUIRED)

1. Wire `WS_PATH` composition mount to `ws://localhost:8765`
2. Use standard WebSocket client (browser `WebSocket` API or `tokio-tungstenite`)
3. Send JSON-RPC requests, receive responses
4. Handle connection drop gracefully (petalTongue may restart)
5. Test with `health.check` first to confirm connectivity

---

## Composition Wiring

In footPrint composition config:
```toml
[mounts.petaltongue]
type = "websocket"
url = "ws://${PETALTONGUE_WS_BIND_HOST}:${PETALTONGUE_WS_PORT}"
protocol = "jsonrpc"
```

For Docker/sporeGate deployments, set `PETALTONGUE_WS_BIND_HOST=0.0.0.0`.

---

## Testing the Bridge Locally

```bash
# Start petalTongue with WS bridge active
cargo run -- server

# In another terminal, test with websocat (or any WS client):
echo '{"jsonrpc":"2.0","id":1,"method":"health.check","params":{}}' | \
  websocat ws://127.0.0.1:8765
```

---

*Wave 145b: Bridge shipped, tested, operational. footPrint client wiring is the remaining step.
Report completion to overwatch when `WS_PATH` composition is live.*
