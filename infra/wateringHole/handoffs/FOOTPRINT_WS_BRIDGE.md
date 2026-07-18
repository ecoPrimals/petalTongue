# footPrint WebSocket Bridge — Integration Handoff

**Wave**: 150b | **Date**: July 18, 2026 | **For**: footPrint team
**Status**: petalTongue bridge **SHIPPED and OPERATIONAL**. Awaiting footPrint client wiring.

---

## What Shipped

`petal-tongue-platform` exposes a WebSocket JSON-RPC bridge at
`PETALTONGUE_WS_PORT` (default: 8765). This enables footPrint compositions
to communicate with embedded petalTongue instances over the network.

```
footPrint / browser ──► WebSocket ──► ws_bridge ──► EmbeddedRuntime.ipc_request()
```

**Commits**: `470d7b5` (bridge), `337e1d0` (docs), `f99c6a9` (feature-gate + composition),
`d9c2ddb` (E2E tests), latest (pt.metrics method).

---

## Configuration

| Env Var | Default | Purpose |
|---------|---------|---------|
| `PETALTONGUE_WS_PORT` | `8765` | WebSocket listener port |
| `PETALTONGUE_WS_BIND_HOST` | `127.0.0.1` | Bind address (set `0.0.0.0` for remote/Docker) |

---

## Protocol

Standard JSON-RPC 2.0 over WebSocket text frames.

### Available Methods

| Method | Purpose |
|--------|---------|
| `health.check` | Heartbeat — returns `{"status":"ok","state":"Running"}` |
| `capabilities.list` | List all supported JSON-RPC methods |
| `pt.metrics` | Platform resource snapshot (CPU, memory, source) |
| `pt.state` | Runtime state (`Created`, `Running`, `Paused`, `Stopped`) |
| `pt.scenarios` | List available visualization scenarios |
| `pt.render_svg` | Render a scenario scene → SVG |
| `pt.render_binding` | Render a data binding → SVG |

### Example: Health Check

```json
// Client → petalTongue
{"jsonrpc":"2.0","id":1,"method":"health.check","params":{}}

// petalTongue → Client
{"jsonrpc":"2.0","id":1,"result":{"status":"ok","state":"Running"}}
```

### Example: Platform Metrics

```json
// Client → petalTongue
{"jsonrpc":"2.0","id":2,"method":"pt.metrics","params":{}}

// petalTongue → Client
{"jsonrpc":"2.0","id":2,"result":{"cpu_percent":12.5,"memory_total":16777216000,"memory_used":8388608000,"memory_percent":50.0,"cpu_count":8,"source":"linux-proc"}}
```

---

## Integration Steps (footPrint ACTION REQUIRED)

1. Wire `WS_PATH` composition mount to `ws://localhost:8765`
2. Use standard WebSocket client (browser `WebSocket` API or `tokio-tungstenite`)
3. Send JSON-RPC requests, receive responses
4. Handle connection drop gracefully (petalTongue may restart)
5. Test with `health.check` first to confirm connectivity
6. Use `pt.metrics` for real-time resource monitoring in composition dashboard

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

# Platform metrics:
echo '{"jsonrpc":"2.0","id":2,"method":"pt.metrics","params":{}}' | \
  websocat ws://127.0.0.1:8765
```

---

## E2E Test Coverage

10 platform tests + 4 E2E WebSocket tests validate the full stack:
- WebSocket connection + upgrade
- `health.check` round-trip
- `capabilities.list` round-trip
- `pt.metrics` round-trip
- Unknown method error response (-32601)

---

*Wave 150b: Bridge shipped, tested (10+4 tests), operational. `pt.metrics` added for
real-time resource monitoring. footPrint client wiring is the remaining step.
Report completion to overwatch when `WS_PATH` composition is live.*
