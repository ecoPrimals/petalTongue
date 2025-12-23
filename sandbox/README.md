# 🧪 petalTongue Sandbox

**Purpose**: Mock services and test data for developing petalTongue against realistic ecosystem behaviors.

---

## Overview

The `sandbox/` directory provides:
- **Mock BiomeOS API** - Simulates BiomeOS/Songbird discovery endpoints
- **Mock Primal Services** - Simulates BearDog, ToadStool, Songbird, NestGate, Squirrel
- **Test Scenarios** - Realistic ecosystem topologies and events
- **Development Server** - Local HTTP server for testing UI against realistic data

This allows petalTongue to evolve independently while staying aligned with actual ecosystem behaviors.

---

## Structure

```
sandbox/
├── README.md                    # This file
├── mock-biomeos/                # Mock BiomeOS orchestrator
│   ├── main.rs                  # Simple HTTP server
│   ├── Cargo.toml              
│   └── data/                    # Static JSON responses
│       ├── primals.json         # Discovery response
│       ├── topology.json        # Topology edges
│       └── health.json          # Health status
├── scenarios/                   # Test scenarios
│   ├── simple.json              # 3-5 primals, basic topology
│   ├── complex.json             # 20+ primals, multiple connections
│   ├── unhealthy.json           # Mixed health states
│   ├── dynamic.json             # Simulates changes over time
│   └── chaos.json               # High churn, failures
└── scripts/                     # Helper scripts
    ├── start-mock.sh            # Start mock BiomeOS server
    ├── generate-scenario.sh     # Generate random scenarios
    └── test-integration.sh      # Test petalTongue against mocks
```

---

## Quick Start

### 1. Start Mock BiomeOS Server

```bash
cd sandbox/
./scripts/start-mock.sh
# Listening on http://localhost:3333
```

### 2. Run petalTongue Against Mock

```bash
cd ..
BIOMEOS_URL=http://localhost:3333 cargo run --release -p petal-tongue-ui
```

### 3. Test Different Scenarios

```bash
# Switch scenario
cd sandbox/
./scripts/generate-scenario.sh complex

# Restart mock server (picks up new scenario)
./scripts/start-mock.sh
```

---

## Scenarios

### `simple.json` - Basic Ecosystem
- 5 primals (1 of each type)
- 4-5 connections
- All healthy
- **Use for**: Basic UI testing, layout validation

### `complex.json` - Production-Like
- 20+ primals (multiple instances)
- 30+ connections
- Mixed health states
- **Use for**: Performance testing, layout stress

### `unhealthy.json` - Degraded State
- 10 primals
- 2 critical, 3 warning, 5 healthy
- **Use for**: Audio testing, alert visualization

### `dynamic.json` - Simulates Changes
- Primals come and go
- Health changes every 5s
- **Use for**: Real-time update testing

### `chaos.json` - High Churn
- Rapid topology changes
- Frequent failures
- **Use for**: Stress testing, edge cases

---

## Mock BiomeOS API

### Endpoints

#### `GET /api/v1/primals`
Returns list of discovered primals.

**Response:**
```json
{
  "primals": [
    {
      "id": "beardog-1",
      "name": "BearDog Security",
      "primal_type": "Security",
      "endpoint": "http://localhost:8001",
      "capabilities": ["authentication", "authorization"],
      "health": "healthy",
      "last_seen": 1703376000
    }
  ]
}
```

#### `GET /api/v1/topology`
Returns topology edges.

**Response:**
```json
[
  {
    "from": "beardog-1",
    "to": "toadstool-1",
    "edge_type": "authenticates",
    "label": "Auth Flow"
  }
]
```

#### `GET /api/v1/health`
Returns overall ecosystem health.

**Response:**
```json
{
  "status": "healthy",
  "primal_count": 5,
  "healthy": 4,
  "warning": 1,
  "critical": 0
}
```

---

## Development Workflow

### 1. Develop Against Mock
```bash
# Terminal 1: Start mock server
cd sandbox/
./scripts/start-mock.sh

# Terminal 2: Run petalTongue
cd ..
BIOMEOS_URL=http://localhost:3333 cargo run --release -p petal-tongue-ui
```

### 2. Test New Features
```bash
# Add new scenario
echo '{"primals": [...]}' > sandbox/scenarios/my-test.json

# Load scenario
cd sandbox/
./scripts/generate-scenario.sh my-test

# Test UI
cd ..
BIOMEOS_URL=http://localhost:3333 cargo run --release -p petal-tongue-ui
```

### 3. Validate Against Real Ecosystem
```bash
# Test against real BiomeOS (when available)
BIOMEOS_URL=http://localhost:3000 cargo run --release -p petal-tongue-ui
```

---

## Adding New Scenarios

### Create Scenario File

```json
{
  "primals": [
    {
      "id": "unique-id",
      "name": "Display Name",
      "primal_type": "Security|Compute|Discovery|Storage|AI",
      "endpoint": "http://localhost:PORT",
      "capabilities": ["cap1", "cap2"],
      "health": "healthy|warning|critical|unknown",
      "last_seen": 1703376000
    }
  ],
  "topology": [
    {
      "from": "source-id",
      "to": "target-id",
      "edge_type": "connection-type",
      "label": "Optional Label"
    }
  ]
}
```

### Use Generator Script

```bash
cd sandbox/
./scripts/generate-scenario.sh my-scenario --primals 10 --health mixed
```

---

## Mock Server Implementation

The mock server is intentionally simple:
- **Language**: Rust (minimal dependencies)
- **Framework**: Axum or Actix (lightweight)
- **Data**: Static JSON files (easy to edit)
- **Hot Reload**: Watches scenario files, reloads on change
- **Logging**: Shows all requests for debugging

### Why Mocks Here?

1. **Isolation** - Develop petalTongue without full ecosystem
2. **Speed** - Fast iteration without waiting for services
3. **Scenarios** - Test edge cases easily
4. **Determinism** - Reproducible test cases
5. **Evolution** - Scaffold towards real behaviors

---

## Integration with Tests

### Unit Tests
petalTongue unit tests use `BiomeOSClient::with_mock_mode(true)` (built-in mocks).

### Integration Tests
Integration tests can use the sandbox mock server:

```rust
#[tokio::test]
async fn test_against_sandbox() {
    // Start mock server (or assume running)
    let client = BiomeOSClient::new("http://localhost:3333");
    
    let primals = client.discover_primals().await.unwrap();
    assert!(primals.len() > 0);
}
```

### Manual Testing
Developers run sandbox server and interact with UI manually.

---

## Comparison: Built-in Mocks vs Sandbox

| Feature | Built-in Mocks | Sandbox |
|---------|----------------|---------|
| **Use** | Unit tests | Integration, manual testing |
| **Data** | Hardcoded in Rust | JSON files (editable) |
| **Network** | No HTTP | Real HTTP server |
| **Scenarios** | Single default | Multiple scenarios |
| **Hot Reload** | Recompile required | Edit JSON, instant update |
| **Complexity** | Simple | Realistic |

**Both are valuable** - built-in for unit tests, sandbox for integration/manual.

---

## Guidelines

### ✅ Do
- Add realistic scenarios based on actual ecosystem behaviors
- Keep mock server simple and maintainable
- Document new scenarios in this README
- Use for testing new petalTongue features
- Validate features work against both mock and real

### ❌ Don't
- Don't use for production (mocks only)
- Don't add complex business logic (keep it a data server)
- Don't commit sensitive data (all scenarios are public)
- Don't let mocks drift from real API contracts

---

## Maintenance

### Updating for API Changes

When BiomeOS API changes:
1. Update `sandbox/mock-biomeos/data/*.json` schemas
2. Update scenario files to match
3. Update this README with new endpoints
4. Test petalTongue against updated mocks

### Adding New Primal Types

When new primal types are added to ecosystem:
1. Add to `scenarios/*.json` examples
2. Add instrument mapping to audio renderer (if needed)
3. Test visualization with new types

---

## Future Enhancements

- [ ] **WebSocket Support** - Real-time event streaming
- [ ] **Dynamic Scenarios** - Scenarios that evolve over time
- [ ] **Failure Injection** - Simulate network errors, timeouts
- [ ] **Performance Metrics** - Track request latency, throughput
- [ ] **Scenario Generator** - Create random valid topologies
- [ ] **Docker Compose** - Package mock ecosystem
- [ ] **Recording Mode** - Capture real API calls, replay as scenarios

---

## Examples

### Example: Testing Layout Performance

```bash
# Generate large topology
cd sandbox/
./scripts/generate-scenario.sh stress --primals 100 --edges 500

# Start mock
./scripts/start-mock.sh

# Test petalTongue performance
cd ..
BIOMEOS_URL=http://localhost:3333 cargo run --release -p petal-tongue-ui
```

### Example: Testing Audio Sonification

```bash
# Use unhealthy scenario
cd sandbox/
cp scenarios/unhealthy.json mock-biomeos/data/current.json

# Start mock
./scripts/start-mock.sh

# Listen to audio descriptions
cd ..
BIOMEOS_URL=http://localhost:3333 cargo run --release -p petal-tongue-ui
```

---

*Sandbox: Where petalTongue learns to speak the ecosystem's language.* 🌸

