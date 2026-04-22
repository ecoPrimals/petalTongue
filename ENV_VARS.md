# Environment Variables Reference

This document describes all environment variables used by petalTongue.

## Socket Configuration (biomeOS Standard)

### **PETALTONGUE_SOCKET**
**Type**: String (absolute path)  
**Default**: None (uses XDG runtime or /tmp fallback)  
**Required**: No  
**Example**: `PETALTONGUE_SOCKET=/run/user/1000/biomeos/petaltongue.sock`

**HIGHEST PRIORITY** socket path override. When set, petalTongue will use this exact socket path.

**biomeOS Socket Standard** (per `IPC_COMPLIANCE_MATRIX.md` v1.3):
- Priority 1: `PETALTONGUE_SOCKET` (explicit override)
- Priority 2: `$XDG_RUNTIME_DIR/biomeos/petaltongue.sock` (standard)
- Priority 3: `/tmp/biomeos/petaltongue.sock` (fallback)

**Use Cases**:
- Atomic deployments with custom socket locations
- Testing with specific socket paths
- Multi-instance deployments with explicit coordination

---

### **FAMILY_ID**
**Type**: String  
**Default**: `nat0` (development / standalone)  
**Required**: No  
**Example**: `FAMILY_ID=staging`

Family identifier for this petalTongue instance. Controls both identity and BTSP security posture.

**BTSP Phase 1 behavior** (per `BTSP_PROTOCOL_STANDARD.md`):
- **Not set / empty / `"default"`**: Development posture — socket is `petaltongue.sock`, no BTSP handshake.
- **Set to a non-default value** (e.g. `staging`, `prod-a`): Production posture — socket becomes `petaltongue-{family_id}.sock`, domain symlink `visualization-{family_id}.sock`. BTSP Phase 2 handshake required when BearDog enforces it.

**Precedence**: `PETALTONGUE_FAMILY_ID` > `FAMILY_ID` (primal-specific override per Self-Knowledge Standard).

---

### **BIOMEOS_INSECURE**
**Type**: Boolean (`1` | `true`)  
**Default**: Not set  
**Required**: No  
**Example**: `BIOMEOS_INSECURE=1`

Development-only flag that explicitly opts into cleartext JSON-RPC without BTSP handshake.

**BTSP Guard**: If both `FAMILY_ID` (non-default) AND `BIOMEOS_INSECURE=1` are set, petalTongue **refuses to start** with a FATAL error. This prevents accidentally running a production family in insecure mode.

**Valid combinations**:
- `FAMILY_ID` unset + `BIOMEOS_INSECURE=1` → development (cleartext, no handshake)
- `FAMILY_ID=prod-a` + `BIOMEOS_INSECURE` unset → production (BTSP handshake when enforced)
- `FAMILY_ID=prod-a` + `BIOMEOS_INSECURE=1` → **FATAL: conflicting posture, startup refused**

**Security Note**: Never set `BIOMEOS_INSECURE=1` in production deployments.

---

### **PETALTONGUE_NODE_ID**
**Type**: String  
**Default**: `default`  
**Required**: No  
**Example**: `PETALTONGUE_NODE_ID=node1`

Node identifier for multi-instance deployments. Used for registration and identity.

**Note**: Like `FAMILY_ID`, the node ID is not embedded in the default socket filename.
Use `PETALTONGUE_SOCKET` for explicit multi-instance socket placement.

---

### **XDG_RUNTIME_DIR**
**Type**: String (directory path)  
**Default**: `/run/user/<uid>` (auto-detected)  
**Required**: No  
**Example**: `XDG_RUNTIME_DIR=/run/user/1000`

Standard XDG runtime directory for socket placement. This is the standard Unix location for user-level runtime files.

**TRUE PRIMAL Principle**: Uses standard Unix conventions rather than hardcoded paths.

---

## Server Ports

### **PETALTONGUE_WEB_PORT**
**Type**: Integer (port number)  
**Default**: `3000`  
**Required**: No  
**Example**: `PETALTONGUE_WEB_PORT=8080`

Port for the web server (`petaltongue web`). Used by `constants::default_web_bind()`.

---

### **PETALTONGUE_HEADLESS_PORT**
**Type**: Integer (port number)  
**Default**: `8080`  
**Required**: No  
**Example**: `PETALTONGUE_HEADLESS_PORT=9000`

Port for the headless API server (`petaltongue headless`). Used by `constants::default_headless_bind()`.

---

### **PETALTONGUE_BIND_ADDR**
**Type**: String (IP address)  
**Default**: `127.0.0.1` (loopback only)  
**Required**: No  
**Example**: `PETALTONGUE_BIND_ADDR=0.0.0.0`

Bind address for web and headless servers. Set to `0.0.0.0` for external access.

---

### **PETALTONGUE_TCP_BIND_HOST**
**Type**: String (IP address or hostname)  
**Default**: `127.0.0.1` (loopback; see `constants::DEFAULT_LOOPBACK_HOST`)  
**Required**: No  
**Example**: `PETALTONGUE_TCP_BIND_HOST=0.0.0.0`

Host address used when the IPC server binds an ephemeral TCP listener (Phase 3 TCP transport). Does not affect Unix socket paths.

---

## Discovery & Integration

### **BIOMEOS_URL**
**Type**: String (URL)  
**Default**: None (discovered at runtime)  
**Required**: No  
**Example**: `BIOMEOS_URL=unix:///run/user/1000/biomeos-device-management.sock`

URL of the BiomeOS API endpoint. Supports both Unix sockets (primary) and HTTP (fallback).

**Formats**:
- `unix:///run/user/1000/biomeos.sock` - Unix socket (PRIMARY protocol)
- `http://biomeos.local:3000` - HTTP endpoint (FALLBACK only)

**TRUE PRIMAL Behavior**:
- If set: Uses this URL directly
- If not set: Discovers BiomeOS via socket scanning at runtime
- Graceful degradation: Falls back to fixture data if no BiomeOS found

**Production**: Set to Unix socket for fast IPC (JSON-RPC 2.0 required for cross-primal IPC; tarpc may be used for Rust-to-Rust hot paths where supported).  
**Development**: Can omit to test runtime discovery.

---

### **BIOMEOS_NEURAL_API_SOCKET**
**Type**: String (absolute path to Unix socket)  
**Default**: None (runtime discovery under XDG runtime dir or legacy `/tmp`)  
**Required**: No  
**Example**: `BIOMEOS_NEURAL_API_SOCKET=/run/user/1000/biomeos/neural-api.sock`

Explicit path to the biomeOS Neural API Unix socket. Highest-priority override when resolving the neural API client; if unset or missing, petalTongue falls back to standard discovery paths.

---

### **PETALTONGUE_FIXTURE_MODE**
**Type**: Boolean (`true` | `false`)  
**Default**: `false`  
**Required**: No  
**Example**: `PETALTONGUE_FIXTURE_MODE=true`

**⚠️ DEVELOPMENT ONLY** — Enable fixture mode to use deterministic data when
biomeOS is unavailable. Requires the `test-fixtures` feature; production builds
reject this at runtime.

> **Migration note**: The config field was renamed from `mock_mode` to
> `fixture_mode` in v1.6.6. The TOML alias `mock_mode` is still accepted for
> backwards compatibility but will be removed in a future release.

**When `true`** (with `test-fixtures` feature):
- Uses built-in deterministic fixture data instead of connecting to biomeOS
- Useful for development without running the full biomeOS stack
- Provides realistic test data for UI development

**When `true`** (without `test-fixtures` feature):
- Returns `FixtureModeUnavailable` error at runtime

**When `false`** (production):
- Connects to real biomeOS at `BIOMEOS_URL`
- Recommended for all non-development environments

**Security Note**: Never set to `true` in production deployments.

---

### **PETALTONGUE_DISCOVERY_HINTS**
**Type**: String (comma-separated list)  
**Default**: None  
**Required**: No  
**Example**: `PETALTONGUE_DISCOVERY_HINTS=unix:///tmp/custom.sock,http://fallback:3000`

Comma-separated list of discovery hints for finding BiomeOS and other primals.

**Format**:
- Unix sockets: `unix:///path/to/socket` or just `/path/to/socket`
- HTTP endpoints: `http://hostname:port`

**Priority**:
1. Standard Unix socket paths (auto-discovered)
2. Discovery hints (if set)
3. Environment variables (`BIOMEOS_URL`)

---

### **PETALTONGUE_ENABLE_MDNS**
**Type**: Boolean (`true` | `false`)  
**Default**: `true`  
**Required**: No  
**Example**: `PETALTONGUE_ENABLE_MDNS=false`

When `true`, HTTP/mDNS-based visualization provider discovery runs after JSON-RPC discovery. Set to `false` to skip mDNS (e.g., air-gapped or CI environments).

---

### **PETALTONGUE_DISCOVERY_PORTS**
**Type**: String (comma-separated port numbers)  
**Default**: `8080,8081,3000,9000`  
**Required**: No  
**Example**: `PETALTONGUE_DISCOVERY_PORTS=8080,8081,9000,9001`

Ports to probe during HTTP-based capability discovery. Also accepts `DISCOVERY_PORTS` as alias.

---

### **PETALTONGUE_DISCOVERY_BASE**
**Type**: String (URL base)  
**Default**: `http://localhost`  
**Required**: No  
**Example**: `PETALTONGUE_DISCOVERY_BASE=http://192.0.2.10`

Base URL for HTTP probing; ports from `PETALTONGUE_DISCOVERY_PORTS` are appended.

---

### **DISCOVERY_SERVICE_SOCKET**
**Type**: String (socket basename)  
**Default**: `discovery-service`  
**Required**: No  
**Example**: `DISCOVERY_SERVICE_SOCKET=my-discovery-registry`

Basename used to resolve the Unix socket for the ecosystem discovery/registry service
(`{basename}-{FAMILY_ID}.sock` under `XDG_RUNTIME_DIR`, `/run/user/<uid>`, and `/tmp`).
Override when your deployment installs the registry under a non-default name.

---

### **COMPUTE_SOCKET**
**Type**: String (absolute path to Unix socket)  
**Default**: None (runtime scan)  
**Required**: No  
**Example**: `COMPUTE_SOCKET=/run/user/1000/ecoPrimals/physics-compute.sock`

Explicit path to the compute-primal JSON-RPC socket used by the physics/compute bridge.
When unset, the bridge scans standard ecosystem paths (see `PHYSICS_COMPUTE_SOCKET_NAME`).

---

### **PHYSICS_COMPUTE_SOCKET_NAME**
**Type**: String  
**Default**: `physics-compute`  
**Required**: No  
**Example**: `PHYSICS_COMPUTE_SOCKET_NAME=physics-gpu`

Socket basename used when searching for a compute primal (only when `COMPUTE_SOCKET` is unset).

---

### **AUDIO_PROVIDER_URL**
**Type**: String (URL)  
**Default**: None  
**Required**: No  
**Example**: `AUDIO_PROVIDER_URL=http://127.0.0.1:8090`

HTTP endpoint for remote audio synthesis when using the capability-discovered audio provider.
The UI prefers this variable for explicit configuration.

---

### **DISPLAY_BACKEND_PORT**
**Type**: Integer (port number)  
**Default**: `9001`  
**Required**: No  
**Example**: `DISPLAY_BACKEND_PORT=9100`

Port used for local display-backend / tarpc defaults when discovery falls back to loopback
(see `petal_tongue_core::constants::display_backend_port()`).

---

### **ENTROPY_SOURCE_ENDPOINT**
**Type**: String (URL)  
**Default**: None  
**Required**: No  
**Example**: `ENTROPY_SOURCE_ENDPOINT=http://127.0.0.1:8443`

HTTP base URL for streaming human-entropy payloads to a primal that advertises entropy ingestion.
Used by the human entropy UI when capability discovery does not yield an endpoint.

---

### **PETALTONGUE_GPU_COMPUTE_ENDPOINT**
**Type**: String (URL)
**Default**: `http://localhost:8090`
**Required**: No
**Example**: `PETALTONGUE_GPU_COMPUTE_ENDPOINT=http://compute.local:8090`

Endpoint for GPU compute offload. Used by the compute bridge for
statistics, tessellation, projection, and physics operations.

---

### **PETALTONGUE_HEADLESS_ENDPOINT**
**Type**: String (URL)  
**Default**: `http://localhost:9000`  
**Required**: No  
**Example**: `PETALTONGUE_HEADLESS_ENDPOINT=http://discovery:9000`

Demo/topology endpoint for headless mode (e.g., discovery service URL).

---

## Self-Awareness & AI Integration

### **PETALTONGUE_STATUS_FILE**
**Type**: String (file path)  
**Default**: `/tmp/petaltongue_status.json`  
**Required**: No  
**Example**: `PETALTONGUE_STATUS_FILE=/var/run/petaltongue/status.json`

Path to write machine-readable status file for AI systems.

**Format**: JSON with system state, health, events, and issues.

**Use Cases**:
- External AI agents monitoring petalTongue
- Integration tests checking system state
- DevOps dashboards
- Automated diagnosis tools

---

### **PETALTONGUE_SOUNDS_DIR**
**Type**: String (directory path)  
**Default**: `./sounds` (current directory)  
**Required**: No (falls back to generated sounds)  
**Example**: `PETALTONGUE_SOUNDS_DIR=/usr/share/petaltongue/sounds`

Directory containing user-supplied sound files for audio system.

**Supported Files**:
- `startup.mp3` - Startup anthem
- `success.mp3` - Success notification
- `error.mp3` - Error notification
- `click.mp3` - UI click sound
- (etc. - see audio system docs)

**Fallback**: If directory doesn't exist or files are missing, petalTongue generates pure Rust audio.

---

## Logging & Debugging

### **RUST_LOG**
**Type**: Log Level (`error` | `warn` | `info` | `debug` | `trace`)  
**Default**: `info`  
**Required**: No  
**Example**: `RUST_LOG=debug`

Controls Rust logging verbosity across all crates.

**Levels**:
- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information (recommended)
- `debug`: Detailed debugging information
- `trace`: Very verbose tracing

---

## UI Configuration

### **PETALTONGUE_UI_BACKEND**
**Type**: String  
**Default**: None (backend selection uses defaults elsewhere)  
**Required**: No  
**Example**: `PETALTONGUE_UI_BACKEND=auto`

Selects the UI backend when set. Valid values include `auto`, `eframe`, `egui`, `compute.provider`, `pure-rust`, and `discovered` (see `petal_tongue_ui::backend::backend_from_env` / `BackendChoice`).

---

### **PETALTONGUE_WINDOW_WIDTH**
**Type**: Integer (pixels)  
**Default**: Auto (from egui)  
**Required**: No  
**Example**: `PETALTONGUE_WINDOW_WIDTH=1920`

Initial window width in pixels.

---

### **PETALTONGUE_WINDOW_HEIGHT**
**Type**: Integer (pixels)  
**Default**: Auto (from egui)  
**Required**: No  
**Example**: `PETALTONGUE_WINDOW_HEIGHT=1080`

Initial window height in pixels.

---

## Tuning & Timing

### **PETALTONGUE_RPC_TIMEOUT_SECS**
**Type**: Integer (seconds)  
**Default**: `5`  
**Required**: No  
**Example**: `PETALTONGUE_RPC_TIMEOUT_SECS=10`

Timeout for RPC clients (JSON-RPC 2.0 cross-primal paths; tarpc may apply on Rust-to-Rust hot paths where used).

---

### **PETALTONGUE_HEARTBEAT_INTERVAL_SECS**
**Type**: Integer (seconds)  
**Default**: `30`  
**Required**: No  
**Example**: `PETALTONGUE_HEARTBEAT_INTERVAL_SECS=60`

Interval between Neural API lifecycle heartbeats.

---

### **PETALTONGUE_REFRESH_INTERVAL_SECS**
**Type**: Integer (seconds)  
**Default**: `2`  
**Required**: No  
**Example**: `PETALTONGUE_REFRESH_INTERVAL_SECS=5`

biomeOS UI manager refresh interval for data updates.

---

### **PETALTONGUE_DISCOVERY_TIMEOUT_SECS**
**Type**: Integer (seconds)  
**Default**: `5`  
**Required**: No  
**Example**: `PETALTONGUE_DISCOVERY_TIMEOUT_SECS=10`

Timeout for mDNS and capability discovery operations.

---

### **PETALTONGUE_TUI_TICK_MS**
**Type**: Integer (milliseconds)  
**Default**: `100`  
**Required**: No  
**Example**: `PETALTONGUE_TUI_TICK_MS=50`

Terminal UI event loop tick rate.

---

### **PETALTONGUE_TELEMETRY_BUFFER**
**Type**: Integer (event count)  
**Default**: `10000`  
**Required**: No  
**Example**: `PETALTONGUE_TELEMETRY_BUFFER=50000`

Maximum telemetry event buffer size before oldest events are dropped.

---

### **PETALTONGUE_RETRY_INITIAL_MS**
**Type**: Integer (milliseconds)  
**Default**: `100`  
**Required**: No  
**Example**: `PETALTONGUE_RETRY_INITIAL_MS=200`

Initial delay for exponential backoff retry on failed connections.

---

### **PETALTONGUE_RETRY_MAX_SECS**
**Type**: Integer (seconds)  
**Default**: `10`  
**Required**: No  
**Example**: `PETALTONGUE_RETRY_MAX_SECS=30`

Maximum delay cap for exponential backoff retry.

---

## Security & Privacy

### **PETALTONGUE_NO_TELEMETRY**
**Type**: Boolean (`true` | `false`)  
**Default**: `false`  
**Required**: No  
**Example**: `PETALTONGUE_NO_TELEMETRY=true`

Disable telemetry collection (if/when implemented).

**Note**: Currently petalTongue does not collect any telemetry. This variable is reserved for future use.

---

### **DISCOVERY_PORTS**
**Type**: String (comma-separated port numbers)  
**Default**: `8080,8081,3000,9000` (or see `PETALTONGUE_DISCOVERY_PORTS`)  
**Required**: No  
**Example**: `DISCOVERY_PORTS=8080,8081,9000,9001`

Alias for `PETALTONGUE_DISCOVERY_PORTS`. Ports to probe during HTTP-based capability discovery.

---

## Quick Reference

### Minimal Production Configuration
```bash
# ZERO required variables! Pure runtime discovery works.
# Optionally set for faster startup:
BIOMEOS_URL=http://your-biomeos-instance:3000
```

### Development Configuration
```bash
# Development with fixture mode (deterministic offline data)
BIOMEOS_URL=http://localhost:3000
PETALTONGUE_FIXTURE_MODE=true
RUST_LOG=debug
PETALTONGUE_STATUS_FILE=/tmp/petaltongue_dev_status.json
```

### High-Performance Configuration
```bash
# Fast updates, smooth animations
BIOMEOS_URL=http://biomeos.local:3000
PETALTONGUE_REFRESH_INTERVAL_SECS=1
RUST_LOG=warn
```

### Low-Resource Configuration
```bash
# Slower updates, lower CPU usage
BIOMEOS_URL=http://biomeos.local:3000
PETALTONGUE_REFRESH_INTERVAL_SECS=10
RUST_LOG=error
```

---

## Telemetry & Data

### **PETALTONGUE_TELEMETRY_DIR**
**Type**: String (directory path)  
**Default**: Falls back to `$XDG_DATA_HOME/petaltongue/telemetry/` then `/tmp/petaltongue-telemetry/`  
**Required**: No  
**Example**: `PETALTONGUE_TELEMETRY_DIR=/var/lib/petaltongue/telemetry`

Directory containing JSONL telemetry files for the file-based provider
(hotSpring, groundSpring). Reads `{t, section, ...fields}` JSONL from
all `.jsonl` files in this directory.

---

## Capability-Based Design

**Key Principle**: petalTongue never hardcodes assumptions about primals or external services.

**Configuration Philosophy**:
- **Environment-driven**: All configuration via environment variables
- **Runtime discovery**: Primals discovered dynamically, not hardcoded
- **Honest capabilities**: System knows what it can actually do
- **Graceful degradation**: Missing services don't crash the application

---

## Deployment Checklist

Before deploying to production:

- [ ] Set `BIOMEOS_URL` to production endpoint
- [ ] Ensure `PETALTONGUE_FIXTURE_MODE` is `false` (or unset)
- [ ] Set `RUST_LOG` to `info` or `warn`
- [ ] Configure `PETALTONGUE_STATUS_FILE` for monitoring
- [ ] Set `PETALTONGUE_SOUNDS_DIR` if using custom sounds
- [ ] Configure refresh interval based on load requirements
- [ ] Test with actual BiomeOS instance
- [ ] Verify capability detection works correctly
- [ ] Confirm no hardcoded development values
- [ ] Test AI monitoring tools can read status file

---

## Additional Resources

- [README.md](README.md) - Project overview
- [START_HERE.md](START_HERE.md) - Getting started guide

---

**Last Updated**: April 21, 2026  
**Maintainer**: ecoPrimals Project  
**License**: AGPL-3.0-or-later

