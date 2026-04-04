// SPDX-License-Identifier: AGPL-3.0-or-later
//! DNS message header wire format and parsing (RFC 1035).

use crate::errors::{DiscoveryError, DiscoveryResult};

/// Parsed DNS header (RFC 1035)
#[derive(Debug)]
pub struct DnsHeader {
    pub transaction_id: u16,
    pub flags: u16,
    pub questions: u16,
    pub answers: u16,
    pub authority: u16,
    pub additional: u16,
}

impl DnsHeader {
    pub fn parse(data: &[u8]) -> DiscoveryResult<Self> {
        if data.len() < 12 {
            return Err(DiscoveryError::DnsParseError {
                message: "DNS header too short".to_string(),
            });
        }

        Ok(Self {
            transaction_id: u16::from_be_bytes([data[0], data[1]]),
            flags: u16::from_be_bytes([data[2], data[3]]),
            questions: u16::from_be_bytes([data[4], data[5]]),
            answers: u16::from_be_bytes([data[6], data[7]]),
            authority: u16::from_be_bytes([data[8], data[9]]),
            additional: u16::from_be_bytes([data[10], data[11]]),
        })
    }

    pub const fn is_response(&self) -> bool {
        (self.flags & 0x8000) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_header_parse() {
        let data = [
            0x00, 0x01, // Transaction ID
            0x84, 0x00, // Flags (response, authoritative)
            0x00, 0x01, // Questions
            0x00, 0x02, // Answers
            0x00, 0x00, // Authority
            0x00, 0x00, // Additional
        ];

        let header = DnsHeader::parse(&data).unwrap();
        assert_eq!(header.transaction_id, 1);
        assert!(header.is_response());
        assert_eq!(header.questions, 1);
        assert_eq!(header.answers, 2);
    }

    #[test]
    fn test_dns_header_too_short() {
        let data = [0x00, 0x01, 0x84];
        let result = DnsHeader::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_dns_header_not_response() {
        let data = [
            0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let header = DnsHeader::parse(&data).expect("parse");
        assert!(!header.is_response());
    }

    #[test]
    fn test_dns_header_authority_additional() {
        let data = [
            0x12, 0x34, 0x80, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04,
        ];
        let header = DnsHeader::parse(&data).expect("parse");
        assert_eq!(header.transaction_id, 0x1234);
        assert_eq!(header.authority, 3);
        assert_eq!(header.additional, 4);
    }

    #[test]
    fn test_dns_header_flags_query() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let header = DnsHeader::parse(&data).expect("parse");
        assert!(!header.is_response());
    }
}
