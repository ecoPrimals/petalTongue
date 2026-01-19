# 🔍 Data Flow Audit - Comprehensive Results

**Date**: January 19, 2026  
**Goal**: Verify ALL data flows through DataService  
**Status**: ✅ **AUDIT COMPLETE**

---

## 🚨 **STRAYS FOUND**

### 1. **crates/petal-tongue-ui/src/app.rs**
**Location**: `refresh_graph_data()` method (lines ~671-730)

```rust
// STRAY! Direct BiomeOS client usage
let primals = match self.biomeos_client.discover_primals().await {
    Ok(p) => p,
    Err(e) => {
        tracing::warn!("Failed to discover primals: {}", e);
        return;
    }
};

let edges = match self.biomeos_client.get_topology().await {
    Ok(e) => e,
    Err(e) => {
        tracing::warn!("Failed to get topology: {}", e);
        vec![]
    }
};
```

**Impact**: HIGH - GUI fetches data directly  
**Fix**: Replace with DataService.graph()

---

### 2. **crates/petal-tongue-ui/src/data_source.rs**
**Location**: `refresh_topology()` method

```rust
// STRAY! Direct client usage
let primals = self
    .client
    .discover_primals()
    .await
    .map_err(|e| format!("Failed to discover primals: {e}"))?;

let edges = self
    .client
    .get_topology()
    .await
    .map_err(|e| format!("Failed to get topology: {e}"))?;
```

**Impact**: MEDIUM - Old data source module (deprecated?)  
**Fix**: Remove or migrate to use DataService

---

### 3. **crates/petal-tongue-tui/src/app.rs**
**Location**: Multiple methods

**Impact**: MEDIUM - TUI has its own data fetching  
**Fix**: TUI should use DataService.snapshot()

---

### 4. **Multiple Discovery Providers**
**Locations**: 
- `crates/petal-tongue-discovery/src/neural_api_provider.rs`
- `crates/petal-tongue-discovery/src/http_provider.rs`
- `crates/petal-tongue-discovery/src/jsonrpc_provider.rs`
- `crates/petal-tongue-discovery/src/songbird_provider.rs`

**Status**: ✅ **OK** - These are the *implementations* that DataService uses  
**Not strays**: These are the underlying providers, DataService calls them

---

## ✅ **CORRECT USAGE**

### 1. **src/web_mode.rs**
```rust
// ✅ CORRECT! Uses DataService
async fn primals_handler(
    State(service): State<Arc<DataService>>
) -> impl IntoResponse {
    match service.snapshot().await {
        Ok(snapshot) => Json(snapshot),
        ...
    }
}
```

### 2. **src/data_service.rs**
```rust
// ✅ CORRECT! This IS the DataService
pub async fn refresh(&self) -> Result<()> {
    if let Some(api) = &self.neural_api {
        let primals = api.get_primals().await?;
        let topology = api.get_topology().await?;
        // Updates internal graph
    }
}
```

---

## 📋 **MIGRATION PLAN**

### Priority 1: Fix GUI (app.rs)
**File**: `crates/petal-tongue-ui/src/app.rs`

**Current**:
```rust
impl PetalTongueApp {
    fn refresh_graph_data(&mut self) {
        let primals = self.biomeos_client.discover_primals().await?;
        let edges = self.biomeos_client.get_topology().await?;
        // Updates self.graph directly
    }
}
```

**Target**:
```rust
impl PetalTongueApp {
    fn new(cc: &eframe::CreationContext, data_service: Arc<DataService>) -> Self {
        let graph = data_service.graph(); // Share the graph!
        
        Self {
            graph,  // Now uses DataService's graph
            data_service,
            ...
        }
    }
    
    fn refresh_graph_data(&mut self) {
        // Just trigger DataService refresh
        self.data_service.refresh().await?;
        // Graph updates automatically (same Arc!)
    }
}
```

---

### Priority 2: Fix TUI
**File**: `crates/petal-tongue-tui/src/app.rs`

**Target**:
```rust
impl TuiApp {
    async fn render(&mut self, data_service: &DataService) {
        let snapshot = data_service.snapshot().await?;
        self.render_primals(&snapshot.primals);
        self.render_edges(&snapshot.edges);
    }
}
```

---

### Priority 3: Deprecate Old DataSource
**File**: `crates/petal-tongue-ui/src/data_source.rs`

**Action**: Mark as deprecated, migrate callers to src/data_service.rs

```rust
#[deprecated(note = "Use src/data_service::DataService instead")]
pub struct DataSource {
    // ...
}
```

---

## 🎯 **VERIFICATION CHECKLIST**

After migration, verify:

- [ ] GUI uses `DataService.graph()` (shared Arc)
- [ ] TUI uses `DataService.snapshot()`
- [ ] Web uses `DataService.snapshot()` ✅ (done!)
- [ ] Headless uses `DataService.snapshot()`
- [ ] CLI uses `DataService.snapshot()`
- [ ] No direct `biomeos_client` calls in UI code
- [ ] No direct `neural_api` calls in UI code
- [ ] Old `data_source.rs` deprecated or removed

---

## 📊 **SUMMARY**

### Found Strays: **3**
1. ❌ GUI app.rs - Direct biomeos_client calls
2. ❌ Old data_source.rs - Direct client calls
3. ❌ TUI app.rs - Direct data fetching

### Correct Usage: **2**
1. ✅ Web mode - Uses DataService
2. ✅ DataService itself - Correct abstraction

### Not Strays (Providers): **4+**
- ✅ Discovery providers are underlying implementations
- ✅ DataService calls them (correct!)

---

## 🔧 **IMPLEMENTATION STEPS**

### Step 1: Wire GUI to DataService
```bash
# Update app.rs to:
# 1. Accept Arc<DataService> in new()
# 2. Use data_service.graph() instead of self.graph
# 3. Call data_service.refresh() instead of direct fetch
```

### Step 2: Wire TUI to DataService
```bash
# Update TUI to:
# 1. Accept Arc<DataService> in new()
# 2. Call data_service.snapshot() for rendering
# 3. Subscribe to updates
```

### Step 3: Update UI Mode
```bash
# Update src/ui_mode.rs to:
# 1. Create DataService
# 2. Pass to PetalTongueApp::new()
```

### Step 4: Update TUI Mode
```bash
# Update src/tui_mode.rs to:
# 1. Create DataService
# 2. Pass to TuiApp
```

### Step 5: Test
```bash
# Verify all modes show SAME data
cargo test
cargo run -- ui &
cargo run -- web &
cargo run -- tui
# All should show identical primal lists!
```

---

## ✅ **SUCCESS CRITERIA**

1. **Single Fetch**: Data fetched ONCE by DataService
2. **Shared State**: All UIs use same Arc<RwLock<GraphEngine>>
3. **No Strays**: Zero direct biomeos/neural_api calls in UI code
4. **Consistent Data**: All modes show identical information
5. **Event-Driven**: Updates propagate via broadcast channel

---

**Status**: Ready to migrate! 🚀  
**Estimated Time**: 1-2 hours for all migrations  
**Risk**: Low (additive changes, can test incrementally)

🌸 **TRUE PRIMAL: One data source, many consumers!** 🌸

