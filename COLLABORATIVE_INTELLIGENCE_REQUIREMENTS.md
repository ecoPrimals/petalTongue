# Collaborative Intelligence Requirements - biomeOS Handoff

**Date**: January 11, 2026  
**From**: biomeOS Team  
**Priority**: High  
**Timeline**: 4 weeks (petalTongue critical path)  
**Status**: 🚧 Requirements received, planning phase

---

## 🎯 Vision

Evolve from **"AI decides, user watches"** to **"Human and AI collaborate as equals"**.

Users will be able to:
- ✅ **View** live graph execution (what's happening)
- ✅ **Understand** why decisions are made (AI reasoning)
- ✅ **Modify** graphs in real-time (active collaboration)
- ✅ **Pre-configure** niches before deployment (bootstrap new systems)
- ✅ **Learn together** - AI learns from user, user learns from AI

---

## 🌸 petalTongue Requirements

### 1. Interactive Graph Editor

**Capabilities Needed**:
- Drag-and-drop graph builder
- Add/remove/modify nodes in real-time
- Visual connection editor (dependencies)
- Live graph execution visualization

**Current State**:
- ✅ We have graph visualization (existing)
- ✅ We have topology display (existing)
- ❌ No interactive editing yet
- ❌ No drag-and-drop builder
- ❌ No real-time node modification

**Gap**: Need to build interactive editing layer on top of existing visualization

---

### 2. JSON-RPC Methods (8 new)

**Required Methods**:

```rust
// Graph Editor Operations
ui.graph.editor_open(graph_id) → Open editor
ui.graph.add_node(node) → Add node to graph
ui.graph.modify_node(node_id, changes) → Modify node
ui.graph.remove_node(node_id) → Remove node
ui.graph.add_edge(from, to) → Add dependency
ui.graph.save_template(name) → Save as template
ui.graph.apply_template(template_id) → Load template
ui.graph.get_preview(graph) → Preview execution plan
```

**Current State**:
- ✅ We have JSON-RPC infrastructure (existing)
- ✅ We have tarpc IPC (existing)
- ❌ No graph manipulation methods yet
- ❌ No template system yet

**Gap**: Need to implement 8 new RPC methods

---

### 3. Real-Time Updates

**Capabilities Needed**:
- WebSocket for live graph execution streaming
- Node status updates (running, completed, failed)
- Decision reasoning display
- Modification conflict handling

**Current State**:
- ✅ We have real-time UI updates (egui)
- ✅ We have bidirectional communication (existing)
- ❌ No WebSocket support yet
- ❌ No graph streaming protocol yet

**Gap**: Need to add WebSocket support and streaming protocol

---

## 📊 Current Architecture Assessment

### What We Have ✅

**UI Foundation**:
- egui + winit (pure Rust GUI)
- SAME DAVE proprioception (bidirectional feedback)
- Sensor discovery (runtime capability detection)
- Pure Rust audio/visual systems

**Communication**:
- tarpc IPC (primary)
- JSON-RPC 2.0 (secondary)
- Unix sockets (Songbird discovery)
- Protocol priority system

**Visualization**:
- Graph rendering (existing)
- Topology display (existing)
- Real-time updates (existing)

**Discovery**:
- Songbird client (runtime discovery)
- Capability-based system
- Graceful degradation

### What We Need ❌

**Interactive Editing**:
- Drag-and-drop system
- Node manipulation UI
- Edge editing UI
- Graph validation UI

**Graph Management**:
- Template system (save/load)
- Preview system
- Modification tracking
- Conflict resolution

**Real-Time Streaming**:
- WebSocket support
- Graph execution streaming
- Status updates protocol
- Reasoning display

---

## 🏗️ Implementation Plan

### Phase 1: Foundation (Week 1)
**Goal**: Set up interactive graph editing infrastructure

**Tasks**:
1. Design graph data model (nodes, edges, templates)
2. Create interactive graph widget (egui)
3. Implement drag-and-drop system
4. Basic node add/remove/modify

**Deliverable**: User can add/remove nodes in UI

---

### Phase 2: RPC Methods (Week 2)
**Goal**: Implement all 8 JSON-RPC methods

**Tasks**:
1. Implement `ui.graph.editor_open(graph_id)`
2. Implement `ui.graph.add_node(node)`
3. Implement `ui.graph.modify_node(node_id, changes)`
4. Implement `ui.graph.remove_node(node_id)`
5. Implement `ui.graph.add_edge(from, to)`
6. Implement `ui.graph.save_template(name)`
7. Implement `ui.graph.apply_template(template_id)`
8. Implement `ui.graph.get_preview(graph)`

**Deliverable**: All RPC methods working, tested

---

### Phase 3: Real-Time Streaming (Week 3)
**Goal**: Add WebSocket support and streaming

**Tasks**:
1. Add WebSocket dependency (pure Rust)
2. Implement streaming protocol
3. Add node status display
4. Add reasoning display
5. Handle modification conflicts

**Deliverable**: Live graph execution visible in UI

---

### Phase 4: Polish & Integration (Week 4)
**Goal**: Production-ready integration

**Tasks**:
1. End-to-end testing with biomeOS
2. Error handling and edge cases
3. UI polish and UX improvements
4. Documentation and examples

**Deliverable**: Production-ready collaborative intelligence

---

## 🎯 Dependencies

### Internal (petalTongue)
- ✅ egui UI framework (have)
- ✅ tarpc IPC (have)
- ✅ JSON-RPC support (have)
- ❌ WebSocket support (need)
- ❌ Graph data model (need)
- ❌ Template system (need)

### External (Other Primals)
- **Squirrel**: AI suggestions, learning
- **NestGate**: Template storage
- **Songbird**: Graph validation
- **BearDog**: Security validation
- **ToadStool**: Resource estimation

**Note**: We can mock these during development

---

## 🚦 Readiness Assessment

### Ready Now ✅
- UI infrastructure (egui + winit)
- Communication layer (tarpc + JSON-RPC)
- Visualization foundation
- Bidirectional feedback (SAME DAVE)

### Need to Build 🚧
- Interactive editing (drag-and-drop)
- Graph manipulation RPC methods (8 new)
- WebSocket streaming
- Template system
- Conflict resolution

### Blockers ⚠️
- **None!** We have all the foundation needed

**Estimated Effort**: 4 weeks (aligns with biomeOS timeline)

---

## 🌸 TRUE PRIMAL Alignment

This request **perfectly aligns** with TRUE PRIMAL principles:

✅ **Human-AI Collaboration**: Equals, not subservient  
✅ **Transparency**: AI explains reasoning  
✅ **User Control**: User can override AI  
✅ **Learn Together**: Both improve  
✅ **Bootstrap Fast**: User expertise + AI learning  

This is **exactly** what we want to build!

---

## 📋 Action Items

### Immediate (This Week)
1. ✅ Acknowledge requirements (this document)
2. ⏳ Design graph data model
3. ⏳ Prototype interactive editor
4. ⏳ Plan WebSocket integration

### Short-Term (Week 1-2)
1. Implement interactive editing
2. Implement RPC methods
3. Create graph template system

### Medium-Term (Week 3-4)
1. Add WebSocket streaming
2. Integrate with biomeOS
3. End-to-end testing

### Coordination
- Join #collaborative-intelligence Slack
- Attend Wednesday 2pm UTC sync
- Tag issues with `collaborative-intelligence`

---

## 🎊 Impact

**Before**:
- petalTongue visualizes AI decisions (passive viewing)
- User watches graph execution
- No user input during deployment

**After**:
- petalTongue enables human-AI collaboration (active)
- User modifies graphs in real-time
- Bootstrap new systems 10x faster

**This is transformative for petalTongue!**

---

## 📚 References

- `specs/COLLABORATIVE_INTELLIGENCE_SPEC.md` (from biomeOS)
- `COLLABORATIVE_INTELLIGENCE_EVOLUTION.md` (from biomeOS)
- Interactive UI specs (from biomeOS)

---

**Status**: 🚧 Requirements received, planning in progress  
**Timeline**: 4 weeks (critical path)  
**Readiness**: ✅ Foundation ready, can start implementation  
**Blocker Status**: ⚠️ None  

🤝 **Ready to build human-AI collaboration!** ✨

