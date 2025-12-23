# 08 - Integration Testing

**Duration**: 30-45 minutes  
**Purpose**: End-to-end validation of petalTongue ↔ BiomeOS ↔ Primals integration

---

## 🎯 What This Demo Does

1. **Tests complete data pipeline** (Primal → BiomeOS → petalTongue → User)
2. **Validates API contracts** (do responses match expectations?)
3. **Checks error handling** (what happens when things fail?)
4. **Verifies state consistency** (is everyone agreeing on truth?)
5. **Ensures production readiness** (can we ship this?)

**Goal**: Confidence in full-stack integration, no surprises in production.

---

## 🚀 Quick Start

```bash
./demo.sh
```

This will run comprehensive integration tests.

---

## 📋 Prerequisites

- Completed scenarios 00-07
- BiomeOS running
- Multiple primals running
- petalTongue UI open
- Understanding of API contracts

---

## 🎬 Demo Flow

### Phase 1: API Contract Validation

**Objective**: Ensure all APIs return expected data structures

#### Test 1: `/api/v1/primals` Endpoint

```bash
curl http://localhost:3000/api/v1/primals | jq
```

**Validate**:
- Returns array of primals
- Each primal has: `id`, `name`, `type`, `endpoint`, `capabilities`, `health`, `last_seen`
- Health is one of: "Healthy", "Warning", "Critical", "Unknown"
- Last_seen is recent timestamp

**petalTongue must handle**: All fields correctly, graceful degradation if fields missing

#### Test 2: `/api/v1/topology` Endpoint

```bash
curl http://localhost:3000/api/v1/topology | jq
```

**Validate**:
- Returns array of edges
- Each edge has: `from`, `to`, `type`, optional `label`
- From/to match primal IDs
- Type describes relationship

**petalTongue must handle**: Edge rendering, unknown types, dangling references

#### Test 3: Error Responses

```bash
curl -X GET http://localhost:3000/api/v1/nonexistent
```

**Validate**:
- Returns 404
- Error message is clear
- No crash

**petalTongue must handle**: Network errors, 404s, 500s, timeouts

### Phase 2: State Consistency

**Objective**: Ensure all components agree on ecosystem state

#### Test: Compare Sources

1. **BiomeOS view**:
   ```bash
   curl http://localhost:3000/api/v1/primals | jq '[.[] | .name]'
   ```

2. **petalTongue view**:
   - Count nodes in UI
   - List node names

3. **Actual processes**:
   ```bash
   ps aux | grep -E "beardog|nestgate|songbird|toadstool|squirrel" | grep -v grep
   ```

**Validation**:
- All three match
- No phantom nodes (in UI but not running)
- No missing nodes (running but not in UI)

**If mismatch**: Root cause investigation required

### Phase 3: Error Injection

**Objective**: Test resilience to failures

#### Scenario 1: BiomeOS Unavailable

```bash
# Stop BiomeOS temporarily
systemctl stop biomeos  # or kill process
```

**petalTongue should**:
- Show "Connection Lost" indicator
- Keep displaying last known state
- Retry connection automatically
- Not crash

**Restore BiomeOS, validate**: Connection restored, data fresh

#### Scenario 2: Malformed API Response

**Simulate**: BiomeOS returns invalid JSON

**petalTongue should**:
- Log error
- Show error message to user
- Not crash
- Retry on next refresh

#### Scenario 3: Partial API Failure

**Simulate**: `/primals` works, `/topology` returns 500

**petalTongue should**:
- Display nodes (from working `/primals`)
- Show error for edges
- Still usable (degraded mode)

### Phase 4: Load and Timing

**Objective**: Ensure integration handles load

#### Test 1: Rapid Refresh

Set auto-refresh to 1 second, run for 5 minutes.

**Validate**:
- No memory leak
- No excessive CPU
- No dropped requests
- UI remains responsive

#### Test 2: Concurrent Users

Open 5 petalTongue tabs simultaneously.

**Validate**:
- All tabs work independently
- BiomeOS handles load
- No race conditions
- Data consistency across tabs

### Phase 5: End-to-End Workflow

**Objective**: Validate complete operational workflow

#### Workflow: Deploy → Monitor → Incident → Resolve

1. **Deploy**: Add new primal
   - petalTongue shows new node within 10s

2. **Monitor**: Watch for 5 minutes
   - All health data updates
   - Connections visible
   - No errors

3. **Incident**: Kill a primal
   - Failure detected within 15s
   - Visual feedback clear
   - Cascade effects visible

4. **Resolve**: Restart primal
   - Recovery detected within 15s
   - Node returns to green
   - Ecosystem stable

**If any step fails**: Integration gap exists, document in GAPS.md

---

## ✅ Success Criteria

After all tests, you should have validated:

- [x] All APIs return expected data structures
- [x] petalTongue handles errors gracefully
- [x] State is consistent across components
- [x] Performance under load is acceptable
- [x] End-to-end workflows are smooth
- [x] Production readiness confirmed

---

## 🌱 Fermentation Notes

### Integration Gaps to Watch For

- **API Versioning**: What if BiomeOS updates API?
- **Schema Evolution**: What if new fields are added?
- **Backwards Compatibility**: Can old petalTongue work with new BiomeOS?
- **Network Flakiness**: How does it handle intermittent connectivity?
- **Clock Skew**: What if timestamps are off?
- **Large Responses**: Can it handle 1000s of primals in API response?

**Document ALL integration gaps in**: `../GAPS.md`

---

## 💡 Integration Best Practices

### API Design

**Versioning**: `/api/v1/` allows future changes  
**Consistency**: Same structure for all primals  
**Extensibility**: Optional fields for backwards compatibility  
**Errors**: Clear, actionable error messages

### Client Resilience

**Retries**: Automatic retry with exponential backoff  
**Timeouts**: Don't hang forever  
**Fallbacks**: Cached data if API unavailable  
**Graceful Degradation**: Partial data > no data

### Testing Strategy

**Unit Tests**: Individual components  
**Integration Tests**: Component pairs  
**End-to-End Tests**: Full stack  
**Chaos Tests**: Random failures

---

## ⏭️ Next Steps

**Congratulations!** You've completed all 8 fermentation scenarios!

### Fermentation Retrospective

1. Review `../GAPS.md` - What needs fixing?
2. Prioritize gaps - Critical vs. nice-to-have
3. Plan fixes - Sprint planning for next phase
4. Document learnings - Update README, STATUS

### Move to Next Phase

After fermentation (Month 2), proceed to:

**Month 3: Abstraction**
- Refactor concrete implementations to interfaces
- Make modalities pluggable
- Extract rendering engines
- Build capability system

**Month 4+: Expansion**
- New modalities (VR, haptic, etc.)
- Advanced features
- Performance optimizations
- Production deployment

---

## 🎓 Final Reflection

### What We've Built

Through 8 fermentation scenarios, we've:
1. **Infrastructure** - Setup, testing, validation
2. **Core Functionality** - Discovery, topology, health
3. **Accessibility** - Audio, keyboard, screen readers
4. **Performance** - Benchmarking, limits
5. **Real-World** - Operational scenarios
6. **Integration** - End-to-end validation

### What We've Learned

- petalTongue works with real primals ✅
- Performance is acceptable for typical ecosystems ✅
- Accessibility needs more work ⚠️
- Integration is solid but has edge cases ⚠️
- Production-ready with documented limitations ✅

### What's Next

Fermentation revealed gaps. Now we evolve:
- Address high-priority gaps
- Refactor for abstraction
- Add new modalities
- Harden for production

---

**Status**: 🌱 Complete!  
**Complexity**: Very High (full-stack testing)  
**Dependencies**: All scenarios 00-07  
**Learning Value**: Very High (production confidence)

---

*Integration is where theory meets reality.  
Testing is where confidence is earned.  
Fermentation is where software matures.* 🌸

**CONGRATULATIONS ON COMPLETING FERMENTATION!** 🎉

