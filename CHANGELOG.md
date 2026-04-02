# Changelog

All notable changes to petalTongue are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Push delivery for callback dispatches (PT-06): subscribers providing
  `callback_socket` and `callback_method` now receive JSON-RPC notifications
  via UDS or TCP instead of polling. Background delivery task with graceful
  fallback to poll on failure.
- `push_delivery` module in `petal-tongue-ipc` with `spawn_push_delivery()`.
- `callback_socket` field on `CallbackDispatch` and `InteractionSubscriber`.
- `sensor_stream` module extracted from `interaction.rs` — `SensorStreamRegistry`
  now lives in its own module for clearer domain separation.
- `DEFAULT_WEBSOCKET_PORT` constant in `petal-tongue-core::constants`.
- `#![warn(missing_docs)]` on `petal-tongue-headless`.
- `#![expect(missing_docs)]` on `doom-core` (tracked for incremental completion).
- Zero-copy `Arc<InteractionEventNotification>` in subscriber queues —
  broadcast shares one allocation across N subscribers instead of N clones.
- Awakening coordinator `process_event` test coverage for all 6 event types.
- `DataService::snapshot_sync` tests (happy path, populated graph, poisoned lock).
- `CHANGELOG.md` (this file).
- Phase 3 wateringHole handoff document.

### Changed
- `ENV_VARS.md`: `PETALTONGUE_MOCK_MODE` → `PETALTONGUE_FIXTURE_MODE` with
  migration note and removal timeline.
- `README.md`: updated `forbid(unsafe_code)` to reflect unconditional status.
- `wateringHole/petaltongue/README.md`: updated stats (coverage, date, forbid).
- `sandbox/scenarios/README.md`: corrected stale directory structure, updated date.
- **BREAKING**: Renamed `mock_mode` → `fixture_mode` across config, API client,
  and UI manager. `BiomeOSClient::with_mock_mode()` → `with_fixture_mode()`,
  `is_mock_mode()` → `is_fixture_mode()`, `MockModeUnavailable` →
  `FixtureModeUnavailable`. Config field `mock_mode` accepted as serde alias
  for backwards compatibility.
- UI label changed from "Mock Mode" to "Fixture Mode (offline)".

### Fixed
- Removed unfulfilled `#[expect(clippy::unused_self)]` and
  `#[expect(clippy::cast_sign_loss)]` in `petal-tongue-api` — clippy now
  passes with zero warnings across the workspace.
- Replaced `as u64` cast with `.cast_unsigned()` in `biomeos_client.rs`.
- Eliminated the last `unsafe` block in `petal-tongue-ipc` (provenance_trio test):
  replaced raw `env::set_var`/`remove_var` with `temp_env::with_vars` via
  `petal_tongue_core::test_fixtures::env_test_helpers`. Upgraded crate to
  unconditional `#![forbid(unsafe_code)]`.
- Hardcoded `127.0.0.1` in `display/backends/software.rs` replaced with
  `constants::DEFAULT_LOOPBACK_HOST`; hardcoded port `8765` replaced with
  `constants::DEFAULT_WEBSOCKET_PORT`.

## [1.6.6] — 2026-04-01

### Added
- `visualization.render.scene` RPC method (PT-04) — direct `SceneGraph` submission.
- `visualization.export` RPC method (PT-06 listen surface).
- `ExportFormat::Html` for headless CLI export (SVG wrapped in standalone HTML).
- `CallbackDispatch` struct and `pending_callbacks` response field (PT-06 data model).
- `RenderingAwareness` auto-init in `UnixSocketServer` (PT-05).
- Server-mode periodic discovery refresh (PT-07).
- blake3 provenance hashing (pure Rust, zero C/asm deps).
- Centralized discovery timeouts (12 named constants, env-overridable).
- `primal_names` and `socket_roles` constants — no hardcoded primal strings.
- `rust-toolchain.toml` (pins stable + rustfmt, clippy, llvm-tools-preview).
- `deny.toml` for advisory, license, ban, and source auditing.
- `llvm-cov.toml` with 90% `fail-under` threshold.
- Spring data adapter (`SpringDataAdapter`) for ludoSpring / ecoPrimals formats.
- `DispatchOutcome`, `IpcErrorPhase`, `StreamItem` structured IPC tracing.
- `CircuitBreaker` and `RetryPolicy` in resilience module.

### Changed
- All JSON-RPC serialization hot paths evolved from `serde_json::to_string` to
  `serde_json::to_vec` (zero intermediate `String` allocation per frame).
- Documentation reconciled to "tarpc PRIMARY, JSON-RPC universal fallback".
- TUI devices panel renders real discovered `PrimalInfo` instead of placeholders.
- Extracted `parse_primal_array()`, `world_to_screen()`, `adjacency_list()`
  to reduce duplication.

### Fixed
- `#![forbid(unsafe_code)]` on all crates (one justified test exception).
- Zero clippy warnings (pedantic + nursery, `-D warnings`).
- Zero `TODO`/`FIXME`/`HACK` markers in committed code.
- All files under 1000 lines (max 852).

### Removed
- `DefaultHasher` placeholder in `provenance_trio.rs` (replaced by blake3).
- Hardcoded primal name strings in 7 files.

## [1.6.5] — 2026-03-28

### Added
- `deny(clippy::unwrap_used, clippy::expect_used)` at workspace level.
- Enriched `capability.list` response with transport and protocol metadata.

## [1.6.4] — 2026-03-26

### Changed
- Version bump, root doc cleanup, stale script path fixes.

[Unreleased]: https://github.com/ecoPrimals/petalTongue/compare/v1.6.6...HEAD
[1.6.6]: https://github.com/ecoPrimals/petalTongue/compare/v1.6.5...v1.6.6
[1.6.5]: https://github.com/ecoPrimals/petalTongue/compare/v1.6.4...v1.6.5
[1.6.4]: https://github.com/ecoPrimals/petalTongue/releases/tag/v1.6.4
