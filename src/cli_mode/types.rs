// SPDX-License-Identifier: AGPL-3.0-or-later
//! Serializable status types for `petaltongue status`.

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct SystemStatus {
    pub version: String,
    pub mode: String,
    pub unibin: UniBinStatus,
    pub ecobin: EcoBinStatus,
    pub system: SystemInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed: Option<DetailedStatus>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UniBinStatus {
    pub compliant: bool,
    pub binary_count: u8,
    pub mode_count: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct EcoBinStatus {
    pub percentage: u8,
    pub pure_rust_modes: u8,
    pub total_modes: u8,
    pub modes: Vec<ModeInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ModeInfo {
    pub name: String,
    pub pure_rust: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_total: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DetailedStatus {
    pub modes: Vec<ModeDetails>,
    pub features: Vec<String>,
    pub dependencies: DependencyInfo,
}

#[derive(Clone, Debug, Serialize)]
pub struct ModeDetails {
    pub name: String,
    pub description: String,
    pub pure_rust: bool,
    pub command: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DependencyInfo {
    pub total: usize,
    pub c_deps: usize,
    pub rust_deps: usize,
}
