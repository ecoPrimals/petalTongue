// SPDX-License-Identifier: AGPL-3.0-or-later
//! Utility functions for UI rendering

use petal_tongue_core::{PrimalHealthStatus, PrimalInfo};

/// Convert health status to percentage (for UI display)
#[must_use]
pub const fn health_to_percentage(health: &PrimalHealthStatus) -> u8 {
    match health {
        PrimalHealthStatus::Healthy => 100,
        PrimalHealthStatus::Warning => 75,
        PrimalHealthStatus::Critical => 25,
        PrimalHealthStatus::Unknown => 50,
    }
}

/// Convert health status to color (hex)
#[must_use]
pub const fn health_to_color(health: &PrimalHealthStatus) -> &'static str {
    match health {
        PrimalHealthStatus::Healthy => "#4ade80",  // green-400
        PrimalHealthStatus::Warning => "#facc15",  // yellow-400
        PrimalHealthStatus::Critical => "#f87171", // red-400
        PrimalHealthStatus::Unknown => "#9ca3af",  // gray-400
    }
}

/// Convert health status to emoji
#[must_use]
pub const fn health_to_emoji(health: &PrimalHealthStatus) -> &'static str {
    match health {
        PrimalHealthStatus::Healthy => "🟢",
        PrimalHealthStatus::Warning => "🟡",
        PrimalHealthStatus::Critical => "🔴",
        PrimalHealthStatus::Unknown => "⚪",
    }
}

/// Get trust level from primal info (from properties or deprecated field)
#[expect(deprecated)]
#[must_use]
pub fn get_trust_level(info: &PrimalInfo) -> String {
    // Try properties first
    if let Some(trust) = info.properties.get("trust_level")
        && let Some(num) = trust.as_number()
    {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "trust_level is 0-5, clamped for display"
        )]
        return format!("{}", num.clamp(0.0, 255.0) as u8);
    }

    // Fall back to deprecated field
    if let Some(trust) = info.trust_level {
        return format!("{trust}");
    }

    "unknown".to_string()
}

/// Get family lineage from primal info (from properties or deprecated field)
#[expect(deprecated)]
#[must_use]
pub fn get_family_lineage(info: &PrimalInfo) -> String {
    // Try properties first
    if let Some(family) = info.properties.get("family_id")
        && let Some(s) = family.as_string()
    {
        return s.to_string();
    }

    // Fall back to deprecated field
    if let Some(ref family) = info.family_id {
        return family.clone();
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_to_percentage() {
        assert_eq!(health_to_percentage(&PrimalHealthStatus::Healthy), 100);
        assert_eq!(health_to_percentage(&PrimalHealthStatus::Warning), 75);
        assert_eq!(health_to_percentage(&PrimalHealthStatus::Critical), 25);
        assert_eq!(health_to_percentage(&PrimalHealthStatus::Unknown), 50);
    }

    #[test]
    fn test_health_to_color() {
        assert_eq!(health_to_color(&PrimalHealthStatus::Healthy), "#4ade80");
        assert_eq!(health_to_color(&PrimalHealthStatus::Warning), "#facc15");
    }

    #[test]
    fn test_health_to_emoji() {
        assert_eq!(health_to_emoji(&PrimalHealthStatus::Healthy), "🟢");
        assert_eq!(health_to_emoji(&PrimalHealthStatus::Warning), "🟡");
    }
}
