# 🌸 petalTongue — What's Next

**Implementation Roadmap**

---

## Phase 1: Foundation + Migration (Week 1) 🔄 IN PROGRESS

**Goal**: Working primal with migrated UI code from biomeOS

### Tasks
- [x] Scaffold primal structure with sourDough
- [x] Create crate structure (core, graph, animation, telemetry, api, ui)
- [x] Copy specification
- [x] Update workspace dependencies
- [ ] Move UI code from biomeOS
  - [ ] Copy views/primals.rs → petal-tongue-ui
  - [ ] Copy views/dashboard.rs → petal-tongue-ui
  - [ ] Copy api.rs (visualization parts) → petal-tongue-api
  - [ ] Copy types.rs (visualization types) → petal-tongue-core
- [ ] Set up egui app structure
- [ ] Basic compilation test

**Deliverables**:
- ✅ Scaffolded primal with 6 crates
- ✅ Specification in place
- ⏸️ Migrated UI code compiling
- ⏸️ Basic egui window rendering

---

## Phase 2: Graph Visualization (Week 2-3)

**Goal**: Interactive graph showing primal topology

### Tasks
- [ ] Integrate egui_graphs
- [ ] Integrate petgraph for graph data structures
- [ ] Implement node rendering
  - [ ] Primal nodes (color-coded by type)
  - [ ] Health status indicators
  - [ ] Node labels and metadata
- [ ] Implement edge rendering
  - [ ] Connection lines between primals
  - [ ] Directional arrows
  - [ ] Edge labels (capabilities)
- [ ] Layout algorithms
  - [ ] Force-directed layout (default)
  - [ ] Hierarchical layout
  - [ ] Circular layout
- [ ] Interactive controls
  - [ ] Pan and zoom
  - [ ] Drag and drop nodes
  - [ ] Click to select
  - [ ] Details panel

**Deliverables**:
- Static graph visualization
- Multiple layout options
- Interactive controls working
- Details panel showing primal info

---

## Phase 3: Real-Time Updates (Week 3-4)

**Goal**: Live graph updates as primals come and go

### Tasks
- [ ] Telemetry integration
  - [ ] Event stream from biomeOS
  - [ ] Primal discovery events
  - [ ] Health status updates
- [ ] Live graph updates
  - [ ] Add nodes dynamically
  - [ ] Remove nodes dynamically
  - [ ] Update node states
  - [ ] Smooth animations
- [ ] Filtering and search
  - [ ] Filter by primal type
  - [ ] Filter by health status
  - [ ] Search by name/capability
  - [ ] Highlight matches

**Deliverables**:
- Live topology updates
- Smooth animations
- Filtering working
- Search functional

---

## Phase 4: Flow Animation (Week 4-6)

**Goal**: Animated particles showing message flow

### Tasks
- [ ] Flow animation engine
  - [ ] Particle system
  - [ ] Path following along edges
  - [ ] Color coding by message type
  - [ ] Speed based on traffic volume
- [ ] Telemetry integration
  - [ ] Capture API calls between primals
  - [ ] Parse message metadata
  - [ ] Stream to animation engine
- [ ] Visual enhancements
  - [ ] Traffic volume indicators
  - [ ] Hotspot highlighting
  - [ ] Flow statistics overlay

**Deliverables**:
- Animated message flow
- Real-time traffic visualization
- Performance metrics overlay

---

## Phase 5: Multi-View Dashboard (Week 6-7)

**Goal**: Multiple visualization modes

### Tasks
- [ ] Timeline view
  - [ ] Sequence diagram of interactions
  - [ ] Time-based event listing
  - [ ] Zoom in/out on timeline
- [ ] Traffic view
  - [ ] Traffic matrix (primal-to-primal)
  - [ ] Bandwidth graphs
  - [ ] Latency heatmaps
- [ ] Health view
  - [ ] Status dashboard
  - [ ] Alert indicators
  - [ ] Historical health trends
- [ ] View switching
  - [ ] Tab-based navigation
  - [ ] Persist view preferences

**Deliverables**:
- 4 distinct views (Topology, Timeline, Traffic, Health)
- Smooth view switching
- Preference persistence

---

## Phase 6: API Server (Week 7-8)

**Goal**: REST + WebSocket API for external consumers

### Tasks
- [ ] REST API
  - [ ] GET /topology (current graph state)
  - [ ] GET /primals (list of primals)
  - [ ] GET /primals/:id (primal details)
  - [ ] GET /traffic (traffic statistics)
- [ ] WebSocket API
  - [ ] Live topology updates
  - [ ] Live traffic streams
  - [ ] Event subscriptions
- [ ] Authentication
  - [ ] BearDog DID integration
  - [ ] API key support
  - [ ] Rate limiting
- [ ] Documentation
  - [ ] OpenAPI spec
  - [ ] Client examples
  - [ ] Integration guide

**Deliverables**:
- Working REST API
- Working WebSocket API
- BearDog auth integration
- Complete API documentation

---

## Phase 7: Polish & Optimization (Week 8-9)

**Goal**: Production-ready quality

### Tasks
- [ ] Performance optimization
  - [ ] 60 FPS with 50+ nodes
  - [ ] Efficient rendering
  - [ ] Memory optimization
  - [ ] CPU profiling
- [ ] Visual polish
  - [ ] Themes (light/dark)
  - [ ] Custom color schemes
  - [ ] Smooth transitions
  - [ ] Accessibility (color-blind friendly)
- [ ] Export functionality
  - [ ] Export graph as PNG/SVG
  - [ ] Export topology as JSON
  - [ ] Export traffic logs
- [ ] Configuration
  - [ ] User preferences
  - [ ] Layout persistence
  - [ ] Custom themes

**Deliverables**:
- 60 FPS performance target met
- Light/dark themes
- Export functionality working
- User preferences persisting

---

## Phase 8: Testing & Documentation (Week 9-10)

**Goal**: Comprehensive tests and docs

### Tasks
- [ ] Unit tests
  - [ ] Graph rendering logic
  - [ ] Animation calculations
  - [ ] Telemetry parsing
  - [ ] API endpoints
- [ ] Integration tests
  - [ ] BiomeOS integration
  - [ ] Songbird discovery
  - [ ] BearDog auth
- [ ] E2E tests
  - [ ] Full visualization pipeline
  - [ ] Multi-primal scenarios
  - [ ] Failure scenarios
- [ ] Documentation
  - [ ] User guide
  - [ ] Developer guide
  - [ ] API reference
  - [ ] Architecture docs

**Deliverables**:
- 80%+ test coverage
- All integration tests passing
- Complete documentation suite

---

## Future Enhancements (Post-Launch)

### Advanced Features
- [ ] 3D graph visualization
- [ ] Geographic topology view (map-based)
- [ ] AI-powered insights
  - [ ] Anomaly detection
  - [ ] Performance suggestions
  - [ ] Capacity planning
- [ ] Mobile app
- [ ] CLI tool for quick topology checks
- [ ] IDE plugin integration

### Ecosystem Integration
- [ ] RhizoCrypt integration (query DAG for history)
- [ ] LoamSpine integration (permanent topology records)
- [ ] SweetGrass integration (attribution visualization)

---

## Success Criteria

### Functional Requirements
- ✅ Display all discovered primals
- ✅ Show connections and health
- ✅ Interactive controls (pan, zoom, select)
- ✅ Real-time updates < 100ms latency
- ✅ 60 FPS with 50 nodes

### Non-Functional Requirements
- ✅ Intuitive UX (industry standard)
- ✅ Accessible (WCAG 2.1 AA)
- ✅ Reliable (graceful degradation)
- ✅ Maintainable (>80% test coverage)
- ✅ Performant (< 50MB memory, < 10% CPU idle)

---

## Dependencies Roadmap

| Milestone | Depends On | Blocks |
|-----------|------------|--------|
| Phase 1 | sourDough scaffold | Phase 2-8 |
| Phase 2 | Phase 1 | Phase 3 |
| Phase 3 | Phase 2 | Phase 4 |
| Phase 4 | Phase 3 | Phase 5 |
| Phase 5 | Phase 3 | Phase 6 |
| Phase 6 | Phase 1 | External consumers |
| Phase 7 | Phase 5 | Production deployment |
| Phase 8 | Phase 7 | Launch |

---

## Team & Timeline

**Estimated Timeline**: 10 weeks (2.5 months)

**Team Size**: 2-3 developers
- 1 UI/UX engineer (graph, animation, polish)
- 1 Backend engineer (API, telemetry, integration)
- 1 Optional: Graphics engineer (advanced visualizations)

**Current Status**: Week 1, Phase 1 (Foundation + Migration)

---

*petalTongue: Visualizing the ecosystem, one petal at a time.*

