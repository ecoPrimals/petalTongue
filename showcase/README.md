# petalTongue Showcase

**Progressive demonstrations from local primal to full ecosystem**

SPDX-License-Identifier: AGPL-3.0-or-later

---

## Quick Start

Build first:

```bash
cargo build --release   # or cargo build (debug is fine)
```

Run the 5-minute automated tour:

```bash
./QUICK_START.sh
```

Or start with hello:

```bash
cd 01-local-primal/00-hello-petaltongue/
./demo.sh
```

---

## Structure

### 01-local-primal/ (petalTongue standalone, no external deps)

| # | Demo | What it proves | Time |
|---|------|---------------|------|
| 00 | [hello-petaltongue](01-local-primal/00-hello-petaltongue/) | status, version, JSON output | 30s |
| 01 | [unibin-modes](01-local-primal/01-unibin-modes/) | All 5 modes exist and run | 60s |
| 02 | [scenario-loading](01-local-primal/02-scenario-loading/) | Load 3 scenarios via web mode | 60s |
| 03 | [web-server](01-local-primal/03-web-server/) | HTTP endpoints, HTML dashboard | 60s |
| 04 | [headless-api](01-local-primal/04-headless-api/) | Headless rendering pipeline | 30s |
| 05 | [tui-dashboard](01-local-primal/05-tui-dashboard/) | Terminal UI (ratatui, Pure Rust) | 15s |
| 06 | [audio-export](01-local-primal/06-audio-export/) | Pure Rust WAV generation (hound) | 30s |
| 07 | [graph-layouts](01-local-primal/07-graph-layouts/) | Layout algorithms via topologies | 60s |
| 08 | [clinical-data](01-local-primal/08-clinical-data/) | healthSpring DataChannel rendering | 45s |
| 09 | [domain-themes](01-local-primal/09-domain-themes/) | Domain-aware color palettes | 45s |
| 10 | [visualization-push](01-local-primal/10-visualization-push/) | Spring IPC push + UiConfig | 45s |
| 11 | [scatter3d-data](01-local-primal/11-scatter3d-data/) | Scatter3D, Heatmap, FieldMap, Spectrum | 45s |
| 12 | [scene-graph](01-local-primal/12-scene-graph/) | Declarative scene graph operations | 30s |
| 13 | [grammar-compilation](01-local-primal/13-grammar-compilation/) | GrammarExpr -> SceneGraph pipeline | 30s |
| 14 | [tufte-constraints](01-local-primal/14-tufte-constraints/) | Machine-checked visualization quality | 30s |
| 15 | [math-objects](01-local-primal/15-math-objects/) | Manim-style: Axes, FunctionPlot, VectorField | 30s |
| 16 | [animation-system](01-local-primal/16-animation-system/) | Easing, transitions, sequences | 30s |
| 17 | [svg-modality](01-local-primal/17-svg-modality/) | Grammar -> SVG + Audio + Accessibility | 30s |
| 18 | [physics-bridge](01-local-primal/18-physics-bridge/) | barraCuda N-body IPC bridge | 30s |

**Total**: ~12 minutes | **Dependencies**: none

### 02-ipc-protocol/ (JSON-RPC, local only)

| # | Demo | What it proves | Time |
|---|------|---------------|------|
| 01 | [unix-socket-server](02-ipc-protocol/01-unix-socket-server/) | IPC over Unix domain sockets | 45s |
| 02 | [jsonrpc-methods](02-ipc-protocol/02-jsonrpc-methods/) | All supported JSON-RPC methods | 30s |
| 03 | [health-monitoring](02-ipc-protocol/03-health-monitoring/) | Health check protocol stability | 30s |

**Total**: ~2 minutes | **Dependencies**: curl

### 03-inter-primal/ (with other primals running)

| # | Demo | What it proves | Requires |
|---|------|---------------|----------|
| 01 | [songbird-discovery](03-inter-primal/01-songbird-discovery/) | Capability-based registration | Songbird (graceful skip) |
| 02 | [biomeos-topology](03-inter-primal/02-biomeos-topology/) | Topology visualization | biomeOS (graceful skip) |
| 03 | [ecosystem-health](03-inter-primal/03-ecosystem-health/) | Health dashboard | Any primal (graceful skip) |
| 04 | [multi-primal-tui](03-inter-primal/04-multi-primal-tui/) | TUI with ecosystem data | Any primal (graceful skip) |
| 05 | [full-ecosystem](03-inter-primal/05-full-ecosystem/) | All primals together | Multiple (graceful skip) |

**Total**: ~3 minutes | **Dependencies**: other primals (all gracefully skip if absent)

### 04-spring-integration/ (springs and biomeOS atomic deployments)

| # | Demo | What it proves | Requires |
|---|------|---------------|----------|
| 01 | [healthspring-push](04-spring-integration/01-healthspring-push/) | Clinical data push via IPC | None (standalone) |
| 02 | [biomeos-atomic-viz](04-spring-integration/02-biomeos-atomic-viz/) | Atomic deployment topology | biomeOS (graceful skip) |
| 03 | [scene-engine-pipeline](04-spring-integration/03-scene-engine-pipeline/) | Full grammar -> multi-modality pipeline | None |

**Total**: ~3 minutes | **Dependencies**: biomeOS optional (graceful skip)

---

## Principles

1. **Real commands, no mocks**: Every demo runs actual `petaltongue` subcommands
2. **Verifiable output**: Exit codes, JSON validation, HTTP status checks
3. **Progressive complexity**: local -> IPC -> inter-primal
4. **Graceful degradation**: Inter-primal demos skip cleanly if deps are absent
5. **Scenario-driven data**: Uses `sandbox/scenarios/*.json` (26 scenarios)
6. **No hardcoded ports**: Demos use high ephemeral ports to avoid conflicts

---

## Prerequisites

```bash
# Required
cargo build --release    # petalTongue binary
curl                     # HTTP probing (most demos)
python3                  # JSON validation

# Optional
socat                    # Unix socket communication
aplay / paplay           # Audio playback
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PETALTONGUE_BIN` | auto-detected | Path to petaltongue binary |
| `PAUSE_DURATION` | 1 | Seconds between steps (0 for CI) |
| `VERBOSE` | false | Extra debug output |
| `DEMO_OUTPUT_DIR` | /tmp/petaltongue-showcase | Artifact output dir |

---

Last updated: March 2026
