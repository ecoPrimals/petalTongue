// SPDX-License-Identifier: AGPL-3.0-only
// mDNS-based visualization provider discovery
//
// This module implements network-based discovery of visualization data providers
// using UDP multicast and mDNS service discovery (RFC 6762, RFC 6763).
//
// Based on Songbird's proven UDP multicast implementation.

use crate::dns_parser::{DnsHeader, RecordType, ResourceRecord};
use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use anyhow::{Context, Result};
use async_trait::async_trait;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

/// mDNS multicast address (224.0.0.251)
const MDNS_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);

/// mDNS standard port
const MDNS_PORT: u16 = 5353;

/// Service name for visualization providers
const SERVICE_NAME: &str = "_visualization-provider._tcp.local";

/// Discovery timeout
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(5);

/// mDNS-based visualization provider
///
/// Discovers visualization data providers on the local network using
/// UDP multicast and mDNS service discovery.
///
/// # Example
///
/// ```no_run
/// use petal_tongue_discovery::MdnsVisualizationProvider;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Discover providers on the network
///     let providers = MdnsVisualizationProvider::discover().await?;
///     
///     println!("Found {} providers", providers.len());
///     
///     Ok(())
/// }
/// ```
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
    ///
    /// # Returns
    ///
    /// A vector of discovered providers. Returns empty vector if discovery fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use petal_tongue_discovery::MdnsVisualizationProvider;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let providers = MdnsVisualizationProvider::discover().await?;
    /// for provider in providers {
    ///     println!("Found: {}", provider.get_metadata().name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover() -> Result<Vec<Box<dyn VisualizationDataProvider>>> {
        tracing::info!("Starting mDNS discovery for visualization providers...");

        // Create UDP socket with multicast support
        let socket =
            Self::create_multicast_socket().context("Failed to create multicast socket")?;

        // Convert to tokio socket
        let socket = UdpSocket::from_std(socket).context("Failed to convert to tokio socket")?;

        // Query for services
        let providers = match timeout(DISCOVERY_TIMEOUT, Self::query_services(socket)).await {
            Ok(Ok(providers)) => {
                tracing::info!("mDNS discovery found {} provider(s)", providers.len());
                providers
            }
            Ok(Err(e)) => {
                tracing::warn!("mDNS discovery failed: {}", e);
                vec![]
            }
            Err(_) => {
                tracing::warn!("mDNS discovery timed out after {:?}", DISCOVERY_TIMEOUT);
                vec![]
            }
        };

        Ok(providers)
    }

    /// Create a UDP socket configured for multicast
    fn create_multicast_socket() -> Result<std::net::UdpSocket> {
        // Create UDP socket using socket2 for low-level control
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .context("Failed to create socket")?;

        // Enable address reuse (multiple processes can bind)
        socket
            .set_reuse_address(true)
            .context("Failed to set SO_REUSEADDR")?;

        // Enable port reuse (BSD systems)
        // NOTE: set_reuse_port() not available in socket2 v0.5, only in v0.6+
        // This is optional optimization for BSD systems, not required
        // #[cfg(all(unix, not(target_os = "windows")))]
        // socket
        //     .set_reuse_port(true)
        //     .context("Failed to set SO_REUSEPORT")?;

        // Bind to 0.0.0.0:5353 to receive multicast
        let bind_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, MDNS_PORT);
        socket
            .bind(&bind_addr.into())
            .context("Failed to bind socket")?;

        // Join multicast group (CRITICAL for receiving multicast!)
        let interface = Ipv4Addr::UNSPECIFIED; // Use default network interface
        socket
            .join_multicast_v4(&MDNS_MULTICAST_ADDR, &interface)
            .context("Failed to join multicast group")?;

        // Set multicast TTL
        socket
            .set_multicast_ttl_v4(255)
            .context("Failed to set multicast TTL")?;

        // Enable broadcast as fallback
        socket
            .set_broadcast(true)
            .context("Failed to enable broadcast")?;

        // Set non-blocking
        socket
            .set_nonblocking(true)
            .context("Failed to set non-blocking")?;

        tracing::debug!(
            "Created multicast socket: joined group {}, bound to port {}",
            MDNS_MULTICAST_ADDR,
            MDNS_PORT
        );

        Ok(socket.into())
    }

    /// Query for visualization provider services
    async fn query_services(socket: UdpSocket) -> Result<Vec<Box<dyn VisualizationDataProvider>>> {
        // Build mDNS query packet
        let query = Self::build_mdns_query(SERVICE_NAME);

        // Send query to multicast address
        let multicast_target = SocketAddr::V4(SocketAddrV4::new(MDNS_MULTICAST_ADDR, MDNS_PORT));
        socket
            .send_to(&query, multicast_target)
            .await
            .context("Failed to send mDNS query")?;

        tracing::debug!("Sent mDNS query for service: {}", SERVICE_NAME);

        // Listen for responses
        let mut providers = Vec::new();
        let mut buf = vec![0u8; 1024];

        // Collect responses for the timeout duration
        loop {
            match timeout(Duration::from_millis(100), socket.recv_from(&mut buf)).await {
                Ok(Ok((len, addr))) => {
                    tracing::debug!("Received mDNS response from {}: {} bytes", addr, len);

                    // Parse response
                    if let Ok(provider) = Self::parse_mdns_response(&buf[..len], addr) {
                        providers.push(provider);
                    }
                }
                Ok(Err(e)) => {
                    tracing::debug!("Socket error: {}", e);
                    break;
                }
                Err(_) => {
                    // Timeout - no more responses
                    break;
                }
            }
        }

        Ok(providers)
    }

    /// Build an mDNS query packet
    ///
    /// Simplified implementation - just enough to query for our service.
    /// Real mDNS would use a full DNS packet parser.
    fn build_mdns_query(service_name: &str) -> Vec<u8> {
        // For now, return a simple query packet
        // TODO: Implement full DNS packet building

        // mDNS query packet structure (simplified):
        // - Transaction ID: 0x0000
        // - Flags: 0x0000 (standard query)
        // - Questions: 1
        // - Answer RRs: 0
        // - Authority RRs: 0
        // - Additional RRs: 0
        // - Question: service_name, type PTR (12), class IN (1)

        let mut packet = Vec::new();

        // DNS header (12 bytes)
        packet.extend_from_slice(&[0x00, 0x00]); // Transaction ID
        packet.extend_from_slice(&[0x00, 0x00]); // Flags
        packet.extend_from_slice(&[0x00, 0x01]); // Questions: 1
        packet.extend_from_slice(&[0x00, 0x00]); // Answer RRs: 0
        packet.extend_from_slice(&[0x00, 0x00]); // Authority RRs: 0
        packet.extend_from_slice(&[0x00, 0x00]); // Additional RRs: 0

        // Question section
        // Encode service name as DNS labels
        for label in service_name.split('.') {
            packet.push(label.len() as u8);
            packet.extend_from_slice(label.as_bytes());
        }
        packet.push(0); // End of labels

        // Type: PTR (12)
        packet.extend_from_slice(&[0x00, 0x0C]);

        // Class: IN (1)
        packet.extend_from_slice(&[0x00, 0x01]);

        tracing::trace!("Built mDNS query packet: {} bytes", packet.len());

        packet
    }

    /// Parse an mDNS response packet
    ///
    /// Extracts service information from DNS-SD response using proper DNS parsing.
    ///
    /// Parses PTR, SRV, TXT, and A records to build provider information.
    fn parse_mdns_response(
        data: &[u8],
        addr: SocketAddr,
    ) -> Result<Box<dyn VisualizationDataProvider>> {
        // Parse DNS header
        let header = DnsHeader::parse(data)?;

        if !header.is_response() {
            anyhow::bail!("Not a DNS response packet");
        }

        tracing::trace!("DNS response: {} answers from {}", header.answers, addr);

        // Skip question section (we know what we asked for)
        let mut offset = 12; // After header

        // Skip questions
        for _ in 0..header.questions {
            let parser = crate::dns_parser::NameParser::new(data);
            let (_, name_len) = parser.parse_name(offset)?;
            offset += name_len + 4; // name + type (2) + class (2)
        }

        // Parse answer records
        let mut service_port: Option<u16> = None;
        let mut _service_host: Option<String> = None; // TODO: Use for hostname resolution if needed
        let mut txt_records: Vec<crate::dns_parser::TxtRecord> = Vec::new();
        let mut a_records: Vec<Ipv4Addr> = Vec::new();

        for _ in 0..header.answers {
            let (record, consumed) = ResourceRecord::parse(data, offset)?;
            let rdata_offset = offset;
            offset += consumed;

            match record.record_type() {
                Some(RecordType::PTR) => {
                    tracing::debug!("Found PTR record: {}", record.name);
                }
                Some(RecordType::SRV) => {
                    if let Ok(srv) = record.as_srv(data, rdata_offset) {
                        service_port = Some(srv.port);
                        _service_host = Some(srv.target.clone());
                        tracing::debug!("Found SRV record: {}:{}", srv.target, srv.port);
                    }
                }
                Some(RecordType::TXT) => {
                    if let Ok(txt) = record.as_txt() {
                        tracing::debug!(
                            "Found TXT record with {} attributes",
                            txt.attributes.len()
                        );
                        txt_records.push(txt);
                    }
                }
                Some(RecordType::A) => {
                    if let Ok(a) = record.as_a() {
                        a_records.push(a.addr);
                        tracing::debug!("Found A record: {}", a.addr);
                    }
                }
                _ => {
                    tracing::trace!("Skipping record type: {}", record.rtype);
                }
            }
        }

        // Build provider from parsed data
        let ip = if a_records.is_empty() {
            // Fall back to response address
            match addr {
                SocketAddr::V4(v4) => v4.ip().to_string(),
                SocketAddr::V6(v6) => format!("[{}]", v6.ip()),
            }
        } else {
            a_records[0].to_string()
        };

        let Some(port) = service_port else {
            tracing::warn!(
                "mDNS service at {} has no SRV port record - skipping (no port assumptions)",
                ip
            );
            anyhow::bail!("No port advertised in mDNS service - refusing to assume default");
        };
        let endpoint = format!("http://{ip}:{port}");

        // Extract capabilities from TXT records
        let mut capabilities = vec![
            "visualization.primal-provider".to_string(),
            "visualization.topology-provider".to_string(),
        ];

        for txt in &txt_records {
            if let Some(caps) = txt.get("capabilities") {
                for cap in caps.split(',') {
                    capabilities.push(cap.trim().to_string());
                }
            }
        }

        let name = txt_records
            .iter()
            .find_map(|txt| txt.get("name"))
            .unwrap_or("mDNS Provider")
            .to_string();

        let metadata = ProviderMetadata {
            name,
            endpoint: endpoint.clone(),
            protocol: "http".to_string(),
            capabilities,
        };

        tracing::info!(
            "Discovered provider via mDNS: {} at {}",
            metadata.name,
            endpoint
        );

        Ok(Box::new(MdnsVisualizationProvider::new(endpoint, metadata)))
    }
}

#[async_trait]
impl VisualizationDataProvider for MdnsVisualizationProvider {
    async fn get_primals(&self) -> Result<Vec<PrimalInfo>> {
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
            .context("Failed to query primals from mDNS-discovered provider")?;

        if !response.status().is_success() {
            anyhow::bail!("Provider returned error status: {}", response.status());
        }

        let data: PrimalsResponse = response
            .json()
            .await
            .context("Failed to parse primals response")?;

        Ok(data.primals)
    }

    async fn get_topology(&self) -> Result<Vec<TopologyEdge>> {
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
            .context("Failed to query topology from mDNS-discovered provider")?;

        if !response.status().is_success() {
            anyhow::bail!("Provider returned error status: {}", response.status());
        }

        let data: TopologyResponse = response
            .json()
            .await
            .context("Failed to parse topology response")?;

        Ok(data.edges)
    }

    async fn health_check(&self) -> Result<String> {
        let url = format!("{}/api/v1/health", self.endpoint);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .context("Failed to health check mDNS-discovered provider")?;

        if response.status().is_success() {
            Ok(format!("mDNS provider {} is healthy", self.metadata.name))
        } else {
            anyhow::bail!("Health check failed: {}", response.status())
        }
    }

    fn get_metadata(&self) -> ProviderMetadata {
        self.metadata.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_mdns_query() {
        let query = MdnsVisualizationProvider::build_mdns_query(SERVICE_NAME);

        // Should have DNS header (12 bytes) + question
        assert!(query.len() > 12, "Query packet too short");

        // Check DNS header
        assert_eq!(&query[0..2], &[0x00, 0x00], "Transaction ID should be 0");
        assert_eq!(&query[4..6], &[0x00, 0x01], "Should have 1 question");
    }

    #[test]
    fn test_multicast_constants() {
        assert_eq!(MDNS_MULTICAST_ADDR, Ipv4Addr::new(224, 0, 0, 251));
        assert_eq!(MDNS_PORT, 5353);
        assert_eq!(SERVICE_NAME, "_visualization-provider._tcp.local");
    }

    #[tokio::test]
    async fn test_discover_timeout() {
        // Discovery should timeout gracefully if no responses
        let result = MdnsVisualizationProvider::discover().await;

        // Should succeed but return empty list (no providers on localhost by default)
        assert!(result.is_ok());
        let providers = result.unwrap();

        // Might find providers or might not - both are valid
        tracing::info!("Discovery found {} providers", providers.len());
    }
}
