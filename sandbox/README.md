# petalTongue Sandbox

Testing and development utilities for petalTongue.

---

## 📁 Contents

### `mock-biomeos/`
**Mock biomeOS Topology Server** - Serves biomeOS-formatted topology data for integration testing.

**Usage**:
```bash
# Start the mock server
cd sandbox/mock-biomeos
cargo run

# Test with curl
curl http://localhost:3000/api/v1/topology | jq
curl http://localhost:3000/api/v1/health | jq
curl http://localhost:3000/api/v1/capabilities | jq
curl http://localhost:3000/api/v1/primals/beardog-node-alpha | jq
```

**Endpoints**:
- `GET /api/v1/topology` - Full ecosystem topology (primals + connections)
- `GET /api/v1/health` - Server health status
- `GET /api/v1/capabilities` - Server capabilities
- `GET /api/v1/primals/:id` - Specific primal info

**Mock Data**:
- 3 primals: BearDog, Songbird, PetalTongue
- 1 connection: Songbird → BearDog (encryption capability)
- All with Unix socket endpoints + metadata
- Connection metrics included

**Purpose**:
- Test petalTongue HTTP discovery with biomeOS format
- Verify topology parsing and rendering
- E2E integration testing

---

### `scenarios/`
Pre-defined topology scenarios for testing.

### `scripts/`
Automation scripts for testing.

---

## 🚀 Quick Start

```bash
# Terminal 1: Start mock biomeOS
cd sandbox/mock-biomeos
cargo run

# Terminal 2: Start petalTongue with biomeOS hint
cd ../..
export PETALTONGUE_DISCOVERY_HINTS="http://localhost:3000"
cargo run --bin petal-tongue

# Terminal 3: Test with curl
curl http://localhost:3000/api/v1/topology | jq
```

---

## 📊 Integration Test

```bash
# 1. Start mock biomeOS (port 3000)
cd sandbox/mock-biomeos && cargo run &

# 2. Start petalTongue pointing to mock biomeOS
export PETALTONGUE_DISCOVERY_HINTS="http://localhost:3000"
cargo run --bin petal-tongue

# Expected: petalTongue discovers mock biomeOS and renders 3 primals
```

---

## 🧪 Test Scenarios

### Scenario 1: Basic Discovery
- Mock biomeOS on port 3000
- 3 primals, 1 connection
- Unix socket endpoints

### Scenario 2: HTTP Endpoints
- Mix of Unix socket and HTTP endpoints
- Test endpoint preference logic

### Scenario 3: Connection Metrics
- Multiple connections with metrics
- Test metric visualization

---

## 🔧 Development

**Add new mock primals**:
Edit `mock-biomeos/src/main.rs` → `get_topology()` function

**Change port**:
Edit `mock-biomeos/src/main.rs` → `main()` → `SocketAddr::from()`

**Add new endpoints**:
Add routes to the `Router` in `main()`

---

🌸 **petalTongue Sandbox: Testing made easy!**
