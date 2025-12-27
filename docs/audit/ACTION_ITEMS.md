# 📋 petalTongue Action Items

**Generated**: December 27, 2025  
**From**: Comprehensive Audit Report  
**Priority**: Categorized by impact and urgency

---

## 🔴 CRITICAL - Do Immediately (This Session)

### 1. Fix Formatting Issues
**Impact**: Blocks CI/CD  
**Effort**: 5 minutes  
**Status**: ⚠️ IN PROGRESS

```bash
# Fix trailing whitespace in app.rs:605
# Then run:
cargo fmt --all --check
```

### 2. Add Missing Documentation
**Impact**: API clarity  
**Effort**: 15 minutes  
**Files**: `crates/petal-tongue-ui/src/toadstool_bridge.rs`

```rust
// Add doc comments to:
/// Tool name for ToadStool bridge
pub tool_name: String,

/// Input data for tool execution
pub input: serde_json::Value,

/// Execution status: "success" | "error"
pub status: String,

/// Optional output data from tool execution
pub output: Option<serde_json::Value>,

/// Optional error message if execution failed
pub error: Option<String>,
```

### 3. Create Environment Variable Documentation
**Impact**: Deployment clarity  
**Effort**: 20 minutes  

```bash
# Create .env.example
cat > .env.example << 'EOF'
# BiomeOS Connection
BIOMEOS_URL=http://localhost:3000

# Mock Mode (for testing without BiomeOS)
PETALTONGUE_MOCK_MODE=false

# Refresh Interval (milliseconds)
PETALTONGUE_REFRESH_INTERVAL=500

# Audio Settings
PETALTONGUE_AUDIO_ENABLED=true

# Logging
RUST_LOG=info
EOF
```

### 4. Remove Dead Code
**Impact**: Code cleanliness  
**Effort**: 5 minutes  

```rust
// toadstool_bridge.rs:160
// Remove unused field or add usage:
pub struct PythonToolPanel {
    metadata: ToolMetadata,
    // bridge: Arc<ToadStoolBridge>,  // ❌ Remove if not needed
    // ... other fields
}
```

---

## 🟡 IMPORTANT - Do This Week

### 5. Wire Up Animation Rendering
**Impact**: Feature completion  
**Effort**: 2-4 hours  
**TODOs**: 4 high-priority items

**Files**:
- `app.rs:31` - Activate animation rendering in visual_renderer
- `app.rs:44` - Wire up animation toggle to visual_renderer  
- `capabilities.rs:103` - Actually test animation
- `app.rs:129` - Move to background task with channels

**Implementation Plan**:
```rust
// 1. In app.rs, connect animation engine to visual renderer
impl eframe::App for PetalTongueApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update animation engine
        let dt = ctx.input(|i| i.stable_dt);
        self.animation_engine.update(dt);
        
        // Pass animations to visual renderer
        self.state.visual_renderer.set_animations(
            self.animation_engine.get_active_animations()
        );
        
        // Render with animations
        self.state.visual_renderer.render(ui, &graph);
    }
}

// 2. Add animation test in capabilities.rs
fn test_animation_capability() -> bool {
    let engine = AnimationEngine::new();
    // Try to create and update an animation
    engine.spawn_flow_particle(...).is_ok()
}
```

### 6. Make ALSA Dependency Optional
**Impact**: Enables full linting, broader platform support  
**Effort**: 1-2 hours  

**Changes Needed**:
```toml
# Cargo.toml - Make audio optional
[features]
default = ["audio"]
audio = ["bingocube-core/audio"]

[dependencies]
bingocube-core = { version = "0.1.0", default-features = false, optional = true }
```

```rust
// lib.rs - Conditional compilation
#[cfg(feature = "audio")]
pub mod audio_sonification;

#[cfg(feature = "audio")]
pub use audio_sonification::AudioSonificationRenderer;
```

### 7. Add E2E Test Framework
**Impact**: Production confidence  
**Effort**: 1 day  

**Implementation**:
```rust
// tests/e2e/mod.rs
#[cfg(test)]
mod e2e_tests {
    use petal_tongue_ui::PetalTongueApp;
    
    #[test]
    fn test_full_discovery_cycle() {
        // 1. Start mock BiomeOS
        // 2. Launch petalTongue
        // 3. Verify discovery
        // 4. Verify topology rendering
        // 5. Verify audio playback
        // 6. Clean shutdown
    }
    
    #[test]
    fn test_unhealthy_primal_detection() {
        // 1. Mock primal goes unhealthy
        // 2. Verify visual update (color change)
        // 3. Verify audio update (pitch change)
        // 4. Verify alerts triggered
    }
}
```

### 8. Add Chaos Test Suite
**Impact**: Reliability assurance  
**Effort**: 2 days  

**Scenarios**:
```rust
// tests/chaos/mod.rs
#[tokio::test]
async fn chaos_primal_churn() {
    // Rapidly add/remove primals
    // Verify no crashes, no deadlocks
}

#[tokio::test]
async fn chaos_network_partition() {
    // Simulate BiomeOS unreachable
    // Verify graceful degradation
}

#[tokio::test]
async fn chaos_memory_pressure() {
    // Simulate low memory
    // Verify no allocations fail
}

#[tokio::test]
async fn chaos_high_update_rate() {
    // 1000 updates/second
    // Verify performance degradation is graceful
}
```

---

## 🟢 NICE-TO-HAVE - Do Next Month

### 9. Improve UI Test Coverage (0% → 50%)
**Impact**: Bug prevention  
**Effort**: 1 week  

**Strategy**:
- Use egui test harness
- Mock rendering context
- Test state transitions
- Test user interactions

### 10. Add Performance Benchmarks
**Impact**: Optimization guidance  
**Effort**: 2-3 days  

```rust
// benches/graph_rendering.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_graph_layout(c: &mut Criterion) {
    let mut graph = create_test_graph(100); // 100 nodes
    
    c.bench_function("force_directed_100_nodes", |b| {
        b.iter(|| {
            graph.apply_force_directed_layout(black_box(100));
        });
    });
}

criterion_group!(benches, bench_graph_layout);
criterion_main!(benches);
```

### 11. Complete Phase 1 Spec Features
**Impact**: Feature completeness  
**Effort**: 2-3 weeks  

**Missing**:
- Timeline sequence diagram view
- Traffic Sankey diagram
- Multi-window support
- Alert list view

### 12. Create Deployment Guide
**Impact**: User adoption  
**Effort**: 1 day  

**Sections**:
- System requirements
- Installation methods (cargo, binary, Docker)
- Configuration guide
- Integration with BiomeOS
- Troubleshooting
- Monitoring and logs

---

## 📊 Coverage Improvement Plan

### Goal: 47% → 90% (A Grade)

**Current Coverage**:
```
Overall:    47.08%
Core:       78.56% ✅
UI:          0.00% ❌
```

**Target Coverage** (by component):
```
Core Types:         100% (already achieved ✅)
Error Handling:     100% (already achieved ✅)
Capabilities:        98% (already achieved ✅)
Audio:               96% (already achieved ✅)
Telemetry:           96% (already achieved ✅)
BiomeOS Client:      90% (already achieved ✅)
Animation:           85% (from 79%, +6%)
Graph Engine:        90% (from 79%, +11%)
Audio Export:        85% (from 70%, +15%)
Config:              85% (from 61%, +24%)
Visual 2D:           60% (from 38%, +22%)
App/UI:              40% (from 0%, +40%)
```

**Strategy**:
1. **Week 1**: Config tests (+24%)
2. **Week 2**: Visual 2D tests (+22%)
3. **Week 3**: App/UI tests (+40%)
4. **Week 4**: Polish remaining gaps

**Estimated Time**: 4 weeks to 90% coverage

---

## 🔧 Quick Wins (< 1 hour each)

### 1. Fix Trailing Whitespace ✅
```bash
# Already identified: app.rs:605
sed -i 's/[[:space:]]*$//' crates/petal-tongue-ui/src/app.rs
```

### 2. Add .gitignore Entry
```bash
echo "*.env" >> .gitignore
echo ".env.local" >> .gitignore
```

### 3. Update README with Env Vars
```markdown
## Configuration

petalTongue uses environment variables for configuration:

- `BIOMEOS_URL` - BiomeOS endpoint (default: `http://localhost:3000`)
- `PETALTONGUE_MOCK_MODE` - Enable mock mode (default: `false`)
- `PETALTONGUE_AUDIO_ENABLED` - Enable audio (default: `true`)
- `RUST_LOG` - Logging level (default: `info`)
```

### 4. Add CONTRIBUTING.md
```markdown
# Contributing to petalTongue

## Code Quality Standards
- All code must pass `cargo fmt --check`
- All code must pass `cargo clippy --all-targets`
- All tests must pass: `cargo test --all`
- New code must have tests (target: 90% coverage)
- Public APIs must have doc comments
```

---

## 🎯 Milestones

### Milestone 1: Clean Codebase (This Week)
- [x] Run audit
- [ ] Fix formatting issues
- [ ] Add missing docs
- [ ] Create .env.example
- [ ] Remove dead code

**ETA**: 1-2 days  
**Blockers**: None

### Milestone 2: Feature Complete (Next Week)
- [ ] Wire up animation
- [ ] Complete Phase 1 features
- [ ] Add E2E tests
- [ ] Make ALSA optional

**ETA**: 1 week  
**Blockers**: None

### Milestone 3: Production Ready (Next Month)
- [ ] 90% test coverage
- [ ] Performance benchmarks
- [ ] Chaos test suite
- [ ] Deployment guide

**ETA**: 1 month  
**Blockers**: Time allocation

---

## 📈 Tracking Progress

### Dashboard

| Category | Current | Target | Progress |
|----------|---------|--------|----------|
| Test Coverage | 47.08% | 90% | ███████░░░░░░░ 52% |
| Documentation | 85% | 100% | ████████████░░ 85% |
| Features | 65% | 100% | █████████░░░░░ 65% |
| Code Quality | 85% | 95% | ████████████░░ 89% |

### Next Review
**Date**: After Milestone 1 completion  
**Focus**: Feature completeness and test coverage

---

## 💬 Questions for Team

1. **ALSA Dependency**: Should we make audio fully optional or keep it?
   - **Recommendation**: Make optional via feature flag

2. **Coverage Target**: Is 90% realistic for UI-heavy code?
   - **Recommendation**: Yes, with egui test harness

3. **Phase 2 Timeline**: When do we start REST API work?
   - **Recommendation**: After Phase 1 is complete

4. **Deployment Priority**: Docker first or native binary?
   - **Recommendation**: Native binary (Rust compiles easily)

---

## 📝 Notes

- All critical items can be completed in **2-4 hours**
- Important items need **1 week focused work**
- Nice-to-have items are **1 month effort**
- Current quality is **production-ready** with improvements

**Recommendation**: Focus on critical items first, then important items in parallel.

---

**Status**: Ready for implementation 🚀  
**Confidence**: HIGH  
**Risk**: LOW

---

*Action items derived from comprehensive audit findings. Prioritized for maximum impact with minimum effort.*

