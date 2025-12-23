# 🌱 petalTongue Fermentation Guide

**Letting it mature through real-world use**

---

## 🎯 What is Fermentation?

Fermentation is the **2-4 week period** after initial development where we:
- Use petalTongue daily in real scenarios
- Gather feedback from actual users
- Refine designs based on experience
- Validate architectural decisions
- Build confidence before major abstractions

**Philosophy**: "Don't rush to abstract. Let concrete implementations prove themselves first."

Inspired by the sourDough primal - good bread takes time to ferment. Good software does too.

---

## 📅 Timeline

**Duration**: Weeks 3-6 (after Month 1 completion)  
**Current Phase**: Ready to begin ✅  
**Status**: Foundations complete, ready for daily use

---

## 🌟 Fermentation Goals

### Primary Goals

1. **Confidence in Design**
   - Prove the graph engine abstraction is correct
   - Validate the separation of renderers
   - Confirm BiomeOS integration works smoothly
   - Test layout algorithms with varied topologies

2. **Performance Validation**
   - Measure FPS with 50, 100, 200+ nodes
   - Identify bottlenecks
   - Optimize hot paths
   - Ensure smooth experience at scale

3. **Accessibility Validation**
   - Test with actual blind users
   - Refine audio mappings based on feedback
   - Validate that sonification is informative
   - Iterate on instrument/pitch/pan choices

4. **User Experience Refinement**
   - Identify pain points in daily use
   - Improve defaults (zoom, layout, colors)
   - Add missing features users actually need
   - Remove features users don't use

5. **Robustness**
   - Find and fix edge cases
   - Improve error messages
   - Handle network failures gracefully
   - Validate with chaotic scenarios

---

## 📋 Daily Activities

### Week 1: Initial Fermentation

**Monday-Tuesday**: Setup Daily Use
- [ ] Point petalTongue at live BiomeOS (when available)
- [ ] Set up auto-launch on system boot
- [ ] Create shortcuts for quick access
- [ ] Document first impressions

**Wednesday-Thursday**: Performance Baseline
- [ ] Benchmark with 10, 20, 50 node scenarios
- [ ] Measure FPS, memory, CPU
- [ ] Profile with `cargo flamegraph`
- [ ] Document baseline metrics

**Friday**: Community Setup
- [ ] Announce on ecoPrimals channels
- [ ] Write "How to Contribute" guide
- [ ] Tag "good first issues" on GitHub
- [ ] Respond to any early feedback

### Week 2: Refinement

**Monday-Wednesday**: User Testing
- [ ] Recruit 3-5 test users (devs, SREs)
- [ ] Walk them through showcase demos
- [ ] Observe where they struggle
- [ ] Document pain points

**Thursday**: Accessibility Testing
- [ ] Recruit 1-2 blind users (if possible)
- [ ] Test audio descriptions (text-to-speech)
- [ ] Validate sonification concepts
- [ ] Iterate on mappings

**Friday**: Iteration
- [ ] Fix top 5 issues from user testing
- [ ] Improve UX based on observations
- [ ] Update docs based on confusion points

### Week 3: Optimization

**Monday-Tuesday**: Performance Tuning
- [ ] Address identified bottlenecks
- [ ] Optimize layout algorithms
- [ ] Add edge bundling (if needed)
- [ ] Test improvements

**Wednesday-Thursday**: Feature Refinement
- [ ] Add most-requested features
- [ ] Improve auto-refresh UX
- [ ] Enhance layout switching
- [ ] Polish visual details

**Friday**: Documentation
- [ ] Update README with learnings
- [ ] Write performance optimization guide
- [ ] Document known limitations
- [ ] Update roadmap

### Week 4: Stabilization

**Monday-Wednesday**: Bug Fixing
- [ ] Fix all critical bugs
- [ ] Address edge cases
- [ ] Improve error handling
- [ ] Expand test coverage

**Thursday**: Cross-Platform Testing
- [ ] Test on macOS (if available)
- [ ] Test on Windows (if available)
- [ ] Document platform differences
- [ ] Fix platform-specific issues

**Friday**: Fermentation Complete
- [ ] Write fermentation retrospective
- [ ] Update STATUS.md
- [ ] Plan Month 3 (Abstraction)
- [ ] Celebrate! 🎉

---

## 📊 Metrics to Track

### Performance Metrics

| Metric | Target | Current | Notes |
|--------|--------|---------|-------|
| FPS (10 nodes) | 60 | TBD | Should be trivial |
| FPS (50 nodes) | 60 | TBD | Current stress test |
| FPS (100 nodes) | 45+ | TBD | Acceptable if smooth |
| FPS (200 nodes) | 30+ | TBD | Stretch goal |
| Memory (50 nodes) | <100 MB | ~50 MB | Current baseline |
| Initial load time | <2s | <2s | Already fast |
| Layout stabilization | <5s | ~10s | Could optimize |

### User Experience Metrics

- **Time to first insight**: How long until user understands graph?
- **Task completion rate**: Can users find critical nodes?
- **Error recovery**: Can users recover from mistakes?
- **Subjective satisfaction**: 1-10 rating from users

### Accessibility Metrics

- **Audio description clarity**: Do blind users understand?
- **Health state recognition**: Can they identify warnings/critical?
- **Spatial awareness**: Can they navigate the graph by sound?
- **Time to detection**: How long to find a critical node?

---

## 🎯 Success Criteria

### Must Have (Before Month 3)

- ✅ **Performance**: 60 FPS with 50 nodes, 45+ FPS with 100 nodes
- ✅ **Stability**: No crashes or freezes during 1 hour session
- ✅ **Usability**: 3+ users can complete showcase demos independently
- ✅ **Accessibility**: 1+ blind user validates audio is informative
- ✅ **Documentation**: All major features documented clearly

### Nice to Have

- ✅ Cross-platform tested (macOS, Windows)
- ✅ 5+ GitHub stars / community interest
- ✅ 3+ external contributors
- ✅ Blog post or conference talk submitted

### Red Flags (Requires Rework)

- ❌ FPS drops below 30 with 50 nodes (performance issue)
- ❌ Blind users can't distinguish health states (audio mapping issue)
- ❌ Users consistently struggle with basic tasks (UX issue)
- ❌ Major architectural changes needed (design issue)

---

## 🧪 Test Scenarios for Fermentation

### Scenario 1: Healthy Ecosystem
- **Use**: `sandbox/scenarios/simple.json`
- **Focus**: Basic navigation, layout switching, node selection
- **Duration**: 5 minutes
- **Success**: User can identify all 5 primals and their connections

### Scenario 2: Degraded System
- **Use**: `sandbox/scenarios/unhealthy.json`
- **Focus**: Health state recognition, visual + audio encoding
- **Duration**: 5 minutes
- **Success**: User can identify critical nodes immediately

### Scenario 3: Production Scale
- **Use**: `sandbox/scenarios/performance.json` (50 nodes)
- **Focus**: Performance, scalability, layout algorithms
- **Duration**: 10 minutes
- **Success**: Smooth interactions, no lag, can find specific nodes

### Scenario 4: Dynamic Changes
- **Use**: Edit scenarios during runtime (hot-reload)
- **Focus**: Real-time updates, auto-refresh
- **Duration**: 5 minutes
- **Success**: Changes reflected within 5s

### Scenario 5: Chaos Testing
- **Use**: Rapid scenario switching, zoom extremes, rapid clicks
- **Focus**: Robustness, error handling
- **Duration**: 5 minutes
- **Success**: No crashes, graceful degradation

---

## 🗣️ User Feedback Collection

### Interview Script

**Opening**:
> "Thanks for testing petalTongue. This is a monitoring tool for distributed systems. I'll show you a quick demo, then I'd like you to try it yourself while I observe. Feel free to think aloud."

**Demo** (2 minutes):
- Show simple.json
- Demonstrate pan, zoom, select
- Switch layouts once
- Point out health colors

**Tasks** (10 minutes):
1. "Find the Squirrel AI primal and tell me its health state."
2. "Show me all connections from the Songbird node."
3. "Switch to a different layout. Which do you prefer?"
4. "Find a critical node. What makes it stand out?"
5. "Explore freely. What do you notice?"

**Questions** (5 minutes):
1. "On a scale of 1-10, how easy was it to navigate?"
2. "What was confusing or frustrating?"
3. "What would you change?"
4. "Would you use this to monitor your systems?"
5. "Anything surprising or delightful?"

**Closing**:
> "Thank you! Your feedback is invaluable. We'll iterate based on what we learned today."

---

## 🔧 Common Issues and Fixes

### Performance Issues

**Issue**: FPS drops with 50+ nodes  
**Diagnosis**: Profile with `cargo flamegraph`  
**Fixes**:
- Reduce edge rendering complexity
- Add edge bundling
- Optimize layout algorithm iterations
- Use WebGL rendering (future)

**Issue**: Layout takes too long to stabilize  
**Diagnosis**: Force-directed algorithm iterations  
**Fixes**:
- Reduce max iterations
- Add adaptive convergence detection
- Use simpler layout as default
- Pre-compute good starting positions

### UX Issues

**Issue**: Users can't find specific nodes  
**Diagnosis**: No search/filter feature  
**Fixes**:
- Add search box
- Highlight nodes matching query
- Add filter by type/health
- Show node list panel

**Issue**: Graph is cluttered at scale  
**Diagnosis**: Too many overlapping elements  
**Fixes**:
- Add edge bundling
- Group nodes by type
- Add zoom-dependent rendering (LOD)
- Improve layout spacing

### Accessibility Issues

**Issue**: Audio descriptions unclear  
**Diagnosis**: Mapping not intuitive  
**Fixes**:
- Iterate on instrument choices
- Test with blind users
- Add audio descriptions tutorial
- Provide customization options

---

## 📝 Fermentation Journal

### Template

Create `fermentation-journal.md` and update daily:

```markdown
# petalTongue Fermentation Journal

## Week 1

### Day 1 (YYYY-MM-DD)
**What I did**: 
**What I learned**: 
**What surprised me**: 
**What needs fixing**: 
**Ideas for improvement**: 

### Day 2 (YYYY-MM-DD)
...
```

This journal will be invaluable when writing the retrospective.

---

## 🎓 Learning Goals

### For the Team

- **Validate assumptions**: Did the modality-agnostic architecture work?
- **Understand users**: What do they actually need vs what we thought?
- **Performance insights**: Where are the bottlenecks?
- **Accessibility lessons**: What makes audio representations effective?
- **Prioritization**: What features matter most?

### For the Community

- **Documentation**: What do new users struggle with?
- **Onboarding**: How quickly can someone be productive?
- **Contribution**: What are good first issues?
- **Feedback**: What excites people? What disappoints?

---

## 🌟 After Fermentation

### Retrospective Questions

1. **Design**: Was the graph engine abstraction correct?
2. **Performance**: Does it scale to production use?
3. **Accessibility**: Is the audio approach viable?
4. **UX**: Do users find it intuitive?
5. **Architecture**: Ready to abstract to `RepresentationModality` trait?

### Decision Points

**If successful**:
- Proceed to Month 3 (Abstraction)
- Expand to VR/AR/Haptic renderers
- Consider separating as independent primal

**If issues found**:
- Extend fermentation another 2 weeks
- Address fundamental issues before abstracting
- Pivot if necessary (rare)

---

## 💡 Fermentation Principles

1. **Use it daily**: Dogfooding reveals issues docs can't
2. **Listen to users**: They'll find things you never imagined
3. **Measure everything**: Data guides decisions
4. **Iterate quickly**: Fix issues as soon as they're found
5. **Document learnings**: Future you will thank present you
6. **Stay humble**: Early assumptions are often wrong
7. **Enjoy the process**: Fermentation is where good software becomes great

---

## 🎉 Celebrate Small Wins

Fermentation is gradual. Celebrate progress:
- ✅ First blind user validation
- ✅ First external contributor
- ✅ First 100-node stress test passing
- ✅ First user says "This is awesome!"
- ✅ First bug found and fixed same day

---

**Remember**: Good bread takes time. Good software takes time. Trust the process. 🌱

---

**Status**: Ready to begin  
**Duration**: 2-4 weeks  
**Next Review**: After Week 2

