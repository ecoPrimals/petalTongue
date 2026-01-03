# 🍄 Toadstool Compute - Inter-Primal Showcase

**Duration**: 15 minutes  
**Complexity**: Medium  
**Prerequisites**: Toadstool running with sample workloads

---

## 🎯 What This Demonstrates

This showcase demonstrates petalTongue's ability to:
1. **Discover** toadstool compute nodes
2. **Visualize** compute mesh topology
3. **Monitor** active workloads in real-time
4. **Sonify** workload execution (running/complete/failed)
5. **Show** resource utilization patterns

---

## 🚀 Quick Start

```bash
# 1. Start toadstool (if not running)
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool
cargo run --release &

# 2. Wait for startup (10 seconds)
sleep 10

# 3. Submit a sample workload
cd showcase/local-capabilities
./01-native-hello.sh

# 4. Run demo
cd /home/eastgate/Development/ecoPrimals/phase2/petalTongue/showcase/03-inter-primal/04-toadstool-compute
./demo.sh
```

---

## 📋 Prerequisites

### **Minimal Setup** (Single Toadstool)
```bash
# Just toadstool running locally
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool
cargo run --release
```

**Expected**: Single compute node visualization

---

### **Distributed Setup** (Multiple Toadstool) - OPTIONAL
```bash
# Start toadstool A
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool
TOADSTOOL_PORT=8080 cargo run --release &

# Start toadstool B
TOADSTOOL_PORT=8081 cargo run --release &

# Verify both running
curl http://localhost:8080/health
curl http://localhost:8081/health
```

**Expected**: Multi-node compute mesh

---

## 🎨 What You'll See

### **Visual Representation**
- **Nodes**: Each toadstool instance
- **Node Colors**:
  - 🟢 Green = Idle (ready for work)
  - 🟡 Yellow = Busy (executing workload)
  - 🔵 Blue = Queued (workload waiting)
  - 🔴 Red = Failed/Error
- **Node Size**: Proportional to compute capacity
- **Edges**: Workload routing (if distributed)
- **Animation**: Pulsing during execution

### **Audio Representation**
- **Synth**: Workload submission (rising pitch)
- **Drums**: Execution start (kick drum)
- **Strings**: Execution progress (sustained note)
- **Chime**: Completion (success bell)
- **Bass**: Failure (low rumble)
- **Spatial**: Left=CPU, Center=Memory, Right=GPU

---

## 📊 Expected Output

### **Console Output**
```
🌸 petalTongue Showcase: Toadstool Compute
==========================================

[00:00] Starting discovery...
[00:02] ✅ Discovered: toadstool-1 (http://localhost:8080)
[00:02]    - Status: Healthy
[00:03]    - Capabilities: [native, container, python]
[00:04]    - Active workloads: 2
[00:04]    - Resource usage: CPU=35% MEM=128MB

[00:05] 🍄 Workload Activity:
[00:05]    - workload-abc123: RUNNING (Python ML inference)
[00:06]    - workload-def456: QUEUED (Native Rust task)

[00:07] 🎵 Generating visual representation...
[00:07]    - Layout: ForceDirected
[00:07]    - Nodes: 1 (toadstool)
[00:07]    - Node color: Yellow (busy)

[00:08] 🎵 Generating audio sonification...
[00:08]    - Instrument: Strings (execution)
[00:08]    - Note: C4 sustained (35% CPU)
[00:08]    - Panning: Left (CPU-bound)

[00:10] Press Ctrl+C to stop demo
```

### **UI Window**
- **Main panel**: Compute mesh graph
- **Top-left**: FPS, active workloads count
- **Top-right**: Total compute capacity
- **Bottom**: Audio controls + workload timeline
- **Side panel**: Workload details (ID, status, resources, duration)

---

## 🎓 What You're Learning

### **Concept 1: Compute Discovery**
petalTongue discovers toadstool via:
1. **mDNS** (local network)
2. **HTTP** (metadata + health)
3. **Capabilities** (native, container, GPU, etc)

**Watch for**: petalTongue detecting compute types automatically

### **Concept 2: Workload Visualization**
Toadstool's compute patterns:
- Native execution (fast, direct)
- Container isolation (secure, portable)
- Python ML (flexible, GPU-capable)
- Distributed routing (multi-instance)

**Watch for**: Node color changing during execution

### **Concept 3: Resource Monitoring**
petalTongue shows:
- CPU utilization (node brightness)
- Memory usage (node size)
- GPU activity (node border)
- Queue depth (node badge)

**Watch for**: Visual feedback matching resource usage

### **Concept 4: Multi-Modal Feedback**
Both visual AND audio:
- Sighted: See compute mesh + colors
- Blind: Hear execution patterns + spatial audio
- Both: Complete operational awareness

**Watch for**: Audio pitch correlating with CPU %

---

## 🛠️ Troubleshooting

### **Problem**: "No compute nodes discovered"
**Solution**:
```bash
# Check toadstool is running
curl http://localhost:8080/health

# If not running:
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool
cargo run --release &
sleep 10

# Try demo again
./demo.sh
```

---

### **Problem**: "No workloads shown"
**Solution**:
```bash
# Submit a sample workload
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool/showcase/local-capabilities
./01-native-hello.sh

# Refresh petalTongue (press R in UI)
```

---

### **Problem**: "Graph is static (no updates)"
**Solution**:
```bash
# Workload may have already completed
# Submit a longer-running workload:
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool/showcase/local-capabilities
./02-long-running-task.sh

# Or enable continuous monitoring:
CONTINUOUS_REFRESH=1 ./demo.sh
```

---

### **Problem**: "Audio sounds wrong"
**Solution**:
```bash
# CPU-bound should pan left
# Memory-bound should pan center
# GPU-bound should pan right

# Check audio system:
pactl info

# Restart demo with audio debug:
RUST_LOG=debug,petal_tongue_audio=trace ./demo.sh
```

---

## 🧪 Experiments to Try

### **Experiment 1: Submit Workload During Demo**
```bash
# While demo running, submit workload in another terminal:
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool/showcase/local-capabilities
./01-native-hello.sh

# Watch graph update:
# - Node color changes (green → yellow)
# - Animation intensifies
# - Audio starts (drums + strings)
```

**Expected**: Real-time visual + audio feedback

---

### **Experiment 2: Distributed Compute**
```bash
# Start second toadstool
TOADSTOOL_PORT=8081 cargo run --release &

# Submit workload to both:
curl -X POST http://localhost:8080/workload -d '{"type":"native"}'
curl -X POST http://localhost:8081/workload -d '{"type":"native"}'

# Watch graph:
# - Two nodes appear
# - Both change color
# - Audio spatially separates (left/right)
```

**Expected**: Multi-node coordination visualization

---

### **Experiment 3: Resource Saturation**
```bash
# Submit multiple workloads to saturate CPU
for i in {1..5}; do
  curl -X POST http://localhost:8080/workload -d '{"type":"cpu_intensive"}'
done

# Watch graph:
# - Node color deepens (darker yellow)
# - Node pulses faster
# - Audio pitch rises (higher CPU %)
```

**Expected**: Visual stress indication

---

### **Experiment 4: GPU Workload** (if GPU available)
```bash
# Check GPU capability
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool/showcase/gpu-universal
./check-gpu.sh

# If available, run GPU workload:
./local/01-vector-add.sh

# Watch graph:
# - Node border glows (GPU active)
# - Audio pans right (GPU = right channel)
# - Higher frequency (GPU is fast)
```

**Expected**: GPU-specific visualization

---

## 📚 Related Demos

### **Before This**:
- **Phase 2**: `02-primal-discovery` (general discovery)
- **Toadstool**: `showcase/local-capabilities/` (toadstool basics)

### **After This**:
- **03-inter-primal**: `01-songbird-discovery` (orchestration)
- **03-inter-primal**: `02-beardog-security` (secure compute)

### **Advanced**:
- **03-inter-primal**: `07-full-ecosystem` (orchestrated compute)

---

## 🌟 Key Takeaways

1. **Compute is visual** - See execution in real-time
2. **Workloads are audible** - Hear operational state
3. **Resources are intuitive** - Color/size/sound = usage
4. **Distributed works** - Multi-instance coordination
5. **Multi-modal matters** - Accessibility + operations

---

## 📊 Success Criteria

After this demo, you should be able to:
- ✅ Discover toadstool compute nodes
- ✅ Visualize active workloads
- ✅ Hear execution patterns
- ✅ Monitor resource utilization
- ✅ Understand distributed routing

---

## 🚀 What's Next?

### **Immediate**:
Try: `../02-beardog-security/` (secure workload execution)

### **Build On This**:
Try: `../07-full-ecosystem/` (songbird orchestrating toadstool)

### **Deep Dive**:
Read: `/home/eastgate/Development/ecoPrimals/phase1/toadstool/showcase/README.md`

---

*Demo Ready: January 2026*  
*Status: 📋 Planned (Week 2)*  
*Integration: toadstool + petalTongue*

🌸🍄 **Visualize the compute!** 🚀

