# 🔍 Discovery Infrastructure Evolution Specification

**Version**: 2.0  
**Date**: January 3, 2026  
**Status**: Specification → Implementation  
**Philosophy**: Modern, Idiomatic, Async, Concurrent Rust

---

## 🎯 Purpose

Evolve petalTongue's discovery infrastructure to **production-grade**, **multi-protocol**, **async**, and **concurrent** Rust with comprehensive error handling, retry logic, caching, and graceful degradation.

**Core Principle**: Zero hardcoded dependencies, capability-based discovery, multi-provider aggregation.

---

## 📊 Current State Assessment

### **What We Have** ✅

**Architecture**:
- ✅ Trait-based abstraction (`VisualizationDataProvider`)
- ✅ Multi-provider support (HTTP, mDNS, Mock)
- ✅ Environment-based configuration
- ✅ Zero hardcoded primal knowledge (TRUE PRIMAL)

**Implementation**:
- ✅ Basic async support (`async_trait`)
- ✅ HTTP provider with health checks
- ✅ mDNS provider (feature-gated)
- ✅ Mock provider for testing
- ✅ Built-in caching (per-provider)

**Testing**:
- ✅ Basic unit tests
- ✅ Mock mode for development

### **What Needs Evolution** 📋

**1. Concurrency & Performance**:
- ❌ No concurrent provider discovery
- ❌ Sequential health checks (slow)
- ❌ No timeout management
- ❌ No connection pooling

**2. Resilience & Reliability**:
- ❌ No retry logic with exponential backoff
- ❌ No circuit breaker pattern
- ❌ Limited error context
- ❌ No graceful degradation

**3. Protocol Support**:
- ❌ HTTP only (no tarpc, gRPC)
- ❌ No WebSocket for real-time updates
- ❌ No mDNS fallback chain

**4. Observability**:
- ❌ Limited metrics
- ❌ No structured logging context
- ❌ No distributed tracing
- ❌ No health monitoring dashboard

**5. Testing**:
- ❌ No integration tests with real services
- ❌ No chaos/fault testing
- ❌ Limited async test coverage

---

## 🚀 Evolution Goals

### **Phase 1: Modern Async & Concurrency** (This Session)

**Objectives**:
1. ✅ Concurrent provider discovery
2. ✅ Parallel health checks with timeout
3. ✅ Modern async patterns (tokio select, join, timeout)
4. ✅ Connection pooling (reqwest client reuse)
5. ✅ Comprehensive error types

**Deliverables**:
- [ ] `discovery/concurrent.rs` - Concurrent discovery coordinator
- [ ] `discovery/retry.rs` - Retry logic with exponential backoff
- [ ] `discovery/errors.rs` - Rich error types with context
- [ ] `discovery/pool.rs` - Connection pool manager
- [ ] Integration tests with tokio

**Success Metrics**:
- Discovery time: < 2s for 5 providers (parallel)
- Retry logic: 3 attempts with backoff
- Error context: Full chain with root cause
- Test coverage: 85%+ for new code

---

### **Phase 2: Multi-Protocol Support** (Future)

**Protocols to Add**:
1. **tarpc** - RPC for high-performance inter-primal communication
2. **gRPC** - Standard RPC with streaming support
3. **WebSocket** - Real-time event streaming
4. **Unix Sockets** - Local IPC (same machine)

**Architecture**:
```rust
pub enum DiscoveryProtocol {
    Http(HttpConfig),
    Tarpc(TarpcConfig),
    Grpc(GrpcConfig),
    WebSocket(WsConfig),
    UnixSocket(PathBuf),
}

pub struct MultiProtocolProvider {
    protocols: Vec<DiscoveryProtocol>,
    fallback_chain: Vec<DiscoveryProtocol>,
}
```

**Benefits**:
- Efficient RPC for frequent queries
- Real-time updates without polling
- Local optimization for same-machine primals

---

### **Phase 3: Resilience & Circuit Breaking** (Future)

**Patterns to Implement**:
1. **Circuit Breaker** - Fast-fail on repeated failures
2. **Bulkhead** - Isolate provider failures
3. **Retry with Jitter** - Avoid thundering herd
4. **Graceful Degradation** - Partial results on failure

**Implementation**:
```rust
pub struct CircuitBreaker {
    state: CircuitState,
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
    half_open_timeout: Duration,
}

pub enum CircuitState {
    Closed,       // Normal operation
    Open,         // Fast-fail mode
    HalfOpen,     // Testing recovery
}
```

**Metrics**:
- Failure rate per provider
- Circuit state changes
- Recovery time

---

### **Phase 4: Observability & Monitoring** (Future)

**Instrumentation**:
1. **Structured Logging** - tracing spans with context
2. **Metrics** - Prometheus-compatible exports
3. **Distributed Tracing** - OpenTelemetry support
4. **Health Dashboard** - Real-time provider status

**Example**:
```rust
#[tracing::instrument(skip(provider))]
async fn discover_from_provider(
    provider: &impl VisualizationDataProvider,
) -> Result<Vec<PrimalInfo>> {
    let span = tracing::info_span!("discover", 
        provider = %provider.get_metadata().name,
        protocol = %provider.get_metadata().protocol
    );
    
    async move {
        // Discovery logic with full tracing
    }.instrument(span).await
}
```

---

## 💡 Modern Rust Patterns to Apply

### **1. Concurrent Discovery with `tokio::select!`**

**Pattern**: Try multiple providers concurrently, use first success

```rust
use tokio::time::{timeout, Duration};

pub async fn discover_first_available(
    providers: Vec<Box<dyn VisualizationDataProvider>>,
    timeout_duration: Duration,
) -> Result<Box<dyn VisualizationDataProvider>> {
    use tokio::select;
    
    let mut futures = providers
        .into_iter()
        .map(|p| Box::pin(async move {
            timeout(timeout_duration, p.health_check()).await??;
            Ok::<_, anyhow::Error>(p)
        }))
        .collect::<Vec<_>>();
    
    // Return first provider that succeeds
    loop {
        if futures.is_empty() {
            anyhow::bail!("No providers available");
        }
        
        let (result, _index, remaining) = select_all(futures).await;
        
        match result {
            Ok(provider) => return Ok(provider),
            Err(_) => {
                futures = remaining;
                continue;
            }
        }
    }
}
```

**Benefits**:
- Fast discovery (first success wins)
- Automatic failover
- Timeout protection

---

### **2. Parallel Health Checks with `join_all`**

**Pattern**: Check all providers concurrently

```rust
use futures::future::join_all;
use tokio::time::timeout;

pub async fn check_all_providers(
    providers: &[Box<dyn VisualizationDataProvider>],
) -> Vec<ProviderHealth> {
    let checks = providers.iter().map(|provider| {
        let metadata = provider.get_metadata();
        async move {
            let result = timeout(
                Duration::from_secs(5),
                provider.health_check()
            ).await;
            
            ProviderHealth {
                metadata,
                status: match result {
                    Ok(Ok(msg)) => HealthStatus::Healthy(msg),
                    Ok(Err(e)) => HealthStatus::Unhealthy(e.to_string()),
                    Err(_) => HealthStatus::Timeout,
                },
                checked_at: Instant::now(),
            }
        }
    });
    
    join_all(checks).await
}
```

**Benefits**:
- Parallel execution (O(1) time, not O(n))
- Individual timeouts
- Complete health picture

---

### **3. Retry with Exponential Backoff**

**Pattern**: Retry failures with increasing delays

```rust
use tokio::time::{sleep, Duration};
use rand::Rng;

pub struct RetryPolicy {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
    pub jitter: bool,
}

impl RetryPolicy {
    pub async fn execute<F, Fut, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut delay = self.initial_delay;
        let mut last_error = None;
        
        for attempt in 1..=self.max_attempts {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.max_attempts {
                        let sleep_duration = if self.jitter {
                            let jitter = rand::thread_rng()
                                .gen_range(0.8..1.2);
                            delay.mul_f64(jitter)
                        } else {
                            delay
                        };
                        
                        tracing::warn!(
                            "Attempt {} failed, retrying in {:?}",
                            attempt,
                            sleep_duration
                        );
                        
                        sleep(sleep_duration).await;
                        
                        delay = (delay * self.backoff_factor as u32)
                            .min(self.max_delay);
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
}
```

**Benefits**:
- Handles transient failures
- Prevents thundering herd (jitter)
- Configurable backoff

---

### **4. Connection Pool with `Arc<reqwest::Client>`**

**Pattern**: Reuse HTTP client across requests

```rust
use std::sync::Arc;
use reqwest::Client;

pub struct HttpProvider {
    client: Arc<Client>,
    base_url: String,
    timeout: Duration,
}

impl HttpProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .build()
            .expect("Failed to build HTTP client");
        
        Self {
            client: Arc::new(client),
            base_url: base_url.into(),
            timeout: Duration::from_secs(30),
        }
    }
    
    pub async fn get_primals(&self) -> Result<Vec<PrimalInfo>> {
        let url = format!("{}/api/v1/primals", self.base_url);
        
        let response = self.client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await?;
        
        let primals = response.json().await?;
        Ok(primals)
    }
}
```

**Benefits**:
- Connection reuse (performance)
- TCP keepalive (fewer handshakes)
- Configurable pooling

---

### **5. Rich Error Types with `thiserror`**

**Pattern**: Structured errors with context

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("No providers found after trying {attempted} sources")]
    NoProvidersFound { attempted: Vec<String> },
    
    #[error("Provider {name} failed health check: {source}")]
    HealthCheckFailed {
        name: String,
        #[source]
        source: anyhow::Error,
    },
    
    #[error("Discovery timeout after {duration:?}")]
    Timeout { duration: Duration },
    
    #[error("All {count} providers failed")]
    AllProvidersFailed { count: usize },
    
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("mDNS discovery failed: {0}")]
    MdnsError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
```

**Benefits**:
- Type-safe error handling
- Automatic error chaining
- Rich context for debugging

---

### **6. Graceful Degradation**

**Pattern**: Return partial results on failure

```rust
pub struct DiscoveryResult {
    pub providers: Vec<Box<dyn VisualizationDataProvider>>,
    pub failures: Vec<DiscoveryFailure>,
}

pub struct DiscoveryFailure {
    pub source: String,
    pub error: anyhow::Error,
    pub timestamp: Instant,
}

pub async fn discover_with_degradation() -> DiscoveryResult {
    let mut providers = Vec::new();
    let mut failures = Vec::new();
    
    // Try mDNS
    match discover_mdns().await {
        Ok(p) => providers.extend(p),
        Err(e) => failures.push(DiscoveryFailure {
            source: "mDNS".to_string(),
            error: e,
            timestamp: Instant::now(),
        }),
    }
    
    // Try environment hints
    match discover_env_hints().await {
        Ok(p) => providers.extend(p),
        Err(e) => failures.push(DiscoveryFailure {
            source: "ENV".to_string(),
            error: e,
            timestamp: Instant::now(),
        }),
    }
    
    // Log failures but don't fail overall if we have at least one provider
    for failure in &failures {
        tracing::warn!(
            "Discovery source {} failed: {}",
            failure.source,
            failure.error
        );
    }
    
    DiscoveryResult { providers, failures }
}
```

**Benefits**:
- Continue with partial results
- Better observability
- User-friendly degradation

---

## 📝 Implementation Plan

### **Step 1: Create Error Types**

File: `crates/petal-tongue-discovery/src/errors.rs`

```rust
use thiserror::Error;
use std::time::Duration;

#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("No providers found")]
    NoProvidersFound,
    
    #[error("Provider health check failed")]
    HealthCheckFailed(#[source] anyhow::Error),
    
    #[error("Discovery timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("mDNS error: {0}")]
    Mdns(String),
}
```

---

### **Step 2: Add Retry Logic**

File: `crates/petal-tongue-discovery/src/retry.rs`

```rust
use tokio::time::{sleep, Duration};

pub struct RetryPolicy {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
        }
    }
}

// Implementation from above
```

---

### **Step 3: Concurrent Discovery**

File: `crates/petal-tongue-discovery/src/concurrent.rs`

```rust
use futures::future::join_all;
use tokio::time::timeout;

pub async fn discover_concurrent(
    sources: Vec<DiscoverySource>,
) -> Vec<Result<Box<dyn VisualizationDataProvider>>> {
    let discoveries = sources.into_iter().map(|source| {
        async move {
            timeout(
                Duration::from_secs(5),
                source.discover()
            ).await?
        }
    });
    
    join_all(discoveries).await
}
```

---

### **Step 4: Connection Pooling**

Update: `crates/petal-tongue-discovery/src/http_provider.rs`

```rust
use std::sync::Arc;

pub struct HttpVisualizationProvider {
    client: Arc<Client>,  // Reuse client
    base_url: String,
    cache: Arc<RwLock<ProviderCache>>,
}

// Implementation from above
```

---

### **Step 5: Integration Tests**

File: `crates/petal-tongue-discovery/tests/concurrent_tests.rs`

```rust
#[tokio::test]
async fn test_parallel_discovery() {
    // Start multiple mock servers
    let servers = start_mock_servers(3).await;
    
    let start = Instant::now();
    let providers = discover_all(&servers).await.unwrap();
    let elapsed = start.elapsed();
    
    assert_eq!(providers.len(), 3);
    assert!(elapsed < Duration::from_secs(2), "Should be parallel");
}

#[tokio::test]
async fn test_retry_on_failure() {
    let flaky_server = start_flaky_server(2).await; // Fails 2x
    
    let policy = RetryPolicy::default();
    let result = policy.execute(|| async {
        discover_from(&flaky_server).await
    }).await;
    
    assert!(result.is_ok(), "Should succeed after retries");
}
```

---

## 🎯 Success Criteria

### **Code Quality**:
- [ ] All clippy warnings resolved
- [ ] cargo fmt applied
- [ ] No unsafe code
- [ ] Comprehensive documentation

### **Performance**:
- [ ] Parallel discovery < 2s for 5 providers
- [ ] Connection pooling (reuse clients)
- [ ] Timeout protection on all operations

### **Resilience**:
- [ ] Retry logic with exponential backoff
- [ ] Graceful degradation on partial failure
- [ ] Rich error context

### **Testing**:
- [ ] 85%+ test coverage (new code)
- [ ] Integration tests with tokio
- [ ] Concurrent test scenarios
- [ ] Fault injection tests

### **Documentation**:
- [ ] This spec document
- [ ] Updated module docs
- [ ] Code examples in docs
- [ ] Migration guide

---

## 📚 Dependencies to Add

```toml
[dependencies]
# Existing
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
reqwest = { version = "0.11", features = ["json"] }

# New for modern async patterns
futures = "0.3"
thiserror = "1.0"
tracing = "0.1"

# For retry and resilience
backoff = "0.4"  # Or implement our own

# For concurrent testing
tokio-test = "0.4"

[dev-dependencies]
wiremock = "0.5"  # Mock HTTP servers for testing
```

---

## 🚀 Migration Strategy

### **Backward Compatibility**:
1. Keep existing `discover_visualization_providers()` function
2. Add new `discover_concurrent()` as alternative
3. Gradual migration in UI code
4. Feature flag for new implementation

### **Rollout Plan**:
1. **Week 1**: Implement concurrent discovery + tests
2. **Week 2**: Add retry logic + fault tolerance
3. **Week 3**: Integration testing with live primals
4. **Week 4**: Replace old implementation

---

## 📊 Metrics to Track

**Discovery Performance**:
- Discovery time (p50, p95, p99)
- Provider count discovered
- Failure rate per source

**Resilience**:
- Retry success rate
- Circuit breaker trips
- Degraded mode activations

**Observability**:
- Log volume
- Tracing span counts
- Error rates by category

---

**Status**: ✅ **Specification Complete**  
**Next**: Begin implementation (concurrent.rs, retry.rs, errors.rs)  
**Philosophy**: Modern, idiomatic, async, concurrent Rust

🌸 **Discovery infrastructure evolution: from working → production-grade!** 🚀

