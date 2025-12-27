# Multi-Primal Integration Plan

**Date**: December 26, 2025  
**Goal**: Integrate petalTongue with real primal binaries from `phase1bins/`  
**Vision**: Real multi-primal orchestration + external tools ecosystem

---

## 🎯 Available Primals

All binaries are **executable and ready** in `../phase1bins/`:

| Primal | Binary | Size | Purpose |
|--------|--------|------|---------|
| **BearDog** | `beardog-bin` | 9.2M | Key/secret management & entropy |
| **LoamSpine** | `loamspine-bin` | 9.2M | Trust & identity substrate |
| **NestGate** | `nestgate-bin` | 3.4M | Network gateway & discovery |
| **RhizoCrypt** | `rhizocrypt-bin` | 1.8M | Cryptographic operations |
| **Songbird** | `songbird-bin` | 21M | Primal discovery & announcement |
| **Squirrel** | `squirrel-bin` | 15M | Data/artifact management |
| **SweetGrass** | `sweetgrass-bin` | 4.6M | Configuration management |
| **ToadStool** | `toadstool-bin` | 4.3M | Compute primal (Python bridge target!) |

---

## 🏗️ Architecture Evolution

### Current State (Mock Data)
```
petalTongue (UI)
  ↓ HTTP
BiomeOS (Mocked)
  ↓ (simulated)
[Mock Primal Data]
```

### Target State (Real Primals)
```
petalTongue (UI/Viz)
  ↓ HTTP
BiomeOS API
  ↓ Discovers
Real Primals (running binaries)
  ├─ Songbird (discovery)
  ├─ NestGate (network)
  ├─ BearDog (secrets)
  ├─ ToadStool (compute)
  └─ [Others...]
```

### With External Tools
```
petalTongue (UI/Viz)
  ├─ Rust Tools (direct)
  │  ├─ System Monitor
  │  ├─ Process Viewer
  │  └─ Graph Metrics
  │
  ├─ Primal Tools (via BiomeOS)
  │  ├─ Songbird (primal discovery)
  │  ├─ BearDog (key status)
  │  └─ ToadStool (compute status)
  │
  └─ Python Tools (via ToadStool)
     ├─ matplotlib
     ├─ pandas
     └─ scikit-learn
```

---

## 📋 Implementation Phases

### Phase 1: BiomeOS Connection ✅ (Already Done)
- [x] HTTP client in `petal-tongue-api`
- [x] Discovery API
- [x] Topology API
- [x] Mock mode for testing

### Phase 2: Songbird Integration (NEXT - High Priority)
**Why First**: Songbird provides primal discovery - the foundation!

**Tasks**:
1. Launch `songbird-bin` from petalTongue (or expect it running)
2. Query Songbird for available primals
3. Display real primal topology in Graph Metrics
4. Update Process Viewer to show primal processes

**Expected Output**:
- Real-time list of running primals
- Their capabilities
- Network topology
- Health status

### Phase 3: ToadStool Integration (Critical for Python)
**Why**: Enables Python tool ecosystem!

**Tasks**:
1. Launch `toadstool-bin` (or connect to running instance)
2. Implement `/api/tools/list` endpoint in ToadStool
3. Implement `/api/tools/execute` endpoint in ToadStool
4. Create first Python tool (matplotlib plotter)
5. Test Python tool in petalTongue UI

**Expected Output**:
- Python tools visible in petalTongue tool menu
- Execute matplotlib plot from petalTongue
- Display result in UI

### Phase 4: Other Primals (Showcase Features)

**BearDog Tool Panel**:
- View entropy pools
- Key status
- Secret management UI

**RhizoCrypt Tool Panel**:
- Encryption operations
- Key generation
- Signature verification

**Squirrel Tool Panel**:
- Artifact browser
- Storage status
- Cache visualization

**NestGate Tool Panel**:
- Network topology
- Gateway status
- Traffic visualization

---

## 🚀 Quick Start Implementation

### Step 1: Launch Songbird

**Option A: From petalTongue** (Managed)
```rust
// In app.rs startup
let songbird = std::process::Command::new("../phase1bins/songbird-bin")
    .arg("--config")
    .arg("../phase1bins/configs/songbird.toml")
    .spawn()?;
```

**Option B: User Launches** (Recommended for now)
```bash
cd /path/to/phase1bins
./songbird-bin --config configs/songbird.toml
```

### Step 2: Connect petalTongue to Songbird

```rust
// In BiomeOSClient
let songbird_url = env::var("SONGBIRD_URL")
    .unwrap_or_else(|_| "http://localhost:8080".to_string());

let response = client.get(format!("{}/api/primals/list", songbird_url))
    .send()
    .await?;
```

### Step 3: Display Real Data

```rust
// In Graph Metrics tool
let primals = songbird_client.get_primal_list().await?;
for primal in primals {
    println!("Found: {} at {}", primal.name, primal.endpoint);
}
```

---

## 💡 Key Design Decisions

### 1. Process Management

**Who starts primals?**
- **Phase 1**: User starts them manually
- **Phase 2**: petalTongue can spawn them
- **Phase 3**: BiomeOS orchestrates them

**Why manual first**: Simpler, safer, easier to debug

### 2. Discovery Pattern

**Songbird as Hub**:
- All primals announce to Songbird
- petalTongue queries Songbird
- Songbird provides topology

**Why**: Maintains primal sovereignty, no hardcoding

### 3. Tool Categories

1. **System Tools** (Rust): System Monitor, Process Viewer
2. **Graph Tools** (Rust): Graph Metrics, Topology Viewer
3. **Primal Tools** (Rust→Primal): Per-primal panels
4. **Python Tools** (Rust→ToadStool→Python): Data science

All use same `ToolPanel` trait!

---

## 📊 Implementation Priority

### High Priority (Do First)
1. ✅ Rust tools (DONE - 4 tools)
2. ✅ Python bridge (DONE - ToadStool bridge)
3. → **Songbird integration** (real primal discovery)
4. → **ToadStool API** (Python tool execution)

### Medium Priority (Do Next)
5. BearDog tool panel
6. NestGate topology viewer
7. Squirrel artifact browser

### Low Priority (Polish)
8. Primal health monitoring
9. Multi-primal workflows
10. Advanced visualization

---

## 🎭 User Experience Goals

### When petalTongue Launches

**Discover Phase**:
1. Connect to Songbird
2. Query for primals
3. Display in Graph Metrics
4. Show in left panel

**Tool Phase**:
1. User clicks primal (e.g., "ToadStool")
2. petalTongue shows ToadStool-specific panel
3. User can interact with primal's capabilities

**Python Phase**:
1. ToadStool advertises Python tools
2. petalTongue shows them in menu
3. User executes matplotlib plot
4. Display in UI

### Example Session

```
User: cargo run --release

petalTongue: Connecting to Songbird...
petalTongue: Found 5 primals:
  • Songbird @ localhost:8080
  • ToadStool @ localhost:4000
  • BearDog @ localhost:3001
  • NestGate @ localhost:3002
  • LoamSpine @ localhost:3003

User: [Clicks "Graph Metrics"]
petalTongue: Displaying real topology (5 nodes, 12 edges)

User: [Clicks "ToadStool"]
petalTongue: Loading ToadStool panel...
petalTongue: Found 3 Python tools:
  • matplotlib plotter
  • pandas viewer
  • scikit-learn inspector

User: [Clicks "matplotlib plotter"]
petalTongue: Executing plot via ToadStool...
petalTongue: [Displays beautiful matplotlib chart]
```

---

## 🔧 Configuration

### Environment Variables

```bash
# Songbird connection
export SONGBIRD_URL="http://localhost:8080"

# ToadStool connection (for Python tools)
export TOADSTOOL_URL="http://localhost:4000"

# BiomeOS connection (if separate)
export BIOMEOS_URL="http://localhost:3000"

# Primal binary path
export PRIMAL_BINS_PATH="../phase1bins"
```

### Config File

```toml
# petal-tongue.toml
[discovery]
songbird_url = "http://localhost:8080"
auto_discover = true
refresh_interval = 5  # seconds

[tools]
enable_rust_tools = true
enable_python_tools = true
toadstool_url = "http://localhost:4000"

[primals]
# Optional: specific primal endpoints
# Otherwise discovered via Songbird
```

---

## 📈 Success Metrics

### Phase 1: Songbird Integration
- [ ] Connect to real Songbird binary
- [ ] Receive primal list
- [ ] Display in Graph Metrics
- [ ] Show real topology

### Phase 2: ToadStool Python Tools
- [ ] Connect to ToadStool binary
- [ ] List Python tools
- [ ] Execute matplotlib plot
- [ ] Display in petalTongue UI

### Phase 3: Multi-Primal Demo
- [ ] 5+ primals running
- [ ] All discoverable via Songbird
- [ ] Real topology visualization
- [ ] Tool panels for each primal

---

## 🚀 Next Steps

### Immediate (This Session)
1. **Test Songbird launch**:
   ```bash
   cd ../phase1bins
   ./songbird-bin --help
   ```

2. **Check Songbird API**:
   ```bash
   # Start Songbird
   ./songbird-bin &
   
   # Query API
   curl http://localhost:8080/api/primals/list
   ```

3. **Integrate with petalTongue**:
   - Add Songbird client to `petal-tongue-api`
   - Connect in `app.rs` startup
   - Display in Graph Metrics

### Short-Term (Next Session)
1. Implement ToadStool API endpoints
2. Create Python tool protocol
3. Add first matplotlib tool
4. Demo full stack

---

## 💎 Value Proposition

**For Users**:
- **Real multi-primal orchestration** (not mocks!)
- **Rust tools** (fast, native)
- **Python tools** (grammar of graphics)
- **Unified UI** (one place to see everything)

**For Developers**:
- **Build Rust tools** (implement ToolPanel)
- **Build Python tools** (simple stdin/stdout)
- **Build primal-specific tools** (leverage existing binaries)
- **No changes to primals** (they just run as-is)

**For The Ecosystem**:
- **Primal discovery works** (via Songbird)
- **Tool discovery works** (via ToadStool)
- **Everything composable** (mix and match)
- **Community can contribute** (tools and primals)

---

**🎯 Goal: Real Multi-Primal Integration with External Tools**  
**🔧 Status: Ready to implement Songbird connection**  
**🌈 Vision: Unified visualization of entire ecoPrimals ecosystem + Python tools**

*Let's make it real!*

