//! Panel implementations for petalTongue
//!
//! This module contains embeddable panels that can be composed into different UIs.
//! Each panel is self-contained and follows the emerging panel system architecture.

pub mod doom_factory;
pub mod doom_panel;
pub mod doom_stats_panel;
pub mod metrics_panel;
pub mod proprioception_panel;

pub use doom_factory::{DoomPanelFactory, create_doom_factory};
pub use doom_panel::DoomPanel;
pub use doom_stats_panel::{DoomStatsPanel, DoomStatsPanelFactory};
pub use metrics_panel::{MetricsPanel, MetricsPanelFactory};
pub use proprioception_panel::{ProprioceptionPanel, ProprioceptionPanelFactory};
