# Environment Variables Reference

This document describes all environment variables used by petalTongue.

## Socket Configuration (biomeOS Standard)

### **PETALTONGUE_SOCKET**
**Type**: String (absolute path)  
**Default**: None (uses XDG runtime or /tmp fallback)  
**Required**: No  
**Example**: `PETALTONGUE_SOCKET=/run/user/1000/petaltongue-nat0-node1.sock`

**HIGHEST PRIORITY** socket path override. When set, petalTongue will use this exact socket path.

**biomeOS Socket Standard**:
- Priority 1: `PETALTONGUE_SOCKET` (explicit override)
- Priority 2: `/run/user/<uid>/petaltongue/petaltongue-<family>-<node>.sock` (XDG)
- Priority 3: `/tmp/petaltongue-<family>-<node>.sock` (fallback)

**Use Cases**:
- Atomic deployments with custom socket locations
- Testing with specific socket paths
- Multi-instance deployments with explicit coordination

---

### **FAMILY_ID**
**Type**: String  
**Default**: `nat0`  
**Required**: No  
**Example**: `FAMILY_ID=staging`

Family identifier for this petalTongue instance. Used in socket path construction.

**Socket Path Impact**:
- `FAMILY_ID=nat0` → `/run/user/1000/petaltongue/petaltongue-nat0-default.sock`
- `FAMILY_ID=staging` → `/run/user/1000/petaltongue/petaltongue-staging-default.sock`

**Atomic Architecture**:
Multiple families can run on the same machine without conflict.

---

### **PETALTONGUE_NODE_ID**
**Type**: String  
**Default**: `default`  
**Required**: No  
**Example**: `PETALTONGUE_NODE_ID=node1`

Node identifier for multi-instance deployments. Enables multiple petalTongue instances in the same family.

**Socket Path Impact**:
- `NODE_ID=default` → `/run/user/1000/petaltongue/petaltongue-nat0-default.sock`
- `NODE_ID=node1` → `/run/user/1000/petaltongue/petaltongue-nat0-node1.sock`
- `NODE_ID=node2` → `/run/user/1000/petaltongue/petaltongue-nat0-node2.sock`

**Use Cases**:
- Running multiple visualization instances
- Load balancing across instances
- A/B testing different configurations

---

### **XDG_RUNTIME_DIR**
**Type**: String (directory path)  
**Default**: `/run/user/<uid>` (auto-detected)  
**Required**: No  
**Example**: `XDG_RUNTIME_DIR=/run/user/1000`

Standard XDG runtime directory for socket placement. This is the standard Unix location for user-level runtime files.

**TRUE PRIMAL Principle**: Uses standard Unix conventions rather than hardcoded paths.

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
- Graceful degradation: Falls back to mock mode if no BiomeOS found

**Production**: Set to Unix socket for fast, secure JSON-RPC.  
**Development**: Can omit to test runtime discovery.

---

### **PETALTONGUE_MOCK_MODE**
**Type**: Boolean (`true` | `false`)  
**Default**: `false`  
**Required**: No  
**Example**: `PETALTONGUE_MOCK_MODE=true`

**⚠️ DEVELOPMENT ONLY** - Enable mock mode to bypass BiomeOS connection.

**When `true`**:
- Uses built-in mock data instead of connecting to BiomeOS
- Useful for development without running full BiomeOS stack
- Provides realistic test data for UI development

**When `false`** (production):
- Connects to real BiomeOS at `BIOMEOS_URL`
- Falls back to mock data only if connection fails
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
**Type**: String (socket name or path)  
**Default**: `discovery-service`  
**Required**: No  
**Example**: `DISCOVERY_SERVICE_SOCKET=songbird` (for Songbird deployments)

Capability-based discovery service socket name. Set to `songbird` when using Songbird as discovery provider.

---

### **SONGBIRD_SOCKET_FALLBACK**
**Type**: String (absolute path)  
**Default**: `/tmp/<discovery-service>-nat0-default.sock`  
**Required**: No  
**Example**: `SONGBIRD_SOCKET_FALLBACK=/run/user/1000/songbird-nat0.sock`

Fallback socket path when discovery service is not found in standard locations.

---

### **BARRACUDA_SOCKET**
**Type**: String (absolute path)  
**Default**: None (runtime scan)  
**Required**: No  
**Example**: `BARRACUDA_SOCKET=/run/user/1000/barracuda.sock`

Explicit path to physics/GPU compute primal socket. Overrides runtime scanning.

---

### **PHYSICS_COMPUTE_SOCKET_NAME**
**Type**: String  
**Default**: `barracuda`  
**Required**: No  
**Example**: `PHYSICS_COMPUTE_SOCKET_NAME=physics-gpu`

Socket name for physics compute capability discovery (used when `BARRACUDA_SOCKET` not set).

---

### **PETALTONGUE_HEADLESS_ENDPOINT**
**Type**: String (URL)  
**Default**: `http://localhost:9000`  
**Required**: No  
**Example**: `PETALTONGUE_HEADLESS_ENDPOINT=http://discovery:9000`

Demo/topology endpoint for headless mode (e.g., discovery service URL).

---

### **PETALTONGUE_TUTORIAL_ENDPOINT_COMPUTE**
### **PETALTONGUE_TUTORIAL_ENDPOINT_SECURITY**
### **PETALTONGUE_TUTORIAL_ENDPOINT_DISCOVERY**
**Type**: String (URL)  
**Default**: `http://localhost:3030`, `http://localhost:8001`, `http://localhost:8003`  
**Required**: No  

Tutorial mode placeholder endpoints for compute, security, and discovery primals.

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

### **RUST_LOG_FILTER**
**Type**: String (filter expression)  
**Default**: None  
**Required**: No  
**Example**: `RUST_LOG_FILTER=petal_tongue_ui=trace,petal_tongue_core=debug`

Advanced logging filter for fine-grained control.

**Syntax**: `crate1=level1,crate2=level2`

**Examples**:
```bash
# Trace UI, debug core, info for everything else
RUST_LOG_FILTER=petal_tongue_ui=trace,petal_tongue_core=debug

# Debug all petalTongue crates
RUST_LOG_FILTER=petal_tongue=debug

# Trace specific module
RUST_LOG_FILTER=petal_tongue_graph::visual_2d=trace
```

---

## UI Configuration

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

### **PETALTONGUE_AUDIO_ENABLED**
**Type**: Boolean (`true` | `false`)  
**Default**: Auto-detected at runtime  
**Required**: No  
**Example**: `PETALTONGUE_AUDIO_ENABLED=true`

Force audio rendering on/off. By default, petalTongue detects audio capabilities at runtime using its capability detection system.

---

## Performance Tuning

### **PETALTONGUE_REFRESH_INTERVAL**
**Type**: Float (seconds)  
**Default**: `5.0`  
**Required**: No  
**Example**: `PETALTONGUE_REFRESH_INTERVAL=2.0`

Auto-refresh interval for topology updates (in seconds). Superseded by `PETALTONGUE_REFRESH_INTERVAL_SECS` for the biomeOS UI manager (the old one controlled topology auto-refresh as float seconds; the new one controls UI manager refresh as integer seconds).

**Trade-offs**:
- Lower values (1-2s): More responsive, higher network/CPU usage
- Higher values (10-30s): Less responsive, lower resource usage

---

### **PETALTONGUE_MAX_FPS**
**Type**: Integer (frames per second)  
**Default**: `60`  
**Required**: No  
**Example**: `PETALTONGUE_MAX_FPS=30`

Maximum frame rate for rendering.

**Recommendations**:
- `60`: Smooth animations, high CPU usage
- `30`: Balanced performance
- `15-20`: Low-power mode

---

## Tuning & Timing

### **PETALTONGUE_RPC_TIMEOUT_SECS**
**Type**: Integer (seconds)  
**Default**: `5`  
**Required**: No  
**Example**: `PETALTONGUE_RPC_TIMEOUT_SECS=10`

Timeout for JSON-RPC and tarpc client requests.

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

## Tool Integration

### **BINGOCUBE_PATH**
**Type**: String (file path)  
**Default**: Auto-discovered via PATH  
**Required**: No  
**Example**: `BINGOCUBE_PATH=/usr/local/bin/bingocube`

Path to BingoCube adapter executable (if using external AI tools).

---

### **SYSTEM_MONITOR_INTERVAL**
**Type**: Float (seconds)  
**Default**: `2.0`  
**Required**: No  
**Example**: `SYSTEM_MONITOR_INTERVAL=1.0`

System monitor refresh interval (for resource tracking tool).

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

### **PETALTONGUE_DEBUG_OVERLAY**
**Type**: Boolean (`true` | `false`)  
**Default**: `false`  
**Required**: No  
**Example**: `PETALTONGUE_DEBUG_OVERLAY=true`

Show debug overlay with internal state (FPS, memory, etc.).

---

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
# Development with mock mode
BIOMEOS_URL=http://localhost:3000
PETALTONGUE_MOCK_MODE=true
RUST_LOG=debug
PETALTONGUE_DEBUG_OVERLAY=true
PETALTONGUE_STATUS_FILE=/tmp/petaltongue_dev_status.json
```

### High-Performance Configuration
```bash
# Fast updates, smooth animations
BIOMEOS_URL=http://biomeos.local:3000
PETALTONGUE_REFRESH_INTERVAL=1.0
PETALTONGUE_MAX_FPS=60
RUST_LOG=warn
```

### Low-Resource Configuration
```bash
# Slower updates, lower CPU usage
BIOMEOS_URL=http://biomeos.local:3000
PETALTONGUE_REFRESH_INTERVAL=10.0
PETALTONGUE_MAX_FPS=20
RUST_LOG=error
```

---

## Capability-Based Design

**Key Principle**: petalTongue never hardcodes assumptions about primals or external services.

**Configuration Philosophy**:
- **Environment-driven**: All configuration via environment variables
- **Runtime discovery**: Primals discovered dynamically, not hardcoded
- **Honest capabilities**: System knows what it can actually do
- **Graceful degradation**: Missing services don't crash the application

**Examples**:
- ✅ Good: Discovering primals via BiomeOS API
- ❌ Bad: Hardcoding primal names/endpoints in code
- ✅ Good: Testing audio capabilities at runtime
- ❌ Bad: Assuming audio always works

---

### Deployment Checklist

Before deploying to production:

- [ ] Set `BIOMEOS_URL` to production endpoint
- [ ] Ensure `PETALTONGUE_MOCK_MODE` is `false` (or unset)
- [ ] Set `RUST_LOG` to `info` or `warn`
- [ ] Unset `PETALTONGUE_DEBUG_OVERLAY` (or set to `false`)
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
- [docs/operations/QUICK_START.md](docs/operations/QUICK_START.md) - Quick start instructions

---

**Last Updated**: March 14, 2026  
**Maintainer**: ecoPrimals Project  
**License**: AGPL-3.0-only

