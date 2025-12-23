# 00 - Setup & Verification

**Duration**: 5 minutes  
**Purpose**: Verify environment is ready for fermentation testing

---

## 🎯 What This Demo Does

1. **Checks prerequisites** (BiomeOS built, petalTongue built, primals available)
2. **Launches BiomeOS** in background
3. **Verifies BiomeOS** is running and healthy
4. **Launches petalTongue** pointing at BiomeOS
5. **Validates connection** between petalTongue and BiomeOS

**Goal**: Ensure everything is ready before building more complex scenarios.

---

## 🚀 Quick Start

```bash
./demo.sh
```

This will run the full setup verification automatically.

---

## 📋 Manual Steps

If you want to run each step manually:

### Step 1: Check Prerequisites

```bash
./check-prerequisites.sh
```

**Expected output**:
```
✅ BiomeOS binary exists
✅ petalTongue binary exists
✅ Primals found in bin/: beardog, nestgate, songbird-*, toadstool-*
✅ All prerequisites met!
```

### Step 2: Launch BiomeOS

```bash
./launch-biomeos.sh
```

**Expected output**:
```
🚀 Launching BiomeOS...
✅ BiomeOS started on http://localhost:3000
```

**Terminal**: BiomeOS will run in background (see terminals/biomeos.log)

### Step 3: Verify BiomeOS

```bash
./verify-biomeos.sh
```

**Expected output**:
```
✅ BiomeOS responding on http://localhost:3000
✅ Health check: OK
✅ Primal discovery active
```

### Step 4: Launch petalTongue

```bash
./launch-petaltongue.sh
```

**Expected output**:
```
🌸 Launching petalTongue...
✅ petalTongue UI opening...
✅ Connected to BiomeOS at http://localhost:3000
```

**Window**: petalTongue UI should open showing BiomeOS topology

### Step 5: Validate Connection

```bash
./validate-connection.sh
```

**Expected output**:
```
✅ petalTongue receiving data from BiomeOS
✅ Primals visible: X nodes
✅ Topology edges visible: Y connections
✅ Auto-refresh working
```

---

## ✅ Success Criteria

After running this demo, you should have:

- [x] BiomeOS running on http://localhost:3000
- [x] petalTongue UI open and responsive
- [x] petalTongue connected to BiomeOS
- [x] Can see primals (if any are running)
- [x] No errors in logs

---

## 🔧 Troubleshooting

### BiomeOS won't start

**Problem**: `./launch-biomeos.sh` fails  
**Solutions**:
1. Check if port 3000 is already in use: `lsof -i :3000`
2. Rebuild BiomeOS: `cd ../../../biomeOS && cargo build --release`
3. Check logs: `tail -f terminals/biomeos.log`

### petalTongue won't connect

**Problem**: petalTongue shows "Failed to connect to BiomeOS"  
**Solutions**:
1. Verify BiomeOS is running: `curl http://localhost:3000/health`
2. Check firewall isn't blocking localhost:3000
3. Try manual launch: `BIOMEOS_URL=http://localhost:3000 cargo run --release -p petal-tongue-ui`

### No primals visible

**Problem**: petalTongue shows empty graph  
**Solutions**:
1. This is expected if no primals are running yet
2. Try launching a primal: `../../biomeOS/bin/primals/beardog &`
3. Wait for auto-refresh (5s) or click "Refresh Now"

---

## 📊 What You Should See

### BiomeOS Terminal Output
```
[INFO] BiomeOS v0.1.0 starting
[INFO] Listening on http://localhost:3000
[INFO] Primal discovery active
[INFO] Health endpoint: /health
```

### petalTongue UI
- **Window Title**: "🌸 petalTongue - Universal Representation System"
- **Left Panel**: Controls (may be collapsed)
- **Center**: Empty graph (no primals yet) OR primals if already running
- **Right Panel**: Audio info
- **Top Bar**: Layout selector, refresh controls
- **Statistics**: "0 nodes, 0 edges" (if no primals) or actual counts

---

## 🌱 Fermentation Notes

### Gaps Discovered

*(Document any issues found during this demo)*

- **Gap ID**: TBD
- **Description**: TBD
- **Impact**: TBD

### What Worked Well

*(Document what went smoothly)*

- TBD

### Improvements Needed

*(Document what could be better)*

- TBD

---

## ⏭️ Next Steps

Once this demo passes, proceed to:

```bash
cd ../01-single-primal/
cat README.md
```

This will show you how to visualize a single primal.

---

**Status**: 🌱 Ready to build  
**Complexity**: Low  
**Dependencies**: None (this is the foundation)

---

*This is the first step in fermentation. Let's grow petalTongue! 🌸*

