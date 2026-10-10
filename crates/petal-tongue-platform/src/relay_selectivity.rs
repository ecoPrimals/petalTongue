// SPDX-License-Identifier: AGPL-3.0-or-later

//! Relay membrane selectivity — live Anderson permeability from firewall observations.
//!
//! Reads `/opt/membrane/relay-membrane.json` (produced by `relay-membrane-probe.sh`),
//! maps firewall counters to Anderson membrane modes, and computes both theoretical
//! and observed selectivity S = max(P) − min(P).
//!
//! ## Anderson math
//!
//! The core transport functions are copied from the canonical implementation in
//! `barraCuda/crates/barracuda/src/special/anderson_transport.rs` (Wave 167).
//! When a shared math crate is extracted, both sources should import from there.
//!
//! ## Mode mapping
//!
//! | Mode       | Firewall signal           | W_eff | d_eff | L   |
//! |------------|---------------------------|-------|-------|-----|
//! | self       | Rule 1 accept (whitelist) | 0.01  | 3.0   | 10  |
//! | commensal  | UDP+TCP accept − self     | 0.5   | 3.0   | 10  |
//! | scanner    | Rate-limit drops          | 20.0  | 1.0   | 500 |
//! | dead_port  | Port 21114 rejects        | 25.0  | 1.0   | 500 |

use serde::{Deserialize, Serialize};
use std::path::Path;

// ═══════════════════════════════════════════════════════════════
// Anderson transport math — copied from BarraCuda anderson_transport.rs
// Canonical source: barraCuda/crates/barracuda/src/special/anderson_transport.rs
//
// INVARIANT: These must match canonical exactly. Do not modify here.
// When divergence is found, resync from canonical (barraCuda).
// ═══════════════════════════════════════════════════════════════

/// Thouless-formula approximation for the 1D localization length.
fn localization_length(disorder: f64, energy: f64) -> f64 {
    let w_sq = disorder.mul_add(disorder, 0.01);
    let band_factor = energy.mul_add(-energy, 4.0).max(0.01);
    105.0 * band_factor / w_sq
}

/// Localization length generalized to fractional effective dimension.
fn dimensional_localization_length(disorder: f64, energy: f64, d_eff: f64) -> f64 {
    let d_eff = d_eff.max(1.0);
    let xi_1d = localization_length(disorder, energy);

    if d_eff <= 1.0 {
        return xi_1d;
    }

    let xi_2d = if disorder > 1e-10 {
        let exponent = (core::f64::consts::PI * (xi_1d / 10.0).min(30.0)).min(30.0);
        xi_1d * exponent.exp()
    } else {
        1e15
    };

    if d_eff <= 2.0 {
        let frac = d_eff - 1.0;
        let log_xi = xi_1d.ln().mul_add(1.0 - frac, xi_2d.ln() * frac);
        return log_xi.exp();
    }

    const W_C: f64 = 16.5;
    const NU: f64 = 1.57;
    const XI_EXTENDED: f64 = 1e15;
    let xi_3d = if disorder < 1e-10 {
        XI_EXTENDED
    } else if disorder < W_C {
        XI_EXTENDED
    } else if (disorder - W_C).abs() < 0.01 {
        xi_2d * 10.0
    } else {
        let reduced_w = disorder / W_C - 1.0;
        let xi_0 = xi_1d.max(1.0);
        xi_0 * reduced_w.powf(-NU).max(0.1)
    };

    if d_eff <= 3.0 {
        let frac = d_eff - 2.0;
        let log_xi = xi_2d.ln().mul_add(1.0 - frac, xi_3d.ln() * frac);
        return log_xi.exp();
    }

    xi_3d
}

/// A signal mode's effective Anderson parameters through the membrane.
struct MembraneMode {
    w_eff: f64,
    d_eff: f64,
    system_size: usize,
    energy: f64,
}

/// Compute transmission coefficient for a single mode.
fn membrane_permeability(mode: &MembraneMode) -> f64 {
    if mode.w_eff < 0.0 {
        return 0.0;
    }
    let xi = dimensional_localization_length(mode.w_eff, mode.energy, mode.d_eff);
    let l = mode.system_size as f64;
    if xi <= 0.0 {
        return 0.0;
    }
    (-l / xi).exp().clamp(0.0, 1.0)
}

// ═══════════════════════════════════════════════════════════════
// Relay membrane JSON + mode mapping
// ═══════════════════════════════════════════════════════════════

/// Raw probe data from `relay-membrane-probe.sh`.
#[derive(Debug, Deserialize)]
pub(crate) struct ProbeData {
    ts: String,
    counters: ProbeCounters,
    unique_ips: ProbeUniqueIps,
    hbbs_peers_1h: u64,
    ratelimit_events_5m: ProbeRateLimitEvents,
}

#[derive(Debug, Deserialize)]
struct ProbeCounters {
    self_accept: u64,
    udp_accept: u64,
    tcp_accept: u64,
    udp_ratelimit_drop: u64,
    tcp_ratelimit_drop: u64,
    dead_port_reject: u64,
}

#[derive(Debug, Deserialize)]
struct ProbeUniqueIps {
    udp: u64,
    tcp: u64,
}

#[derive(Debug, Deserialize)]
struct ProbeRateLimitEvents {
    udp: u64,
    tcp: u64,
}

/// Per-mode selectivity result.
#[derive(Debug, Serialize)]
struct ModeResult {
    id: &'static str,
    p_theoretical: f64,
    p_observed: f64,
    pkts: u64,
    drops: u64,
}

/// Full selectivity response.
#[derive(Debug, Serialize)]
pub(crate) struct SelectivityResponse {
    selectivity: f64,
    selectivity_observed: f64,
    modes: Vec<ModeResult>,
    unique_ips: SelectivityIps,
    hbbs_peers_1h: u64,
    ratelimit_events_5m: SelectivityRateLimits,
    probe_ts: String,
}

#[derive(Debug, Serialize)]
struct SelectivityIps {
    udp: u64,
    tcp: u64,
}

#[derive(Debug, Serialize)]
struct SelectivityRateLimits {
    udp: u64,
    tcp: u64,
}

/// Default path to the relay membrane probe JSON.
const DEFAULT_PROBE_PATH: &str = "/opt/membrane/relay-membrane.json";

/// Read probe data and compute selectivity.
///
/// Returns a JSON string suitable for a JSON-RPC result.
pub(crate) fn compute_selectivity() -> Result<SelectivityResponse, String> {
    let path_str =
        std::env::var("RELAY_MEMBRANE_JSON").unwrap_or_else(|_| DEFAULT_PROBE_PATH.to_owned());
    compute_selectivity_from_path(&path_str)
}

/// Compute selectivity from a specific file path. Testable without env vars.
fn compute_selectivity_from_path(path_str: &str) -> Result<SelectivityResponse, String> {
    let path = Path::new(path_str);

    let json_bytes = std::fs::read(path).map_err(|e| format!("probe read: {e}"))?;
    let probe: ProbeData =
        serde_json::from_slice(&json_bytes).map_err(|e| format!("probe parse: {e}"))?;

    // ── Theoretical permeability per mode ──
    let modes_params = [
        ("self", 0.01_f64, 3.0_f64, 10_usize),
        ("commensal", 0.5, 3.0, 10),
        ("scanner", 20.0, 1.0, 500),
        ("dead_port", 25.0, 1.0, 500),
    ];

    let theoretical: Vec<f64> = modes_params
        .iter()
        .map(|(_, w, d, l)| {
            membrane_permeability(&MembraneMode {
                w_eff: *w,
                d_eff: *d,
                system_size: *l,
                energy: 0.0,
            })
        })
        .collect();

    let t_max = theoretical.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let t_min = theoretical.iter().cloned().fold(f64::INFINITY, f64::min);
    let selectivity = t_max - t_min;

    // ── Observed permeability per mode ──
    let c = &probe.counters;
    let commensal_accept = (c.udp_accept + c.tcp_accept).saturating_sub(c.self_accept);
    let commensal_drop = c.udp_ratelimit_drop + c.tcp_ratelimit_drop;
    let scanner_drop = c.udp_ratelimit_drop + c.tcp_ratelimit_drop;

    let obs_self = if c.self_accept > 0 { 1.0 } else { 0.0 };
    let obs_commensal = if commensal_accept + commensal_drop > 0 {
        commensal_accept as f64 / (commensal_accept + commensal_drop) as f64
    } else {
        1.0
    };
    let obs_scanner = if scanner_drop > 0 { 0.0 } else { 0.5 };
    let obs_dead = if c.dead_port_reject > 0 { 0.0 } else { 0.5 };

    let observed = [obs_self, obs_commensal, obs_scanner, obs_dead];
    let o_max = observed.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let o_min = observed.iter().cloned().fold(f64::INFINITY, f64::min);
    let selectivity_observed = o_max - o_min;

    let mode_results = vec![
        ModeResult {
            id: "self",
            p_theoretical: theoretical[0],
            p_observed: obs_self,
            pkts: c.self_accept,
            drops: 0,
        },
        ModeResult {
            id: "commensal",
            p_theoretical: theoretical[1],
            p_observed: obs_commensal,
            pkts: commensal_accept,
            drops: commensal_drop,
        },
        ModeResult {
            id: "scanner",
            p_theoretical: theoretical[2],
            p_observed: obs_scanner,
            pkts: 0,
            drops: scanner_drop,
        },
        ModeResult {
            id: "dead_port",
            p_theoretical: theoretical[3],
            p_observed: obs_dead,
            pkts: 0,
            drops: c.dead_port_reject,
        },
    ];

    Ok(SelectivityResponse {
        selectivity,
        selectivity_observed,
        modes: mode_results,
        unique_ips: SelectivityIps {
            udp: probe.unique_ips.udp,
            tcp: probe.unique_ips.tcp,
        },
        hbbs_peers_1h: probe.hbbs_peers_1h,
        ratelimit_events_5m: SelectivityRateLimits {
            udp: probe.ratelimit_events_5m.udp,
            tcp: probe.ratelimit_events_5m.tcp,
        },
        probe_ts: probe.ts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localization_length_matches_barracuda() {
        let xi = localization_length(1.0, 0.0);
        assert!(xi > 400.0, "ξ(W=1, E=0) should be ~420, got {xi}");
        let xi_strong = localization_length(10.0, 0.0);
        assert!(
            xi_strong < xi,
            "stronger disorder should shorten ξ: {xi_strong} < {xi}"
        );
    }

    #[test]
    fn self_mode_passes() {
        let p = membrane_permeability(&MembraneMode {
            w_eff: 0.01,
            d_eff: 3.0,
            system_size: 10,
            energy: 0.0,
        });
        assert!(p > 0.99, "self mode should pass freely, got P={p}");
    }

    #[test]
    fn commensal_mode_passes() {
        let p = membrane_permeability(&MembraneMode {
            w_eff: 0.5,
            d_eff: 3.0,
            system_size: 10,
            energy: 0.0,
        });
        assert!(p > 0.95, "commensal mode should pass, got P={p}");
    }

    #[test]
    fn scanner_mode_blocks() {
        let p = membrane_permeability(&MembraneMode {
            w_eff: 20.0,
            d_eff: 1.0,
            system_size: 500,
            energy: 0.0,
        });
        assert!(p < 0.01, "scanner mode should block, got P={p}");
    }

    #[test]
    fn dead_port_mode_blocks() {
        let p = membrane_permeability(&MembraneMode {
            w_eff: 25.0,
            d_eff: 1.0,
            system_size: 500,
            energy: 0.0,
        });
        assert!(p < 0.01, "dead_port mode should block, got P={p}");
    }

    #[test]
    fn selectivity_is_near_one() {
        let modes = [
            (0.01_f64, 3.0_f64, 10_usize),
            (0.5, 3.0, 10),
            (20.0, 1.0, 500),
            (25.0, 1.0, 500),
        ];
        let perms: Vec<f64> = modes
            .iter()
            .map(|(w, d, l)| {
                membrane_permeability(&MembraneMode {
                    w_eff: *w,
                    d_eff: *d,
                    system_size: *l,
                    energy: 0.0,
                })
            })
            .collect();
        let s = perms.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            - perms.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(
            s > 0.95,
            "relay selectivity should be near 1.0, got S={s}"
        );
    }

    #[test]
    fn compute_selectivity_with_test_json() {
        let json = r#"{
            "ts": "2026-10-08T21:30:00Z",
            "counters": {
                "self_accept": 12345,
                "udp_accept": 67890,
                "tcp_accept": 45678,
                "udp_ratelimit_drop": 23,
                "tcp_ratelimit_drop": 5,
                "dead_port_reject": 3
            },
            "unique_ips": { "udp": 28, "tcp": 15 },
            "hbbs_peers_1h": 42,
            "ratelimit_events_5m": { "udp": 0, "tcp": 0 }
        }"#;

        let tmp = std::env::temp_dir().join("relay-membrane-test.json");
        std::fs::write(&tmp, json).expect("write test json");

        let result =
            compute_selectivity_from_path(tmp.to_str().unwrap()).expect("should parse");
        assert!(result.selectivity > 0.95, "S={}", result.selectivity);
        assert!(
            result.selectivity_observed > 0.95,
            "S_obs={}",
            result.selectivity_observed
        );
        assert_eq!(result.modes.len(), 4);
        assert_eq!(result.modes[0].id, "self");
        assert!((result.modes[0].p_observed - 1.0).abs() < 0.001);
        assert!(result.modes[2].p_observed < 0.01); // scanner blocked

        let _ = std::fs::remove_file(&tmp);
    }
}
