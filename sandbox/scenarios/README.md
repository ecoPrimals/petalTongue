# Sandbox Scenarios

This directory contains demonstration scenarios for `petalTongue` showcase mode.

## Usage

```bash
# Use default scenario (simple.json)
SHOWCASE_MODE=true ./petal-tongue

# Use specific scenario
SHOWCASE_MODE=true SANDBOX_SCENARIO=complex ./petal-tongue

# Use chaos testing scenario
SHOWCASE_MODE=true SANDBOX_SCENARIO=chaos ./petal-tongue
```

## Available Scenarios

### `simple.json` (Default)
- **Primals**: 5 (BearDog, ToadStool, Songbird, NestGate, Squirrel)
- **Status**: All healthy
- **Purpose**: Basic demonstration of ecosystem
- **Use Case**: First-time users, screenshots, documentation

### `complex.json`
- **Primals**: 10 (includes Federation, multiple compute/storage nodes)
- **Status**: Mostly healthy, 1 degraded (remote tower)
- **Purpose**: Advanced topology with trust relationships
- **Use Case**: Testing graph layout, relationship visualization, trust UI

### `chaos.json`
- **Primals**: 8 (various health states)
- **Status**: Multiple failures (critical, degraded, offline, unstable)
- **Purpose**: Chaos testing, failure visualization, alert systems
- **Use Case**: Testing error handling, health indicators, graceful degradation

## Scenario Format

```json
{
  "name": "Scenario Name",
  "description": "Human-readable description",
  "primals": [
    {
      "id": "unique-id",
      "name": "Display Name",
      "primal_type": "Security|Compute|Storage|etc",
      "endpoint": "http://localhost:port or 'self'",
      "capabilities": ["cap1", "cap2"],
      "health": "healthy|degraded|unhealthy|critical",
      "last_seen": <unix_timestamp>
    }
  ],
  "topology": [
    {
      "from": "primal_id",
      "to": "primal_id",
      "edge_type": "relationship_type",
      "label": "Optional Label" 
    }
  ]
}
```

## Health States

- **`healthy`**: Green, fully operational
- **`degraded`**: Yellow, working but issues detected
- **`unhealthy`**: Orange, significant problems
- **`critical`**: Red, failing or offline

## Creating New Scenarios

1. Copy an existing scenario as template
2. Modify primals and topology
3. Use meaningful IDs and descriptive names
4. Set appropriate `last_seen` timestamps:
   - Current: `date +%s`
   - 10 min ago: `date -d '10 minutes ago' +%s`
   - 1 hour ago: `date -d '1 hour ago' +%s`
5. Test with: `SHOWCASE_MODE=true SANDBOX_SCENARIO=your-scenario ./petal-tongue`

## Integration with Testing

Scenarios can be used for:
- **Visual regression testing**: Compare graph renderings
- **Performance testing**: Large topologies (50+ nodes)
- **Accessibility testing**: Verify color schemes with different health states
- **Audio testing**: Different states trigger different tones

## Directory Structure

```
sandbox/
├── scenarios/
│   ├── simple.json                   (default, 5 primals)
│   ├── complex.json                  (10 primals, advanced)
│   ├── chaos.json                    (failures & errors)
│   ├── doom-*.json                   (5 game/rendering scenarios)
│   ├── healthspring-*.json           (6 clinical domain scenarios)
│   ├── clinical-trt-*.json           (5 clinical trial scenarios)
│   ├── full-dashboard.json           (multi-panel dashboard)
│   ├── live-ecosystem.json           (real ecosystem topology)
│   ├── performance.json              (stress testing)
│   ├── graph-studio.json             (graph layout exploration)
│   ├── neural-api-test.json          (Neural API testing)
│   ├── paint-simple.json             (painting/creative)
│   ├── rootpulse-demo.json           (rootpulse domain)
│   ├── trust-demo.json               (trust elevation flows)
│   ├── unhealthy.json                (degraded states)
│   └── README.md                     (this file)
├── scripts/
│   └── start-mock.sh     (sandbox biomeOS server launcher)
└── mock-biomeos/
    └── (standalone mock biomeOS server for offline development)
```

## Environment Variables

- `SHOWCASE_MODE`: Set to `true` to enable sandbox mode
- `SANDBOX_SCENARIO`: Name of scenario file (without `.json`)
- `PETALTONGUE_SANDBOX_DIR`: Custom sandbox directory path

## Future Scenarios (Ideas)

- `performance-large.json`: 50+ primals for stress testing
- `security-breach.json`: Demonstrates security incidents
- `federation-multi-tower.json`: Multiple tower federation
- `ai-heavy.json`: Many AI/ML primals with GPU workloads
- `minimal.json`: Just 2 primals for minimal demos
- `progressive-trust.json`: Demonstrates trust elevation flows

## Tips

1. **Realistic Timestamps**: Use actual `date +%s` for "now" states
2. **Meaningful Labels**: Edge labels help explain relationships
3. **Varied Health**: Mix health states to test UI robustness
4. **Capability Diversity**: Show different primal types and capabilities
5. **Documentation**: Update this README when adding scenarios

## Contributing

When adding new scenarios:
1. Follow the JSON format above
2. Test thoroughly with showcase mode
3. Update this README with description
4. Add use cases and purpose
5. Consider accessibility (color-blind users)

---

**Status**: 27 scenarios available  
**Last Updated**: April 2, 2026  
**Maintainer**: petalTongue team

