// SPDX-License-Identifier: AGPL-3.0-or-later
//! mDNS-based visualization provider discovery.
//!
//! Implements network-based discovery of visualization data providers
//! using UDP multicast and mDNS service discovery (RFC 6762, RFC 6763).
//!
//! Based on a proven UDP multicast pattern used by discovery/registry providers.

mod packet;

use async_trait::async_trait;
use petal_tongue_core::{PrimalInfo, TopologyEdge, constants};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::errors::{DiscoveryError, DiscoveryResult};
use crate::traits::{ProviderMetadata, VisualizationDataProvider};

pub use packet::parse_mdns_response;

#[cfg(test)]
pub use packet::build_mdns_query;

/// mDNS multicast address (224.0.0.251)
pub const MDNS_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);

/// mDNS standard port
pub const MDNS_PORT: u16 = 5353;

/// Service name for visualization providers
pub const SERVICE_NAME: &str = "_visualization-provider._tcp.local";

/// mDNS-based visualization provider
///
/// Discovers visualization data providers on the local network using
/// UDP multicast and mDNS service discovery.
pub struct MdnsVisualizationProvider {
    /// Discovered endpoint
    endpoint: String,
    /// Provider metadata
    metadata: ProviderMetadata,
    /// HTTP client for queries
    client: reqwest::Client,
}

impl MdnsVisualizationProvider {
    /// Create a new mDNS provider from discovered endpoint
    #[must_use]
    pub fn new(endpoint: String, metadata: ProviderMetadata) -> Self {
        Self {
            endpoint,
            metadata,
            client: reqwest::Client::new(),
        }
    }

    /// Discover visualization providers on the local network
    ///
    /// Uses UDP multicast to query for `_visualization-provider._tcp.local` services.
    pub async fn discover() -> DiscoveryResult<Vec<Box<dyn VisualizationDataProvider>>> {
        tracing::info!("Starting mDNS discovery for visualization providers...");

        let socket = Self::create_multicast_socket().map_err(|e| {
            DiscoveryError::MdnsError(format!("Failed to create multicast socket: {e}"))
        })?;

        let socket = UdpSocket::from_std(socket).map_err(|e| {
            DiscoveryError::MdnsError(format!("Failed to convert to tokio socket: {e}"))
        })?;

        let discovery_timeout = constants::default_discovery_timeout();
        let providers = match timeout(discovery_timeout, Self::query_services(socket)).await {
            Ok(Ok(providers)) => {
                tracing::info!("mDNS discovery found {} provider(s)", providers.len());
                providers
            }
            Ok(Err(e)) => {
                tracing::warn!("mDNS discovery failed: {}", e);
                vec![]
            }
            Err(_) => {
                tracing::warn!("mDNS discovery timed out after {:?}", discovery_timeout);
                vec![]
            }
        };

        Ok(providers)
    }

    /// Create a UDP socket configured for multicast
    fn create_multicast_socket() -> Result<std::net::UdpSocket, std::io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

        socket.set_reuse_address(true)?;

        let bind_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, MDNS_PORT);
        socket.bind(&bind_addr.into())?;

        let interface = Ipv4Addr::UNSPECIFIED;
        socket.join_multicast_v4(&MDNS_MULTICAST_ADDR, &interface)?;

        socket.set_multicast_ttl_v4(255)?;
        socket.set_broadcast(true)?;
        socket.set_nonblocking(true)?;

        tracing::debug!(
            "Created multicast socket: joined group {}, bound to port {}",
            MDNS_MULTICAST_ADDR,
            MDNS_PORT
        );

        Ok(socket.into())
    }

    /// Query for visualization provider services
    async fn query_services(
        socket: UdpSocket,
    ) -> DiscoveryResult<Vec<Box<dyn VisualizationDataProvider>>> {
        let query = packet::build_mdns_query(SERVICE_NAME);

        let multicast_target = SocketAddr::V4(SocketAddrV4::new(MDNS_MULTICAST_ADDR, MDNS_PORT));
        socket
            .send_to(&query, multicast_target)
            .await
            .map_err(|e| DiscoveryError::MdnsError(format!("Failed to send mDNS query: {e}")))?;

        tracing::debug!("Sent mDNS query for service: {}", SERVICE_NAME);

        let mut providers = Vec::new();
        let mut buf = vec![0u8; 1024];

        loop {
            match timeout(Duration::from_millis(100), socket.recv_from(&mut buf)).await {
                Ok(Ok((len, addr))) => {
                    tracing::debug!("Received mDNS response from {}: {} bytes", addr, len);

                    if let Ok(metadata) = packet::parse_mdns_response(&buf[..len], addr) {
                        let endpoint = metadata.endpoint.clone();
                        providers.push(Box::new(Self::new(endpoint, metadata))
                            as Box<dyn VisualizationDataProvider>);
                    }
                }
                Ok(Err(e)) => {
                    tracing::debug!("Socket error: {}", e);
                    break;
                }
                Err(_) => break,
            }
        }

        Ok(providers)
    }
}

#[async_trait]
impl VisualizationDataProvider for MdnsVisualizationProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        #[derive(serde::Deserialize)]
        struct PrimalsResponse {
            primals: Vec<PrimalInfo>,
        }

        let url = format!("{}/api/v1/primals/discovered", self.endpoint);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| DiscoveryError::MdnsError(format!("Failed to query primals: {e}")))?;

        if !response.status().is_success() {
            return Err(DiscoveryError::ProviderHttpError {
                status: response.status().as_u16(),
                endpoint: Some(self.endpoint.clone()),
            });
        }

        let data: PrimalsResponse =
            response
                .json()
                .await
                .map_err(|e| DiscoveryError::ParseError {
                    data_type: "primals response".to_string(),
                    message: e.to_string(),
                })?;

        Ok(data.primals)
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        #[derive(serde::Deserialize)]
        struct TopologyResponse {
            edges: Vec<TopologyEdge>,
        }

        let url = format!("{}/api/v1/topology", self.endpoint);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| DiscoveryError::MdnsError(format!("Failed to query topology: {e}")))?;

        if !response.status().is_success() {
            return Err(DiscoveryError::ProviderHttpError {
                status: response.status().as_u16(),
                endpoint: Some(self.endpoint.clone()),
            });
        }

        let data: TopologyResponse =
            response
                .json()
                .await
                .map_err(|e| DiscoveryError::ParseError {
                    data_type: "topology response".to_string(),
                    message: e.to_string(),
                })?;

        Ok(data.edges)
    }

    async fn health_check(&self) -> DiscoveryResult<String> {
        let url = format!("{}/api/v1/health", self.endpoint);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| DiscoveryError::MdnsError(format!("Failed to health check: {e}")))?;

        if response.status().is_success() {
            Ok(format!("mDNS provider {} is healthy", self.metadata.name))
        } else {
            Err(DiscoveryError::ProviderHttpError {
                status: response.status().as_u16(),
                endpoint: Some(self.endpoint.clone()),
            })
        }
    }

    fn get_metadata(&self) -> ProviderMetadata {
        self.metadata.clone()
    }
}

#[cfg(test)]
mod tests;
