# Tutorial Mode & Graceful Degradation Architecture

## Overview

petalTongue implements a **tutorial/demonstration mode** that serves dual purposes:
1. **Explicit Tutorial Mode**: Educational scenarios when `SHOWCASE_MODE=true`
2. **Graceful Fallback**: Minimal working example when no primals are discovered

This architecture embodies the TRUE PRIMAL principle: **"Graceful degradation, not silent mocking"**

## Architecture Philosophy

### Core Principles

1. **Explicit Over Implicit**
   - Tutorial mode requires `SHOWCASE_MODE=true` environment variable
   - Never silently mocks in production
   - Always logs what data source is being used

2. **Educational First**
   - Tutorial scenarios teach how petalTongue works
   - Progressive complexity (simple → complex scenarios)
   - Self-documenting demonstration data

3. **Graceful Degradation**
   - If discovery fails, user still sees working interface
   - Minimal example shows core concepts
   - Clear messaging about fallback state

4. **Production Purity**
   - Production mode discovers real primals only
   - No hidden test data
   - Capability-based discovery (no hardcoding)

## Implementation

### Module: `tutorial_mode.rs`

Located at `crates/petal-tongue-ui/src/tutorial_mode.rs`

#### Key Components

**1. TutorialMode Struct**
```rust
pub struct TutorialMode {
    enabled: bool,        // Explicit tutorial mode
    scenario_name: String, // Which scenario to load
}
```

**2. Tutorial Mode Detection**
```rust
// Check SHOWCASE_MODE environment variable
let tutorial_mode = TutorialMode::new();

if tutorial_mode.is_enabled() {
    // Use demonstration scenarios
    tutorial_mode.load_into_graph(graph, layout);
}
```

**3. Graceful Fallback**
```rust
// If discovery returns no providers
if !tutorial_mode.is_enabled() && providers.is_empty() {
    if should_fallback(0) {
        // Create minimal working example
        TutorialMode::create_fallback_scenario(graph, layout);
    }
}
```

### Tutorial Scenarios

**Simple Scenario** (3 primals)
- petalTongue (self-awareness!)
- BearDog (Security)
- Songbird (Discovery)
- 2 connections

**Complex Scenarios** (from `sandbox_mock.rs`)
- Loaded from `sandbox/scenarios/` directory
- Multiple ecosystem configurations
- Trust relationships
- Health status variations

## Usage

### Explicit Tutorial Mode

```bash
# Enable tutorial mode
SHOWCASE_MODE=true cargo run

# Use specific scenario
SHOWCASE_MODE=true SANDBOX_SCENARIO=complex cargo run

# Disable graceful fallback
PETALTONGUE_NO_FALLBACK=true cargo run
```

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `SHOWCASE_MODE` | Enable explicit tutorial mode | `false` |
| `SANDBOX_SCENARIO` | Which scenario to load | `simple` |
| `PETALTONGUE_NO_FALLBACK` | Disable graceful fallback | `false` |

### Logs

Tutorial mode is transparent through logging:

```
🎭 TUTORIAL MODE ENABLED
📚 Scenario: simple
💡 This is demonstration data, not production primals
📦 Loading tutorial scenario: simple
✅ Loaded tutorial scenario: Simple ecoPrimals Topology
📊 Populating graph: 3 primals, 2 connections
✅ Tutorial data loaded successfully
```

Graceful fallback:

```
⚠️ No visualization providers discovered
🎭 GRACEFUL FALLBACK: No primals discovered
💡 Loading minimal tutorial scenario so you can explore petalTongue
📚 To see your real primals, ensure they're running and discoverable
✅ Fallback scenario ready
```

## Scenarios

### Minimal Example (Fallback)

Created when discovery fails and user hasn't disabled fallback.

**Primals:**
- `petaltongue-tutorial` - The visualization system itself
- `beardog-tutorial` - Example security primal
- `songbird-tutorial` - Example discovery primal

**Purpose:**
- Show basic graph structure
- Demonstrate node selection
- Enable UI exploration
- Teach petalTongue concepts

### Sandbox Scenarios (Explicit Tutorial)

Loaded from `sandbox/scenarios/` when `SHOWCASE_MODE=true`.

**simple.yaml** - Basic topology
- 5-6 primals
- Simple connections
- All healthy

**complex.yaml** - Advanced features
- 10+ primals
- Trust relationships
- Family lineages
- Mixed health states

**federation.yaml** - Multi-system
- Multiple ecosystems
- Cross-family connections
- Advanced capabilities

## Integration with app.rs

### Before Refactoring

`app.rs` contained:
- `load_sandbox_data()` method (~50 lines)
- `populate_sample_graph_legacy()` method (~135 lines)
- Showcase mode initialization (~10 lines)
- **Total: ~195 lines** of tutorial logic mixed with production code

### After Refactoring

`app.rs` simplified:
```rust
// Initialize tutorial mode
let tutorial_mode = TutorialMode::new();

// Discovery logic...

// Data loading
if tutorial_mode.is_enabled() {
    tutorial_mode.load_into_graph(graph, layout);
} else if needs_fallback {
    TutorialMode::create_fallback_scenario(graph, layout);
} else {
    app.refresh_graph_data(); // Production!
}
```

**Benefits:**
- Clear separation of concerns
- Tutorial logic in dedicated module
- Production code uncluttered
- Easy to test independently

## Design Decisions

### Why Not Always Mock?

**Anti-pattern:**
```rust
// BAD: Silent mocking
let providers = discover().await.unwrap_or_else(|_| vec![mock_provider()]);
```

**TRUE PRIMAL Way:**
```rust
// GOOD: Explicit fallback with logging
match discover().await {
    Ok(providers) if !providers.is_empty() => providers,
    _ => {
        if should_fallback(0) {
            warn!("GRACEFUL FALLBACK: Using tutorial data");
            vec![mock_provider()]
        } else {
            vec![]
        }
    }
}
```

**Rationale:**
- User knows what's happening
- Production never silently mocks
- Debugging is transparent
- Testing is explicit

### Why Minimal Fallback?

The minimal example (3 primals) is deliberately small:
- **Fast to understand** - New users aren't overwhelmed
- **Core concepts only** - Shows essential features
- **Encourages real usage** - Users want to see their actual primals
- **Clear it's a demo** - Names include "-tutorial" suffix

### Why Separate from sandbox_mock?

`tutorial_mode.rs` handles:
- Tutorial mode detection
- Graceful fallback logic
- Minimal example creation
- Integration with app.rs

`sandbox_mock.rs` handles:
- Scenario definitions
- File loading
- Complex test data
- Legacy compatibility

**Clean separation** = easier maintenance and testing

## Testing

```rust
#[test]
fn test_tutorial_mode_detection() {
    // Tutorial mode off by default
    let tutorial = TutorialMode::new();
    assert!(!tutorial.is_enabled());
}

#[test]
fn test_graceful_fallback() {
    // Should fallback when no providers
    assert!(should_fallback(0));
    
    // Should not fallback when providers exist
    assert!(!should_fallback(5));
}

#[test]
fn test_minimal_example() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(graph.clone(), LayoutAlgorithm::ForceDirected);
    
    let graph = graph.read().unwrap();
    assert_eq!(graph.nodes().len(), 3); // Minimal: 3 primals
    assert_eq!(graph.edges().len(), 2); // 2 connections
}
```

## Comparison: Before vs. After

### Before (Mixed Responsibilities)

```
app.rs: 1446 lines
├── Production logic
├── Tutorial logic (scattered)
├── Sample data generation
└── Showcase mode handling
```

### After (Clean Separation)

```
app.rs: ~1250 lines (production-focused)
├── Production logic
├── Tutorial mode integration (minimal)
└── Graceful fallback trigger

tutorial_mode.rs: ~340 lines (tutorial-focused)
├── Tutorial detection
├── Scenario loading
├── Minimal example
└── Fallback logic
```

**Result:** ~195 lines removed from `app.rs`, isolated to dedicated module

## Future Enhancements

1. **Interactive Tutorial**
   - Step-by-step walkthrough
   - Tooltips and hints
   - Progressive feature unlocking

2. **Tutorial Library**
   - More scenarios
   - User-created tutorials
   - Community sharing

3. **Recording Mode**
   - Capture real topology
   - Export as scenario
   - Share configurations

4. **Scenario Validation**
   - Schema validation
   - Completeness checks
   - Version compatibility

## Related Documentation

- [SHOWCASE_MODE](../tutorials/SHOWCASE_MODE.md) - How to use tutorial mode
- [SANDBOX_SCENARIOS](../tutorials/SANDBOX_SCENARIOS.md) - Available scenarios
- [TRUE_PRIMAL_ARCHITECTURE](./TRUE_PRIMAL_ARCHITECTURE.md) - Core principles

---

**Status**: ✅ Implemented  
**Version**: 1.0.0  
**Date**: January 6, 2026  
**Module**: `petal-tongue-ui/src/tutorial_mode.rs`

