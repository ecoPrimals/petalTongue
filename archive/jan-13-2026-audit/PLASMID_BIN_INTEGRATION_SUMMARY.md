# plasmidBin Integration Summary

**Date**: January 13, 2026 PM  
**Purpose**: Quick reference for using stable binaries in testing  
**Compliance**: ✅ TRUE PRIMAL (runtime discovery only)

---

## 🎯 KEY POINT

The stable binaries in `../biomeOS/plasmidBin/` are **perfect for interaction testing** because:

1. ✅ **They discover at runtime** (no hardcoded paths)
2. ✅ **They respect primal self-knowledge** (each knows itself only)
3. ✅ **They create Unix sockets** for discovery
4. ✅ **They're production-ready** (tested, stable)

---

## 📍 LOCATION

```
/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/
├── beardog                 (3.5 MB) - Security primal
├── nestgate                (4.5 MB) - Storage primal  
├── petaltongue             (34 MB)  - Visualization (older build)
├── petaltongue-headless    (3.2 MB) - Headless mode
├── squirrel                (15 MB)  - AI collaboration
├── toadstool               (6.8 MB) - Compute primal
└── toadstool-cli           (22 MB)  - CLI interface
```

**Note**: Current petalTongue development is more recent than the plasmidBin version.

---

## ✅ HOW TO USE (TRUE PRIMAL PATTERN)

### Quick Test Script

```bash
# From petalTongue root
./test-with-plasmid-binaries.sh
```

**What it does**:
1. Starts stable primals from plasmidBin
2. Each primal creates its own socket
3. Runs current petalTongue (from source)
4. petalTongue discovers primals automatically
5. No hardcoded paths anywhere!

### Manual Testing

```bash
# Terminal 1: Start stable primals
cd ../biomeOS/plasmidBin/
./beardog --family-id test &
./toadstool --family-id test &
./nestgate --family-id test &

# Terminal 2: Run current petalTongue
cd ../../petalTongue
export FAMILY_ID=test
cargo run --bin petal-tongue-ui

# Expected: petalTongue discovers all running primals
```

---

## 🔍 DISCOVERY FLOW

```
1. Primal starts (e.g., beardog)
   └─→ Creates socket: /run/user/1000/beardog-test-default.sock

2. petalTongue starts
   └─→ Scans /run/user/1000/*.sock
   └─→ Finds: beardog-test-default.sock
   └─→ Connects via JSON-RPC
   └─→ Queries capabilities
   └─→ Discovers: "BearDog v0.15.0 (security, encryption)"

3. Visualization
   └─→ Shows discovered primals in topology
   └─→ Updates in real-time as primals come/go
```

**No hardcoded paths. No assumptions. Pure runtime discovery.** ✅

---

## 🚨 ANTI-PATTERNS TO AVOID

### ❌ DON'T: Hardcode Binary Paths
```rust
// WRONG - Violates primal self-knowledge
let beardog = Command::new("../biomeOS/plasmidBin/beardog");
```

### ✅ DO: Let Primals Announce Themselves
```rust
// CORRECT - Runtime discovery
let primals = discover_via_unix_sockets().await?;
```

### ❌ DON'T: Assume Binaries Exist
```rust
// WRONG - Assumes beardog exists
assert!(Path::new("../biomeOS/plasmidBin/beardog").exists());
```

### ✅ DO: Gracefully Handle Missing Primals
```rust
// CORRECT - Works with 0, 1, or N primals
match discover_beardog().await {
    Ok(beardog) => use_beardog(beardog),
    Err(_) => continue_without_beardog(),
}
```

---

## 📊 WHY THIS WORKS

### Primal Self-Knowledge

**BearDog knows**:
- ✅ "I am beardog"
- ✅ "I provide security capabilities"
- ✅ "My socket is /run/user/1000/beardog-test-default.sock"
- ❌ Where petalTongue is
- ❌ Where other primals are

**petalTongue knows**:
- ✅ "I am petalTongue"
- ✅ "I need visualization data"
- ✅ "I can discover primals via sockets"
- ❌ Which primals exist
- ❌ Where primals are located

**Result**: Each primal is sovereign, discovers others at runtime.

---

## 🎯 TESTING SCENARIOS

### Scenario 1: All Primals Available
```bash
# Start: beardog, toadstool, nestgate
./test-with-plasmid-binaries.sh

# Expected:
# ✅ petalTongue shows all 3 primals in topology
# ✅ Real-time capabilities displayed
# ✅ Health monitoring working
```

### Scenario 2: Partial Availability
```bash
# Start: only beardog
cd ../biomeOS/plasmidBin/
./beardog --family-id test &

cd ../../petalTongue
cargo run --bin petal-tongue-ui

# Expected:
# ✅ petalTongue shows beardog
# ✅ Gracefully handles missing toadstool/nestgate
# ✅ No crashes, clear UI feedback
```

### Scenario 3: Zero Primals (Standalone)
```bash
# Start: nothing
cargo run --bin petal-tongue-ui

# Expected:
# ✅ Falls back to mock mode
# ✅ Tutorial available
# ✅ Clear message: "No primals discovered"
```

---

## 🔒 TRUE PRIMAL COMPLIANCE

**Verified**:
- ✅ **Zero hardcoding** - No binary paths in code
- ✅ **Runtime discovery** - All primals found dynamically
- ✅ **Self-knowledge** - Each primal knows itself only
- ✅ **Graceful degradation** - Works with any subset
- ✅ **Capability-based** - Discovers by capability, not name

**From comprehensive audit**:
- ✅ Grade: A+ (100/100) on hardcoding
- ✅ Grade: A+ (100/100) on sovereignty
- ✅ Zero violations found

---

## 📚 REFERENCES

**Documentation**:
- `INTERACTION_TESTING_GUIDE.md` - Full testing guide
- `COMPREHENSIVE_AUDIT_JAN_13_2026_PM.md` - Audit results
- `ENV_VARS.md` - Environment variable hints

**Code**:
- `crates/petal-tongue-discovery/src/lib.rs` - Discovery chain
- `crates/petal-tongue-ipc/src/socket_path.rs` - Socket discovery
- `crates/petal-tongue-discovery/src/unix_socket_provider.rs` - Unix socket probing

**Cross-Primal**:
- `wateringHole/INTER_PRIMAL_INTERACTIONS.md` - Coordination patterns
- `../biomeOS/plasmidBin/MANIFEST.md` - Binary manifest

---

## ✅ SUMMARY

**Using plasmidBin Binaries**:
1. ✅ Start primals (they create sockets)
2. ✅ Run petalTongue (discovers via sockets)
3. ✅ No hardcoded paths anywhere
4. ✅ Perfect TRUE PRIMAL compliance

**Quick Start**:
```bash
./test-with-plasmid-binaries.sh
```

**Result**: Runtime discovery + primal self-knowledge = sovereignty! 🌸

---

**Status**: ✅ PRODUCTION PATTERN  
**Compliance**: 100% TRUE PRIMAL  
**Ready**: Immediate use for interaction testing

🌸 **plasmidBin: Stable Binaries, Runtime Discovery, Zero Hardcoding** 🌸

