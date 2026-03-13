# Awakening to Tutorial Transition

**Status**: ✅ Implemented  
**Date**: January 8, 2026  
**Version**: 0.2.0-dev

## Overview

Seamless transition from the 4-stage awakening experience into tutorial/sandbox mode.

## Flow

```
Awakening Experience (12 seconds)
         ↓
  Stage 1: Awakening (0-3s)
         ↓
  Stage 2: Discovery (3-6s)
         ↓
  Stage 3: Connection (6-9s)
         ↓
  Stage 4: Harmony (9-12s)
         ↓
    ┌─────────┴─────────┐
    ▼                   ▼
Tutorial Mode      Production Mode
(SHOWCASE_MODE)    (Discovery)
```

## Implementation

### 1. Awakening Coordinator

The `AwakeningCoordinator::run()` method now returns `bool`:

```rust
pub async fn run(&self) -> Result<bool>
```

- Returns `true` if tutorial mode should be activated
- Checks `SHOWCASE_MODE` environment variable
- Logs transition intent

### 2. Tutorial Mode Integration

Tutorial mode is activated after awakening completes:

```rust
let tutorial_mode = coordinator.run().await?;

if tutorial_mode {
    let tutorial = TutorialMode::new();
    tutorial.load_into_graph(graph, layout);
}
```

### 3. User Experience

**With Tutorial Mode** (`SHOWCASE_MODE=true`):
```
🌸 Awakening: Stage 1 → Awakening...
🌸 Awakening: Stage 2 → Discovery...
🌸 Awakening: Stage 3 → Connection...
🌸 Awakening: Stage 4 → Harmony...
✨ Awakening complete! Transitioning to main experience...
🎭 Tutorial mode detected - will load demonstration scenarios
📦 Loading tutorial scenario: simple
🌸 Seamless transition from awakening to tutorial experience
✅ Tutorial data loaded successfully
🎓 Tutorial mode active - explore the sandbox!
```

**Without Tutorial Mode** (Production):
```
🌸 Awakening: Stage 1 → Awakening...
🌸 Awakening: Stage 2 → Discovery...
🌸 Awakening: Stage 3 → Connection...
🌸 Awakening: Stage 4 → Harmony...
✨ Awakening complete! Transitioning to main experience...
🔍 Discovering primals via capability-based discovery...
```

## Environment Variables

- `AWAKENING_ENABLED=true` - Enable awakening experience (default: true)
- `SHOWCASE_MODE=true` - Enable tutorial mode after awakening
- `SANDBOX_SCENARIO=<name>` - Select tutorial scenario (default: "simple")

## Scenarios

Available tutorial scenarios:

1. **simple** - Basic 3-primal topology
2. **complex** - Multi-layer ecosystem
3. **distributed** - Geographic distribution
4. **stressed** - Performance testing

## Benefits

✅ **Seamless**: No jarring transition between awakening and tutorial  
✅ **Contextual**: Tutorial mode only when explicitly requested  
✅ **Logged**: User always knows what's happening  
✅ **Flexible**: Works with any scenario  
✅ **Production-Safe**: Tutorial mode is explicit, not a fallback

## Code Locations

- **Coordinator**: `crates/petal-tongue-core/src/awakening_coordinator.rs`
- **Tutorial Mode**: `crates/petal-tongue-ui/src/tutorial_mode.rs`
- **Integration**: Application startup code

## Testing

```bash
# Test awakening → tutorial transition
AWAKENING_ENABLED=true SHOWCASE_MODE=true cargo run

# Test awakening → production
AWAKENING_ENABLED=true cargo run

# Test without awakening
AWAKENING_ENABLED=false cargo run
```

## Future Enhancements

- [ ] Fade transition between awakening and tutorial
- [ ] Tutorial intro overlay after awakening
- [ ] Guided tour system
- [ ] Interactive tutorial steps
- [ ] Progress tracking

## Philosophy

> "The awakening experience should flow naturally into the user's
> chosen path - whether that's learning through tutorials or
> discovering real primals in production."

The transition is:
- **Explicit**: User chooses tutorial mode
- **Seamless**: No interruption in experience
- **Contextual**: Different paths for different needs
- **Logged**: Transparent about what's happening

---

**Status**: ✅ Complete  
**Quality**: ✅ Production Ready  
**Documentation**: ✅ Comprehensive

