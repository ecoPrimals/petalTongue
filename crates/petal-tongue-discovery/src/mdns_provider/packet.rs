// SPDX-License-Identifier: AGPL-3.0-or-later
//! mDNS packet building and parsing.
//!
//! Handles DNS query construction and DNS-SD response parsing for
//! visualization provider discovery.

use crate::dns_parser::{DnsHeader, RecordType, ResourceRecord};
use crate::errors::{DiscoveryError, DiscoveryResult};
use crate::traits::ProviderMetadata;
use std::net::{Ipv4Addr, SocketAddr};

/// Build an mDNS query packet
///
/// Simplified implementation - just enough to query for our service.
/// Real mDNS would use a full DNS packet parser.
///
/// Supports single-question queries; multi-question packets are not needed for our use case.
#[must_use]
pub fn build_mdns_query(service_name: &str) -> Vec<u8> {
    let mut packet = Vec::new();

    // DNS header (12 bytes)
    packet.extend_from_slice(&[0x00, 0x00]); // Transaction ID
    packet.extend_from_slice(&[0x00, 0x00]); // Flags
    packet.extend_from_slice(&[0x00, 0x01]); // Questions: 1
    packet.extend_from_slice(&[0x00, 0x00]); // Answer RRs: 0
    packet.extend_from_slice(&[0x00, 0x00]); // Authority RRs: 0
    packet.extend_from_slice(&[0x00, 0x00]); // Additional RRs: 0

    // Question section - encode service name as DNS labels
    for label in service_name.split('.') {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "DNS label length is at most 63"
        )]
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

/// Parse an mDNS response packet into provider metadata.
///
/// Extracts service information from DNS-SD response using proper DNS parsing.
/// Parses PTR, SRV, TXT, and A records to build provider information.
pub fn parse_mdns_response(data: &[u8], addr: SocketAddr) -> DiscoveryResult<ProviderMetadata> {
    let header = DnsHeader::parse(data)?;

    if !header.is_response() {
        return Err(DiscoveryError::NotDnsResponse);
    }

    tracing::trace!("DNS response: {} answers from {}", header.answers, addr);

    let mut offset = 12; // After header

    // Skip questions
    for _ in 0..header.questions {
        let parser = crate::dns_parser::NameParser::new(data);
        let (_, name_len) = parser.parse_name(offset)?;
        offset += name_len + 4; // name + type (2) + class (2)
    }

    let mut service_port: Option<u16> = None;
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
                    tracing::debug!("Found SRV record: {}:{}", srv.target, srv.port);
                }
            }
            Some(RecordType::TXT) => {
                if let Ok(txt) = record.as_txt() {
                    tracing::debug!("Found TXT record with {} attributes", txt.attributes.len());
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

    let ip = if a_records.is_empty() {
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
        return Err(DiscoveryError::NoPortAdvertisedInMdns);
    };
    let endpoint = format!("http://{ip}:{port}");

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

    Ok(metadata)
}
