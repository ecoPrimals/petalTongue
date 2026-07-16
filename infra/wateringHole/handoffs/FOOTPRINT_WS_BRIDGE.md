# footPrint WebSocket Bridge — Integration Handoff

**Wave**: 142b | **Date**: July 16, 2026 | **For**: footPrint team

---

## What Shipped

`petal-tongue-platform` now exposes a WebSocket JSON-RPC bridge at
`PETALTONGUE_WS_PORT` (default: 8765). This enables footPrint compositions
to communicate with embedded petalTongue instances over the network.

```
footPrint / browser ──► WebSocket ──► ws_bridge ──► EmbeddedRuntime.ipc_request()
```

---

## Configuration

| Env Var | Default | Purpose |
|---------|---------|---------|
| `PETALTONGUE_WS_PORT` | `8765` | WebSocket listener port |
| `PETALTONGUE_WS_BIND_HOST` | `127.0.0.1` | Bind address (set `0.0.0.0` for remote) |

---

## Protocol

Standard JSON-RPC 2.0 over WebSocket text frames. Same methods as the UDS IPC
server: `visualization.render.*`, `health.check`, `capabilities.list`, etc.

### Example

```json
// Client → petalTongue
{"jsonrpc":"2.0","id":1,"method":"visualization.render.grammar","params":{"grammar":"..."}}

// petalTongue → Client
{"jsonrpc":"2.0","id":1,"result":{"svg":"<svg>...</svg>","format":"svg"}}
```

---

## Integration Steps (footPrint)

1. Wire `WS_PATH` composition mount to `ws://localhost:8765`
2. Use standard WebSocket client (browser `WebSocket` API or `tokio-tungstenite`)
3. Send JSON-RPC requests, receive responses
4. Handle connection drop gracefully (petalTongue may restart)

---

## Composition Wiring

In `footPrint` composition config:
```toml
[mounts.petaltongue]
type = "websocket"
url = "ws://${PETALTONGUE_WS_BIND_HOST}:${PETALTONGUE_WS_PORT}"
protocol = "jsonrpc"
```

---

*From petalTongue team. Bridge is shipped and tested. footPrint side wiring is TODO.*
