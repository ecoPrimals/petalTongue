# 06 - Performance Benchmarking

**Duration**: 30-45 minutes  
**Purpose**: Stress test petalTongue with large-scale ecosystems and measure performance limits

---

## 🎯 What This Demo Does

1. **Tests scalability** (10, 50, 100, 500 nodes)
2. **Measures FPS** under load
3. **Benchmarks layout algorithms** (which is fastest?)
4. **Identifies bottlenecks** (rendering vs. layout vs. data fetching)
5. **Validates real-world limits** (how big can we go?)

**Goal**: Understand performance characteristics and set realistic expectations for production use.

---

## 🚀 Quick Start

```bash
./demo.sh
```

This will run a progressive stress test from small to large ecosystems.

---

## 📋 Prerequisites

- Completed scenarios 00-05
- BiomeOS running
- petalTongue UI open
- System monitor running (`htop`, Activity Monitor, Task Manager)
- Stopwatch/timer for manual timing

---

## 🎬 Demo Flow

### Phase 1: Baseline (10 Nodes)

**Objective**: Establish baseline performance

```bash
./bench-10-nodes.sh
```

**Measure**:
- **FPS**: Should be 60 FPS (smooth)
- **CPU**: Should be low (<20%)
- **Memory**: Baseline usage
- **Layout time**: <100ms

**Expected**: Silky smooth, no issues

### Phase 2: Medium Scale (50 Nodes)

**Objective**: Test typical production scenario

```bash
./bench-50-nodes.sh
```

**Measure**:
- **FPS**: Should still be 60 FPS
- **CPU**: Moderate (20-40%)
- **Memory**: Moderate increase
- **Layout time**: <500ms

**Expected**: Still smooth, some CPU usage

### Phase 3: Large Scale (100 Nodes)

**Objective**: Push toward limits

```bash
./bench-100-nodes.sh
```

**Measure**:
- **FPS**: May drop to 30-40 FPS
- **CPU**: High (40-60%)
- **Memory**: Noticeable increase
- **Layout time**: <2s

**Expected**: Noticeable lag, but usable

### Phase 4: Stress Test (500 Nodes)

**Objective**: Find the breaking point

```bash
./bench-500-nodes.sh
```

**Measure**:
- **FPS**: Likely <20 FPS (slideshow)
- **CPU**: Very high (60-100%)
- **Memory**: Significant
- **Layout time**: 5-15s

**Expected**: Unusable, but doesn't crash

---

## 📊 Benchmark Metrics

### Frame Rate (FPS)

| FPS | User Experience |
|-----|-----------------|
| 60+ | Silky smooth |
| 30-60 | Smooth |
| 15-30 | Noticeable lag |
| <15 | Sluggish, painful |

**Goal**: ≥30 FPS for production use

### Layout Algorithm Performance

Run each layout with 100 nodes, measure time:

```bash
./bench-layouts.sh
```

**Expected Results** (estimates):
- **Force-Directed**: ~500ms (many iterations)
- **Hierarchical**: ~100ms (single pass)
- **Circular**: ~50ms (simple math)
- **Random**: ~10ms (trivial)

**Insight**: Hierarchical is fastest for large graphs

### Memory Usage

| Nodes | Expected Memory |
|-------|-----------------|
| 10 | ~50 MB |
| 50 | ~100 MB |
| 100 | ~150 MB |
| 500 | ~300 MB |

**Watch for**: Memory leaks (does it grow over time?)

### CPU Usage

| Nodes | Expected CPU (idle) | Expected CPU (layout) |
|-------|---------------------|------------------------|
| 10 | <10% | 20% |
| 50 | 10-20% | 40% |
| 100 | 20-30% | 60% |
| 500 | 30-40% | 90%+ |

**Watch for**: CPU spikes during auto-refresh

---

## ✅ Success Criteria

After this demo, you should know:

- [x] Maximum usable node count (FPS ≥30)
- [x] Fastest layout algorithm for large graphs
- [x] Memory and CPU characteristics
- [x] Where bottlenecks are (rendering, layout, data)
- [x] Real-world production limits

**Document**: "petalTongue performs well up to X nodes, acceptable up to Y nodes, unusable beyond Z nodes."

---

## 🔧 Troubleshooting

### FPS always low

**Problem**: Even with 10 nodes, FPS is poor  
**Solutions**:
1. Close other applications
2. Check GPU acceleration enabled
3. Try different browser/system
4. Check for debug mode overhead
5. Monitor system load (`htop`)

### Memory leak

**Problem**: Memory grows indefinitely  
**Solutions**:
1. This is a critical bug - document in GAPS.md
2. Check for graph not being cleared on refresh
3. Check for event listeners not being removed
4. Profile with browser dev tools

### Layout hangs

**Problem**: Force-directed layout never finishes  
**Solutions**:
1. Reduce iteration count
2. Add timeout
3. Use hierarchical instead
4. Document as gap

---

## 🌱 Fermentation Notes

### Gaps to Watch For

- **Scalability**:
  - What's the practical limit?
  - Does performance degrade gracefully?
  - Can we virtualize (only render visible nodes)?

- **Layout Performance**:
  - Can we run layout in Web Worker?
  - Can we incrementally update?
  - Do we need LOD (level of detail)?

- **Rendering**:
  - Canvas vs. WebGL?
  - Can we batch draw calls?
  - Culling off-screen nodes?

- **Memory**:
  - Any leaks?
  - Can we paginate/stream?
  - LRU cache for old data?

- **User Experience**:
  - Loading indicators?
  - Progressive rendering?
  - "Performance mode" toggle?

**Document ALL performance gaps in**: `../GAPS.md`

---

## 💡 Performance Optimization Strategies

### Already Implemented

1. **Efficient Data Structures** (HashMap for nodes)
2. **Minimal Redraws** (only on change)
3. **Layout Caching** (don't re-layout unnecessarily)

### Future Optimizations

1. **Web Workers** for layout computation
2. **Virtual Scrolling** (only render visible area)
3. **Level of Detail** (simplify distant nodes)
4. **WebGL Rendering** (GPU acceleration)
5. **Incremental Layout** (update vs. recompute)
6. **Data Streaming** (load nodes on-demand)
7. **Spatial Indexing** (quad-tree for click detection)

---

## 🎓 Learning Points

### Performance is a Feature

Users won't use a slow tool. Performance = usability.

### Scalability Limits

All systems have limits. The goal is to:
1. **Know** the limit
2. **Document** the limit
3. **Fail gracefully** at the limit

**petalTongue**: Designed for 10-100 nodes (typical ecosystems)

### Premature Optimization

"Premature optimization is the root of all evil" - Knuth

**Don't optimize** until you:
1. Have real performance problems
2. Have profiled and identified bottlenecks
3. Have user feedback

**This benchmark** provides that data!

### Trade-offs

**Performance** vs. **Features** vs. **Code Simplicity**

Example:
- Force-directed layout: Beautiful, slow
- Circular layout: Simple, fast, less useful
- Hierarchical: Balanced

**Choose** based on use case.

---

## ⏭️ Next Steps

Once performance is characterized, proceed to:

```bash
cd ../07-real-world-scenarios/
cat README.md
```

This will test **production-like** scenarios with real primal behaviors.

---

## 🎮 Advanced Experiments

### Stress Test to Failure

Keep adding nodes until it crashes:
```bash
./bench-until-failure.sh
```

**Document**: What node count causes crash? OOM? Freeze?

### Layout Algorithm Comparison

Visual side-by-side:
```bash
./compare-layouts.sh 100
```

**Screenshots** of each layout at 100 nodes, time each

### Memory Leak Detection

Run for 1 hour with auto-refresh:
```bash
./long-running-test.sh
```

**Monitor**: Does memory grow linearly? Is there a leak?

### Multi-Tab Test

Open 5 petalTongue tabs:
```bash
for i in {1..5}; do xdg-open http://localhost:8080 & done
```

**Observe**: Does system handle multiple instances?

---

## 📐 Performance Targets

### MVP (Minimum Viable Performance)

- **10 nodes**: 60 FPS
- **50 nodes**: 30 FPS
- **100 nodes**: 15 FPS (usable for static viewing)

### Production-Ready

- **50 nodes**: 60 FPS
- **100 nodes**: 30 FPS
- **500 nodes**: Graceful degradation (with warning)

### Stretch Goals

- **100 nodes**: 60 FPS
- **500 nodes**: 30 FPS
- **1000 nodes**: 15 FPS (with WebGL)

---

**Status**: 🌱 Ready to build  
**Complexity**: High (requires performance profiling)  
**Dependencies**: 00-05 complete  
**Learning Value**: Very High (sets realistic expectations)

---

*You can't improve what you don't measure.  
Performance benchmarking is the foundation of optimization.* 🌸

