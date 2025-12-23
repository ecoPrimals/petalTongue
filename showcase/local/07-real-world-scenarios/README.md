# 07 - Real-World Scenarios

**Duration**: 45-60 minutes  
**Purpose**: Test petalTongue with production-like primal behaviors and realistic operational scenarios

---

## 🎯 What This Demo Does

1. **Simulates real operational scenarios** (deploy, scale, incident, maintenance)
2. **Uses actual primal binaries** (not mocks)
3. **Tests end-to-end workflows** (detection → response → validation)
4. **Validates production readiness** (would you trust this in prod?)
5. **Discovers real-world gaps** that theoretical tests miss

**Goal**: Confidence that petalTongue works in actual production environments.

---

## 🚀 Quick Start

```bash
./demo.sh
```

This will guide you through realistic operational scenarios.

---

## 📋 Prerequisites

- Completed scenarios 00-06
- BiomeOS running
- Actual primal binaries available (`biomeOS/bin/primals/`)
- petalTongue UI open
- ~1 hour of uninterrupted time

---

## 🎬 Demo Flow

### Scenario 1: New Service Deployment

**Situation**: Deploy a new primal to production

#### Timeline

1. **T+0**: Healthy ecosystem (5 primals)
2. **T+1m**: Deploy new Squirrel primal
3. **T+2m**: Squirrel discovered, appears in graph
4. **T+3m**: Connections form to existing primals
5. **T+5m**: Ecosystem stabilizes

#### What to Observe

- **Discovery latency**: How long until Squirrel visible?
- **Connection formation**: Edges appear automatically?
- **Layout adaptation**: Graph re-arranges smoothly?
- **No disruption**: Other primals unaffected?

#### Validation

- [ ] New primal visible within 10 seconds
- [ ] Connections form automatically
- [ ] Layout adjusts without jarring jumps
- [ ] Existing primals remain healthy

### Scenario 2: Service Scaling

**Situation**: Scale NestGate from 1 → 3 replicas

#### Timeline

1. **T+0**: Single NestGate instance
2. **T+1m**: Launch NestGate-2
3. **T+2m**: Launch NestGate-3
4. **T+5m**: Load balanced across 3 instances

#### What to Observe

- **Multiple instances**: Do all 3 appear in graph?
- **Load distribution**: Can you see traffic patterns?
- **Health monitoring**: All 3 reporting separately?
- **Replica naming**: Clear which is which?

#### Validation

- [ ] All replicas visible
- [ ] Health tracked independently
- [ ] Connections show load distribution
- [ ] Easy to identify instances

### Scenario 3: Incident Response

**Situation**: BearDog crashes, must be detected and restored

#### Timeline

1. **T+0**: Healthy ecosystem
2. **T+1m**: Kill BearDog process (`kill -9`)
3. **T+2m**: BearDog disappears from graph
4. **T+3m**: Dependent services show warnings
5. **T+5m**: Restart BearDog
6. **T+7m**: BearDog returns, health restored

#### What to Observe

- **Failure detection**: How long until BearDog goes gray/red?
- **Cascade effects**: Do other primals degrade?
- **Alert visibility**: Is the problem obvious?
- **Recovery tracking**: BearDog return visible?

#### Validation

- [ ] Failure detected within 15 seconds
- [ ] Cascading effects visible
- [ ] Problem root cause obvious (BearDog)
- [ ] Recovery clearly shown

### Scenario 4: Planned Maintenance

**Situation**: Rolling restart of all primals (zero-downtime upgrade)

#### Timeline

1. **T+0**: Healthy ecosystem
2. **T+5m**: Drain + restart BearDog
3. **T+10m**: Drain + restart NestGate
4. **T+15m**: Drain + restart Songbird
5. **T+20m**: All services upgraded, healthy

#### What to Observe

- **Drain process**: Primal goes warning during drain?
- **Brief disappearance**: Node vanishes, then returns?
- **Version tracking**: Can you see version change? (if supported)
- **Zero downtime**: Other primals compensate?

#### Validation

- [ ] Drain visible (warning state)
- [ ] Restart visible (disappear + reappear)
- [ ] Other services remain healthy
- [ ] Process completes smoothly

### Scenario 5: Network Partition

**Situation**: Simulate network split (primal can't reach others)

#### Timeline

1. **T+0**: Healthy ecosystem
2. **T+1m**: Block NestGate network (`iptables` or similar)
3. **T+2m**: NestGate shows as disconnected
4. **T+3m**: Other primals can't reach NestGate
5. **T+5m**: Restore network
6. **T+7m**: Connections re-establish

#### What to Observe

- **Partition detection**: NestGate goes gray/disconnected?
- **Split topology**: Two separate graphs?
- **Partial functionality**: Some primals still work?
- **Healing**: Edges reform on restore?

#### Validation

- [ ] Network issue detectable (not health issue)
- [ ] Topology reflects split
- [ ] Recovery is visible
- [ ] No false positives

### Scenario 6: Load Spike

**Situation**: Sudden traffic surge (Black Friday scenario)

#### Timeline

1. **T+0**: Normal load (all green)
2. **T+1m**: Inject load (stress test tool)
3. **T+2m**: ToadStool goes yellow (high CPU)
4. **T+3m**: ToadStool goes red (saturated)
5. **T+5m**: Scale ToadStool (add replica)
6. **T+7m**: Load distributed, health restored

#### What to Observe

- **Degradation visibility**: ToadStool color changes?
- **Bottleneck identification**: Easy to see ToadStool is problem?
- **Scaling effect**: New replica helps?
- **Real-time feedback**: See load redistribute?

#### Validation

- [ ] Load-induced degradation visible
- [ ] Bottleneck obvious
- [ ] Scaling solution verifiable
- [ ] Useful for capacity planning

---

## ✅ Success Criteria

After all scenarios, you should be able to say:

- [x] petalTongue handles real primal deployments
- [x] petalTongue detects actual failures
- [x] petalTongue supports operational workflows
- [x] petalTongue is useful for incident response
- [x] petalTongue is production-ready

---

## 🌱 Fermentation Notes

### Real-World Gaps

Real scenarios reveal gaps that tests miss:

- **Discovery edge cases**: What if primal starts then immediately crashes?
- **Timing issues**: What if multiple events happen simultaneously?
- **State synchronization**: What if BiomeOS and primal disagree on health?
- **Network flakiness**: What if discovery is intermittent?
- **Version skew**: What if BiomeOS and primal use different protocols?

**Document ALL real-world gaps in**: `../GAPS.md`

---

## 💡 Operational Workflows

### Incident Response

1. **Alert**: Monitoring system detects issue
2. **Visualize**: Open petalTongue
3. **Identify**: Find red node(s)
4. **Diagnose**: Check connections, health details
5. **Remediate**: Restart, scale, or fix
6. **Validate**: Watch node return to green

**petalTongue Goal**: Make steps 2-6 fast and obvious

### Capacity Planning

1. **Baseline**: Observe normal operation
2. **Load test**: Inject traffic
3. **Identify bottleneck**: Which node degrades first?
4. **Scale**: Add resources to bottleneck
5. **Re-test**: Verify improvement

**petalTongue Goal**: Visualize bottlenecks clearly

### Change Management

1. **Pre-change**: Snapshot healthy state
2. **Execute change**: Deploy, scale, config
3. **Monitor**: Watch for degradation
4. **Validate**: All green after change?
5. **Rollback if needed**: Return to snapshot

**petalTongue Goal**: Make before/after comparison obvious

---

## ⏭️ Next Steps

Once real-world scenarios are validated, proceed to:

```bash
cd ../08-integration-testing/
cat README.md
```

This will test **end-to-end integration** with BiomeOS and full ecosystem.

---

**Status**: 🌱 Ready to build  
**Complexity**: Very High (requires real primal orchestration)  
**Dependencies**: 00-06 complete, actual primals available  
**Learning Value**: Very High (production confidence)

---

*Theory is knowing. Practice is understanding.  
Real-world scenarios bridge the gap.* 🌸

