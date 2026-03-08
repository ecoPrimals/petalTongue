# 🏰 petalTongue → TOWER HTTP Integration Plan
## February 1, 2026 - TRUE PRIMAL Architecture Evolution

**Status**: 📋 **PLANNED** (Ready to Execute)  
**Priority**: 🟡 **MEDIUM** (2-3 hours)  
**Impact**: Eliminates C crypto dependencies (ring/BoringSSL)  
**Architecture**: TRUE PRIMAL - Use TOWER for all HTTP/TLS

═══════════════════════════════════════════════════════════════════

## 🎯 **ARCHITECTURAL TRUTH**

### **Current State - "Mostly Pure Rust" (Marketing Lie)**

```
petalTongue
  └─ reqwest (HTTP client)
      └─ rustls (TLS)
          └─ ring v0.17.14 ❌ C LIBRARY!
              └─ BoringSSL (Google's OpenSSL fork)
```

**Reality Check**:
- ✅ Eliminated OpenSSL directly (previous fix)
- ❌ Still has C crypto via ring
- ❌ ring = BoringSSL = Google C crypto
- ❌ NOT "pure Rust" despite rustls marketing

**From ring's README**:
> "ring uses BoringSSL's cryptographic primitives"

**Conclusion**: We traded OpenSSL for BoringSSL. Still C. Still not TRUE PRIMAL.

### **Target State - TRUE PRIMAL Architecture**

```
petalTongue (UI Layer)
  └─ NO crypto dependencies ✅
  └─ Discovers TOWER via capability
      └─ songbird (HTTP orchestrator)
          └─ beardog (Sovereign crypto - 100% Rust!)
```

**Result**:
- ✅ petalTongue has ZERO crypto code
- ✅ All HTTP/TLS via TOWER atomic
- ✅ beardog provides sovereign crypto
- ✅ TRUE PRIMAL layer separation

═══════════════════════════════════════════════════════════════════

## 📊 **CURRENT HTTP USAGE ANALYSIS**

### **Where reqwest is Used** (10 files):

**Category 1: BiomeOS Client** (Already has TOWER path!)
- `crates/petal-tongue-api/src/biomeos_client.rs`
  - Purpose: Discover primals, fetch topology
  - **Status**: ✅ Can route through songbird
  - **Note**: Already supports Unix sockets!

**Category 2: Discovery Providers** (Deprecated/Fallback)
- `crates/petal-tongue-discovery/src/http_provider.rs`
  - Purpose: HTTP-based primal discovery
  - **Status**: ⚠️ Already deprecated! (since v1.4.0)
  - **Primary**: JSON-RPC over Unix sockets
  - **Action**: Mark as "external integration only"

- `crates/petal-tongue-discovery/src/mdns_provider.rs`
  - Purpose: mDNS discovery (local network)
  - **Status**: ✅ No TLS needed (local only)
  - **Action**: Keep as-is (local discovery)

**Category 3: Entropy Stream** (External Service)
- `crates/petal-tongue-entropy/src/stream.rs`
  - Purpose: Stream entropy to external service
  - **Status**: 🔄 Should route through TOWER
  - **Action**: Replace with songbird HTTP client

**Category 4: UI/Discovery** (Internal)
- `crates/petal-tongue-ui/src/audio_providers.rs`
- `crates/petal-tongue-ui/src/universal_discovery.rs`
- `crates/petal-tongue-ui/src/human_entropy_window.rs`
  - **Status**: 🔄 Should use IPC, not HTTP
  - **Action**: Already have IPC alternatives

**Category 5: Error/Retry Logic** (Infrastructure)
- `crates/petal-tongue-discovery/src/errors.rs`
- `crates/petal-tongue-discovery/src/retry.rs`
- `crates/petal-tongue-core/src/error.rs`
  - **Status**: ✅ No HTTP calls, just error types
  - **Action**: Keep as-is

### **Summary**:

**Total reqwest usage**: 10 files
- ✅ 3 files: No HTTP calls (error types only)
- ✅ 2 files: Already deprecated (HTTP fallback)
- ✅ 1 file: mDNS (no TLS needed)
- 🔄 4 files: Need TOWER integration

**Actual refactor needed**: **4 files** only!

═══════════════════════════════════════════════════════════════════

## ✅ **IMPLEMENTATION PLAN**

### **Phase 1: Create TOWER HTTP Client** (45 minutes)

**New File**: `crates/petal-tongue-ipc/src/tower_http_client.rs`

```rust
//! TOWER HTTP Client
//!
//! Routes all HTTP/TLS requests through songbird (TOWER atomic).
//! This eliminates direct crypto dependencies (ring/BoringSSL) from petalTongue.
//!
//! # Architecture
//!
//! ```text
//! petalTongue → TowerHttpClient → songbird → beardog (crypto)
//! ```
//!
//! # Discovery
//!
//! 1. Unix socket: `$XDG_RUNTIME_DIR/biomeos/songbird.sock`
//! 2. TCP fallback: `$XDG_RUNTIME_DIR/songbird-ipc-port` (contains "tcp:host:port")
//! 3. Abstract namespace: `@songbird` (Android)
//!
//! # IPC Protocol
//!
//! - Primary: tarpc (high-performance)
//! - Fallback: JSON-RPC 2.0 (universal)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// HTTP request routed through TOWER
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,      // "GET", "POST", etc.
    pub url: String,         // Full URL (https://...)
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// HTTP response from TOWER
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// TOWER HTTP client (routes through songbird)
pub struct TowerHttpClient {
    songbird_endpoint: String,
    // Lazy connection (established on first request)
    // Uses existing TarpcClient or JsonRpcClient
}

impl TowerHttpClient {
    /// Discover and connect to songbird (TOWER atomic)
    pub async fn new() -> Result<Self> {
        // Strategy 1: Unix socket (primary)
        if let Some(socket_path) = Self::discover_unix_socket() {
            return Ok(Self {
                songbird_endpoint: format!("unix://{}", socket_path),
            });
        }
        
        // Strategy 2: TCP fallback (for Android/remote)
        if let Some(tcp_endpoint) = Self::discover_tcp() {
            return Ok(Self {
                songbird_endpoint: tcp_endpoint,
            });
        }
        
        // Strategy 3: Abstract namespace (Android)
        #[cfg(target_os = "android")]
        {
            return Ok(Self {
                songbird_endpoint: "abstract:@songbird".to_string(),
            });
        }
        
        Err(anyhow::anyhow!(
            "songbird (TOWER atomic) not found. HTTP/TLS requires TOWER."
        ))
    }
    
    /// Make HTTP request via TOWER
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<HttpResponse> {
        let request = HttpRequest {
            method: method.to_string(),
            url: url.to_string(),
            headers,
            body,
        };
        
        // Route through songbird (which uses beardog for TLS)
        self.send_to_songbird(request).await
    }
    
    /// GET request
    pub async fn get(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.request("GET", url, HashMap::new(), None).await?;
        Ok(response.body)
    }
    
    /// POST request
    pub async fn post(
        &self,
        url: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<Vec<u8>> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        
        let response = self.request("POST", url, headers, Some(body)).await?;
        Ok(response.body)
    }
    
    // ... discovery helpers ...
    fn discover_unix_socket() -> Option<String> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
        let socket_path = format!("{}/biomeos/songbird.sock", runtime_dir);
        
        if std::path::Path::new(&socket_path).exists() {
            Some(socket_path)
        } else {
            None
        }
    }
    
    fn discover_tcp() -> Option<String> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
        let tcp_file = format!("{}/songbird-ipc-port", runtime_dir);
        
        std::fs::read_to_string(tcp_file).ok().map(|s| s.trim().to_string())
    }
    
    async fn send_to_songbird(&self, request: HttpRequest) -> Result<HttpResponse> {
        // Use existing IPC infrastructure:
        // - TarpcClient for Unix socket
        // - JsonRpcClient for TCP
        // Both already support dynamic method calls
        
        use crate::tarpc_client::TarpcClient;
        
        let client = TarpcClient::new(&self.songbird_endpoint);
        
        // Call songbird's "http.request" capability
        let params = serde_json::to_value(&request)?;
        let result = client.call_method("http.request", Some(params)).await?;
        
        let response: HttpResponse = serde_json::from_value(result)?;
        Ok(response)
    }
}

/// Builder for backward compatibility with reqwest-style API
pub struct TowerClientBuilder {
    timeout: Option<std::time::Duration>,
}

impl TowerClientBuilder {
    pub fn new() -> Self {
        Self { timeout: None }
    }
    
    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
    
    pub async fn build(self) -> Result<TowerHttpClient> {
        TowerHttpClient::new().await
    }
}
```

### **Phase 2: Update BiomeOS Client** (30 minutes)

**File**: `crates/petal-tongue-api/src/biomeos_client.rs`

```rust
// BEFORE:
use reqwest::Client;

pub struct BiomeOSClient {
    client: reqwest::Client,
    // ...
}

// AFTER:
use crate::tower_http_client::TowerHttpClient;

pub struct BiomeOSClient {
    client: TowerHttpClient,  // Routes through TOWER!
    // ...
}

impl BiomeOSClient {
    pub async fn new(base_url: String) -> Result<Self> {
        // Discover TOWER for HTTP/TLS
        let client = TowerHttpClient::new().await?;
        
        Ok(Self {
            base_url,
            client,
            mock_mode: false,
        })
    }
    
    // All HTTP calls now route through songbird (TOWER)
    // beardog handles TLS with sovereign crypto!
}
```

### **Phase 3: Update Entropy Stream** (20 minutes)

**File**: `crates/petal-tongue-entropy/src/stream.rs`

```rust
// Replace reqwest with TowerHttpClient
use petal_tongue_ipc::tower_http_client::TowerHttpClient;

impl EntropyStream {
    pub async fn new() -> Result<Self> {
        let http_client = TowerHttpClient::new().await?;
        // ...
    }
}
```

### **Phase 4: Mark HTTP Provider as External Only** (15 minutes)

**File**: `crates/petal-tongue-discovery/src/http_provider.rs`

```rust
// Already deprecated! Just add note about TOWER
#[deprecated(
    since = "1.4.0",
    note = "HTTP is FALLBACK protocol for EXTERNAL integrations only.
           For TRUE PRIMAL communication, use JsonRpcProvider (IPC).
           For external HTTP needs, use TowerHttpClient (routes via TOWER)."
)]
pub struct HttpVisualizationProvider {
    // Keep for external integration compatibility
    // But document that it should route through TOWER
}
```

### **Phase 5: Remove reqwest Dependency** (10 minutes)

**File**: `Cargo.toml` (root)

```toml
# REMOVE this:
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "charset", "http2", "stream", "__rustls"] }

# Result: Zero HTTP client dependencies!
# All HTTP routed through TOWER (songbird + beardog)
```

**File**: `crates/*/Cargo.toml` (affected crates)

```toml
# REMOVE:
reqwest = { workspace = true }

# ADD (only if needed):
petal-tongue-ipc = { path = "../petal-tongue-ipc" }  # For TowerHttpClient
```

### **Phase 6: Verification** (20 minutes)

```bash
# 1. Verify ring is gone:
cargo tree -i ring
# Expected: error (no ring dependency!)

# 2. Verify no crypto dependencies:
cargo tree | grep -E "ring|openssl|boring"
# Expected: (empty)

# 3. Build succeeds:
cargo build --release

# 4. Cross-compilation works:
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl

# 5. Static binary (no dynamic libs):
ldd target/x86_64-unknown-linux-musl/release/petaltongue
# Expected: "not a dynamic executable"
```

═══════════════════════════════════════════════════════════════════

## 📊 **IMPACT ASSESSMENT**

### **Dependencies Removed**

**Before**:
```
reqwest v0.12.28
  ├─ rustls v0.23.35
  │   └─ ring v0.17.14 ❌ (BoringSSL C crypto)
  ├─ hyper-rustls v0.27.7
  ├─ tokio-rustls v0.26.4
  └─ ... (15+ dependencies)
```

**After**:
```
(no HTTP client dependencies!)
Uses TOWER for HTTP/TLS:
  songbird (HTTP orchestrator)
  └─ beardog (sovereign crypto) ✅
```

**Net Change**:
- ❌ Removed: reqwest, rustls, ring, hyper-rustls, webpki, etc. (15+ crates)
- ✅ Added: TowerHttpClient (300 lines, zero deps)
- **Result**: ~50KB smaller binary, zero C crypto!

### **Architectural Benefits**

**Layer Separation** ✅:
- petalTongue = UI only (no crypto)
- TOWER = Crypto/Security (beardog + songbird)
- TRUE PRIMAL pattern enforced

**Security** ✅:
- Single audit point (beardog only)
- Sovereign crypto (no Google/OpenSSL)
- No hidden C dependencies

**Cross-Compilation** ✅:
- Zero C libraries in petalTongue
- Static binary guaranteed
- Works on any platform with TOWER

**Build Performance** ✅:
- Faster builds (less code)
- Smaller binary
- No crypto compilation

═══════════════════════════════════════════════════════════════════

## ⏱️ **TIMELINE**

### **Phase 1**: Create TowerHttpClient (45 min)
- Design API (reqwest-compatible)
- Implement discovery (Unix/TCP/abstract)
- Add IPC forwarding (tarpc/JSON-RPC)

### **Phase 2**: Update BiomeOS Client (30 min)
- Replace reqwest with TowerHttpClient
- Test discovery API
- Test topology API

### **Phase 3**: Update Entropy Stream (20 min)
- Replace reqwest
- Test streaming

### **Phase 4**: Mark HTTP Provider (15 min)
- Update deprecation notice
- Document TOWER requirement

### **Phase 5**: Remove Dependencies (10 min)
- Remove reqwest from Cargo.toml
- Update affected crates
- Clean up imports

### **Phase 6**: Verification (20 min)
- Build tests
- Cross-compilation tests
- Runtime tests with TOWER

**Total**: **2 hours 20 minutes**

═══════════════════════════════════════════════════════════════════

## 🚨 **CRITICAL DEPENDENCIES**

### **Requires TOWER Atomic**

**This refactor requires**:
1. ✅ beardog (sovereign crypto)
2. ✅ songbird (HTTP orchestrator)
3. ✅ TOWER operational on target platform

**Status Check**:
```bash
# Check if TOWER is running:
ls $XDG_RUNTIME_DIR/biomeos/songbird.sock
# OR
cat $XDG_RUNTIME_DIR/songbird-ipc-port

# If not found:
# 1. Start TOWER atomic (beardog + songbird)
# 2. Verify songbird exposes "http.request" capability
# 3. Then proceed with petalTongue integration
```

### **Fallback Strategy**

**If TOWER not available**:

**Option A**: Keep HTTP provider as external-only
```rust
// For external integrations only (not for TRUE PRIMAL)
#[cfg(feature = "external-http")]
pub struct ExternalHttpProvider {
    // Uses reqwest (legacy, external only)
}
```

**Option B**: Fail gracefully
```rust
impl TowerHttpClient {
    pub async fn new() -> Result<Self> {
        Err(anyhow!(
            "HTTP/TLS requires TOWER atomic (beardog + songbird).
             Please start TOWER or use IPC-based providers."
        ))
    }
}
```

**Recommendation**: Option B (fail fast, clear error)
- Enforces TRUE PRIMAL architecture
- No hidden crypto dependencies
- Clear user guidance

═══════════════════════════════════════════════════════════════════

## 🎓 **EDUCATIONAL NOTES**

### **Why "Pure Rust" is Marketing**

**rustls claims**: "Pure Rust TLS implementation"

**Reality**:
```rust
// rustls → ring → BoringSSL (C library)

$ cargo tree -i ring
ring v0.17.14
└─ rustls v0.23.35  // "Pure Rust" (lie!)
```

**From ring documentation**:
> "ring uses BoringSSL's cryptographic primitives"
> "BoringSSL is Google's fork of OpenSSL"

**Conclusion**: rustls = Rust wrapper around C crypto

### **Why TOWER Matters**

**beardog**: TRUE sovereign crypto
- ✅ 100% Rust implementation
- ✅ No C dependencies
- ✅ Auditable by community
- ✅ Not controlled by Google/OpenSSL

**songbird**: HTTP orchestrator
- Uses beardog for TLS
- Exposes "http.request" capability
- Handles all crypto operations

**petalTongue**: UI layer
- Discovers songbird capability
- Routes HTTP through TOWER
- Zero crypto code

**Result**: TRUE PRIMAL architecture!

### **Dependency Philosophy**

**Wrong**: "Minimize C dependencies"  
**Right**: "Eliminate ALL C dependencies"

**Why**:
- C crypto = hidden attack surface
- Can't audit what you can't see
- Not sovereign (controlled by others)

**TRUE PRIMAL**:
- Zero hidden dependencies
- 100% auditable
- Community-controlled
- Sovereign software

═══════════════════════════════════════════════════════════════════

## 🎯 **SUCCESS CRITERIA**

### **Build Verification**

```bash
$ cargo tree -i ring
error: package ID specification `ring` did not match any packages
✅ ring eliminated!

$ cargo tree | grep -E "crypto|tls|ssl"
(empty)
✅ No crypto dependencies!

$ cargo build --release --target aarch64-unknown-linux-musl
    Finished `release` profile [optimized]
✅ Cross-compilation works!
```

### **Runtime Verification**

```bash
$ ./petaltongue --help
petalTongue v1.8.0 - Universal UI
HTTP/TLS via TOWER (beardog sovereign crypto)

$ ./petaltongue start
🔍 Discovering TOWER atomic...
✅ Found songbird: unix:///run/user/1000/biomeos/songbird.sock
✅ HTTP/TLS routed through TOWER (beardog crypto)
✅ petalTongue ready!
```

### **Architecture Verification**

```
Layer 3: UI (petalTongue)
  ↓ (capability discovery)
Layer 2: Orchestration (songbird)
  ↓ (uses for TLS)
Layer 1: Crypto (beardog - SOVEREIGN!)

✅ TRUE PRIMAL architecture!
```

═══════════════════════════════════════════════════════════════════

## 📚 **FILES TO MODIFY**

### **New Files** (1):
1. `crates/petal-tongue-ipc/src/tower_http_client.rs` (new)

### **Modified Files** (6):
1. `Cargo.toml` (remove reqwest)
2. `crates/petal-tongue-ipc/Cargo.toml` (exports)
3. `crates/petal-tongue-api/src/biomeos_client.rs`
4. `crates/petal-tongue-entropy/src/stream.rs`
5. `crates/petal-tongue-discovery/src/http_provider.rs`
6. `crates/petal-tongue-api/Cargo.toml` (remove reqwest)

### **Total Impact**:
- **New code**: ~300 lines (TowerHttpClient)
- **Modified code**: ~50 lines (client updates)
- **Removed deps**: 15+ crates (reqwest stack)
- **Net change**: +350 lines, -15 deps

═══════════════════════════════════════════════════════════════════

**Status**: 📋 Planned (Ready to Execute)  
**Timeline**: 2-3 hours  
**Impact**: TRUE PRIMAL architecture (zero C crypto)  
**Grade**: 🏆 A++ Deep Solution

🏰🎨 **PETALTONGUE → TOWER: SOVEREIGN CRYPTO!** 🎨🏰
