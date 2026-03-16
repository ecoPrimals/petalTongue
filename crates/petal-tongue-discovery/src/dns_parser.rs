// SPDX-License-Identifier: AGPL-3.0-only
// DNS packet parsing utilities for mDNS
//
// Simplified DNS parser focused on service discovery (RFC 1035, RFC 6762, RFC 6763)

use crate::errors::{DiscoveryError, DiscoveryResult};
use std::net::{Ipv4Addr, Ipv6Addr};

/// DNS record type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(clippy::upper_case_acronyms)]
pub enum RecordType {
    A = 1,     // IPv4 address
    NS = 2,    // Name server
    CNAME = 5, // Canonical name
    PTR = 12,  // Pointer
    TXT = 16,  // Text
    AAAA = 28, // IPv6 address
    SRV = 33,  // Service locator
}

impl RecordType {
    const fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::A),
            2 => Some(Self::NS),
            5 => Some(Self::CNAME),
            12 => Some(Self::PTR),
            16 => Some(Self::TXT),
            28 => Some(Self::AAAA),
            33 => Some(Self::SRV),
            _ => None,
        }
    }
}

/// DNS class (RFC 1035 completeness; used in tests)
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordClass {
    IN = 1, // Internet
}

/// Parsed DNS header (RFC 1035; authority/additional used in tests)
#[cfg_attr(not(test), allow(dead_code))]
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

/// DNS name parser
pub struct NameParser<'a> {
    data: &'a [u8],
}

impl<'a> NameParser<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Parse a DNS name starting at offset
    ///
    /// Returns (name, `bytes_consumed`)
    pub fn parse_name(&self, mut offset: usize) -> DiscoveryResult<(String, usize)> {
        let start_offset = offset;
        let mut name = String::new();
        let mut jumped = false;
        let mut jump_offset = 0;

        loop {
            if offset >= self.data.len() {
                return Err(DiscoveryError::DnsParseError {
                    message: "Name parsing exceeded packet bounds".to_string(),
                });
            }

            let len = self.data[offset];

            // Check for compression pointer (top 2 bits set)
            if (len & 0xC0) == 0xC0 {
                if offset + 1 >= self.data.len() {
                    return Err(DiscoveryError::DnsParseError {
                        message: "Compression pointer incomplete".to_string(),
                    });
                }

                // Extract pointer offset (14 bits)
                let pointer = (((len & 0x3F) as usize) << 8) | (self.data[offset + 1] as usize);

                if !jumped {
                    jump_offset = offset + 2;
                    jumped = true;
                }

                offset = pointer;
                continue;
            }

            // End of name
            if len == 0 {
                offset += 1;
                break;
            }

            // Label
            if offset + 1 + len as usize > self.data.len() {
                return Err(DiscoveryError::DnsParseError {
                    message: "Label length exceeds packet bounds".to_string(),
                });
            }

            if !name.is_empty() {
                name.push('.');
            }

            let label = &self.data[offset + 1..offset + 1 + len as usize];
            name.push_str(&String::from_utf8_lossy(label));

            offset += 1 + len as usize;
        }

        let bytes_consumed = if jumped {
            jump_offset - start_offset
        } else {
            offset - start_offset
        };

        Ok((name, bytes_consumed))
    }
}

/// SRV record data
#[derive(Debug, Clone)]
#[expect(
    dead_code,
    reason = "RFC 2782 completeness; priority/weight parsed for future SRV routing logic"
)]
pub struct SrvRecord {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

impl SrvRecord {
    pub fn parse(data: &[u8], offset: usize, rdata: &[u8]) -> DiscoveryResult<Self> {
        if rdata.len() < 6 {
            return Err(DiscoveryError::DnsParseError {
                message: "SRV rdata too short".to_string(),
            });
        }

        let priority = u16::from_be_bytes([rdata[0], rdata[1]]);
        let weight = u16::from_be_bytes([rdata[2], rdata[3]]);
        let port = u16::from_be_bytes([rdata[4], rdata[5]]);

        let parser = NameParser::new(data);
        let (target, _) = parser.parse_name(offset + 6)?;

        Ok(Self {
            priority,
            weight,
            port,
            target,
        })
    }
}

/// TXT record data (key=value pairs)
#[derive(Debug, Clone)]
pub struct TxtRecord {
    pub attributes: Vec<(String, String)>,
}

impl TxtRecord {
    #[expect(clippy::unnecessary_wraps, reason = "Ok wrapper for struct literal")]
    pub fn parse(rdata: &[u8]) -> DiscoveryResult<Self> {
        let mut attributes = Vec::new();
        let mut offset = 0;

        while offset < rdata.len() {
            let len = rdata[offset] as usize;
            offset += 1;

            if offset + len > rdata.len() {
                break;
            }

            let txt = String::from_utf8_lossy(&rdata[offset..offset + len]);

            // Parse key=value pairs
            if let Some((key, value)) = txt.split_once('=') {
                attributes.push((key.to_string(), value.to_string()));
            }

            offset += len;
        }

        Ok(Self { attributes })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// A record data (IPv4)
#[derive(Debug, Clone)]
pub struct ARecord {
    pub addr: Ipv4Addr,
}

impl ARecord {
    pub fn parse(rdata: &[u8]) -> DiscoveryResult<Self> {
        if rdata.len() != 4 {
            return Err(DiscoveryError::DnsParseError {
                message: "A record must be 4 bytes".to_string(),
            });
        }

        Ok(Self {
            addr: Ipv4Addr::new(rdata[0], rdata[1], rdata[2], rdata[3]),
        })
    }
}

/// AAAA record data (IPv6; reserved for future IPv6 mDNS)
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone)]
pub struct AaaaRecord {
    pub addr: Ipv6Addr,
}

impl AaaaRecord {
    pub fn parse(rdata: &[u8]) -> DiscoveryResult<Self> {
        if rdata.len() != 16 {
            return Err(DiscoveryError::DnsParseError {
                message: "AAAA record must be 16 bytes".to_string(),
            });
        }

        let mut segments = [0u16; 8];
        for (i, segment) in segments.iter_mut().enumerate() {
            *segment = u16::from_be_bytes([rdata[i * 2], rdata[i * 2 + 1]]);
        }

        Ok(Self {
            addr: Ipv6Addr::from(segments),
        })
    }
}

/// Generic DNS resource record (rclass/ttl used in tests)
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug)]
pub struct ResourceRecord {
    pub name: String,
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    pub rdata: Vec<u8>,
}

impl ResourceRecord {
    /// Parse a resource record starting at offset
    ///
    /// Returns (record, `bytes_consumed`)
    pub fn parse(data: &[u8], offset: usize) -> DiscoveryResult<(Self, usize)> {
        let parser = NameParser::new(data);
        let (name, name_len) = parser.parse_name(offset)?;

        let mut pos = offset + name_len;

        if pos + 10 > data.len() {
            return Err(DiscoveryError::DnsParseError {
                message: "Resource record truncated".to_string(),
            });
        }

        let rtype = u16::from_be_bytes([data[pos], data[pos + 1]]);
        let rclass = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);
        let ttl = u32::from_be_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
        let rdlength = u16::from_be_bytes([data[pos + 8], data[pos + 9]]) as usize;

        pos += 10;

        if pos + rdlength > data.len() {
            return Err(DiscoveryError::DnsParseError {
                message: "Resource record data truncated".to_string(),
            });
        }

        let rdata = data[pos..pos + rdlength].to_vec();
        pos += rdlength;

        Ok((
            Self {
                name,
                rtype,
                rclass,
                ttl,
                rdata,
            },
            pos - offset,
        ))
    }

    /// Parse as SRV record
    pub fn as_srv(&self, full_data: &[u8], offset: usize) -> DiscoveryResult<SrvRecord> {
        SrvRecord::parse(full_data, offset, &self.rdata)
    }

    /// Parse as TXT record
    pub fn as_txt(&self) -> DiscoveryResult<TxtRecord> {
        TxtRecord::parse(&self.rdata)
    }

    /// Parse as A record
    pub fn as_a(&self) -> DiscoveryResult<ARecord> {
        ARecord::parse(&self.rdata)
    }

    /// Parse as AAAA record
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn as_aaaa(&self) -> DiscoveryResult<AaaaRecord> {
        AaaaRecord::parse(&self.rdata)
    }

    /// Get record type enum
    pub const fn record_type(&self) -> Option<RecordType> {
        RecordType::from_u16(self.rtype)
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
    fn test_name_parser_simple() {
        let data = [
            4, b't', b'e', b's', b't', // "test"
            3, b'c', b'o', b'm', // "com"
            0,    // end
        ];

        let parser = NameParser::new(&data);
        let (name, len) = parser.parse_name(0).unwrap();

        assert_eq!(name, "test.com");
        assert_eq!(len, 10);
    }

    #[test]
    fn test_txt_record_parse() {
        let data = [
            7, b'k', b'e', b'y', b'=', b'v', b'a', b'l', // "key=val"
            9, b'p', b'o', b'r', b't', b'=', b'8', b'0', b'8', b'0', // "port=8080"
        ];

        let txt = TxtRecord::parse(&data).unwrap();
        assert_eq!(txt.attributes.len(), 2);
        assert_eq!(txt.get("key"), Some("val"));
        assert_eq!(txt.get("port"), Some("8080"));
    }

    #[test]
    fn test_a_record_parse() {
        let data = [192, 168, 1, 100];
        let a = ARecord::parse(&data).unwrap();
        assert_eq!(a.addr.to_string(), "192.0.2.100");
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
    fn test_name_parser_compression() {
        let data = [
            4, b't', b'e', b's', b't', 3, b'c', b'o', b'm', 0, 0xC0, 0x00,
        ];
        let parser = NameParser::new(&data);
        let (name, len) = parser.parse_name(10).expect("parse with compression");
        assert_eq!(name, "test.com");
        assert_eq!(len, 2);
    }

    #[test]
    fn test_txt_record_no_equals() {
        let data = [4, b't', b'e', b's', b't'];
        let txt = TxtRecord::parse(&data).expect("parse");
        assert!(txt.attributes.is_empty());
    }

    #[test]
    fn test_a_record_wrong_length() {
        let data = [192, 168, 1];
        let result = ARecord::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_aaaa_record_parse() {
        let mut data = [0u8; 16];
        data[0] = 0x20;
        data[1] = 0x01;
        data[2] = 0x0d;
        data[3] = 0xb8;
        let aaaa = AaaaRecord::parse(&data).expect("parse");
        assert!(aaaa.addr.to_string().starts_with("2001:"));
    }

    #[test]
    fn test_aaaa_record_wrong_length() {
        let data = [0u8; 8];
        let result = AaaaRecord::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_record_type_from_u16() {
        assert_eq!(RecordType::from_u16(1), Some(RecordType::A));
        assert_eq!(RecordType::from_u16(33), Some(RecordType::SRV));
        assert_eq!(RecordType::from_u16(99), None);
    }

    #[test]
    fn test_txt_get_missing() {
        let data = [7, b'k', b'e', b'y', b'=', b'v', b'a', b'l'];
        let txt = TxtRecord::parse(&data).unwrap();
        assert_eq!(txt.get("missing"), None);
    }

    #[test]
    fn test_record_type_all_variants() {
        assert_eq!(RecordType::from_u16(1), Some(RecordType::A));
        assert_eq!(RecordType::from_u16(2), Some(RecordType::NS));
        assert_eq!(RecordType::from_u16(5), Some(RecordType::CNAME));
        assert_eq!(RecordType::from_u16(12), Some(RecordType::PTR));
        assert_eq!(RecordType::from_u16(16), Some(RecordType::TXT));
        assert_eq!(RecordType::from_u16(28), Some(RecordType::AAAA));
        assert_eq!(RecordType::from_u16(33), Some(RecordType::SRV));
        assert_eq!(RecordType::from_u16(0), None);
        assert_eq!(RecordType::from_u16(255), None);
    }

    #[test]
    fn test_resource_record_parse() {
        let data = [
            4, b't', b'e', b's', b't', 3, b'c', b'o', b'm', 0, // name "test.com"
            0x00, 0x01, // A record
            0x00, 0x01, // IN class
            0x00, 0x00, 0x00, 0x3c, // TTL 60
            0x00, 0x04, // rdlength 4
            192, 168, 1, 1, // A record data
        ];
        let (rr, consumed) = ResourceRecord::parse(&data, 0).expect("parse");
        assert_eq!(rr.name, "test.com");
        assert_eq!(rr.rtype, 1);
        assert_eq!(rr.rclass, 1);
        assert_eq!(rr.ttl, 60);
        assert_eq!(rr.rdata.len(), 4);
        assert_eq!(rr.record_type(), Some(RecordType::A));
        assert!(consumed > 0);
    }

    #[test]
    fn test_resource_record_as_a() {
        let data = [
            4, b't', b'e', b's', b't', 0, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3c, 0x00,
            0x04, 10, 0, 0, 1,
        ];
        let (rr, _) = ResourceRecord::parse(&data, 0).expect("parse");
        let a = rr.as_a().expect("A record");
        assert_eq!(a.addr.to_string(), "10.0.0.1");
    }

    #[test]
    fn test_resource_record_as_txt() {
        let data = [
            4, b't', b'e', b's', b't', 0, 0x00, 0x10, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3c, 0x00,
            0x0a, 9, b'v', b'e', b'r', b'=', b'1', b'.', b'0', b'.', b'0',
        ];
        let (rr, _) = ResourceRecord::parse(&data, 0).expect("parse");
        let txt = rr.as_txt().expect("TXT record");
        assert_eq!(txt.get("ver"), Some("1.0.0"));
    }

    #[test]
    fn test_resource_record_as_aaaa() {
        let mut data = [0u8; 32]; // name(6) + header(10) + AAAA rdata(16)
        data[0] = 4;
        data[1] = b't';
        data[2] = b'e';
        data[3] = b's';
        data[4] = b't';
        data[5] = 0;
        data[6] = 0x00;
        data[7] = 0x1c; // AAAA type
        data[8] = 0x00;
        data[9] = 0x01;
        data[10] = 0x00;
        data[11] = 0x00;
        data[12] = 0x00;
        data[13] = 0x3c;
        data[14] = 0x00;
        data[15] = 0x10; // rdlength 16
        data[16] = 0x20;
        data[17] = 0x01;
        data[18] = 0x0d;
        data[19] = 0xb8;
        let (rr, _) = ResourceRecord::parse(&data, 0).expect("parse");
        let aaaa = rr.as_aaaa().expect("AAAA record");
        assert!(aaaa.addr.to_string().starts_with("2001:"));
    }

    #[test]
    fn test_resource_record_truncated() {
        let data = [4, b't', b'e', b's', b't', 0];
        let result = ResourceRecord::parse(&data, 0);
        assert!(result.is_err());
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
    fn test_name_parser_exceeds_bounds() {
        let data = [4, b't', b'e', b's', b't'];
        let parser = NameParser::new(&data);
        let result = parser.parse_name(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_srv_record_parse_rdata_too_short() {
        let data = [0u8; 5];
        let result = SrvRecord::parse(&[], 0, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_srv_record_parse_success() {
        let mut packet = [0u8; 12];
        packet[6] = 4;
        packet[7] = b't';
        packet[8] = b'e';
        packet[9] = b's';
        packet[10] = b't';
        packet[11] = 0;
        let rdata = [
            0x00, 0x00, 0x00, 0x00, 0x1F, 0x90, 4, b't', b'e', b's', b't', 0,
        ];
        let result = SrvRecord::parse(&packet, 0, &rdata);
        assert!(result.is_ok());
        let srv = result.unwrap();
        assert_eq!(srv.port, 8080);
        assert_eq!(srv.target, "test");
    }

    #[test]
    fn test_record_class_in() {
        let _ = RecordClass::IN;
    }

    #[test]
    fn test_name_parser_single_label() {
        let data = [4, b't', b'e', b's', b't', 0];
        let parser = NameParser::new(&data);
        let (name, len) = parser.parse_name(0).expect("parse");
        assert_eq!(name, "test");
        assert_eq!(len, 6);
    }

    #[test]
    fn test_txt_record_multiple_pairs() {
        let data = [
            3, b'a', b'=', b'1', 3, b'b', b'=', b'2', 5, b'k', b'e', b'y', b'=', b'x',
        ];
        let txt = TxtRecord::parse(&data).expect("parse");
        assert_eq!(txt.attributes.len(), 3);
        assert_eq!(txt.get("a"), Some("1"));
        assert_eq!(txt.get("b"), Some("2"));
        assert_eq!(txt.get("key"), Some("x"));
    }

    #[test]
    fn test_record_type_ns_cname() {
        assert_eq!(RecordType::from_u16(2), Some(RecordType::NS));
        assert_eq!(RecordType::from_u16(5), Some(RecordType::CNAME));
    }

    #[test]
    fn test_dns_header_flags_query() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let header = DnsHeader::parse(&data).expect("parse");
        assert!(!header.is_response());
    }

    #[test]
    fn test_txt_record_empty_rdata() {
        let data: [u8; 0] = [];
        let txt = TxtRecord::parse(&data).expect("parse");
        assert!(txt.attributes.is_empty());
    }
}
