# petalTongue Discovery Infrastructure Evolution Specification

**Version**: 1.0  
**Date**: January 3, 2026  
**Status**: Specification & Roadmap  
**Based On**: Cross-project analysis of Songbird, biomeOS, and petalTongue

---

## 🎯 Executive Summary

This specification defines the evolution of petalTongue's discovery infrastructure from its current HTTP-only implementation to a comprehensive multi-protocol, multi-transport discovery system that combines the best practices from across the ecoPrimals ecosystem.

**Current State**: HTTP-based provider discovery with multi-provider aggregation  
**Target State**: Full network discovery with mDNS, caching, multiple protocols, and trust verification

---

## 📊 Current Architecture

### Existing Components

```
petalTongue Discovery (Current)
├── VisualizationDataProvider trait
│   ├── get_primals() -> Vec<PrimalInfo>
│   ├── get_topology() -> Vec<TopologyEdge>
│   ├── health_check() -> String
│   └── get_metadata() -> ProviderMetadata
├── HttpVisualizationProvider (✅ Complete)
├── MockVisualizationProvider (✅ Complete)
└── discover_visualization_providers() function
    ├── Try PETALTONGUE_DISCOVERY_HINTS
    ├── Try BIOMEOS_URL (legacy)
    └── Fallback to mock
```

### Strengths
- ✅ Zero hardcoded primal dependencies
- ✅ Multi-provider aggregation
- ✅ Graceful degradation
- ✅ Mock support for testing
- ✅ Clean trait-based abstraction

### Gaps
- ⚠️ No network discovery (mDNS/multicast)
- ⚠️ No caching layer
- ⚠️ Only HTTP protocol support
- ⚠️ No trust verification
- ⚠️ No retry logic
- ⚠️ No connection pooling optimization

---

## 🎓 Lessons from Other Primals

### From Songbird: Network Discovery

**Key Techniques**:
- UDP multicast with proper `IP_ADD_MEMBERSHIP`
- mDNS-compatible service announcements (224.0.0.251)
- Hybrid approach: multicast + known peers + broadcast
- Uses `socket2` crate for low-level socket control

**Application to petalTongue**:
- Implement `MdnsVisualizationProvider`
- Query `_visualization-provider._tcp.local` service
- Support known providers list
- Automatic zero-config discovery

### From biomeOS: Universal Client

**Key Techniques**:
- Format-agnostic (auto-detect wrapped vs unwrapped)
- Protocol-agnostic (HTTP, tarpc, gRPC)
- Comprehensive caching (schemas, formats, discoveries)
- Trust policies (genetic lineage)
- Retry logic with backoff

**Application to petalTongue**:
- Add caching layer
- Implement tarpc support for Songbird
- Add trust verification
- Implement retry/timeout logic

---

## 🏗️ Target Architecture

### Phase 1: Network Discovery (Week 1)

```
Network Discovery Layer
├── MdnsVisualizationProvider
│   ├── UDP socket with socket2
│   ├── Join multicast group (224.0.0.251)
│   ├── Listen for mDNS service announcements
│   ├── Query: _visualization-provider._tcp.local
│   └── Returns: Vec<ProviderMetadata>
├── Known Providers Configuration
│   ├── PETALTONGUE_KNOWN_PROVIDERS env var
│   ├── Direct HTTP probing
│   └── Guaranteed connectivity
└── Discovery Priority
    1. mDNS auto-discovery
    2. Known providers list
    3. Discovery hints (existing)
    4. Legacy BIOMEOS_URL
    5. Mock fallback
```

**Implementation Details**:
- Use `socket2` crate for multicast joins
- Use `tokio::net::UdpSocket` for async I/O
- Parse DNS-SD (RFC 6763) service records
- Support TXT records for metadata (capabilities, protocols, versions)
- Timeout: 5 seconds for discovery
- Refresh interval: 30 seconds

**Configuration**:
```bash
# Enable mDNS discovery (default: true)
PETALTONGUE_ENABLE_MDNS=true

# Known providers (guaranteed connectivity)
PETALTONGUE_KNOWN_PROVIDERS="http://localhost:9000,http://tower2:9000"

# Discovery timeout (seconds)
PETALTONGUE_DISCOVERY_TIMEOUT=5

# Discovery refresh interval (seconds)
PETALTONGUE_DISCOVERY_REFRESH=30
```

### Phase 2: Caching Layer (Week 2)

```
Caching Layer
├── ProviderCache
│   ├── Discovered providers (TTL: 30s)
│   ├── Primal lists (TTL: 10s)
│   ├── Topology data (TTL: 10s)
│   └── Health status (TTL: 5s)
├── CachedVisualizationProvider (wrapper)
│   ├── Wraps any provider
│   ├── Caches responses
│   ├── Automatic invalidation
│   └── Cache statistics
└── Configuration
    ├── Cache sizes (LRU)
    ├── TTLs per data type
    └── Enable/disable per cache type
```

**Implementation Details**:
- Use `lru` crate for LRU cache
- Async-safe with `tokio::sync::RwLock`
- Configurable TTLs per data type
- Optional cache statistics/metrics
- Cache invalidation on errors

**API**:
```rust
pub struct CachedVisualizationProvider {
    inner: Box<dyn VisualizationDataProvider>,
    cache: Arc<RwLock<ProviderCache>>,
    config: CacheConfig,
}

pub struct CacheConfig {
    pub provider_ttl: Duration,      // 30s
    pub primal_list_ttl: Duration,   // 10s
    pub topology_ttl: Duration,      // 10s
    pub health_ttl: Duration,        // 5s
    pub max_entries: usize,          // 100
}
```

### Phase 3: Protocol Support (Week 3)

```
Protocol Support
├── TarpcVisualizationProvider
│   ├── Uses Songbird's tarpc client
│   ├── Direct primal-to-primal communication
│   ├── More efficient than HTTP
│   └── Native async RPC
├── Protocol Negotiation
│   ├── Detect supported protocols from mDNS
│   ├── Prefer tarpc over HTTP (lower latency)
│   ├── Fallback to HTTP if tarpc unavailable
│   └── Cache protocol preferences
└── Connection Pooling
    ├── Connection reuse
    ├── Configurable pool size
    └── Idle timeout
```

**Implementation Details**:
- Depend on `songbird-client` crate (if public)
- Or implement minimal tarpc client for specific endpoints
- Protocol detection from mDNS TXT records
- Connection pooling with `deadpool` or similar

**API**:
```rust
pub struct TarpcVisualizationProvider {
    client: Arc<SongbirdClient>,
    endpoint: SocketAddr,
    capabilities: Vec<String>,
}

impl VisualizationDataProvider for TarpcVisualizationProvider {
    async fn get_primals(&self) -> Result<Vec<PrimalInfo>> {
        // Call Songbird's get_peers() RPC
        let peers = self.client.get_peers().await?;
        Ok(peers.into_iter().map(Into::into).collect())
    }
    
    async fn get_topology(&self) -> Result<Vec<TopologyEdge>> {
        // Call Songbird's get_topology() RPC
        self.client.get_topology().await
    }
}
```

### Phase 4: Trust & Resilience (Week 4)

```
Trust & Resilience
├── Trust Verification
│   ├── Query BearDog for trust levels
│   ├── Genetic lineage validation
│   ├── Family membership checks
│   └── Capability restrictions
├── Retry Logic
│   ├── Exponential backoff
│   ├── Configurable max retries
│   ├── Per-operation timeouts
│   └── Circuit breaker pattern
└── Health Monitoring
    ├── Periodic health checks
    ├── Provider availability tracking
    ├── Automatic failover
    └── Health metrics
```

**Implementation Details**:
- Integrate with BearDog for trust evaluation
- Use `tokio-retry` or similar for retry logic
- Circuit breaker with `failsafe` crate
- Health check background task

**Configuration**:
```rust
pub struct ResilienceConfig {
    pub retry_max_attempts: u32,          // 3
    pub retry_initial_delay: Duration,    // 100ms
    pub retry_max_delay: Duration,        // 5s
    pub retry_multiplier: f64,            // 2.0
    pub circuit_breaker_threshold: u32,   // 5 failures
    pub circuit_breaker_timeout: Duration,// 30s
    pub health_check_interval: Duration,  // 10s
}
```

---

## 📋 Implementation Phases

### Phase 1: Network Discovery (Week 1)

**Goal**: Zero-config mDNS-based provider discovery

**Tasks**:
1. Add `socket2` dependency
2. Implement `MdnsVisualizationProvider`
   - UDP socket setup with multicast join
   - mDNS query for `_visualization-provider._tcp.local`
   - Parse DNS-SD service records
   - Extract provider metadata from TXT records
3. Add known providers support
4. Update discovery function priority
5. Add configuration options
6. Write tests (unit + integration)
7. Update documentation

**Success Criteria**:
- ✅ Auto-discovers providers on local network
- ✅ Works across subnets with IGMP
- ✅ Falls back gracefully if multicast unavailable
- ✅ Configuration via environment variables
- ✅ All tests passing

### Phase 2: Caching Layer (Week 2)

**Goal**: Reduce redundant API calls with intelligent caching

**Tasks**:
1. Add `lru` dependency
2. Implement `ProviderCache` with TTLs
3. Implement `CachedVisualizationProvider` wrapper
4. Add cache configuration
5. Add cache invalidation on errors
6. Add cache statistics (optional)
7. Write tests
8. Update documentation

**Success Criteria**:
- ✅ Reduces API calls by 80%+
- ✅ Configurable TTLs per data type
- ✅ Cache statistics available
- ✅ All tests passing

### Phase 3: Protocol Support (Week 3)

**Goal**: Support multiple protocols, prioritize tarpc for Songbird

**Tasks**:
1. Add tarpc dependencies
2. Implement `TarpcVisualizationProvider`
3. Add protocol negotiation logic
4. Update discovery to detect protocols
5. Add connection pooling
6. Write tests
7. Update documentation

**Success Criteria**:
- ✅ Can talk to Songbird via tarpc
- ✅ Auto-selects best protocol
- ✅ Connection pooling works
- ✅ All tests passing

### Phase 4: Trust & Resilience (Week 4)

**Goal**: Production hardening with trust and resilience

**Tasks**:
1. Add retry dependencies
2. Implement retry logic with exponential backoff
3. Add circuit breaker
4. Integrate trust verification (BearDog)
5. Add health monitoring
6. Write tests
7. Update documentation
8. Performance benchmarks

**Success Criteria**:
- ✅ Handles transient failures gracefully
- ✅ Trust levels displayed in UI
- ✅ Health monitoring works
- ✅ All tests passing
- ✅ Performance acceptable

---

## 🔌 API Changes

### New Trait Methods (Optional Extensions)

```rust
#[async_trait]
pub trait VisualizationDataProvider: Send + Sync {
    // Existing methods
    async fn get_primals(&self) -> Result<Vec<PrimalInfo>>;
    async fn get_topology(&self) -> Result<Vec<TopologyEdge>>;
    async fn health_check(&self) -> Result<String>;
    fn get_metadata(&self) -> ProviderMetadata;
    
    // NEW: Optional advanced methods
    async fn get_trust_level(&self) -> Result<TrustLevel> {
        Ok(TrustLevel::Unknown)  // Default implementation
    }
    
    async fn supports_protocol(&self, protocol: &str) -> bool {
        false  // Default: only base protocol supported
    }
    
    async fn get_capabilities(&self) -> Result<Vec<String>> {
        Ok(vec![])  // Default: no capabilities
    }
}
```

### New Configuration Options

```bash
# Network Discovery
PETALTONGUE_ENABLE_MDNS=true
PETALTONGUE_KNOWN_PROVIDERS="http://localhost:9000,http://tower2:9000"
PETALTONGUE_DISCOVERY_TIMEOUT=5
PETALTONGUE_DISCOVERY_REFRESH=30

# Caching
PETALTONGUE_ENABLE_CACHE=true
PETALTONGUE_CACHE_PROVIDER_TTL=30
PETALTONGUE_CACHE_PRIMAL_TTL=10
PETALTONGUE_CACHE_TOPOLOGY_TTL=10
PETALTONGUE_CACHE_MAX_ENTRIES=100

# Protocol
PETALTONGUE_PREFER_PROTOCOL="tarpc"  # tarpc, http, auto
PETALTONGUE_CONNECTION_POOL_SIZE=10

# Resilience
PETALTONGUE_RETRY_MAX_ATTEMPTS=3
PETALTONGUE_RETRY_INITIAL_DELAY_MS=100
PETALTONGUE_RETRY_MAX_DELAY_MS=5000
PETALTONGUE_CIRCUIT_BREAKER_THRESHOLD=5
PETALTONGUE_HEALTH_CHECK_INTERVAL=10

# Trust
PETALTONGUE_ENABLE_TRUST_VERIFICATION=false
PETALTONGUE_MIN_TRUST_LEVEL=0  # 0=any, 1=limited, 2=full
```

---

## 🧪 Testing Strategy

### Unit Tests
- Each provider implementation
- Cache behavior (hit/miss/expiry)
- Protocol negotiation
- Retry logic
- Circuit breaker

### Integration Tests
- mDNS discovery on localhost
- Multi-provider aggregation
- Cache integration
- Protocol fallback
- Trust verification

### Manual Testing
- Cross-network mDNS discovery
- Multi-tower scenarios
- Network failure scenarios
- Trust level display

### Performance Tests
- Discovery latency
- Cache hit rate
- Protocol comparison (HTTP vs tarpc)
- Multi-provider overhead

---

## 📊 Success Metrics

### Performance
- Discovery latency: < 1s for mDNS
- Cache hit rate: > 80%
- API call reduction: > 80%
- tarpc latency: < 10ms vs HTTP

### Reliability
- Discovery success rate: > 95%
- Retry success rate: > 90%
- Circuit breaker prevents cascading failures
- Graceful degradation in all scenarios

### Usability
- Zero-config for local networks
- Clear error messages
- Trust levels visible in UI
- Configuration well-documented

---

## 🔐 Security Considerations

### Trust Verification
- Query BearDog for trust evaluation
- Display trust levels in UI
- Filter untrusted primals (optional)
- Respect family boundaries

### Network Security
- Use HTTPS when available
- Verify TLS certificates
- Support mutual TLS
- Genetic lineage validation

### Data Privacy
- No telemetry to third parties
- Local-only discovery
- User-controlled trust levels
- Transparent operation

---

## 📚 Dependencies

### New Crates (Phase 1)
```toml
socket2 = "0.5"           # Low-level socket for multicast
```

### New Crates (Phase 2)
```toml
lru = "0.12"              # LRU cache implementation
```

### New Crates (Phase 3)
```toml
# Option 1: Use Songbird's client (if public)
songbird-client = { path = "../../phase1/songbird/crates/songbird-client" }

# Option 2: Use tarpc directly
tarpc = { version = "0.33", features = ["tokio1", "serde1"] }
```

### New Crates (Phase 4)
```toml
tokio-retry = "0.3"       # Retry logic
failsafe = "1.2"          # Circuit breaker
```

---

## 🎯 Backward Compatibility

### Guaranteed Compatibility
- ✅ Existing `BIOMEOS_URL` still works
- ✅ Existing `PETALTONGUE_DISCOVERY_HINTS` still works
- ✅ Existing `PETALTONGUE_MOCK_MODE` still works
- ✅ All existing provider implementations continue working
- ✅ No breaking changes to trait interface

### Migration Path
1. Phase 1: Add mDNS, old methods still work
2. Phase 2: Add caching, transparent to users
3. Phase 3: Add protocols, auto-selected
4. Phase 4: Add trust, opt-in feature

**Users can upgrade incrementally with zero downtime.**

---

## 📖 Documentation Updates

### User Documentation
- Update `README.md` with mDNS discovery
- Update `DEPLOYMENT_GUIDE.md` with new config options
- Update `QUICK_REFERENCE.md` with discovery commands
- Add troubleshooting section for mDNS

### Developer Documentation
- Update `cargo doc` with new provider types
- Add discovery architecture diagram
- Add protocol negotiation flowchart
- Add caching behavior documentation

### Operations Documentation
- Add mDNS troubleshooting guide
- Add network requirements
- Add firewall configuration
- Add monitoring recommendations

---

## 🎊 Success Criteria

### Phase 1 Complete When:
- [ ] mDNS discovery works on local network
- [ ] Known providers work
- [ ] All tests passing (15+ new tests)
- [ ] Documentation updated
- [ ] Zero regressions

### Phase 2 Complete When:
- [ ] Caching reduces API calls by 80%+
- [ ] All tests passing (10+ new tests)
- [ ] Cache statistics available
- [ ] Documentation updated
- [ ] Zero regressions

### Phase 3 Complete When:
- [ ] tarpc provider works with Songbird
- [ ] Protocol negotiation works
- [ ] All tests passing (12+ new tests)
- [ ] Documentation updated
- [ ] Zero regressions

### Phase 4 Complete When:
- [ ] Retry logic handles transient failures
- [ ] Trust verification works
- [ ] All tests passing (15+ new tests)
- [ ] Documentation updated
- [ ] Performance benchmarks pass
- [ ] Zero regressions

---

## 🚀 Deployment

### Rollout Strategy
1. **Week 1**: Deploy Phase 1 (mDNS) to dev environment
2. **Week 2**: Deploy to staging, add Phase 2 (caching)
3. **Week 3**: Production deployment, add Phase 3 (protocols)
4. **Week 4**: Full deployment, add Phase 4 (trust)

### Monitoring
- Discovery success/failure rates
- Cache hit rates
- Protocol usage statistics
- Trust verification metrics
- Error rates by phase

### Rollback Plan
- Each phase is independent
- Can disable via configuration
- Existing methods always work
- Zero downtime rollback

---

## 📞 References

### Specifications
- This document (specification)
- `DISCOVERY_INFRASTRUCTURE_COMPARISON_JAN_3_2026.md` (analysis)
- `specs/archive/PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md` (archived; superseded by GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md and UNIVERSAL_VISUALIZATION_PIPELINE.md)

### Implementation
- `crates/petal-tongue-discovery/` (current implementation)
- Songbird: `crates/songbird-discovery/` (mDNS reference)
- biomeOS: `crates/biomeos-core/src/primal_client/` (client reference)

### Standards
- RFC 6762: Multicast DNS
- RFC 6763: DNS-Based Service Discovery
- RFC 1918: Private IPv4 Address Space
- RFC 2365: Administratively Scoped IP Multicast

---

**Version**: 1.0  
**Status**: Approved for Implementation  
**Start Date**: January 3, 2026  
**Estimated Completion**: February 7, 2026 (5 weeks)

---

*"Evolution through learning: Take the best from Songbird and biomeOS, keep what makes petalTongue unique."* 🌸

