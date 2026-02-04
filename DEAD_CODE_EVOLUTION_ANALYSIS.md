//! Dead Code Evolution - Implementation Plan
//!
//! Analysis of dead code fields and evolution strategy

/// Dead Code Field: data_providers
///
/// Location: app.rs:66
/// 
/// Status: KEEP - This is forward-looking architecture
/// 
/// Justification:
/// - Part of multi-provider data aggregation system
/// - Will be used when we support multiple visualization data sources
/// - Not dead code, just future-facing architecture
/// - Already has discover_visualization_providers() infrastructure
///
/// Evolution Plan:
/// - Phase 1: Keep with #[allow(dead_code)] (current)
/// - Phase 2: Implement aggregation logic (Q2 2026)
/// - Phase 3: Active use with multiple providers
///
/// No action needed: This is intentional future-proofing

/// Dead Code Field: session_manager
///
/// Location: app.rs:131
///
/// Status: KEEP - Session persistence is planned
///
/// Justification:
/// - Session persistence system exists (SessionManager type)
/// - Will be activated when persistence feature is enabled
/// - Infrastructure ready, just not enabled yet
/// - User data sovereignty feature (important!)
///
/// Evolution Plan:
/// - Phase 1: Keep with #[allow(dead_code)] (current)
/// - Phase 2: Enable session persistence (Q1 2026)
/// - Phase 3: Full state save/restore
///
/// No action needed: Feature implementation in progress

/// Dead Code Field: instance_id
///
/// Location: app.rs:134
///
/// Status: KEEP - Multi-instance coordination planned
///
/// Justification:
/// - Required for multi-instance coordination
/// - InstanceId type already exists in core
/// - Will be used for collaborative editing
/// - Part of broader multi-user vision
///
/// Evolution Plan:
/// - Phase 1: Keep with #[allow(dead_code)] (current)
/// - Phase 2: Implement instance tracking (Q2 2026)
/// - Phase 3: Multi-instance graph synchronization
///
/// No action needed: Future architecture

/// Dead Code Field: bridge (in toadstool_bridge.rs)
///
/// Location: toadstool_bridge.rs:170
///
/// Status: REMOVE - This is superseded by new Toadstool integration
///
/// Justification:
/// - toadstool_bridge.rs is legacy Python bridge
/// - New Toadstool integration uses tarpc (display/backends/toadstool_v2.rs)
/// - Python bridge is no longer needed
/// - Technical debt from previous architecture
///
/// Evolution Plan:
/// - Immediate: Remove toadstool_bridge.rs entirely
/// - Reason: Superseded by modern tarpc integration
///
/// Action: REMOVE toadstool_bridge.rs (deprecated module)

/// Dead Code Fields: time_labels, time_axis (in graph_metrics_plotter.rs)
///
/// Locations: graph_metrics_plotter.rs:15, :29
///
/// Status: KEEP - Time-series plotting planned
///
/// Justification:
/// - Required for time-based metric visualization
/// - Part of metrics dashboard evolution
/// - Will be used for historical data plots
/// - Missing feature, not dead code
///
/// Evolution Plan:
/// - Phase 1: Keep with #[allow(dead_code)] (current)
/// - Phase 2: Implement time-axis visualization (Q1 2026)
/// - Phase 3: Historical metric tracking
///
/// No action needed: Feature implementation pending

/// Dead Code Field: show_system_processes (in process_viewer_integration.rs)
///
/// Location: process_viewer_integration.rs:33
///
/// Status: KEEP - Feature toggle planned
///
/// Justification:
/// - UI toggle for showing/hiding system processes
/// - Makes sense as a filter option
/// - Common pattern in process viewers
/// - Small feature, easy to implement
///
/// Evolution Plan:
/// - Immediate: Implement toggle in UI
/// - Add checkbox in process viewer panel
/// - Filter processes based on flag
///
/// Action: IMPLEMENT (simple UI toggle)

//---

## Summary

Total Fields: 7

**KEEP (6 fields)** - Future-facing architecture:
1. data_providers - Multi-provider aggregation
2. session_manager - Session persistence
3. instance_id - Multi-instance coordination
4. time_labels - Time-series metrics
5. time_axis - Time-based plotting
6. show_system_processes - UI toggle (easy to implement)

**REMOVE (1 field)** - Superseded:
1. bridge (toadstool_bridge.rs) - Old Python bridge, superseded by tarpc

## Actions

1. Remove toadstool_bridge.rs (deprecated)
2. Implement show_system_processes toggle (15 minutes)
3. Document remaining fields as intentional future-proofing

## Philosophy

"Dead code" is often future-proofing. We distinguish:
- **Technical Debt**: Code that should never have existed → REMOVE
- **Forward Architecture**: Code waiting for features → KEEP & DOCUMENT

All 6 kept fields are forward-looking architecture with clear evolution plans.
Only 1 field (bridge) is true technical debt from architecture migration.
