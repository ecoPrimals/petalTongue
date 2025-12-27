# Environment Variables Reference

This document describes all environment variables used by petalTongue.

## 🔧 Configuration Variables

### **BIOMEOS_URL**
**Type**: String (URL)  
**Default**: `http://localhost:3000`  
**Required**: No  
**Example**: `BIOMEOS_URL=http://biomeos.local:3000`

URL of the BiomeOS API endpoint. petalTongue will connect to this endpoint to discover primals and retrieve topology data.

**Production**: Set to your actual BiomeOS instance.  
**Development**: Can use localhost or mock-biomeos server.

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

## 📊 Logging & Debugging

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

## 🎨 UI Configuration

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

**Note**: Audio requires ALSA libraries on Linux (`libasound2-dev`).

---

## ⚡ Performance Tuning

### **PETALTONGUE_REFRESH_INTERVAL**
**Type**: Float (seconds)  
**Default**: `5.0`  
**Required**: No  
**Example**: `PETALTONGUE_REFRESH_INTERVAL=2.0`

Auto-refresh interval for topology updates (in seconds).

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

## 🔌 Tool Integration

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

## 🔒 Security & Privacy

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

## 📋 Quick Reference

### Minimal Production Configuration
```bash
# Only one required variable (defaults work for everything else)
BIOMEOS_URL=http://your-biomeos-instance:3000
```

### Development Configuration
```bash
# Development with mock mode
BIOMEOS_URL=http://localhost:3000
PETALTONGUE_MOCK_MODE=true
RUST_LOG=debug
PETALTONGUE_DEBUG_OVERLAY=true
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

## 🎯 Capability-Based Design

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

## 🚀 Deployment Checklist

Before deploying to production:

- [ ] Set `BIOMEOS_URL` to production endpoint
- [ ] Ensure `PETALTONGUE_MOCK_MODE` is `false` (or unset)
- [ ] Set `RUST_LOG` to `info` or `warn`
- [ ] Unset `PETALTONGUE_DEBUG_OVERLAY` (or set to `false`)
- [ ] Configure refresh interval based on load requirements
- [ ] Test with actual BiomeOS instance
- [ ] Verify capability detection works correctly
- [ ] Confirm no hardcoded development values

---

## 📚 Additional Resources

- [README.md](README.md) - Project overview
- [START_HERE.md](START_HERE.md) - Getting started guide
- [docs/operations/QUICK_START.md](docs/operations/QUICK_START.md) - Quick start instructions
- [AUDIT_REPORTS_INDEX.md](AUDIT_REPORTS_INDEX.md) - Audit reports

---

**Last Updated**: December 27, 2025  
**Maintainer**: ecoPrimals Project  
**License**: AGPL-3.0

