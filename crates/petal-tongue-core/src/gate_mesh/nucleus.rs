// SPDX-License-Identifier: AGPL-3.0-or-later
//! NUCLEUS composition — primal→gate assignments and atomics.
//!
//! Defines the 4 NUCLEUS atomics (Tower, Node, Nest, Meta) and their
//! constituent primals. This is the offline snapshot; live composition
//! state comes from `biomeOS orchestrator` capability discovery.

use serde::{Deserialize, Serialize};

/// A primal in the NUCLEUS composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleusPrimal {
    /// Primal identifier (e.g. "bearDog").
    pub id: &'static str,
    /// Short role description.
    pub role: &'static str,
    /// Primary gate assignment.
    pub gate: &'static str,
}

/// An atomic grouping of primals.
#[derive(Debug, Clone, Serialize)]
pub struct NucleusAtomic {
    /// Atomic name (e.g. "Tower Atomic").
    pub name: &'static str,
    /// Primals in this atomic.
    pub primals: &'static [NucleusPrimal],
}

/// Tower Atomic — trust, transport, defense.
pub const TOWER_ATOMIC: &[NucleusPrimal] = &[
    NucleusPrimal {
        id: "bearDog",
        role: "Crypto identity, BTSP auth, TLS",
        gate: "flockGate",
    },
    NucleusPrimal {
        id: "songBird",
        role: "Mesh routing, STUN/TURN, relay",
        gate: "flockGate",
    },
    NucleusPrimal {
        id: "skunkBat",
        role: "Threat detection, MethodGate",
        gate: "flockGate",
    },
];

/// Node Atomic — compute, fleet, shaders.
pub const NODE_ATOMIC: &[NucleusPrimal] = &[
    NucleusPrimal {
        id: "toadStool",
        role: "Fleet management, dispatch",
        gate: "ironGate",
    },
    NucleusPrimal {
        id: "barraCuda",
        role: "GPU compute, LSTM, Vulkan",
        gate: "ironGate",
    },
    NucleusPrimal {
        id: "coralReef",
        role: "Shader pipelines, SPIR-V",
        gate: "ironGate",
    },
];

/// Nest Atomic — storage, provenance.
pub const NEST_ATOMIC: &[NucleusPrimal] = &[
    NucleusPrimal {
        id: "nestGate",
        role: "Content-addressed storage",
        gate: "sporeGate",
    },
    NucleusPrimal {
        id: "rhizoCrypt",
        role: "DAG sessions, Merkle roots",
        gate: "sporeGate",
    },
    NucleusPrimal {
        id: "loamSpine",
        role: "Ledger commits, spine",
        gate: "sporeGate",
    },
    NucleusPrimal {
        id: "sweetGrass",
        role: "Provenance braids",
        gate: "sporeGate",
    },
];

/// Meta — orchestration, AI, visualization.
pub const META_ATOMIC: &[NucleusPrimal] = &[
    NucleusPrimal {
        id: "biomeOS",
        role: "Composition orchestrator",
        gate: "eastGate",
    },
    NucleusPrimal {
        id: "squirrel",
        role: "AI dispatch, Ollama",
        gate: "eastGate",
    },
    NucleusPrimal {
        id: "petalTongue",
        role: "Visualization, dashboards",
        gate: "eastGate",
    },
];

/// All 4 NUCLEUS atomics.
pub const NUCLEUS_ATOMICS: &[NucleusAtomic] = &[
    NucleusAtomic {
        name: "Tower Atomic",
        primals: TOWER_ATOMIC,
    },
    NucleusAtomic {
        name: "Node Atomic",
        primals: NODE_ATOMIC,
    },
    NucleusAtomic {
        name: "Nest Atomic",
        primals: NEST_ATOMIC,
    },
    NucleusAtomic {
        name: "Meta",
        primals: META_ATOMIC,
    },
];

/// Total primal count across all atomics.
#[must_use]
pub fn nucleus_primal_count() -> usize {
    NUCLEUS_ATOMICS.iter().map(|a| a.primals.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nucleus_has_13_primals() {
        assert_eq!(nucleus_primal_count(), 13);
    }

    #[test]
    fn all_atomics_named() {
        for atomic in NUCLEUS_ATOMICS {
            assert!(!atomic.name.is_empty());
            assert!(!atomic.primals.is_empty());
        }
    }

    #[test]
    fn all_primals_have_gate_assignment() {
        for atomic in NUCLEUS_ATOMICS {
            for primal in atomic.primals {
                assert!(!primal.gate.is_empty(), "{} has no gate", primal.id);
            }
        }
    }
}
