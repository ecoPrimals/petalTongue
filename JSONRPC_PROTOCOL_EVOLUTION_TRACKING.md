# 🌸 JSON-RPC Protocol Evolution - Progress Tracking

**Start Date**: January 11, 2026  
**Target**: Full JSON-RPC + tarpc integration  
**Priority**: High (Blocking biomeOS integration)

---

## 📊 **Overall Progress**

```
[░░░░░░░░░░░░░░░░░░░░] 0% Complete

Phase 1: JSON-RPC Foundation    [░░░░░░░░░░] 0/6 hours
Phase 2: HTTP Deprecation       [░░░░░░░░░░] Not Started
Phase 3: tarpc Integration      [░░░░░░░░░░] Not Started
```

---

## 🎯 **Phase 1: JSON-RPC Foundation** (4-6 hours)

**Goal**: Implement line-delimited JSON-RPC 2.0 client over Unix sockets

### Hour 1-2: Core JSON-RPC Client

- [ ] **Create `jsonrpc_provider.rs` module**
  - [ ] `JsonRpcProvider` struct
  - [ ] `JsonRpcRequest` struct (with serde)
  - [ ] `JsonRpcResponse` struct (with serde)
  - [ ] `JsonRpcError` struct (with serde)
  - Status: Not Started
  - ETA: 1 hour

- [ ] **Implement `call()` method**
  - [ ] Connect to Unix socket (`tokio::net::UnixStream`)
  - [ ] Split into reader/writer
  - [ ] Send line-delimited request
  - [ ] Read line-delimited response
  - [ ] Parse JSON-RPC response
  - [ ] Error handling (connection, protocol, RPC errors)
  - Status: Not Started
  - ETA: 1 hour

### Hour 2-3: Trait Implementation

- [ ] **Implement `VisualizationDataProvider` trait**
  - [ ] `get_primals()` → `get_primals_extended` RPC
  - [ ] `get_topology()` → graceful fallback (empty vec)
  - [ ] `health_check()` → RPC call test
  - [ ] `metadata()` → provider info
  - Status: Not Started
  - ETA: 1 hour

### Hour 3-4: Discovery Integration

- [ ] **Update `discover_all_providers()` in `lib.rs`**
  - [ ] Add `JsonRpcProvider` before `HttpProvider`
  - [ ] Implement `discover()` method for `JsonRpcProvider`
  - [ ] Auto-detect standard Unix socket paths
  - [ ] Environment variable support (`BIOMEOS_URL=unix://...`)
  - [ ] Logging (info/warn/debug)
  - Status: Not Started
  - ETA: 1 hour

### Hour 4-5: Testing

- [ ] **Unit Tests** (20+ tests)
  - [ ] JSON-RPC request serialization
  - [ ] JSON-RPC response deserialization
  - [ ] Error response handling
  - [ ] Request ID generation (atomic)
  - [ ] Socket path validation
  - Status: Not Started
  - ETA: 30 minutes

- [ ] **Integration Tests** (5+ tests)
  - [ ] Mock Unix socket server
  - [ ] Concurrent requests
  - [ ] Connection retry logic
  - [ ] Discovery auto-detection
  - Status: Not Started
  - ETA: 30 minutes

- [ ] **E2E Tests** (3+ tests)
  - [ ] Real biomeOS server connection
  - [ ] All supported methods (`get_devices`, `get_primals_extended`, etc.)
  - [ ] Error scenarios
  - Status: Not Started
  - ETA: 30 minutes

### Hour 5-6: Documentation & Polish

- [ ] **Inline Documentation**
  - [ ] Module-level docs (`jsonrpc_provider.rs`)
  - [ ] Struct docs
  - [ ] Method docs with examples
  - Status: Not Started
  - ETA: 20 minutes

- [ ] **Update Discovery Documentation**
  - [ ] Update `README.md` with JSON-RPC section
  - [ ] Update `STATUS.md`
  - [ ] Update `NAVIGATION.md`
  - Status: Not Started
  - ETA: 20 minutes

- [ ] **Examples**
  - [ ] Create `examples/jsonrpc_demo.rs`
  - [ ] Update existing examples
  - Status: Not Started
  - ETA: 20 minutes

---

## 🎯 **Phase 2: HTTP Deprecation** (2-4 hours)

**Goal**: Mark HTTP as external fallback only

### Tasks

- [ ] **Mark `HttpProvider` as deprecated**
  - [ ] Add `#[deprecated]` attribute
  - [ ] Update struct documentation
  - [ ] Add warning log on instantiation
  - Status: Not Started

- [ ] **Update Discovery Priority**
  - [ ] Ensure JSON-RPC is checked before HTTP
  - [ ] Add warning if HTTP is used as primary
  - [ ] Document priority order
  - Status: Not Started

- [ ] **Update All Examples**
  - [ ] Replace HTTP examples with JSON-RPC
  - [ ] Add note about HTTP as "external only"
  - Status: Not Started

- [ ] **Update Documentation**
  - [ ] README: Emphasize JSON-RPC as primary
  - [ ] STATUS: Update protocol status
  - [ ] SPECS: Add HTTP deprecation note
  - Status: Not Started

---

## 🎯 **Phase 3: tarpc Integration** (8-12 hours, Future)

**Goal**: High-performance binary RPC for streaming updates

### Tasks

- [ ] **Research tarpc Patterns**
  - [ ] Review ToadStool implementation
  - [ ] Review Songbird implementation
  - [ ] Document common patterns
  - Status: Not Started

- [ ] **Design `TarpcProvider`**
  - [ ] Service trait definition
  - [ ] Client implementation
  - [ ] Server stubs (for testing)
  - Status: Not Started

- [ ] **Streaming Updates**
  - [ ] Real-time device events
  - [ ] Primal health updates
  - [ ] Topology changes
  - Status: Not Started

- [ ] **Bi-directional Communication**
  - [ ] Server push to client
  - [ ] Client subscriptions
  - [ ] Event filtering
  - Status: Not Started

---

## 📊 **Metrics**

### Code

- **New Files**: 0
- **Lines of Code**: 0
- **Tests**: 0/30+
- **Documentation**: 0 files

### Testing

- **Unit Tests**: 0/20 passing
- **Integration Tests**: 0/5 passing
- **E2E Tests**: 0/3 passing
- **Coverage**: 0%

### Performance

- **Latency**: Not measured
- **Throughput**: Not measured
- **Connection Time**: Not measured

---

## 🚧 **Blockers**

*None identified yet*

---

## ✅ **Success Criteria**

### Phase 1 Complete When:

- ✅ `JsonRpcProvider` implemented and tested
- ✅ All biomeOS methods callable
- ✅ Auto-discovery working
- ✅ 30+ tests passing (unit + integration + E2E)
- ✅ Documentation complete
- ✅ petalTongue connects to biomeOS successfully

### Phase 2 Complete When:

- ✅ HTTP marked as deprecated
- ✅ JSON-RPC is primary protocol
- ✅ All examples updated
- ✅ Documentation updated

### Phase 3 Complete When:

- ✅ tarpc provider implemented
- ✅ Streaming updates working
- ✅ Bi-directional communication working
- ✅ Performance: < 1ms latency

---

## 📅 **Daily Progress Log**

### January 11, 2026

- **Time**: 0 hours
- **Progress**: Created analysis, spec, and tracking docs
- **Next**: Begin Phase 1 implementation

---

## 📞 **Coordination**

### With biomeOS Team:

- ✅ Received handoff document
- ✅ Confirmed JSON-RPC protocol details
- ⏳ Awaiting E2E testing coordination

### With Ecosystem:

- ⏳ Review ToadStool tarpc patterns
- ⏳ Review Songbird discovery patterns
- ⏳ Coordinate Unix socket path conventions

---

## 🎯 **Next Steps**

1. **Begin Phase 1, Hour 1-2**: Create `jsonrpc_provider.rs` module
2. **Set up E2E test environment**: Ensure biomeOS server available for testing
3. **Review ToadStool/Songbird**: Study existing JSON-RPC implementations

---

**Status**: Ready to begin implementation! 🚀  
**Awaiting**: User approval to "proceed to execute on all"

🌸 TRUE PRIMAL evolution in progress!
