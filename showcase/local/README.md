# 🌸 petalTongue Local Showcase

**Practical demonstrations using real primals and BiomeOS**

**Status**: 🌱 Fermentation Phase (Building)  
**Purpose**: Discover gaps, evolve implementation, validate architecture

---

## 🎯 Overview

This local showcase demonstrates petalTongue's capabilities through **practical, hands-on scenarios** using:
- **Real BiomeOS** orchestration
- **Actual primals** from `biomeOS/bin/`
- **Live topology** discovery
- **Production-like** interactions

**Not mocks. Not theory. Real systems.**

---

## 🚀 Quick Start

### Prerequisites

1. **BiomeOS built**:
   ```bash
   cd ../../biomeOS
   cargo build --release
   ```

2. **petalTongue built**:
   ```bash
   cd ../petalTongue
   cargo build --release
   ```

3. **Primals available** in `biomeOS/bin/primals/`

### Run First Demo

```bash
cd 00-setup/
./demo.sh
```

This will verify your environment is ready.

---

## 📚 Showcase Scenarios

### 00 - Setup & Verification
**Duration**: 5 minutes  
**Purpose**: Verify environment, launch BiomeOS, validate setup

```bash
cd 00-setup/
./demo.sh
```

**What you'll see**: BiomeOS running, primals discovered, petalTongue connecting

---

### 01 - Single Primal Visualization
**Duration**: 5 minutes  
**Purpose**: Learn to visualize one primal at a time

```bash
cd 01-single-primal/
./beardog-only.sh    # Just security primal
./nestgate-only.sh   # Just storage primal
```

**What you'll see**: Single node, health status, audio description

---

### 02 - Primal Discovery
**Duration**: 10 minutes  
**Purpose**: Watch real-time primal discovery

```bash
cd 02-primal-discovery/
./demo.sh
```

**What you'll see**: Primals appearing as they start, auto-refresh updates

---

### 03 - Topology Visualization
**Duration**: 10 minutes  
**Purpose**: Visualize full ecosystem topology

```bash
cd 03-topology-visualization/
./5-primal-mesh.sh      # Standard 5 primals
./10-primal-cluster.sh  # Larger topology
```

**What you'll see**: Full graph, layout algorithms, connections

---

### 04 - Health Monitoring
**Duration**: 10 minutes  
**Purpose**: Monitor health state changes

```bash
cd 04-health-monitoring/
./demo.sh
```

**What you'll see**: Health states (healthy/warning/critical), color changes, audio cues

---

### 05 - Accessibility Validation
**Duration**: 15 minutes  
**Purpose**: Test accessibility features

```bash
cd 05-accessibility-validation/
./screen-reader-test.sh
./audio-only-test.sh
```

**What you'll see**: Screen reader compatibility, audio descriptions, keyboard navigation

---

### 06 - Performance Benchmarking
**Duration**: 15 minutes  
**Purpose**: Measure performance at scale

```bash
cd 06-performance-benchmarking/
./benchmark-10-nodes.sh
./benchmark-50-nodes.sh
```

**What you'll see**: FPS metrics, memory usage, CPU load, responsiveness

---

### 07 - Real-World Scenarios
**Duration**: 20 minutes  
**Purpose**: Simulate production scenarios

```bash
cd 07-real-world-scenarios/
./ecosystem-startup.sh
./rolling-update.sh
./failure-cascade.sh
```

**What you'll see**: Cold starts, rolling updates, failure handling, recovery

---

### 08 - Integration Testing
**Duration**: 15 minutes  
**Purpose**: Validate full integration

```bash
cd 08-integration-testing/
./biomeos-integration.sh
./real-vs-mock-comparison.sh
```

**What you'll see**: BiomeOS coordination, capability discovery, mock vs real comparison

---

## 🌱 Fermentation Philosophy

This showcase is **not complete**. It's **growing**.

As we build each scenario, we'll:
1. **Discover gaps** in implementation
2. **Document issues** we encounter
3. **Evolve petalTongue** to address them
4. **Learn** from mature primals

**This is fermentation.** We're growing petalTongue through practical use.

---

## 📊 Progress Tracker

| Scenario | Status | Completion | Modalities | Notes |
|----------|--------|-----------|-----------|-------|
| 00-setup | ✅ Complete | 100% | - | Environment validation |
| 01-single-primal | ✅ Complete | 100% | Visual, Audio | Basic rendering |
| 02-modality-visual | ✅ Complete | 100% | Visual | 4 layouts, full interaction |
| 03-modality-audio | ✅ Complete | 100% | Audio | 5 instruments, full sonification |
| 04-dual-modality | ✅ Complete | 100% | Both | Revolutionary proof! |
| 05-accessibility-validation | ✅ Complete | 100% | Both | Accessibility validated |
| 06-songbird-discovery | 📋 Planned | 0% | Both | Phase 2 |
| 07-nestgate-storage | 📋 Planned | 0% | Both | Phase 2 |
| 08-toadstool-compute | 📋 Planned | 0% | Both | Phase 2 |
| 09-full-ecosystem | 📋 Planned | 0% | Both | Phase 2 |

---

## 🔍 Gap Documentation

As gaps are discovered, they're documented in: [`GAPS.md`](./GAPS.md)

Format:
```markdown
## Gap: [Description]
**Discovered**: [Date]
**Scenario**: [Which demo revealed it]
**Impact**: [High/Medium/Low]
**Status**: [Open/In Progress/Fixed]
```

---

## 📚 Learning from Mature Primals

We studied these showcases:
- `../../../beardog/showcase/` - Security patterns
- `../../../nestgate/showcase/` - Storage patterns (excellent structure!)
- `../../../songbird/showcase/` - Discovery patterns
- `../../../toadstool/showcase/` - Compute patterns
- `../../../squirrel/showcase/` - AI patterns

Key learnings:
- Progressive learning path (nestgate's 00-local-primal approach)
- Clear scripts with expected outputs
- Comprehensive README files
- Real operations, not mocks
- Automated testing where possible

---

## 🎓 How to Use This Showcase

### For petalTongue Development

1. Run scenarios sequentially
2. Document what works and what doesn't
3. Note performance observations
4. Record accessibility findings
5. Update GAPS.md with issues

### For Learning petalTongue

1. Start with 00-setup
2. Progress through scenarios in order
3. Read the READMEs
4. Run the demos
5. Observe the behavior

### For Validation

1. Run all scenarios
2. Check gap status
3. Verify fixes work
4. Update progress tracker

---

## 🚀 Getting Started

**Right now**, run:

```bash
cd 00-setup/
cat README.md
./demo.sh
```

This will verify your environment and launch the first demo!

---

**Status**: Fermentation in progress 🌱  
**Next**: Build 00-setup scenario  
**Goal**: 8 working scenarios + comprehensive gap documentation

---

*"Good software is grown, not built. Let it ferment."* 🌸

