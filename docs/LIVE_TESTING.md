# Live Multi-Primal Testing Guide

Testing petalTongue with real biomeOS, NUCLEUS, and ludoSpring instances.

## Prerequisites

- biomeOS Tower node running (`biomeos nucleus serve --mode tower`)
- Neural API socket available at `$XDG_RUNTIME_DIR/biomeos-neural-api-{family_id}.sock`
- ludoSpring primal registered (part of NUCLEUS full deployment)

## Quick Start

```bash
# Terminal 1: Start biomeOS Tower
cd ../biomeOS && cargo run -- nucleus serve --mode tower

# Terminal 2: Start petalTongue
cd petalTongue && cargo run

# Terminal 3: Verify registration
echo '{"jsonrpc":"2.0","method":"lifecycle.status","id":1}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos-neural-api-nat0.sock
```

## What Happens on Startup

1. petalTongue binds its IPC socket at `$XDG_RUNTIME_DIR/petaltongue-{family_id}-{node_id}.sock`
2. In a background thread, it discovers the Neural API socket
3. Calls `lifecycle.register` with capabilities: `ui.render`, `visualization.render`, `ipc.json-rpc`, `interaction.sensor_stream`
4. Starts a 30s heartbeat loop via `lifecycle.status`

If biomeOS is not running, registration fails gracefully and petalTongue operates standalone.

## Sensor Stream Testing

External primals (ludoSpring, Squirrel) can subscribe to petalTongue's raw sensor events:

```bash
# Subscribe to sensor stream
echo '{"jsonrpc":"2.0","method":"interaction.sensor_stream.subscribe","params":{},"id":1}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/petaltongue-nat0-default.sock

# Poll for events (move mouse/type in petalTongue first)
echo '{"jsonrpc":"2.0","method":"interaction.sensor_stream.poll","params":{"subscription_id":"sensor-sub-1"},"id":2}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/petaltongue-nat0-default.sock
```

## Interaction Event Testing

```bash
# Subscribe to interaction events (selection changes)
echo '{"jsonrpc":"2.0","method":"interaction.subscribe","params":{"subscriber_id":"test"},"id":1}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/petaltongue-nat0-default.sock

# Click a node in petalTongue, then poll
echo '{"jsonrpc":"2.0","method":"interaction.poll","params":{"subscriber_id":"test"},"id":2}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/petaltongue-nat0-default.sock
```

## Game Data Channel Testing (ludoSpring)

Send game science data to petalTongue for visualization:

```bash
# Push an engagement curve
echo '{"jsonrpc":"2.0","method":"visualization.render","params":{
  "session_id":"ludospring-test",
  "title":"Engagement Analysis",
  "domain":"game",
  "bindings":[{
    "channel_type":"timeseries",
    "id":"engagement",
    "label":"Player Engagement",
    "x_label":"Time (s)",
    "y_label":"Engagement",
    "unit":"score",
    "x_values":[0,1,2,3,4,5],
    "y_values":[0.1,0.4,0.8,0.6,0.9,0.7]
  }],
  "thresholds":[]
},"id":1}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/petaltongue-nat0-default.sock
```

## Full NUCLEUS Deployment

For testing with all primals:

```bash
# Start full NUCLEUS (Tower + all primals)
cd ../biomeOS && cargo run -- nucleus start --mode full

# Verify all primals registered
echo '{"jsonrpc":"2.0","method":"lifecycle.status","id":1}' | \
  socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos-neural-api-nat0.sock
```

## Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `FAMILY_ID` | biomeOS family identifier | `nat0` |
| `PETALTONGUE_NODE_ID` | Instance node ID | `default` |
| `PETALTONGUE_SOCKET` | Override socket path | Auto-derived |
| `XDG_RUNTIME_DIR` | Socket directory | `/run/user/<uid>` |
| `BIOMEOS_HOST` | biomeOS WebSocket host | `127.0.0.1` |
| `BIOMEOS_PORT` | biomeOS WebSocket port | `9600` |

## Automated Tests

Unit and integration tests that exercise the pipeline without live primals:

```bash
# Sensor feed tests (pure logic)
cargo test -p petal-tongue-ui -- sensor_feed

# Game data channel mapping tests
cargo test -p petal-tongue-ui -- game_data_channel

# Full pipeline integration tests
cargo test -p petal-tongue-ui --test live_primal_integration_tests

# Neural registration capability tests
cargo test -p petal-tongue-ui -- neural_registration
```
