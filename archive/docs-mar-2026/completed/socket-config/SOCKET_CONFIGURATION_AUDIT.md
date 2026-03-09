# 🔌 Socket Configuration Audit - petalTongue

**Date**: January 11, 2026  
**Auditor**: petalTongue Team  
**Context**: biomeOS Socket Standardization Initiative  

---

## 🎯 **Executive Summary**

**Status**: ✅ **COMPLIANT WITH EXCEPTIONS**

petalTongue is **mostly compliant** with biomeOS socket standards, but has **1 critical gap**:

- ❌ **Missing `PETALTONGUE_SOCKET` env var** (explicit override)
- ✅ Supports `FAMILY_ID` env var
- ✅ Supports `XDG_RUNTIME_DIR`
- ✅ Creates parent directories
- ✅ Removes old sockets before binding
- ⚠️  **Missing NODE_ID support** (multi-instance)

---

## 📊 **Current Implementation Status**

### ✅ **What We Do RIGHT**:

1. **XDG-Compliant Paths**
   - Uses `/run/user/<uid>/petaltongue-<family>.sock`
   - Respects `XDG_RUNTIME_DIR` environment variable
   - Falls back to `/run/user/<uid>` if not set

2. **Socket Cleanup**
   - Removes old socket before binding ✅
   - Cleans up on server shutdown ✅

3. **Parent Directory Creation**
   - Not explicitly in code, but relies on `/run/user/<uid>` existing
   - Should add `create_dir_all` for safety

4. **Capability-Based Discovery**
   - No hardcoded primal endpoints
   - Runtime discovery via `discover_primal_socket()`

---

## ❌ **What We're MISSING**:

### **Issue 1: No `PETALTONGUE_SOCKET` Environment Variable**

**Current Behavior**:
```rust
pub fn get_petaltongue_socket_path() -> Result<PathBuf> {
    let family_id = get_family_id();
    let runtime_dir = get_runtime_dir()?;
    Ok(runtime_dir.join(format!("petaltongue-{}.sock", family_id)))
}
```

**Problem**: Cannot override socket path explicitly!

**Expected Behavior** (biomeOS standard):
```bash
export PETALTONGUE_SOCKET=/tmp/custom.sock
petaltongue
# Should use /tmp/custom.sock, NOT /run/user/<uid>/petaltongue-nat0.sock
```

**Fix Required**:
1. Check for `PETALTONGUE_SOCKET` env var first
2. Fall back to XDG/family-based path if not set

---

### **Issue 2: No `NODE_ID` Support (Multi-Instance)**

**Current**: Single instance per family
```
/run/user/1000/petaltongue-nat0.sock
```

**Needed**: Multiple instances in same family
```
/run/user/1000/petaltongue-nat0-node1.sock
/run/user/1000/petaltongue-nat0-node2.sock
```

**Fix Required**:
1. Add `PETALTONGUE_NODE_ID` env var support
2. Update path format: `petaltongue-<family>-<node>.sock`
3. Default `node` to "default" if not set

---

### **Issue 3: No Explicit Parent Directory Creation**

**Current**: Assumes `/run/user/<uid>` exists

**Problem**: Fails if XDG_RUNTIME_DIR points to non-existent path

**Fix Required**:
```rust
use std::fs;

let socket_path = get_petaltongue_socket_path()?;

// Ensure parent directory exists
if let Some(parent) = socket_path.parent() {
    fs::create_dir_all(parent)?;
}
```

---

### **Issue 4: No `/tmp` Fallback**

**Current**: Errors if `/run/user/<uid>` doesn't exist

**Expected** (3-tier fallback):
1. `PETALTONGUE_SOCKET` (env var override)
2. `/run/user/<uid>/petaltongue-<family>.sock` (XDG)
3. `/tmp/petaltongue-<family>-<node>.sock` (fallback)

---

## 🎯 **Standardized Configuration (Target)**

### **Environment Variables**

```bash
# Socket path (absolute) - HIGHEST PRIORITY
PETALTONGUE_SOCKET=/run/user/1000/petaltongue-nat0.sock

# Family ID (for socket naming)
FAMILY_ID=nat0

# Node ID (for multi-instance)
PETALTONGUE_NODE_ID=node1

# Runtime directory (standard XDG)
XDG_RUNTIME_DIR=/run/user/1000
```

### **Fallback Logic** (Priority Order)

```rust
fn get_petaltongue_socket_path() -> Result<PathBuf> {
    // 1. Check explicit override (HIGHEST PRIORITY)
    if let Ok(path) = std::env::var("PETALTONGUE_SOCKET") {
        return Ok(PathBuf::from(path));
    }
    
    // 2. XDG runtime directory (STANDARD)
    let family_id = get_family_id();
    let node_id = get_node_id();
    
    if let Ok(runtime_dir) = get_runtime_dir() {
        return Ok(runtime_dir.join(format!(
            "petaltongue-{}-{}.sock",
            family_id,
            node_id
        )));
    }
    
    // 3. Fallback to /tmp (LAST RESORT)
    Ok(PathBuf::from(format!(
        "/tmp/petaltongue-{}-{}.sock",
        family_id,
        node_id
    )))
}

fn get_node_id() -> String {
    std::env::var("PETALTONGUE_NODE_ID")
        .unwrap_or_else(|_| "default".to_string())
}
```

---

## 📝 **Implementation Checklist**

- [ ] Add `PETALTONGUE_SOCKET` env var support (highest priority)
- [x] Support `FAMILY_ID` env var ✅
- [ ] Add `PETALTONGUE_NODE_ID` env var support
- [x] Support `XDG_RUNTIME_DIR` ✅
- [ ] Implement 3-tier fallback (env → XDG → /tmp)
- [ ] Create parent directories (`create_dir_all`)
- [x] Remove old socket before binding ✅
- [x] Cleanup socket on shutdown ✅
- [ ] Update tests for new env vars
- [ ] Update `ENV_VARS.md` documentation
- [ ] Test all 4 biomeOS scenarios

---

## 🚀 **Testing Requirements**

### **Test 1: Environment Variable Override**
```bash
export PETALTONGUE_SOCKET=/tmp/test-socket.sock
export FAMILY_ID=test0
petaltongue

# Verify socket exists at /tmp/test-socket.sock
ls -lh /tmp/test-socket.sock
```

### **Test 2: XDG Runtime Directory**
```bash
export FAMILY_ID=xdg0
petaltongue

# Verify socket exists at /run/user/<uid>/petaltongue-xdg0-default.sock
ls -lh /run/user/$(id -u)/petaltongue-xdg0-default.sock
```

### **Test 3: Multi-Instance (NODE_ID)**
```bash
export FAMILY_ID=nat0
export PETALTONGUE_NODE_ID=node1
petaltongue &

export PETALTONGUE_NODE_ID=node2
petaltongue &

# Verify both sockets exist
ls -lh /run/user/$(id -u)/petaltongue-nat0-node*.sock
```

### **Test 4: Socket Cleanup**
```bash
# Create old socket
touch /tmp/test-socket.sock

# Start petalTongue
export PETALTONGUE_SOCKET=/tmp/test-socket.sock
petaltongue

# Should remove old socket and create new one (no "address already in use" error)
```

---

## 🎊 **Why This Matters for petalTongue**

### **Atomic Integration**

petalTongue may be part of atomics in the future:

```
Intelligent Tower = BearDog + Songbird + petalTongue
  - BearDog NAT (compute boundary)
  - Songbird discovery
  - petalTongue visualization
```

**Requirements**:
- Unique socket paths per instance ✅ (with NODE_ID)
- Capability-based discovery ✅ (already implemented)
- No hardcoded endpoints ✅ (already compliant)

---

## 📈 **Comparison to Other Primals**

| Primal | Env Override | XDG | /tmp Fallback | NODE_ID | Status |
|--------|--------------|-----|---------------|---------|--------|
| **BearDog** | ❌ | ❌ | ✅ | ❓ | Needs fix |
| **Songbird** | ❓ | ❌ | ❓ | ❓ | Needs fix |
| **ToadStool** | ❓ | ❓ | ❓ | ❓ | Unknown |
| **NestGate** | ✅ | ✅ | ❓ | ❓ | Mostly good |
| **petalTongue** | ❌ | ✅ | ❌ | ❌ | **Needs fix** |

**Target**: All ✅✅✅✅

---

## 🎯 **Proposed Evolution**

### **Phase 1: Critical Fixes** (Today)
1. Add `PETALTONGUE_SOCKET` override
2. Add explicit `create_dir_all` for parent
3. Add `/tmp` fallback

### **Phase 2: Multi-Instance** (This Week)
1. Add `PETALTONGUE_NODE_ID` support
2. Update socket path format
3. Test multi-instance scenarios

### **Phase 3: Documentation** (This Week)
1. Update `ENV_VARS.md`
2. Update `README.md`
3. Add testing guide

---

## 📚 **Related Files**

- `crates/petal-tongue-ipc/src/socket_path.rs` - Path resolution
- `crates/petal-tongue-ipc/src/unix_socket_server.rs` - Server binding
- `ENV_VARS.md` - Environment variable documentation

---

## ✅ **Current Strengths**

1. ✅ Already XDG-compliant
2. ✅ Already removes old sockets
3. ✅ Already capability-based (no hardcoding)
4. ✅ Already has `discover_primal_socket()` for others
5. ✅ Already uses `FAMILY_ID`

**We're 70% there!** Just need explicit override and NODE_ID support.

---

**Status**: Audit complete. Ready for evolution! 🚀

