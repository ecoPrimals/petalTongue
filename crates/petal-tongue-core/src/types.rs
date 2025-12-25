//! Core types for petalTongue visualization system

use serde::{Deserialize, Serialize};

/// Information about a discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Unique identifier for the primal
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Type of primal (e.g., "Compute", "Storage", "Security")
    pub primal_type: String,
    /// Network endpoint (e.g., <http://localhost:8080>)
    pub endpoint: String,
    /// List of capabilities this primal provides
    pub capabilities: Vec<String>,
    /// Health status ("Healthy", "Warning", "Critical", "Unknown")
    pub health: PrimalHealthStatus,
    /// Last time this primal was seen (Unix timestamp)
    pub last_seen: u64,
}

/// Health status of a primal (visualization-specific)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalHealthStatus {
    /// Primal is operating normally
    Healthy,
    /// Primal has minor issues but is functional
    Warning,
    /// Primal has major issues
    Critical,
    /// Health status is unknown
    Unknown,
}

impl PrimalHealthStatus {
    /// Get the string representation
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
            Self::Unknown => "Unknown",
        }
    }

    /// Parse from string
    #[must_use]
    pub fn parse_health_status(s: &str) -> Self {
        match s {
            "Healthy" => Self::Healthy,
            "Warning" => Self::Warning,
            "Critical" => Self::Critical,
            _ => Self::Unknown,
        }
    }
}

/// Connection status to a primal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Successfully connected
    Connected,
    /// Attempting to connect
    Connecting,
    /// Not connected
    Disconnected,
    /// Connection error
    Error(String),
}

/// Connection to a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConnection {
    /// Primal name
    pub name: String,
    /// Type of primal
    pub primal_type: String,
    /// Connection status
    pub status: ConnectionStatus,
    /// Network endpoint
    pub endpoint: String,
    /// Last heartbeat timestamp (Unix timestamp)
    pub last_heartbeat: Option<u64>,
}

/// Topology graph containing primals and their relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyGraph {
    /// List of primals in the topology
    pub nodes: Vec<PrimalInfo>,
    /// Connections between primals
    pub edges: Vec<TopologyEdge>,
    /// When this topology was captured
    pub timestamp: u64,
}

/// Edge (connection) in the topology graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    /// Source primal ID
    pub from: String,
    /// Target primal ID
    pub to: String,
    /// Type of relationship (e.g., `api_call`, `capability`)
    pub edge_type: String,
    /// Optional label
    pub label: Option<String>,
}

/// Real-time flow event showing message between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEvent {
    /// Event ID
    pub id: String,
    /// Source primal ID
    pub from: String,
    /// Target primal ID
    pub to: String,
    /// Type of message
    pub message_type: String,
    /// When the event occurred (Unix timestamp)
    pub timestamp: u64,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Traffic statistics between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    /// Source primal ID
    pub from: String,
    /// Target primal ID
    pub to: String,
    /// Number of messages
    pub message_count: u64,
    /// Total bytes transferred
    pub bytes_transferred: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Time period for these stats (Unix timestamp)
    pub period_start: u64,
    /// End of time period (Unix timestamp)
    pub period_end: u64,
}
