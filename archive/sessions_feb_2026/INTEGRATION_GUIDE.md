# 🚀 Integration Guide - New Systems

**Quick guide for integrating the new architectural systems**

---

## ⚡ Quick Start (30 minutes)

### 1. Replace Toadstool Implementation (5 minutes)

```bash
cd crates/petal-tongue-ui/src/display/backends/
mv toadstool.rs toadstool_old.rs.bak
mv toadstool_v2.rs toadstool.rs
```

### 2. Update Main.rs to Use Config (10 minutes)

```rust
// At the top of main.rs
use petal_tongue_core::config_system::Config;

// In main() function, add:
let config = Config::from_env()
    .expect("Failed to load configuration");

// Replace hardcoded values throughout with:
let port = config.network.web_port;
let socket = config.network.ipc_socket.clone();
let data_dir = config.paths.data_dir.clone();
// etc.
```

### 3. Migrate to Capability Discovery (15 minutes)

**Replace hardcoded primal lookups:**

```rust
// OLD WAY (hardcoded):
let beardog_client = connect_to_beardog().await?;

// NEW WAY (capability-based):
use petal_tongue_core::{
    capability_discovery::{CapabilityDiscovery, CapabilityQuery},
    biomeos_discovery::BiomeOsBackend,
};

let backend = BiomeOsBackend::from_env()?;
let discovery = CapabilityDiscovery::new(Box::new(backend));

// Discover by capability (not by name!)
let endpoint = discovery.discover_one(
    &CapabilityQuery::new("crypto.signing")
).await?;

// Connect using discovered endpoint
let client = connect_via_tarpc(&endpoint.endpoints.tarpc).await?;
```

---

## 📋 Integration Checklist

### Phase 1: Core Systems (2-3 days)

- [ ] **Config Integration**
  - [ ] Update main.rs to use Config::from_env()
  - [ ] Replace hardcoded ports with config.network.*
  - [ ] Replace hardcoded paths with config.paths.*
  - [ ] Replace hardcoded thresholds with config.thresholds.*
  - [ ] Test with environment variables
  - [ ] Test with config file

- [ ] **Discovery Integration**
  - [ ] Update BiomeOSProvider to use new discovery (already done!)
  - [ ] Update ToadstoolDisplay to use new discovery (already done!)
  - [ ] Find and replace other hardcoded primal lookups
  - [ ] Test discovery with running biomeOS
  - [ ] Test fallback when primals unavailable

- [ ] **Toadstool Integration**
  - [ ] Verify toadstool_v2.rs is now toadstool.rs
  - [ ] Test with running toadStool
  - [ ] Verify tarpc performance
  - [ ] Test graceful degradation

### Phase 2: Testing (1 week)

- [ ] **Unit Tests**
  - [ ] Run cargo test --all
  - [ ] Fix any integration issues
  - [ ] Add missing test cases
  - [ ] Verify adapters tests pass
  - [ ] Verify telemetry tests pass

- [ ] **Integration Tests**
  - [ ] Test with live biomeOS
  - [ ] Test with live toadStool
  - [ ] Test config from environment
  - [ ] Test config from file
  - [ ] Test discovery fallback
  - [ ] Test offline mode

- [ ] **Coverage**
  - [ ] Install llvm-cov: `cargo install cargo-llvm-cov`
  - [ ] Run: `cargo llvm-cov --all --html`
  - [ ] Target: 90% coverage
  - [ ] Add tests for gaps

### Phase 3: Cleanup (Optional)

- [ ] **Smart Refactoring**
  - [ ] Execute app.rs decomposition (3-4 days)
  - [ ] Execute visual_2d.rs pipeline (3-4 days)
  - [ ] Comprehensive testing after refactor

---

## 🔍 Finding Hardcoded Values

Use these patterns to find remaining hardcoded values:

```bash
# Find hardcoded ports
rg -i ":\d{4,5}" --type rust | grep -v "://"

# Find hardcoded paths
rg '"/tmp/' --type rust
rg '"/var/' --type rust
rg '"/run/' --type rust

# Find hardcoded primal names (common ones)
rg -w "beardog|toadstool|biomeos|petaltongue" --type rust -i

# Find potential unwraps in production (should be rare now)
rg "\.unwrap\(\)" --type rust crates/ | grep -v "tests/" | grep -v "#\[cfg(test)\]"
```

---

## 🛠️ Configuration Examples

### Environment Variables

```bash
# Network
export PETALTONGUE_WEB_PORT=8080
export PETALTONGUE_IPC_SOCKET=/run/user/1000/petaltongue.sock

# Paths (XDG-compliant)
export XDG_DATA_HOME=$HOME/.local/share
export XDG_CONFIG_HOME=$HOME/.config
export XDG_CACHE_HOME=$HOME/.cache

# Discovery
export BIOMEOS_SOCKET=/run/user/1000/biomeos-neural-api.sock
export DISCOVERY_TIMEOUT_MS=5000

# Thresholds
export HEALTH_CHECK_INTERVAL_MS=10000
export CONNECTION_TIMEOUT_MS=30000
```

### Config File (config.toml)

```toml
# $XDG_CONFIG_HOME/petaltongue/config.toml

[network]
web_port = 8080
ipc_socket = "/run/user/1000/petaltongue.sock"
api_port = 8081

[paths]
data_dir = "/home/user/.local/share/petaltongue"
config_dir = "/home/user/.config/petaltongue"
cache_dir = "/home/user/.cache/petaltongue"

[discovery]
backend = "biomeos"
timeout_ms = 5000
cache_ttl_seconds = 300

[thresholds]
health_check_interval_ms = 10000
connection_timeout_ms = 30000
max_retry_attempts = 3

[performance]
worker_threads = 4
max_connections = 100
buffer_size_kb = 64
```

### Loading Priority

1. **Environment variables** (highest priority)
2. **Config file** ($XDG_CONFIG_HOME/petaltongue/config.toml)
3. **XDG defaults** (automatic)
4. **Built-in defaults** (fallback)

---

## 🧪 Testing Strategy

### Unit Tests

```bash
# Test everything
cargo test --all

# Test specific modules
cargo test -p petal-tongue-core
cargo test -p petal-tongue-adapters
cargo test -p petal-tongue-telemetry
```

### Integration Tests

```bash
# With live services (requires biomeOS + toadStool running)
BIOMEOS_SOCKET=/run/user/1000/biomeos.sock cargo test --test integration
```

### Coverage

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --all --html

# View report
xdg-open target/llvm-cov/html/index.html

# Target: 90% coverage
```

---

## 🐛 Troubleshooting

### Config Not Loading

```bash
# Check environment
env | grep PETALTONGUE

# Check XDG paths
env | grep XDG

# Test config loading
cargo run -- --config-test
```

### Discovery Failing

```bash
# Verify biomeOS is running
ls -la /run/user/$(id -u)/biomeos*.sock

# Test discovery manually
cargo run --bin test-discovery

# Check logs
RUST_LOG=debug cargo run
```

### Toadstool Not Connecting

```bash
# Verify toadStool is running
ps aux | grep toadstool

# Check socket
ls -la /run/user/$(id -u)/toadstool*.sock

# Test connection
cargo run --bin test-toadstool
```

---

## 📊 Verification

After integration, verify:

✅ No hardcoded primal names in production code  
✅ All config comes from environment or files  
✅ Discovery works with live biomeOS  
✅ Toadstool works with tarpc  
✅ Tests pass with new systems  
✅ Coverage at 90%+ (target)

---

## 🎯 Success Criteria

**Core Integration** (Must have):
- [ ] Config system integrated
- [ ] Discovery system integrated
- [ ] Toadstool tarpc integrated
- [ ] All tests pass
- [ ] No hardcoded values in production

**Quality** (Should have):
- [ ] 90% test coverage
- [ ] Live testing complete
- [ ] Documentation updated
- [ ] Examples working

**Excellence** (Nice to have):
- [ ] Smart refactoring complete
- [ ] Performance benchmarks
- [ ] Stress testing
- [ ] Deployment guide

---

## 📚 References

**Documentation**:
- `COMPREHENSIVE_EVOLUTION_COMPLETE_JAN_31_2026.md` - Full report
- `EVOLUTION_QUICK_REF.md` - Quick reference
- `DEAD_CODE_EVOLUTION_ANALYSIS.md` - Dead code analysis

**New Code**:
- `crates/petal-tongue-core/src/capability_discovery.rs`
- `crates/petal-tongue-core/src/biomeos_discovery.rs`
- `crates/petal-tongue-core/src/config_system.rs`
- `crates/petal-tongue-ui/src/display/backends/toadstool_v2.rs`

**Standards**:
- `wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md`
- `wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md`
- `wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md`
- `wateringHole/PRIMAL_IPC_PROTOCOL.md`

---

## 💡 Tips

1. **Start Small**: Integrate one system at a time
2. **Test Often**: Run tests after each change
3. **Use Logs**: RUST_LOG=debug for troubleshooting
4. **Check Docs**: All new systems are well-documented
5. **Ask Questions**: Systems are designed to be self-explanatory

---

**Integration Time**: 2-3 days  
**Testing Time**: 1 week  
**Optional Refactoring**: 1-2 weeks

**Total to 95%+ Compliance**: 2-4 weeks

🌸 **Happy Integrating!** 🌸
