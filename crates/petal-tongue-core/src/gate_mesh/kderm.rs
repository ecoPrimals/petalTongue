// SPDX-License-Identifier: AGPL-3.0-or-later
//! K-Derm diderm topology — live visualization data.
//!
//! Maps the bacterial cell envelope model to network defense topology.
//! The 5-layer diderm architecture represents sovereign infrastructure
//! where outer membrane data reinforces inner membrane security.

use serde::Serialize;

/// K-Derm membrane layer in the diderm architecture.
///
/// Maps the bacterial cell envelope model to network defense topology.
/// Each layer has distinct security responsibilities and data flows.
#[derive(Debug, Clone, Serialize)]
pub struct KDermLayer {
    /// Layer name (e.g. "Outer membrane").
    pub name: &'static str,
    /// Concise description of what this layer does.
    pub role: &'static str,
    /// Components operating at this layer.
    pub components: &'static [&'static str],
    /// primals.eco routing path at this layer.
    pub path: &'static str,
    /// Security properties this layer provides.
    pub security: &'static [&'static str],
    /// Data flow direction (inbound, outbound, bidirectional).
    pub data_flow: &'static str,
}

/// The 5-layer K-Derm diderm topology (Wave 136b canonical).
///
/// Outer membrane data reinforces inner membrane. Cross-membrane validation
/// ensures integrity (content hash, timing baseline, TLS cert, DNS, routes).
pub const KDERM_LAYERS: &[KDermLayer] = &[
    KDermLayer {
        name: "Extracellular",
        role: "Public internet, hostile traffic",
        components: &["Crawlers", "Scanners", "Public DNS"],
        path: "Hostile traffic (unfiltered)",
        security: &["No trust", "Observable only"],
        data_flow: "inbound",
    },
    KDermLayer {
        name: "Outer membrane",
        role: "DDoS mitigation, CDN, TLS edge",
        components: &["Cloudflare proxy", "Caddy (TLS/CSP/HSTS)", "DNSSEC"],
        path: "lab.primals.eco → CF → Flint H1",
        security: &["DDoS absorb", "TLS termination", "WAF rules", "Cache"],
        data_flow: "bidirectional",
    },
    KDermLayer {
        name: "Periplasm",
        role: "Routing, relay, build pipeline",
        components: &[
            "golgi relay",
            "sporeGate CI",
            "WireGuard mesh",
            "songBird drawbridge",
        ],
        path: "WG overlay → sporeGate → LAN/WAN backends",
        security: &["Authenticated relay", "Signed builds", "Allowlist proxy"],
        data_flow: "bidirectional",
    },
    KDermLayer {
        name: "Plasma membrane",
        role: "Boundary enforcement, firewall",
        components: &[
            "Flint H1 (edge router)",
            "UFW per-gate",
            "fail2ban",
            "Port-forward rules",
        ],
        path: "NAT + firewall → per-gate ingress",
        security: &[
            "Stateful firewall",
            "Rate limiting",
            "Ban automation",
            "Port isolation",
        ],
        data_flow: "inbound",
    },
    KDermLayer {
        name: "Cytoplasm",
        role: "Sovereign compute, UDS IPC",
        components: &[
            "NUCLEUS primals",
            "Unix domain sockets",
            "songBird mesh",
            "bearDog TLS",
        ],
        path: "localhost-only services, mesh capability.call",
        security: &[
            "Zero external ports",
            "Capability-based routing",
            "Signed IPC",
            "Process isolation",
        ],
        data_flow: "internal",
    },
];

/// Hardening status for security controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardeningStatus {
    /// Fully active and validated.
    Active,
    /// Code landed, activation pending.
    Landed,
    /// In progress.
    InProgress,
    /// Planned but not started.
    Planned,
}

/// A security hardening control and its current state.
#[derive(Debug, Clone, Serialize)]
pub struct HardeningControl {
    /// Control identifier (e.g. "DNSSEC", "HSTS").
    pub id: &'static str,
    /// Which K-Derm layer this control operates at.
    pub layer: &'static str,
    /// Current deployment status.
    pub status: HardeningStatus,
    /// Brief description.
    pub description: &'static str,
}

/// Current hardening controls (Wave 136b state).
pub const HARDENING_CONTROLS: &[HardeningControl] = &[
    HardeningControl {
        id: "DNSSEC",
        layer: "Outer membrane",
        status: HardeningStatus::Active,
        description: "DS record at Porkbun, CF DNSSEC enabled (keyTag 2371, alg 13)",
    },
    HardeningControl {
        id: "HSTS",
        layer: "Outer membrane",
        status: HardeningStatus::Active,
        description: "Strict-Transport-Security on all domains",
    },
    HardeningControl {
        id: "CSP",
        layer: "Outer membrane",
        status: HardeningStatus::Active,
        description: "Content-Security-Policy headers (Caddy)",
    },
    HardeningControl {
        id: "fail2ban",
        layer: "Plasma membrane",
        status: HardeningStatus::Active,
        description: "Automated ban on repeated auth failures",
    },
    HardeningControl {
        id: "drawbridge",
        layer: "Periplasm",
        status: HardeningStatus::Active,
        description: "songBird external proxy allowlist (OSM, FEMA, USGS, ArcGIS)",
    },
    HardeningControl {
        id: "cascade-signing",
        layer: "Periplasm",
        status: HardeningStatus::Landed,
        description: "ed25519 signed cascade commits (code landed, activation pending)",
    },
    HardeningControl {
        id: "lab-auth-gate",
        layer: "Periplasm",
        status: HardeningStatus::Landed,
        description: "lab.primals.eco access control (songBird drawbridge → Caddy)",
    },
    HardeningControl {
        id: "bearDog-TLS",
        layer: "Cytoplasm",
        status: HardeningStatus::Landed,
        description: "bearDog ACME gateway replacing Caddy TLS termination",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kderm_layers_complete() {
        assert_eq!(KDERM_LAYERS.len(), 5);
        assert_eq!(KDERM_LAYERS[0].name, "Extracellular");
        assert_eq!(KDERM_LAYERS[4].name, "Cytoplasm");
        for layer in KDERM_LAYERS {
            assert!(!layer.components.is_empty());
            assert!(!layer.security.is_empty());
        }
    }

    #[test]
    fn hardening_controls_consistent() {
        let layer_names: Vec<&str> = KDERM_LAYERS.iter().map(|l| l.name).collect();
        for ctrl in HARDENING_CONTROLS {
            assert!(
                layer_names.contains(&ctrl.layer),
                "control {} references unknown layer: {}",
                ctrl.id,
                ctrl.layer
            );
        }
        let active = HARDENING_CONTROLS
            .iter()
            .filter(|c| c.status == HardeningStatus::Active)
            .count();
        assert!(active >= 4);
    }
}
