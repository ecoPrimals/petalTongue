// SPDX-License-Identifier: AGPL-3.0-only
//! mDNS-based discovery of visualization data providers
//!
//! Discovers primals via multicast DNS that advertise visualization capabilities.
//! Delegates to [`crate::mdns_provider::MdnsVisualizationProvider`] for the real
//! network implementation.

use crate::VisualizationCapability;
use crate::mdns_provider::MdnsVisualizationProvider;
use crate::traits::VisualizationDataProvider;

/// Discover providers via mDNS/multicast.
///
/// Delegates to [`MdnsVisualizationProvider::discover`] which performs real
/// UDP multicast service discovery on the local network.
#[cfg_attr(not(test), allow(dead_code))]
pub async fn discover_via_mdns() -> anyhow::Result<Vec<Box<dyn VisualizationDataProvider>>> {
    tracing::debug!("mDNS discovery: delegating to MdnsVisualizationProvider");
    MdnsVisualizationProvider::discover().await
}

/// Query mDNS for a specific capability.
///
/// Performs real mDNS discovery and filters providers by the requested capability.
#[expect(
    dead_code,
    reason = "API reserved for direct capability queries; callers currently use discover_via_mdns"
)]
pub async fn query_capability(capability: VisualizationCapability) -> anyhow::Result<Vec<String>> {
    let providers = MdnsVisualizationProvider::discover().await?;
    let cap_str = capability.as_str();
    Ok(providers
        .iter()
        .filter(|p| p.get_metadata().capabilities.iter().any(|c| c == cap_str))
        .map(|p| p.get_metadata().endpoint.clone())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn discover_via_mdns_returns_result() {
        let result = discover_via_mdns().await;
        assert!(result.is_ok());
    }

    #[cfg(feature = "mdns")]
    #[tokio::test]
    async fn query_capability_returns_result() {
        let result = query_capability(VisualizationCapability::PrimalProvider).await;
        assert!(result.is_ok());
    }

    #[cfg(feature = "mdns")]
    #[tokio::test]
    async fn query_capability_all_variants() {
        for cap in VisualizationCapability::all() {
            let result = query_capability(*cap).await;
            assert!(result.is_ok());
        }
    }
}
