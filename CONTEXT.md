# Context — petalTongue

## What This Is

petalTongue is the **Universal User Interface primal** for the ecoPrimals ecosystem: a 100% Pure Rust visualization and interaction engine that translates any computational universe into any modality for any user type. Other primals send data to petalTongue via JSON-RPC, and it compiles that data into visual, audio, haptic, terminal, or API representations using a composable Grammar of Graphics pipeline.

## Role in the Ecosystem

petalTongue answers "show me this data" and "let me interact with this system" for every primal and spring in the ecosystem. It does not own compute orchestration (Toadstool/coralReef), cryptography (BearDog), mesh networking (Songbird), or AI inference (Squirrel); it consumes their outputs and renders them for humans and agents through capability-based discovery at runtime.

## Technical Facts

- **Language:** 100% Rust (edition 2024), zero C dependencies in application code
- **License:** AGPL-3.0-or-later (SPDX on all sources)
- **Version:** 1.6.6
- **Workspace:** 18 crates (`Cargo.toml` workspace)
- **MSRV:** 1.87 (`rust-version` in workspace `Cargo.toml`)
- **Tests:** 5,967+ passing (0 failed)
- **Coverage:** ~90% line (llvm-cov, workspace)
- **Unsafe:** 0 production blocks (`forbid(unsafe_code)` workspace-wide)
- **IPC:** JSON-RPC 2.0 over Unix sockets / TCP (REQUIRED); tarpc for Rust-to-Rust hot paths (optional)
- **WASM:** `petal-tongue-wasm` crate compiles to `wasm32-unknown-unknown` for client-side rendering (grammar → SVG pipeline, offline-capable)

## Key Capabilities

- **Modes:** `ui` (egui desktop), `tui` (ratatui terminal), `web` (axum), `headless` (SVG/PNG/JSON), `server` (IPC-only), `status` (system info)
- **Grammar of Graphics:** Composable data→representation pipeline with 11 DataBinding types
- **Tufte constraints:** Machine-checked visualization quality
- **Multi-modal output:** Visual, audio, haptic, terminal, braille, JSON API
- **SAME DAVE model:** Sensory Afferent / Motor Efferent bidirectional feedback loops
- **Sensory Capability Matrix:** Formal input×output negotiation for consumer primals
- **Accessibility:** Switch access, audio inverse pipeline, agent adapter for AI
- **IPC methods:** 60+ JSON-RPC methods across health, capability, visualization, interaction, audio, motor, and UI domains

## What This Does NOT Do

petalTongue is not a GPU compute dispatcher (that is Toadstool/coralReef), not a mesh network (Songbird), not encrypted storage (NestGate), not a cryptography provider (BearDog), and not an AI runtime (Squirrel). It discovers and delegates to those capabilities at runtime via the Neural API or filesystem-based capability discovery.

## Build and Test

```bash
cargo build --release
cargo test --workspace --all-features
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

Run modes: `petaltongue ui`, `petaltongue tui`, `petaltongue web`, `petaltongue server`, `petaltongue headless`, `petaltongue status`.

## Related Repositories

- [wateringHole](https://github.com/ecoPrimals/wateringHole) — ecosystem standards, inter-primal contracts, handoffs
- [ecoPrimals](https://github.com/ecoPrimals) — organization root for sovereign computing primals
