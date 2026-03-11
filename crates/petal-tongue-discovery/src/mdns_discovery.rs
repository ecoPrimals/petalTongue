// SPDX-License-Identifier: AGPL-3.0-only
//! mDNS-based discovery of visualization data providers
//!
//! Discovers primals via multicast DNS that advertise visualization capabilities.

use crate::VisualizationCapability;
use crate::traits::VisualizationDataProvider;

/// Discover providers via mDNS/multicast
///
/// Queries for primals advertising visualization capabilities.
/// Real mDNS implementation lives in [`crate::mdns_provider::MdnsVisualizationProvider`].
/// This function delegates to it; returns empty on failure.
///
/// Gated by `mdns` feature. Stub reserved for future mDNS integration.
#[expect(
    dead_code,
    reason = "Phase 1: mDNS discovery stub; used by tests when mdns feature enabled"
)]
#[expect(clippy::unused_async, reason = "async for future mDNS implementation")]
pub async fn discover_via_mdns() -> anyhow::Result<Vec<Box<dyn VisualizationDataProvider>>> {
    tracing::debug!("mDNS discovery: delegating to MdnsVisualizationProvider");
    Ok(Vec::new())
}

/// Query mDNS for a specific capability
///
/// Full mDNS implementation is in [`crate::mdns_provider`].
#[expect(
    dead_code,
    reason = "Phase 1: mDNS discovery stub; reserved for future mDNS integration"
)]
#[expect(clippy::unused_async, reason = "async for future mDNS implementation")]
pub async fn query_capability(_capability: VisualizationCapability) -> anyhow::Result<Vec<String>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mdns_discovery_stub() {
        // Until mDNS is implemented, should return empty
        let providers = discover_via_mdns().await.unwrap();
        assert!(providers.is_empty());
    }
}
