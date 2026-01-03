# 🌸 petalTongue Showcase - Phase 3 Quick Start

**Focus**: Inter-Primal Integration  
**Goal**: Show petalTongue visualizing other primals  
**Timeline**: 4 weeks (Jan 6 - Feb 2, 2026)

---

## 🎯 Phase 3 Structure

```
showcase/03-inter-primal/
├── README.md                          # This file
├── RUN_ALL_INTER_PRIMAL.sh           # Run all demos
├── 01-songbird-discovery/            # Visualize songbird federation
├── 02-beardog-security/              # Visualize key lineage
├── 03-nestgate-storage/              # Visualize storage patterns
├── 04-toadstool-compute/             # Visualize compute mesh
├── 05-loamspine-permanence/          # Visualize spine (future)
├── 06-rhizocrypt-dag/                # Visualize DAG (future)
└── 07-full-ecosystem/                # All primals together
```

---

## 🚀 Quick Start

### **Prerequisites**

1. **petalTongue built**:
```bash
cd /path/to/petalTongue
cargo build --release
```

2. **At least one other primal running** (pick one):

**Songbird** (easiest):
```bash
cd /path/to/songBird
cargo run --release
```

**BearDog**:
```bash
cd /path/to/bearDog
./start-beardog-server.sh
```

**Toadstool**:
```bash
cd /path/to/ecoPrimals/phase1/toadstool
cargo run --release
```

---

## 📋 Demo Descriptions

### **01-songbird-discovery** (15 min)
**What it shows**: Multi-tower federation visualization

**Prerequisites**:
- Songbird running (local or remote)
- Federation configured (optional for multi-tower)

**Run**:
```bash
cd 01-songbird-discovery
./demo.sh
```

**Expected**: 
- petalTongue discovers songbird
- Shows songbird's federated towers
- Visualizes routing topology
- Real-time protocol selection

---

### **02-beardog-security** (10 min)
**What it shows**: Key lineage and BTSP visualization

**Prerequisites**:
- BearDog running
- Some keys created (demo will create if needed)

**Run**:
```bash
cd 02-beardog-security
./demo.sh
```

**Expected**:
- petalTongue discovers BearDog
- Shows key family tree (genetic lineage)
- Visualizes trust relationships
- BTSP tunnel states (if active)

---

### **03-nestgate-storage** (10 min)
**What it shows**: Storage topology and data flow

**Prerequisites**:
- NestGate running
- Some data stored (demo will create samples)

**Run**:
```bash
cd 03-nestgate-storage
./demo.sh
```

**Expected**:
- petalTongue discovers NestGate
- Shows storage nodes
- Visualizes replication
- Data flow patterns

---

### **04-toadstool-compute** (15 min)
**What it shows**: Compute mesh and workload routing

**Prerequisites**:
- Toadstool running
- Can submit simple workload

**Run**:
```bash
cd 04-toadstool-compute
./demo.sh
```

**Expected**:
- petalTongue discovers toadstool
- Shows active workloads
- Visualizes compute distribution
- Resource usage patterns

---

### **07-full-ecosystem** (20 min)
**What it shows**: All primals together

**Prerequisites**:
- All primals running:
  - Songbird
  - BearDog
  - NestGate
  - Toadstool
  - (Others optional)

**Run**:
```bash
cd 07-full-ecosystem
./demo.sh
```

**Expected**:
- Complete topology graph
- All inter-primal communications
- Trust relationships
- Resource flows
- Health monitoring

---

## 🎓 Learning Path

### **New to Inter-Primal?**
Start with: **01-songbird-discovery**
- Easiest to set up
- Clear visualization
- Foundation for others

### **Want Security Focus?**
Try: **02-beardog-security**
- Key relationships
- Trust visualization
- BTSP tunnels

### **Want Compute Understanding?**
Try: **04-toadstool-compute**
- Workload routing
- Resource visualization
- Distributed patterns

### **Want Everything?**
Run: **07-full-ecosystem**
- Complete picture
- All interactions
- Production-like

---

## 🛠️ Troubleshooting

### **"No primals discovered"**
**Solution**:
1. Check primal is actually running
2. Verify mDNS enabled
3. Try `BIOMEOS_URL` env var

### **"Connection refused"**
**Solution**:
1. Check primal port (3000, 8080, etc)
2. Verify firewall allows connection
3. Check logs: `RUST_LOG=debug`

### **"Visualization empty"**
**Solution**:
1. Wait 5-10 seconds for discovery
2. Check primal has data/workloads
3. Refresh manually (press R)

---

## 📊 Status

| Demo | Status | ETA |
|------|--------|-----|
| 01-songbird-discovery | 📋 Planned | Week 2 |
| 02-beardog-security | 📋 Planned | Week 2 |
| 03-nestgate-storage | 📋 Planned | Week 2 |
| 04-toadstool-compute | 📋 Planned | Week 2 |
| 05-loamspine-permanence | 📋 Future | Q1 2026 |
| 06-rhizocrypt-dag | 📋 Future | Q1 2026 |
| 07-full-ecosystem | 📋 Planned | Week 4 |

---

## 🌟 What Makes This Special

### **Multi-Modal Throughout**
Every demo shows BOTH:
- Visual graph representation
- Audio sonification
- (Unlike other primal showcases)

### **Accessibility-First**
Can be experienced:
- Visually (traditional)
- Audibly (blind users)
- Both (complete experience)

### **Real Operations**
- No mocks (unless primal unavailable)
- Actual discovery
- Live data
- Production patterns

---

*Phase 3 Ready: January 2026*  
*Learn from: songbird/toadstool/beardog*  
*Build: Inter-primal visualization excellence*

🌸 **petalTongue: The ecosystem's visual heartbeat!** 🚀

