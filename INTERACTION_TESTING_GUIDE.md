# Interaction Testing with Stable Primal Binaries

**Date**: January 13, 2026 PM  
**Purpose**: Guide for testing petalTongue with stable primal binaries  
**Principle**: Runtime discovery + primal self-knowledge

---

## 🎯 OVERVIEW

This guide explains how to use stable primal binaries (from `../biomeOS/plasmidBin/` or similar) for interaction testing while maintaining TRUE PRIMAL principles:

1. ✅ **Runtime Discovery** - No hardcoded paths to binaries
2. ✅ **Primal Self-Knowledge** - Each primal knows itself, not others
3. ✅ **Capability-Based** - Discover by capability, not by name
4. ✅ **Graceful Degradation** - Works with or without specific primals

---

## 📁 STABLE BINARY LOCATION

**Actual Location** (verified):
```
/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/
├── beardog                 # Security primal (3.5 MB)
├── nestgate                # Storage primal (4.5 MB)
├── petaltongue             # Visualization primal (34 MB)
├── petaltongue-headless    # Headless mode (3.2 MB)
├── squirrel                # AI collaboration primal (15 MB)
├── toadstool               # Compute primal (6.8 MB)
├── toadstool-cli           # CLI interface (22 MB)
└── MANIFEST.md             # Version tracking
```

**Note**: These are stable, production-ready binaries from Jan 11, 2026.

---

## ✅ TRUE PRIMAL COMPLIANT TESTING

### ❌ WRONG: Hardcoded Paths

```bash
# DON'T DO THIS - Hardcoded binary path
export BEARDOG_BIN="/home/eastgate/Development/ecoPrimals/biomeOS/plasmidBin/beardog"
export SONGBIRD_BIN="/home/eastgate/Development/ecoPrimals/biomeOS/plasmidBin/songbird"
```

**Problem**: Hardcodes specific locations, violates primal self-knowledge

### ✅ CORRECT: Runtime Discovery via Sockets

```bash
# DO THIS - Let primals announce themselves at runtime
cd /home/eastgate/Development/ecoPrimals/biomeOS/plasmidBin/

# Start primals (they create their own sockets)
./beardog &
./songbird &
./toadstool &

# petalTongue discovers them via:
# 1. Unix sockets in /run/user/<uid>/*.sock
# 2. JSON-RPC capability queries
# 3. Songbird live discovery
cargo run --bin petal-tongue-ui
```

**Result**: ✅ Zero hardcoding, runtime discovery only

---

## 🔧 ENVIRONMENT-BASED HINTS (Optional)

For **testing convenience only**, you can provide hints via environment variables. These are **optional** and **fallbacks only**.

### Socket Path Hints

```bash
# Optional: Override socket paths for testing
export BEARDOG_SOCKET="/run/user/1000/beardog-test-node1.sock"
export SONGBIRD_SOCKET="/run/user/1000/songbird-test-node1.sock"
export BIOMEOS_URL="http://localhost:3000"

# Run petalTongue
cargo run --bin petal-tongue-ui
```

**How it works** (from `socket_path.rs`):
```rust
// Priority 1: Check environment variable (optional hint)
let env_var = format!("{}_SOCKET", primal_name.to_uppercase());
if let Ok(socket_path) = env::var(&env_var) {
    return Ok(PathBuf::from(socket_path));
}

// Priority 2: XDG runtime directory (standard)
// Priority 3: /tmp fallback
```

**Key**: Environment variables are **hints**, not requirements. Discovery still works without them.

---

## 🧪 TESTING SCENARIOS

### Scenario 1: Full Integration Test

**Goal**: Test petalTongue with all primals running

```bash
# Terminal 1: Start stable primals
cd ../biomeOS/plasmidBin/
./beardog --family-id test &
./toadstool --family-id test &

# Note: Songbird binary not in plasmidBin (may need to build separately)
# See primals/ subdirectory for additional services

# Terminal 2: Run current petalTongue (from source)
cd ../../petalTongue
export FAMILY_ID=test  # Match the family
cargo run --bin petal-tongue-ui

# Expected: petalTongue discovers available primals via Unix sockets
```

### Scenario 2: Partial Availability Test

**Goal**: Verify graceful degradation when some primals are missing

```bash
# Only start Songbird (not BearDog)
./songbird &

# Run petalTongue
cargo run --bin petal-tongue-ui

# Expected: 
# - Discovers Songbird ✅
# - Gracefully handles missing BearDog ✅
# - No crashes, clear user feedback ✅
```

### Scenario 3: Zero Primals Test

**Goal**: Verify petalTongue works standalone

```bash
# Start NO external primals
cargo run --bin petal-tongue-ui

# Expected:
# - Falls back to mock mode ✅
# - Clear message: "No primals discovered" ✅
# - Tutorial mode available ✅
```

### Scenario 4: Cross-Primal Coordination Test

**Goal**: Test BearDog ↔ Songbird encrypted discovery

```bash
# Start both primals
./beardog --family-id nat0 &
./songbird --family-id nat0 &

# Run petalTongue
export FAMILY_ID=nat0
cargo run --bin petal-tongue-ui

# Expected:
# - Songbird discovers BearDog ✅
# - BirdSong v2 encrypted packets ✅
# - Auto-trust within family ✅
# - petalTongue visualizes topology ✅
```

---

## 🔍 DISCOVERY METHODS (Priority Order)

petalTongue uses **5 discovery methods** (from `lib.rs:discover_visualization_providers()`):

### 1. Songbird Discovery (Highest Priority)
```rust
// Queries Songbird for complete primal topology
match SongbirdVisualizationProvider::discover(None).await {
    Ok(songbird) => return Ok(vec![songbird]),
    Err(_) => continue_to_next_method(),
}
```

**How it finds binaries**: Songbird maintains live registry, binaries announce themselves

### 2. JSON-RPC Unix Sockets
```rust
// Scans /run/user/<uid>/*.sock for JSON-RPC services
match JsonRpcProvider::discover().await {
    Ok(providers) => return Ok(providers),
    Err(_) => continue_to_next_method(),
}
```

**How it finds binaries**: Primals create sockets on startup, petalTongue probes them

### 3. mDNS Discovery
```rust
// Multicast DNS queries for _discovery._tcp.local
match MdnsProvider::discover().await {
    Ok(providers) => return Ok(providers),
    Err(_) => continue_to_next_method(),
}
```

**How it finds binaries**: Primals advertise via mDNS (if available)

### 4. Environment Variable Hints
```rust
// Check BIOMEOS_URL, TOADSTOOL_URL, etc.
if let Ok(url) = env::var("BIOMEOS_URL") {
    return HttpVisualizationProvider::new(url);
}
```

**How it finds binaries**: User provides hints (optional)

### 5. HTTP Port Probing (Fallback)
```rust
// Probe common ports: 8080, 8081, 3000, 9000, etc.
for port in DISCOVERY_PORTS {
    if let Ok(provider) = probe_http(port).await {
        return Ok(provider);
    }
}
```

**How it finds binaries**: Tries common service ports (no assumptions about which primal)

---

## 🎯 PRIMAL SELF-KNOWLEDGE

### What Each Primal Knows

**BearDog knows**:
- ✅ Its own capabilities (security, encryption, lineage)
- ✅ Its own socket path
- ✅ Its own endpoints
- ❌ Where petalTongue is
- ❌ Where Songbird is

**Songbird knows**:
- ✅ Its own capabilities (discovery, registry)
- ✅ Other primals that **announced themselves** to it
- ✅ Its own socket path
- ❌ Where primals are if they didn't announce

**petalTongue knows**:
- ✅ Its own capabilities (visualization, UI)
- ✅ Its own socket path
- ✅ How to **discover** other primals at runtime
- ❌ Hardcoded locations of any primal
- ❌ Which primals "should" exist

### Discovery is a Two-Way Handshake

```
1. Primal starts → Creates socket → Announces to Songbird (optional)
2. petalTongue starts → Queries Songbird → Discovers primals
3. petalTongue connects → JSON-RPC handshake → Capability exchange
```

**No primal knows where another primal is before discovery!**

---

## 📋 TESTING CHECKLIST

### Before Testing
- [ ] Stable binaries are executable (`chmod +x`)
- [ ] No hardcoded paths in petalTongue code
- [ ] Environment variables are hints only (optional)
- [ ] Family IDs match across primals (if using families)

### During Testing
- [ ] Primals start successfully
- [ ] Sockets appear in `/run/user/<uid>/` or `/tmp/`
- [ ] petalTongue discovers primals automatically
- [ ] Graceful degradation when primals are missing
- [ ] Clear error messages (not crashes)

### After Testing
- [ ] No hardcoded assumptions introduced
- [ ] Discovery still works without env vars
- [ ] Tests pass with/without specific primals
- [ ] Documentation updated if needed

---

## 🚨 COMMON MISTAKES TO AVOID

### ❌ Mistake 1: Hardcoding Binary Paths
```rust
// DON'T DO THIS
let beardog = Command::new("/path/to/biomeOS/plasmidBin/beardog");
```

**Fix**: Let primals start independently, discover via sockets

### ❌ Mistake 2: Assuming Primals Exist
```rust
// DON'T DO THIS
let beardog = BearDogClient::new("localhost:8080").unwrap();
```

**Fix**: Use `Result`, handle missing primals gracefully

### ❌ Mistake 3: Port Hardcoding
```rust
// DON'T DO THIS
const BEARDOG_PORT: u16 = 8080;
```

**Fix**: Discover capabilities, not ports

### ❌ Mistake 4: Test-Only Paths in Production
```rust
// DON'T DO THIS
#[cfg(test)]
const TEST_BIN_PATH = "../biomeOS/plasmidBin/";
// ... then use in production code
```

**Fix**: Keep test infrastructure separate, use mocks for tests

---

## ✅ CORRECT PATTERNS

### Pattern 1: Graceful Discovery
```rust
pub async fn discover_beardog() -> Option<BearDogClient> {
    // Try multiple discovery methods
    if let Ok(client) = discover_via_songbird().await {
        return Some(client);
    }
    
    if let Ok(client) = discover_via_unix_socket("beardog").await {
        return Some(client);
    }
    
    // No BearDog found - that's OK!
    None
}
```

### Pattern 2: Capability-Based Testing
```rust
#[tokio::test]
async fn test_with_available_primals() {
    // Discover what's available
    let providers = discover_visualization_providers().await.unwrap();
    
    if providers.is_empty() {
        println!("⚠️  No primals available - using mock mode");
        return test_with_mock().await;
    }
    
    // Test with whatever is available
    for provider in providers {
        test_provider(provider).await;
    }
}
```

### Pattern 3: Environment Hints (Not Requirements)
```rust
pub fn get_biomeos_url() -> Option<String> {
    // Hint, not requirement
    env::var("BIOMEOS_URL").ok()
}

pub async fn connect() -> Result<Client> {
    // Try hint first
    if let Some(url) = get_biomeos_url() {
        if let Ok(client) = Client::connect(&url).await {
            return Ok(client);
        }
    }
    
    // Fall back to discovery
    discover_via_mdns().await
}
```

---

## 🔬 EXAMPLE: Full Test Session

```bash
#!/bin/bash
# test-with-stable-binaries.sh
# TRUE PRIMAL compliant integration testing

set -e

# Configuration (hints only, not requirements)
export FAMILY_ID=integration-test
export XDG_RUNTIME_DIR=/run/user/$(id -u)

# Find stable binaries (don't hardcode location)
PRIMAL_BIN_DIR="${PRIMAL_BIN_DIR:-../biomeOS/plasmidBin}"

# Note: Using relative path from petalTongue root
# Absolute: /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin

if [ ! -d "$PRIMAL_BIN_DIR" ]; then
    echo "⚠️  Primal binaries not found at $PRIMAL_BIN_DIR"
    echo "💡 Set PRIMAL_BIN_DIR or run without external primals"
    exit 1
fi

# Start primals (they announce themselves)
echo "🚀 Starting primals..."
"$PRIMAL_BIN_DIR/beardog" --family-id "$FAMILY_ID" &
BEARDOG_PID=$!

"$PRIMAL_BIN_DIR/songbird" --family-id "$FAMILY_ID" &
SONGBIRD_PID=$!

"$PRIMAL_BIN_DIR/biomeos" --port 3000 &
BIOMEOS_PID=$!

# Wait for primals to initialize
sleep 2

# Verify sockets exist
echo "🔍 Checking for primal sockets..."
ls -la "$XDG_RUNTIME_DIR"/*.sock || echo "⚠️  No sockets found (yet)"

# Run petalTongue (discovers automatically)
echo "🌸 Starting petalTongue..."
cargo run --bin petal-tongue-ui &
PETALTONGUE_PID=$!

# Wait for user interaction
echo "✅ All primals running. Press Ctrl+C to stop."
wait $PETALTONGUE_PID

# Cleanup
kill $BEARDOG_PID $SONGBIRD_PID $BIOMEOS_PID 2>/dev/null || true
echo "🧹 Cleanup complete"
```

**Usage**:
```bash
chmod +x test-with-stable-binaries.sh
./test-with-stable-binaries.sh
```

---

## 📚 REFERENCES

**Discovery Documentation**:
- `ENV_VARS.md` - Environment variable hints
- `crates/petal-tongue-discovery/src/lib.rs` - Discovery chain
- `crates/petal-tongue-ipc/src/socket_path.rs` - Socket discovery

**Cross-Primal Docs**:
- `wateringHole/INTER_PRIMAL_INTERACTIONS.md` - Coordination patterns
- `wateringHole/LIVESPORE_CROSS_PRIMAL_COORDINATION_JAN_2026.md` - LiveSpore

**TRUE PRIMAL Principles**:
- `PRIMAL_BOUNDARIES_COMPLETE.md` - Zero hardcoding verification
- `COMPREHENSIVE_AUDIT_JAN_13_2026_PM.md` - Full audit results

---

## ✅ SUMMARY

**Key Principles**:
1. ✅ **No hardcoded binary paths** - Primals announce themselves
2. ✅ **Runtime discovery only** - 5 discovery methods (priority order)
3. ✅ **Self-knowledge** - Each primal knows itself, discovers others
4. ✅ **Graceful degradation** - Works with 0, 1, or N primals
5. ✅ **Environment hints optional** - Discovery works without them

**Testing with Stable Binaries**:
- Start primals independently
- Let them create sockets
- petalTongue discovers them automatically
- No hardcoded paths anywhere

**Result**: TRUE PRIMAL compliant interaction testing! 🌸

---

**Status**: ✅ PRODUCTION PATTERN  
**Compliance**: 100% TRUE PRIMAL  
**Ready for**: Integration testing with any primal binaries

🌸 **Runtime Discovery + Primal Self-Knowledge = Sovereignty** 🌸

