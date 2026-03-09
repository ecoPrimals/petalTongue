// SPDX-License-Identifier: AGPL-3.0-only
//! Integration tests for Songbird discovery
//!
//! Tests the complete flow from Songbird client to visualization provider.

use petal_tongue_discovery::{
    SongbirdClient, SongbirdVisualizationProvider, VisualizationDataProvider,
};
use std::path::PathBuf;

#[tokio::test]
async fn test_songbird_provider_creation() {
    // Create a mock client for testing
    let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test-songbird.sock"));
    let provider = SongbirdVisualizationProvider::from_client(client);

    // Verify metadata
    let metadata = provider.get_metadata();
    assert_eq!(metadata.name, "Songbird Registry");
    assert_eq!(metadata.protocol, "unix+jsonrpc");
    assert!(
        metadata
            .capabilities
            .contains(&"primal-discovery".to_string())
    );
}

#[tokio::test]
async fn test_songbird_provider_get_topology() {
    // Topology should return empty for now (inferred from capabilities)
    let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
    let provider = SongbirdVisualizationProvider::from_client(client);

    let topology = provider.get_topology().await;
    assert!(topology.is_ok());
    assert!(topology.unwrap().is_empty());
}

// Note: Live integration tests require an actual Songbird instance running.
// These would be in a separate test suite (e.g., `tests/live_songbird_test.rs`)
// that is only run when SONGBIRD_TEST_SOCKET environment variable is set.
