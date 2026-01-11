# 🌸 JSON-RPC Protocol Evolution - Deep Analysis

**Date**: January 11, 2026  
**Priority**: High (Blocking full biomeOS integration)  
**Type**: Architectural Evolution (External → Primal Protocol)

---

## 🎯 **Problem Statement**

petalTongue is currently the **ONLY primal** using HTTP/REST as its primary protocol, violating ecoPrimals architecture principles.

### Current State (Incorrect):
```
petalTongue → HTTP/REST → biomeOS
              ❌ Wrong protocol!
              ❌ External-first architecture
              ❌ HTTP overhead on local IPC
```

### Target State (TRUE PRIMAL):
```
petalTongue → JSON-RPC 2.0 → biomeOS
              ✅ Primal protocol
              ✅ Port-free Unix sockets
              ✅ 100x faster
              ✅ Language agnostic
```

---

## 🌍 **ecoPrimals Philosophy: JSON-RPC & tarpc First**

**Core Principle:**
> ecoPrimals is a **JSON-RPC 2.0 and tarpc-first ecosystem**. HTTP/REST is an **optional, enableable fallback** for external integrations, not the primary protocol.

### Why JSON-RPC & tarpc?

1. **Port-Free Architecture**: Unix sockets eliminate port conflicts
2. **100x Faster**: Local IPC without TCP/IP stack overhead
3. **Language Agnostic**: JSON-RPC 2.0 works with any language
4. **Secure by Default**: File permissions control access
5. **True Primal Sovereignty**: No dependency on HTTP servers
6. **Bi-directional**: Supports notifications and streaming

### Current Ecosystem (petalTongue is the outlier!):

| Primal | JSON-RPC | tarpc | HTTP | Status |
|--------|----------|-------|------|--------|
| **Songbird** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **BearDog** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **ToadStool** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **NestGate** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **Squirrel** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **biomeOS** | ✅ Primary | ⏳ Planned | ⚠️ Minimal | Production |
| **petalTongue** | ⚠️ **Missing** | ⏳ Planned | ✅ Primary | ⚠️ **Needs Evolution** |

---

## 🔍 **Technical Investigation**

### What biomeOS Provides (Correct! ✅):

**Protocol**: JSON-RPC 2.0 over Unix socket (line-delimited)  
**Socket**: `/run/user/{uid}/biomeos-device-management.sock`

**Methods**:
1. `get_devices` → List system devices
2. `get_primals_extended` → List primals with health
3. `get_niche_templates` → List niche templates
4. `assign_device` → Assign device to primal
5. `validate_niche` → Validate niche config
6. `deploy_niche` → Deploy via Neural API

**Protocol Details**:
- Line-delimited JSON-RPC 2.0
- Request: `{"jsonrpc":"2.0","method":"get_primals_extended","params":null,"id":1}\n`
- Response: `{"jsonrpc":"2.0","result":[...],"id":1}\n`

### What petalTongue Is Doing (Incorrect! ❌):

**File**: `crates/petal-tongue-discovery/src/http_provider.rs`  
**Protocol**: HTTP/REST over Unix socket (reqwest client)

**Problem**:
```rust
// ❌ Tries to send HTTP GET request to Unix socket
async fn health_check(&self) -> anyhow::Result<String> {
    let url = format!("{}/api/v1/health", self.endpoint);
    match self.client.get(&url).send().await {
        // reqwest expects HTTP semantics (request line, headers, body)
        // biomeOS expects line-delimited JSON-RPC
    }
}
```

**Error**:
```
ERROR petal_tongue_discovery: ❌ Failed to connect to biomeOS at unix:///run/user/1000/biomeos-device-management.sock: 
  Health check failed: builder error for url (unix:///run/user/1000/biomeos-device-management.sock/api/v1/health): 
  URL scheme is not allowed
```

---

## 💡 **Solution: JSON-RPC + tarpc First Architecture**

### Phase 1: JSON-RPC Provider (Immediate)

**Goal**: Implement line-delimited JSON-RPC 2.0 client over Unix sockets

**New Module**: `crates/petal-tongue-discovery/src/jsonrpc_provider.rs`

**Key Features**:
- ✅ Line-delimited JSON-RPC 2.0
- ✅ Unix socket connection
- ✅ Async/await with tokio
- ✅ Error handling (JSON-RPC error codes)
- ✅ Implements `VisualizationDataProvider` trait
- ✅ Auto-discovery of Unix sockets

### Phase 2: Discovery Chain Evolution

**Priority Order** (TRUE PRIMAL):
1. **Songbird Discovery** (JSON-RPC + NUCLEUS)
2. **JSON-RPC Provider** (Unix sockets)
3. **Environment Variables** (with JSON-RPC first)
4. **Auto-detect Unix Sockets** (standard paths)
5. **HTTP Provider** (fallback only, with warning)

### Phase 3: tarpc Integration (Future)

**Goal**: High-performance binary RPC for streaming updates

**Benefits**:
- Streaming real-time updates (device events, primal health)
- Bi-directional communication (server can push to client)
- Type-safe (derive macros for service traits)
- Even faster than JSON-RPC (binary serialization)

---

## 🏗️ **Implementation Plan**

### Week 1: JSON-RPC Foundation (4-6 hours)

**Hour 1-2**: Core JSON-RPC Client
- [ ] Create `jsonrpc_provider.rs`
- [ ] Implement `JsonRpcProvider` struct
- [ ] Implement `call()` method (connect, send, receive)
- [ ] Error handling (connection, protocol, JSON-RPC errors)

**Hour 2-3**: Trait Implementation
- [ ] Implement `VisualizationDataProvider` trait
- [ ] `get_primals()` → `get_primals_extended` RPC
- [ ] `get_topology()` → graceful fallback
- [ ] `health_check()` → RPC call
- [ ] `metadata()` → provider info

**Hour 3-4**: Discovery Integration
- [ ] Update `discover_all_providers()` in `lib.rs`
- [ ] Add JSON-RPC before HTTP
- [ ] Auto-detect standard Unix socket paths
- [ ] Environment variable: `BIOMEOS_URL=unix:///path/to/socket`

**Hour 4-5**: Testing
- [ ] Unit tests (JSON-RPC request/response serialization)
- [ ] Integration tests (with mock Unix socket server)
- [ ] E2E tests (with real biomeOS server)

**Hour 5-6**: Documentation & Polish
- [ ] Inline documentation
- [ ] Update discovery documentation
- [ ] Add examples
- [ ] Update root docs

### Week 2-3: HTTP Deprecation & tarpc Foundation (8-12 hours)

**HTTP Deprecation**:
- [ ] Mark `HttpProvider` as "external fallback only"
- [ ] Add warning log if HTTP is used
- [ ] Document HTTP as "external integration only"
- [ ] Update all examples to use JSON-RPC

**tarpc Integration** (Future):
- [ ] Research tarpc integration patterns from ToadStool/Songbird
- [ ] Design `TarpcProvider` architecture
- [ ] Implement streaming updates
- [ ] Bi-directional communication

---

## 🎯 **Success Criteria**

### Immediate (JSON-RPC):
- ✅ petalTongue connects to biomeOS via JSON-RPC 2.0
- ✅ All biomeOS methods callable (`get_devices`, `get_primals_extended`, etc.)
- ✅ Auto-discovery of Unix sockets works
- ✅ HTTP is fallback only (with warning)
- ✅ 20+ unit tests passing
- ✅ E2E integration test with biomeOS passing

### Future (tarpc):
- ✅ Real-time streaming updates from biomeOS
- ✅ Bi-directional communication (server push)
- ✅ Type-safe service traits
- ✅ Performance: < 1ms latency for local RPC

---

## 📊 **TRUE PRIMAL Compliance**

### Before (HTTP-first):
- ❌ External protocol as primary
- ❌ HTTP overhead on local IPC
- ❌ Port conflicts possible
- ❌ Incompatible with other primals
- ❌ Hardcoded to HTTP semantics

### After (JSON-RPC + tarpc first):
- ✅ Primal protocol as primary
- ✅ Zero overhead (Unix sockets)
- ✅ Port-free architecture
- ✅ Compatible with entire ecosystem
- ✅ Discovery-based, zero hardcoding
- ✅ Graceful degradation (HTTP as fallback)
- ✅ Self-stable, then network, then externals

---

## 🚀 **Expected Outcome**

Once implemented:

```bash
# Launch biomeOS device_management_server
$ cargo run --bin device_management_server
🌸 Starting biomeOS Device Management Server
📡 Binding to Unix socket: /run/user/1000/biomeos-device-management.sock
✅ biomeOS Device Management Server ready

# Launch petalTongue (auto-discovers!)
$ cargo run --release
🌸 Starting petalTongue
🔍 Discovering visualization providers...
✅ Found JSON-RPC provider at unix:///run/user/1000/biomeos-device-management.sock
📊 Fetching primals...
✅ Discovered 6 primals (Songbird, BearDog, ToadStool, NestGate, Squirrel, biomeOS)
✅ Discovered 7 devices
✅ Discovered 3 niche templates
🎨 Rendering UI...
✅ Full UI with live data!
```

---

## 📚 **References**

- **JSON-RPC 2.0 Spec**: https://www.jsonrpc.org/specification
- **biomeOS RPC Server**: `crates/biomeos-ui/src/bin/device_management_server.rs`
- **ToadStool Example**: `ecoPrimals/phase1/toadstool/` (production JSON-RPC + tarpc)
- **Songbird Example**: `ecoPrimals/phase1/songbird/` (production JSON-RPC + tarpc)
- **tarpc Docs**: https://github.com/google/tarpc

---

## 💬 **Questions for Ecosystem Coordination**

1. **Streaming**: Should we implement tarpc streaming immediately, or JSON-RPC first?
2. **Versioning**: Should we version the JSON-RPC API (e.g., `v1.get_primals`)?
3. **Error Codes**: Should we standardize JSON-RPC error codes across primals?
4. **Discovery**: Should we formalize Unix socket discovery paths (e.g., `/run/user/{uid}/{primal}-{service}.sock`)?

---

**Different orders of the same architecture.** 🍄🐸🌸

Let's evolve petalTongue to be a TRUE primal with JSON-RPC + tarpc first!
