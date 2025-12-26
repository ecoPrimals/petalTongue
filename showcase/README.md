# 🎬 petalTongue Showcase

This directory contains demonstrations of petalTongue's capabilities.

---

## Local Showcases (Standalone petalTongue)

1. **[Graph Visualization](local/01-graph-visualization/)** ✅
   - Basic graph rendering with force-directed layout
   - Interactive zoom, pan, drag
   - Node and edge visualization

2. **[Layout Algorithms](local/02-layout-algorithms/)** ✅
   - Force-Directed layout
   - Hierarchical layout
   - Circular layout
   - Random layout

3. **[Audio Sonification](local/03-audio-sonification/)** ✅
   - Multi-instrument mapping (5 instruments)
   - Spatial audio (stereo panning)
   - Real-time audio generation

4. **[Animation & Flow](local/04-animation-flow/)** ⚠️
   - Flow particles along edges
   - Node pulse effects
   - *Note: Scaffold in place, animations need activation*

5. **[Multi-Modal Representation](local/05-multi-modal/)** ✅
   - Visual + Audio + Text simultaneously
   - Adaptive rendering based on available modalities
   - Real-time updates across all modes

6. **[Configuration & Layout](local/06-config-layout/)** ✅
   - 9-field configuration system
   - Environment variable support
   - Dynamic layout switching

7. **[BingoCube Visualization](local/07-bingocube-visualization/)** ✅
   - Primal tool integration pattern
   - Visual rendering (color grid, progressive reveal)
   - Audio sonification (cell-to-sound mapping)
   - Interactive configuration (grid size, palette, presets)
   - Multi-modal feedback

8. **[Pure Rust Audio Export](local/08-audio-export/)** ✅ **NEW**
   - Self-aware capability detection
   - Honest status reporting (no false claims)
   - Pure Rust WAV generation (no system dependencies)
   - One-click export for graph and BingoCube audio
   - Production-ready accessibility solution

---

## Primal Interaction Showcases (petalTongue + Other Primals)

**Status**: 🔜 Coming Soon

These will demonstrate petalTongue interacting with other ecoPrimals:
- ToadStool (distributed audio generation)
- NestGate (content-addressed storage)
- Songbird (P2P discovery)
- BearDog (identity management)
- Squirrel (resource management)

---

## Running Showcases

### Local Showcases

Each local showcase has a `demo.sh` script:

```bash
cd showcase/local/<showcase-name>/
./demo.sh
```

Or use the automated runner:

```bash
cd showcase/
./run-local-showcase.sh <number>
```

Examples:
```bash
./run-local-showcase.sh 7   # BingoCube visualization
./run-local-showcase.sh 8   # Pure Rust audio export
```

### Requirements

- Rust toolchain (1.70+)
- petalTongue built: `cargo build --release -p petal-tongue-ui`
- For audio playback (optional): `mpv` or `vlc`

---

## Presentation Materials

See individual showcase `README.md` files for:
- Detailed feature descriptions
- Test scenarios
- Troubleshooting guides
- Technical architecture details
- Philosophy and design decisions

---

**Last Updated**: December 26, 2025  
**Version**: 0.1.0  
**Status**: 8 local showcases ready, primal interactions coming soon
