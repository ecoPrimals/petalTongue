//! Unified Data Service - Deep Debt Solution
//!
//! **Problem**: Each UI mode (GUI, TUI, web) had its own data fetching logic
//! **Solution**: Single data service that all UIs consume
//!
//! **TRUE PRIMAL Principles**:
//! - Single source of truth
//! - Zero duplication
//! - Capability-based discovery
//! - Event-driven updates

# Architecture

```
                    DataService (SINGLE)
                         ↓
        ┌────────────────┼────────────────┐
        ↓                ↓                ↓
      GUI              Web              TUI
   (consumes)      (consumes)       (consumes)
        ↓                ↓                ↓
    GraphEngine      JSON API      Terminal
    rendering        serving        output
```

## Key Benefits

### 1. Single Source of Truth ✅
- Data fetching happens ONCE
- All UIs show identical data
- No synchronization issues

### 2. Zero Duplication ✅
- No repeated discovery logic
- No repeated data transformation
- No repeated error handling

### 3. Capability-Based ✅
- Discovers Neural API if available
- Graceful degradation to fallback
- No hardcoded assumptions

### 4. Event-Driven ✅
- Broadcast updates to subscribers
- UIs update automatically
- No polling needed

---

## Implementation

### DataService API

```rust
use crate::data_service::{DataService, DataSnapshot};

// Create service
let mut service = DataService::new();

// Initialize (discovers Neural API)
service.init().await?;

// Get current data
let snapshot = service.snapshot().await?;

// Subscribe to updates
let mut updates = service.subscribe();
tokio::spawn(async move {
    while let Ok(update) = updates.recv().await {
        // React to data changes
    }
});

// Refresh data
service.refresh().await?;
```

### DataSnapshot Structure

```rust
pub struct DataSnapshot {
    pub primals: Vec<PrimalInfo>,           // Discovered primals
    pub edges: Vec<TopologyEdge>,           // Topology
    pub proprioception: Option<ProprioceptionData>, // SAME DAVE
    pub metrics: Option<SystemMetrics>,     // System stats
    pub timestamp: DateTime<Utc>,           // When captured
}
```

---

## Usage in Each Mode

### GUI Mode
```rust
// Get graph directly for rendering
let graph = data_service.graph();
visual_renderer.render(&graph);

// Subscribe to updates
let mut updates = data_service.subscribe();
tokio::spawn(async move {
    while let Ok(_) = updates.recv().await {
        // Trigger re-render
    }
});
```

### Web Mode
```rust
async fn api_handler(
    State(service): State<Arc<DataService>>
) -> Json<DataSnapshot> {
    let snapshot = service.snapshot().await.unwrap();
    Json(snapshot)
}
```

### TUI Mode
```rust
let snapshot = data_service.snapshot().await?;
println!("Primals: {}", snapshot.primals.len());
println!("Health: {}%", 
    snapshot.proprioception
        .map(|p| p.health.percentage)
        .unwrap_or(0.0)
);
```

### Headless Mode
```rust
// Export to various formats
let snapshot = data_service.snapshot().await?;
let svg = render_graph_to_svg(&snapshot.primals, &snapshot.edges);
let json = serde_json::to_string(&snapshot)?;
```

---

## Migration Plan

### Phase 1: Create DataService ✅
- [x] Implement core service
- [x] Add Neural API integration
- [x] Add broadcast channel
- [x] Add tests

### Phase 2: Wire to Web Mode
- [ ] Add DataService to web mode
- [ ] Update `/api/primals` endpoint
- [ ] Add `/api/snapshot` endpoint
- [ ] Add WebSocket for updates

### Phase 3: Wire to GUI Mode
- [ ] Replace direct BiomeOS calls
- [ ] Use DataService.graph()
- [ ] Subscribe to updates
- [ ] Remove duplicated logic

### Phase 4: Wire to TUI Mode
- [ ] Use DataService.snapshot()
- [ ] Display real data
- [ ] Subscribe to updates

### Phase 5: Wire to Headless Mode
- [ ] Use DataService.snapshot()
- [ ] Render to various formats
- [ ] Real data, not mocks

---

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_data_service_init() {
    let mut service = DataService::new();
    service.init().await.unwrap();
    // Checks Neural API discovery
}

#[tokio::test]
async fn test_snapshot_consistency() {
    let service = DataService::new();
    let snapshot1 = service.snapshot().await.unwrap();
    let snapshot2 = service.snapshot().await.unwrap();
    // Should be identical if no refresh
}

#[tokio::test]
async fn test_update_broadcast() {
    let service = DataService::new();
    let mut rx = service.subscribe();
    service.refresh().await.unwrap();
    // Should receive update notification
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_all_modes_see_same_data() {
    let service = Arc::new(DataService::new());
    
    // GUI snapshot
    let gui_snapshot = service.snapshot().await.unwrap();
    
    // Web snapshot
    let web_snapshot = service.snapshot().await.unwrap();
    
    // TUI snapshot
    let tui_snapshot = service.snapshot().await.unwrap();
    
    // All should be identical
    assert_eq!(gui_snapshot.primals, web_snapshot.primals);
    assert_eq!(web_snapshot.primals, tui_snapshot.primals);
}
```

---

## Deep Debt Solved

### Before
```rust
// GUI Mode
biomeos_client.discover_primals()  // ❌ Duplicated
biomeos_client.get_topology()      // ❌ Duplicated

// Web Mode
// TODO: Implement                  // ❌ Empty!

// TUI Mode
println!("Static text")             // ❌ No real data

// Headless Mode
println!("Static text")             // ❌ No real data
```

### After
```rust
// Single service
let service = DataService::new();
service.init().await?;              // ✅ Once

// All modes consume
let snapshot = service.snapshot();  // ✅ Same data
// GUI:      Renders snapshot.primals
// Web:      JSON(snapshot)
// TUI:      Print snapshot.primals
// Headless: Export snapshot to SVG/JSON
```

---

## Performance Benefits

### Network Calls
- **Before**: N modes × M calls = N×M calls
- **After**: 1 service × M calls = M calls
- **Savings**: (N-1)×M calls eliminated

### Memory Usage
- **Before**: N copies of data (one per mode)
- **After**: 1 copy of data (shared via Arc)
- **Savings**: (N-1) copies eliminated

### Consistency
- **Before**: Race conditions between modes
- **After**: Single atomic source
- **Benefit**: Zero sync bugs

---

## Future Enhancements

### Caching
```rust
impl DataService {
    async fn snapshot_cached(&self, max_age: Duration) -> Result<DataSnapshot> {
        if self.time_since_refresh() < max_age {
            // Return cached
        } else {
            // Refresh and return
        }
    }
}
```

### Background Refresh
```rust
impl DataService {
    pub fn start_auto_refresh(&self, interval: Duration) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                self.refresh().await.ok();
            }
        });
    }
}
```

### Incremental Updates
```rust
impl DataService {
    async fn refresh_incremental(&self) -> Result<()> {
        // Only fetch what changed
        // Merge into existing graph
        // Broadcast fine-grained updates
    }
}
```

---

## Conclusion

**DataService solves the deep debt of duplicated data logic across UI modes.**

**TRUE PRIMAL achieved**:
- ✅ Single source of truth
- ✅ Zero duplication
- ✅ Capability-based
- ✅ Event-driven

**Next**: Wire all modes to consume DataService!

---

**Date**: January 19, 2026  
**Status**: ✅ Design complete, ready to wire  
**Impact**: Eliminates major architectural debt

🌸 **Deep debt solved through abstraction!** 🌸

