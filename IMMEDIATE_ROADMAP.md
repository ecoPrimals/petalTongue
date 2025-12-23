# 🌸 petalTongue - Immediate Roadmap

**Date**: December 23, 2025  
**Status**: Production-ready foundation, audio synthesis next

---

## 🎯 Progression Plan

### Phase 1: Audio Synthesis (Next - 2-3 hours) 🎵

**Goal**: Make the audio renderer actually produce sound through speakers.

**Tasks**:
1. Add audio library (`rodio` or `cpal`)
2. Implement tone generation based on instrument types
3. Map audio attributes → actual sounds:
   - Instruments (bass, drums, chimes, strings, synth) → distinct tones
   - Health (healthy/warning/critical) → pitch/harmony
   - Position (x/y) → stereo panning
   - Activity (capabilities) → volume
4. Wire up play/pause controls in UI
5. Test with all 5 primal types
6. Add basic fade-in/fade-out
7. Validate accessibility (sound is informative, not noise)

**Success Criteria**:
- ✅ Hearing distinct sounds for each primal type
- ✅ Recognizing health states by pitch
- ✅ Perceiving position via stereo
- ✅ Volume controls work
- ✅ Blind users can "hear" ecosystem state

**Estimated**: 2-3 hours

---

### Phase 2: Complete sandbox/ (After Audio - 1-2 hours) 🧪

**Goal**: Flesh out sandbox with working demos using real audio.

**Tasks**:
1. **Build mock HTTP server** (`sandbox/mock-biomeos/`)
   - Simple Rust server (axum/actix)
   - Serves JSON from scenarios/
   - Hot-reload on file change
   - ~200 lines of code

2. **Add more scenarios**:
   - `complex.json` - 20+ primals, production-like
   - `dynamic.json` - Simulates changes over time (health degrades, recovers)
   - `chaos.json` - High churn, rapid failures
   - `performance.json` - 100+ nodes for stress testing

3. **Test audio with scenarios**:
   - Run `unhealthy.json` - hear the warnings/critical alerts
   - Run `dynamic.json` - hear ecosystem evolve in real-time
   - Validate sonification is informative

4. **Document sandbox workflows**:
   - "How to add a scenario"
   - "How to test audio with different topologies"
   - "How to validate against real BiomeOS"

**Success Criteria**:
- ✅ Mock server serves scenarios
- ✅ Can switch scenarios and hear differences
- ✅ Audio clearly distinguishes healthy vs unhealthy
- ✅ Sandbox enables rapid iteration

**Estimated**: 1-2 hours

---

### Phase 3: Create showcase/ (After Sandbox - 2-3 hours) 🎬

**Goal**: Live demonstrations for presentations and onboarding.

**Structure**:
```
showcase/
├── README.md                    # Showcase guide
├── demos/                       # Pre-recorded demos
│   ├── 01-basic-topology/       # Simple 5-primal demo
│   ├── 02-degraded-system/      # Health state changes
│   ├── 03-scaling-event/        # Add/remove primals live
│   ├── 04-audio-only/           # Blind user experience
│   └── 05-production-scale/     # Large topology (50+ nodes)
├── scripts/                     # Demo automation
│   ├── run-demo.sh              # Launch specific demo
│   ├── record-demo.sh           # Capture screenshots/video
│   └── presentation-mode.sh     # Fullscreen, clean UI
└── presentations/               # Slide decks, materials
    ├── accessibility-first.pdf  # "How we built for blind users"
    ├── architecture.pdf         # Technical deep-dive
    └── live-demo.md             # Presenter notes
```

**Demo Scenarios**:

1. **Basic Topology** (01)
   - 5 primals, all healthy
   - Show visual graph
   - Show audio descriptions
   - Demonstrate pan/zoom/select
   - Message: "This is petalTongue"

2. **Degraded System** (02)
   - Start healthy
   - Inject failures (ToadStool → warning, Squirrel → critical)
   - Watch colors change
   - Hear audio descriptions change
   - Message: "Same info, visual + audio"

3. **Scaling Event** (03)
   - Start with 5 primals
   - Add 10 more dynamically
   - Watch layout adapt
   - Auto-refresh shows changes
   - Message: "Real-time ecosystem monitoring"

4. **Audio-Only Experience** (04)
   - Close eyes
   - Listen to soundscape description
   - Hear instrument types
   - Detect warnings by pitch
   - Message: "Blind users monitor systems"

5. **Production Scale** (05)
   - 50+ primals
   - Complex topology
   - Test layout algorithms
   - Performance demonstration
   - Message: "Scales to real deployments"

**Success Criteria**:
- ✅ 5 polished demo scenarios
- ✅ Presenter can run any demo quickly
- ✅ Screenshots/video for README
- ✅ Slide decks for presentations
- ✅ Compelling story about accessibility

**Estimated**: 2-3 hours

---

### Phase 4: Fermentation (Ongoing - 2-4 weeks) 🌱

**Goal**: Let petalTongue "ferment" - test, refine, evolve naturally.

**Activities**:

1. **Use It Daily**
   - Monitor real BiomeOS (when available)
   - Test with different topologies
   - Gather user feedback
   - Note pain points and delights

2. **Iterate on Complexity**
   - More layout algorithms (radial, hierarchical-lr, grid)
   - Edge bundling (reduce visual clutter)
   - Node grouping (cluster by type)
   - Filtering (show only healthy, only critical, etc.)
   - Search/highlight (find specific primal)

3. **Refine Audio**
   - Better instrument synthesis
   - More nuanced health → pitch mapping
   - Ambient vs alert modes
   - Audio themes (calm, urgent, analytical)
   - Volume mixing (prevent cacophony)

4. **Polish Interactions**
   - Smoother animations
   - Tooltip on hover
   - Node details panel
   - History/timeline (see past states)
   - Export graph (PNG, SVG, JSON)

5. **Expand Testing**
   - Integration tests with mock server
   - Performance benchmarks (1K+ nodes)
   - Accessibility testing (actual blind users)
   - Cross-platform (Linux, macOS, Windows)

6. **Documentation**
   - User guide
   - Video tutorials
   - Blog posts
   - Conference talk prep

**Success Criteria**:
- ✅ petalTongue is used regularly (dogfooding)
- ✅ No critical bugs found
- ✅ Performance is acceptable (60 FPS with 100+ nodes)
- ✅ Blind users validate audio is useful
- ✅ Team is confident in design decisions

**Duration**: 2-4 weeks of organic use and refinement

---

### Phase 5: Prepare for Next Systems (After Fermentation) 🚀

**Goal**: With petalTongue stable, begin RhizoCrypt/LoamSpine/SweetGrass.

**Prerequisites**:
- ✅ petalTongue production-deployed
- ✅ User feedback incorporated
- ✅ No major architectural changes planned
- ✅ Documentation complete
- ✅ Showcase ready for demos

**Next Systems**:
1. **RhizoCrypt** - Core DAG engine (3-4 weeks)
2. **LoamSpine** - Permanence semantics (2-3 weeks)
3. **SweetGrass** - Attribution queries (2-3 weeks)

**Timeline**:
- Week 1-2: Audio synthesis + sandbox complete
- Week 3-4: Showcase + initial fermentation
- Week 5-6: Continue fermentation, gather feedback
- Week 7-8: Final polish, production deployment
- Week 9+: Begin RhizoCrypt

---

## 📋 Immediate Next Steps (This Week)

### Day 1: Audio Synthesis 🎵
- [ ] Add `rodio` dependency
- [ ] Implement basic tone generation
- [ ] Map instruments to distinct sounds
- [ ] Wire up UI controls
- [ ] Test with all primal types
- [ ] Commit: "🎵 Add real audio synthesis"

### Day 2: Mock Server 🧪
- [ ] Create `sandbox/mock-biomeos/` server
- [ ] Implement `/api/v1/primals` endpoint
- [ ] Implement `/api/v1/topology` endpoint
- [ ] Add hot-reload for scenarios
- [ ] Test with petalTongue UI
- [ ] Commit: "🧪 Add mock BiomeOS server"

### Day 3: More Scenarios 📊
- [ ] Create `complex.json` (20+ primals)
- [ ] Create `dynamic.json` (evolving topology)
- [ ] Create `chaos.json` (high churn)
- [ ] Test audio with each scenario
- [ ] Document scenario creation
- [ ] Commit: "📊 Add complex scenarios"

### Day 4: Showcase Setup 🎬
- [ ] Create `showcase/` structure
- [ ] Build 5 demo scenarios
- [ ] Create presenter scripts
- [ ] Take screenshots
- [ ] Write presenter notes
- [ ] Commit: "🎬 Add showcase demos"

### Day 5-7: Polish & Document ✨
- [ ] Record demo videos
- [ ] Update README with screenshots
- [ ] Write blog post draft
- [ ] Test on multiple machines
- [ ] Gather initial feedback
- [ ] Deploy to production

---

## 🎯 Success Metrics

### Audio Synthesis
- **Functional**: Sound plays through speakers
- **Distinct**: Each instrument type recognizable
- **Informative**: Health states audibly different
- **Accessible**: Blind users can interpret

### Sandbox
- **Isolated**: Develop without full ecosystem
- **Realistic**: Mock behaviors match real APIs
- **Fast**: Instant feedback on changes
- **Documented**: Easy for others to use

### Showcase
- **Polished**: Professional demos ready
- **Compelling**: Story about accessibility clear
- **Reusable**: Can demo at conferences
- **Educational**: Onboards new users

### Fermentation
- **Stable**: No critical bugs
- **Performant**: 60 FPS with 100+ nodes
- **Validated**: Blind users find it useful
- **Confident**: Team ready to build next systems

---

## 🌟 Vision Alignment

This progression ensures:
1. **Audio synthesis completes the multi-modal promise**
2. **Sandbox enables rapid, isolated iteration**
3. **Showcase tells the accessibility story**
4. **Fermentation validates design decisions**
5. **Confidence to proceed to DAG layer**

We're **not rushing** - we're letting petalTongue mature naturally while proving the universal representation concept.

---

## 📅 Estimated Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| Audio Synthesis | 2-3 hours | ⏸️ Next |
| Complete Sandbox | 1-2 hours | ⏸️ After Audio |
| Create Showcase | 2-3 hours | ⏸️ After Sandbox |
| Fermentation | 2-4 weeks | ⏸️ Ongoing |
| Next Systems | Week 9+ | ⏸️ After Ferment |

**Total before DAG layer**: ~1-2 weeks of focused work + 2-4 weeks organic use

---

## 🎉 Today's Achievement

We went from "How's our UI?" to a **production-ready foundation** with:
- ✅ Graph engine
- ✅ Visual renderer
- ✅ Audio renderer (attributes)
- ✅ UI integration
- ✅ BiomeOS client
- ✅ Comprehensive docs
- ✅ Sandbox structure

**Next**: Complete the audio → showcase → ferment → evolve progression! 🌸

---

*petalTongue: Growing organically, one layer at a time.* 🌱


