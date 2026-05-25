# Start Here

SPDX-License-Identifier: AGPL-3.0-or-later

## What is this?

The petalTongue showcase demonstrates the Universal UI primal's capabilities
through runnable shell scripts that exercise real commands - no mocks.

## Before you begin

```bash
# From the petalTongue root:
cargo build --release
```

## First demo (30 seconds)

```bash
cd showcase/01-local-primal/00-hello-petaltongue/
./demo.sh
```

This runs `petaltongue status`, verifies output, and checks JSON serialization.

## Automated tour (~15 minutes, or set PAUSE_DURATION=0 for CI)

```bash
cd showcase/
./QUICK_START.sh
```

Runs all local demos that need no external dependencies.

## What each section covers

- **01-local-primal/**: petalTongue by itself. Status, web server, TUI,
  headless mode, audio export, graph layouts, clinical data rendering,
  domain themes, IPC visualization push, and advanced data types
  (Scatter3D, Heatmap, FieldMap, Spectrum). No external primal dependencies.

- **02-ipc-protocol/**: JSON-RPC over Unix sockets. Demonstrates the
  inter-primal communication protocol locally.

- **03-inter-primal/**: Integration with Songbird, biomeOS, and the full
  ecosystem. These demos gracefully skip if the other primals are not running.

- **04-spring-integration/**: healthSpring push, biomeOS atomic visualization,
  and scene engine pipeline demos. Requires running springs or gracefully skips.

## Next steps

After the showcase:

- Read `specs/` for the Grammar of Graphics architecture
- Read `sandbox/scenarios/` for scenario file format
- Try `petaltongue tui --scenario sandbox/scenarios/live-ecosystem.json` interactively
- Try `petaltongue web --scenario sandbox/scenarios/full-dashboard.json` and open your browser
