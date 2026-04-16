// SPDX-License-Identifier: AGPL-3.0-or-later
//! TUI State Management
//!
//! Central state for the Rich TUI. All state is managed here,
//! with zero global state or unsafe code.

use chrono::{DateTime, Utc};

/// The active view in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// System overview
    Dashboard,
    /// ASCII graph visualization
    Topology,
    /// Device management
    Devices,
    /// Primal status
    Primals,
    /// Real-time log streaming
    Logs,
    /// neuralAPI graph orchestration
    NeuralAPI,
    /// NUCLEUS secure discovery
    Nucleus,
    /// `LiveSpore` live deployment
    LiveSpore,
}

impl View {
    /// Get the keyboard shortcut for this view
    #[must_use]
    pub const fn shortcut(&self) -> char {
        match self {
            Self::Dashboard => '1',
            Self::Topology => '2',
            Self::Devices => '3',
            Self::Primals => '4',
            Self::Logs => '5',
            Self::NeuralAPI => '6',
            Self::Nucleus => '7',
            Self::LiveSpore => '8',
        }
    }

    /// Get the display name for this view
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Topology => "Topology",
            Self::Devices => "Devices",
            Self::Primals => "Primals",
            Self::Logs => "Logs",
            Self::NeuralAPI => "neuralAPI",
            Self::Nucleus => "NUCLEUS",
            Self::LiveSpore => "LiveSpore",
        }
    }

    /// Get all views
    #[must_use]
    pub const fn all() -> [Self; 8] {
        [
            Self::Dashboard,
            Self::Topology,
            Self::Devices,
            Self::Primals,
            Self::Logs,
            Self::NeuralAPI,
            Self::Nucleus,
            Self::LiveSpore,
        ]
    }
}

/// Log message with metadata
#[derive(Debug, Clone)]
pub struct LogMessage {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Source primal (if known)
    pub source: Option<String>,
    /// Log level
    pub level: LogLevel,
    /// Message content
    pub message: String,
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warn,
    /// Error level
    Error,
}

/// System status
#[derive(Debug, Clone)]
pub struct SystemStatus {
    /// Number of active primals
    pub active_primals: usize,
    /// Number of discovered devices
    pub discovered_devices: usize,
    /// System uptime
    pub uptime: std::time::Duration,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            active_primals: 0,
            discovered_devices: 0,
            uptime: std::time::Duration::from_secs(0),
            last_update: Utc::now(),
        }
    }
}

/// TUI statistics
#[derive(Debug, Clone)]
pub struct TUIStats {
    /// Current view
    pub view: View,
    /// Number of primals
    pub primal_count: usize,
    /// Number of topology edges
    pub topology_edge_count: usize,
    /// Number of log messages
    pub log_count: usize,
    /// Standalone mode
    pub standalone: bool,
    /// Number of registered capabilities
    pub registered_capabilities: usize,
}
