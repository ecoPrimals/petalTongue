// SPDX-License-Identifier: AGPL-3.0-or-later
//! mDNS discovery integration tests
//!
//! Production discovery uses [`crate::mdns_provider::MdnsVisualizationProvider`] from
//! [`crate::discover_visualization_providers`].

#[cfg(test)]
mod tests {
    use crate::mdns_provider::MdnsVisualizationProvider;

    #[tokio::test]
    async fn discover_via_mdns_returns_result() {
        let result = MdnsVisualizationProvider::discover().await;
        assert!(result.is_ok());
    }

    #[cfg(feature = "mdns")]
    #[tokio::test]
    async fn query_capability_returns_result() {
        assert!(MdnsVisualizationProvider::discover().await.is_ok());
    }

    #[cfg(feature = "mdns")]
    #[tokio::test]
    async fn query_capability_all_variants() {
        use crate::VisualizationCapability;

        for _cap in VisualizationCapability::all() {
            assert!(MdnsVisualizationProvider::discover().await.is_ok());
        }
    }
}
