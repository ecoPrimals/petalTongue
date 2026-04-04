// SPDX-License-Identifier: AGPL-3.0-or-later
//! DNS resource record types, typed rdata helpers, and generic RR parsing (RFC 1035; SRV RFC 2782).

use super::name::NameParser;
use crate::errors::{DiscoveryError, DiscoveryResult};
use rand::Rng;
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

/// SRV record data
#[derive(Debug, Clone)]
pub struct SrvRecord {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

impl SrvRecord {
    /// Choose one record per [RFC 2782](https://www.rfc-editor.org/rfc/rfc2782): lowest `priority`
    /// first; among ties, random selection weighted by `weight` (weights summing to 0 ⇒ uniform).
    #[must_use]
    pub fn select_by_priority(records: &[Self]) -> Option<&Self> {
        if records.is_empty() {
            return None;
        }
        let min_priority = records.iter().map(|r| r.priority).min()?;
        let candidates: Vec<&Self> = records
            .iter()
            .filter(|r| r.priority == min_priority)
            .collect();

        let sum: u32 = candidates.iter().map(|r| u32::from(r.weight)).sum();
        let mut rng = rand::thread_rng();

        if sum == 0 {
            let idx = rng.gen_range(0..candidates.len());
            return Some(candidates[idx]);
        }

        let mut roll = rng.gen_range(0..sum);
        for r in &candidates {
            let w = u32::from(r.weight);
            if w == 0 {
                continue;
            }
            if roll < w {
                return Some(*r);
            }
            roll -= w;
        }

        candidates.last().copied()
    }

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

/// AAAA record data (IPv6)
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

/// Generic DNS resource record
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
    fn test_srv_select_by_priority_prefers_lower_number() {
        let high = SrvRecord {
            priority: 10,
            weight: 100,
            port: 1,
            target: "a".to_string(),
        };
        let low = SrvRecord {
            priority: 0,
            weight: 1,
            port: 2,
            target: "b".to_string(),
        };
        assert_eq!(
            SrvRecord::select_by_priority(&[high, low])
                .expect("selection")
                .target,
            "b"
        );
    }

    #[test]
    fn test_srv_select_by_priority_empty() {
        assert!(SrvRecord::select_by_priority(&[]).is_none());
    }

    #[test]
    fn test_srv_select_by_priority_zero_weights_uniform() {
        let a = SrvRecord {
            priority: 0,
            weight: 0,
            port: 80,
            target: "only".to_string(),
        };
        assert_eq!(
            SrvRecord::select_by_priority(std::slice::from_ref(&a))
                .expect("selection")
                .target,
            "only"
        );
        let picks: std::collections::HashSet<_> = (0..32)
            .filter_map(|_| {
                SrvRecord::select_by_priority(&[
                    SrvRecord {
                        priority: 0,
                        weight: 0,
                        port: 1,
                        target: "x".to_string(),
                    },
                    SrvRecord {
                        priority: 0,
                        weight: 0,
                        port: 2,
                        target: "y".to_string(),
                    },
                ])
                .map(|r| r.target.clone())
            })
            .collect();
        assert!(picks.contains("x") || picks.contains("y"));
        assert_eq!(picks.len(), 2, "both targets should be reachable");
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
    fn test_txt_record_empty_rdata() {
        let data: [u8; 0] = [];
        let txt = TxtRecord::parse(&data).expect("parse");
        assert!(txt.attributes.is_empty());
    }
}
