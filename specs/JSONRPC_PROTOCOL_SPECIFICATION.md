# 🌸 JSON-RPC Protocol Specification for petalTongue

**Version**: 1.0.0  
**Date**: January 11, 2026  
**Status**: Design Phase

---

## 🎯 **Overview**

This specification defines petalTongue's implementation of **JSON-RPC 2.0** over Unix sockets as the **universal fallback** protocol for inter-primal and tooling IPC: **tarpc is PRIMARY** for inter-primal RPC ecosystem-wide; JSON-RPC provides maximum compatibility when a peer does not expose tarpc or for line-based tooling, aligning with the ecoPrimals ecosystem architecture.

---

## 🏗️ **Architecture**

### Protocol Stack

```
┌─────────────────────────────────────────┐
│  Application (petalTongue UI)           │
├─────────────────────────────────────────┤
│  VisualizationDataProvider Trait        │
├─────────────────────────────────────────┤
│  JsonRpcProvider (NEW!)                 │ ← THIS SPEC
├─────────────────────────────────────────┤
│  JSON-RPC 2.0 (line-delimited)          │
├─────────────────────────────────────────┤
│  Unix Domain Sockets                    │
├─────────────────────────────────────────┤
│  File System                            │
└─────────────────────────────────────────┘
```

### Discovery Priority Order

1. **Songbird Discovery** (JSON-RPC + NUCLEUS) - Highest priority
2. **JsonRpcProvider** (Unix sockets) - Universal fallback (text JSON-RPC path)
3. **Environment Variables** (`BIOMEOS_URL=unix://...`)
4. **Auto-detect Standard Paths** (`/run/user/{uid}/*.sock`)
5. **HttpProvider** (fallback only, with warning) - Lowest priority

---

## 📡 **JSON-RPC 2.0 Protocol**

### Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "primal.list",
  "params": null,
  "id": 1
}
```

**Serialized** (line-delimited):
```
{"jsonrpc":"2.0","method":"primal.list","params":null,"id":1}\n
```

### Response Format (Success)

```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "id": "songbird",
      "name": "Songbird",
      "health": "Healthy",
      "capabilities": ["discovery", "coordination", "neural_api"]
    }
  ],
  "id": 1
}
```

### Response Format (Error)

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32601,
    "message": "Method not found",
    "data": {"method": "invalid_method"}
  },
  "id": 1
}
```

---

## 🔌 **Unix Socket Connection**

### Socket Path Convention

**Standard Path**: `/run/user/{UID}/{primal}-{service}.sock`

**Examples**:
- biomeOS: `/run/user/1000/biomeos-device-management.sock`
- Songbird: `/run/user/1000/songbird-discovery.sock`
- ToadStool: `/run/user/1000/toadstool-compute.sock`

### Connection Flow

```rust
// 1. Connect to Unix socket
let stream = UnixStream::connect("/run/user/1000/biomeos-device-management.sock").await?;

// 2. Split into reader/writer
let (reader, writer) = stream.into_split();
let mut reader = BufReader::new(reader);

// 3. Send request (line-delimited)
let request = JsonRpcRequest { ... };
let request_json = serde_json::to_string(&request)? + "\n";
writer.write_all(request_json.as_bytes()).await?;
writer.flush().await?;

// 4. Read response (line-delimited)
let mut line = String::new();
reader.read_line(&mut line).await?;
let response: JsonRpcResponse = serde_json::from_str(&line)?;

// 5. Handle result or error
if let Some(error) = response.error {
    return Err(anyhow!("RPC error: {}", error.message));
}
Ok(response.result.unwrap())
```

---

## 🛠️ **JsonRpcProvider Implementation**

### Module Structure

```
crates/petal-tongue-discovery/src/
├── lib.rs                    # Discovery orchestration
├── jsonrpc_provider.rs       # ⭐ NEW! JSON-RPC client
├── http_provider.rs          # Fallback (deprecated as primary)
├── songbird_provider.rs      # Songbird-specific (highest priority)
└── traits.rs                 # VisualizationDataProvider trait
```

### JsonRpcProvider Struct

```rust
pub struct JsonRpcProvider {
    socket_path: PathBuf,
    request_id: AtomicU64,  // Thread-safe request ID counter
}

impl JsonRpcProvider {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self { ... }
    
    pub async fn discover() -> Result<Option<Self>> { ... }
    
    async fn call(&self, method: &str, params: Option<Value>) -> Result<Value> { ... }
}
```

### VisualizationDataProvider Implementation

```rust
#[async_trait]
impl VisualizationDataProvider for JsonRpcProvider {
    async fn get_primals(&self) -> Result<Vec<PrimalInfo>> {
        let result = self.call("primal.list", None).await?;
        Ok(serde_json::from_value(result)?)
    }
    
    async fn get_topology(&self) -> Result<Vec<TopologyEdge>> {
        // Optional method - graceful fallback
        match self.call("topology.get", None).await {
            Ok(result) => Ok(serde_json::from_value(result)?),
            Err(_) => Ok(Vec::new()),  // Not all providers support topology
        }
    }
    
    async fn health_check(&self) -> Result<String> {
        // Use primal.list as health check
        self.call("primal.list", None).await?;
        Ok(format!("JSON-RPC provider at {} is healthy", self.socket_path.display()))
    }
    
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "jsonrpc".to_string(),
            name: "JSON-RPC Provider".to_string(),
            version: "1.0.0".to_string(),
            description: "JSON-RPC 2.0 over Unix sockets (universal fallback)".to_string(),
            capabilities: vec!["primals".to_string(), "devices".to_string()],
        }
    }
}
```

---

## 🔍 **Auto-Discovery**

### Standard Socket Paths

```rust
async fn discover_standard_sockets() -> Vec<PathBuf> {
    let uid = users::get_current_uid();
    
    let standard_paths = vec![
        format!("/run/user/{}/biomeos-device-management.sock", uid),
        format!("/run/user/{}/biomeos-ui.sock", uid),
        format!("/run/user/{}/songbird-discovery.sock", uid),
        "/tmp/biomeos.sock".to_string(),
    ];
    
    let mut found = Vec::new();
    for path in standard_paths {
        if tokio::fs::metadata(&path).await.is_ok() {
            found.push(PathBuf::from(path));
        }
    }
    found
}
```

### Environment Variable

```rust
// BIOMEOS_URL=unix:///run/user/1000/biomeos-device-management.sock
if let Ok(url) = std::env::var("BIOMEOS_URL") {
    if let Some(socket_path) = url.strip_prefix("unix://") {
        return Some(JsonRpcProvider::new(socket_path));
    }
}
```

---

## 📋 **Supported Methods**

### biomeOS Methods (wateringHole semantic naming: `{domain}.{operation}[.{variant}]`)

| Method | Params | Returns | Description |
|--------|--------|---------|-------------|
| `device.list` | None | `Vec<Device>` | List system devices |
| `primal.list` | None | `Vec<PrimalInfo>` | List primals with health |
| `niche.list_templates` | None | `Vec<NicheTemplate>` | List niche templates |
| `device.assign` | `{device_id, primal_id}` | `AssignmentResult` | Assign device to primal |
| `niche.validate` | `NicheConfig` | `ValidationResult` | Validate niche config |
| `niche.deploy` | `NicheConfig` | `DeploymentResult` | Deploy niche |

### Optional Methods (Graceful Fallback)

| Method | Fallback Behavior |
|--------|-------------------|
| `topology.get` | Return empty `Vec<TopologyEdge>` |
| `capability.list` | Return empty capabilities list |
| `events.subscribe` | Log warning, no-op |

---

## ⚠️ **Error Handling**

### Standard JSON-RPC Error Codes

| Code | Message | Meaning |
|------|---------|---------|
| `-32700` | Parse error | Invalid JSON |
| `-32600` | Invalid Request | Missing required fields |
| `-32601` | Method not found | Unknown method |
| `-32602` | Invalid params | Wrong parameter types |
| `-32603` | Internal error | Server-side error |

### Custom Error Codes (ecoPrimals)

| Code | Message | Meaning |
|------|---------|---------|
| `1001` | Primal not found | Unknown primal ID |
| `1002` | Device not available | Device in use or missing |
| `1003` | Validation failed | Niche config invalid |
| `1004` | Deployment failed | Neural API error |

### Error Recovery

```rust
async fn call_with_retry(&self, method: &str, max_retries: u32) -> Result<Value> {
    for attempt in 1..=max_retries {
        match self.call(method, None).await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                warn!("RPC call failed (attempt {}/{}): {}", attempt, max_retries, e);
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

---

## 🧪 **Testing Strategy**

### Unit Tests

- [ ] JSON-RPC request serialization
- [ ] JSON-RPC response deserialization
- [ ] Error response handling
- [ ] Request ID generation (atomicity)

### Integration Tests

- [ ] Mock Unix socket server
- [ ] Concurrent requests
- [ ] Connection retry logic
- [ ] Socket path discovery

### E2E Tests

- [ ] Real biomeOS server connection
- [ ] All supported methods
- [ ] Error scenarios
- [ ] Performance (< 1ms latency)

---

## 🚀 **Migration Plan**

### Phase 1: JSON-RPC Implementation (Week 1)

1. Create `jsonrpc_provider.rs`
2. Implement `JsonRpcProvider`
3. Update discovery chain
4. Add unit tests
5. Integration testing with biomeOS

### Phase 2: HTTP Deprecation (Week 2)

1. Mark `HttpProvider` as "external fallback only"
2. Add warning if HTTP is used
3. Update all documentation
4. Update examples

### Phase 3: tarpc Integration (Week 3-4)

1. Research tarpc patterns from ToadStool/Songbird
2. Implement `TarpcProvider`
3. Streaming updates
4. Bi-directional communication

---

## 📊 **Performance Targets**

| Metric | Target | Rationale |
|--------|--------|-----------|
| Latency (local) | < 1ms | Unix socket IPC is extremely fast |
| Throughput | 10,000+ RPC/s | Limited by JSON serialization |
| Connection time | < 10ms | Unix socket connection is instant |
| Memory overhead | < 1MB per connection | Minimal buffering needed |

---

## 🔐 **Security**

### Unix Socket Permissions

```bash
# Socket should be owned by user, accessible by group
$ ls -l /run/user/1000/biomeos-device-management.sock
srwxrwx--- 1 user user 0 Jan 11 12:00 biomeos-device-management.sock
```

### Authentication

- Unix socket file permissions provide authentication
- Only processes running as the same user can connect
- No additional auth needed for local IPC

---

## 📚 **References**

- **JSON-RPC 2.0**: https://www.jsonrpc.org/specification
- **Unix Domain Sockets**: `man 7 unix`
- **tokio::net::UnixStream**: https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html

---

**Status**: Ready for implementation ✅  
**Blocking**: Full biomeOS integration  
**Priority**: High

🌸 TRUE PRIMAL: tarpc PRIMARY for inter-primal RPC, JSON-RPC universal fallback!
