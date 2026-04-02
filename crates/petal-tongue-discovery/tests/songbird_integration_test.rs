// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for discovery service client
//!
//! Tests the complete flow from discovery service client to visualization provider.

use petal_tongue_discovery::{
    DiscoveryServiceClient, DiscoveryServiceProvider, VisualizationDataProvider,
};
use std::path::PathBuf;

#[tokio::test]
async fn test_discovery_service_provider_creation() {
    let client =
        DiscoveryServiceClient::with_socket_path(PathBuf::from("/tmp/test-discovery.sock"));
    let provider = DiscoveryServiceProvider::from_client(client);

    let metadata = provider.get_metadata();
    assert_eq!(metadata.name, "Discovery Service Registry");
    assert_eq!(metadata.protocol, "unix+jsonrpc");
    assert!(
        metadata
            .capabilities
            .contains(&"primal-discovery".to_string())
    );
}

#[tokio::test]
async fn test_discovery_service_provider_get_topology() {
    let client = DiscoveryServiceClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
    let provider = DiscoveryServiceProvider::from_client(client);

    let topology = provider.get_topology().await;
    assert!(topology.is_ok());
    assert!(topology.unwrap().is_empty());
}

// Live integration tests require an actual discovery service running.
// Run when DISCOVERY_SERVICE_TEST_SOCKET is set.
