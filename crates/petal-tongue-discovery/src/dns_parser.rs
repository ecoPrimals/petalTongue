// DNS packet parsing utilities for mDNS
//
// Simplified DNS parser focused on service discovery (RFC 1035, RFC 6762, RFC 6763)

use anyhow::Result;
use std::net::{Ipv4Addr, Ipv6Addr};

/// DNS record type
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
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
    fn from_u16(value: u16) -> Option<Self> {
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

/// DNS class
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum RecordClass {
    IN = 1, // Internet
}

/// Parsed DNS header
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
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            anyhow::bail!("DNS header too short");
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

    pub fn is_response(&self) -> bool {
        (self.flags & 0x8000) != 0
    }
}

/// DNS name parser
pub struct NameParser<'a> {
    data: &'a [u8],
}

impl<'a> NameParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Parse a DNS name starting at offset
    ///
    /// Returns (name, bytes_consumed)
    pub fn parse_name(&self, mut offset: usize) -> Result<(String, usize)> {
        let start_offset = offset;
        let mut name = String::new();
        let mut jumped = false;
        let mut jump_offset = 0;

        loop {
            if offset >= self.data.len() {
                anyhow::bail!("Name parsing exceeded packet bounds");
            }

            let len = self.data[offset];

            // Check for compression pointer (top 2 bits set)
            if (len & 0xC0) == 0xC0 {
                if offset + 1 >= self.data.len() {
                    anyhow::bail!("Compression pointer incomplete");
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
                anyhow::bail!("Label length exceeds packet bounds");
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
pub struct SrvRecord {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

impl SrvRecord {
    pub fn parse(data: &[u8], offset: usize, rdata: &[u8]) -> Result<Self> {
        if rdata.len() < 6 {
            anyhow::bail!("SRV rdata too short");
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
    pub fn parse(rdata: &[u8]) -> Result<Self> {
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
    pub fn parse(rdata: &[u8]) -> Result<Self> {
        if rdata.len() != 4 {
            anyhow::bail!("A record must be 4 bytes");
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
    pub fn parse(rdata: &[u8]) -> Result<Self> {
        if rdata.len() != 16 {
            anyhow::bail!("AAAA record must be 16 bytes");
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
    /// Returns (record, bytes_consumed)
    pub fn parse(data: &[u8], offset: usize) -> Result<(Self, usize)> {
        let parser = NameParser::new(data);
        let (name, name_len) = parser.parse_name(offset)?;

        let mut pos = offset + name_len;

        if pos + 10 > data.len() {
            anyhow::bail!("Resource record truncated");
        }

        let rtype = u16::from_be_bytes([data[pos], data[pos + 1]]);
        let rclass = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);
        let ttl = u32::from_be_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
        let rdlength = u16::from_be_bytes([data[pos + 8], data[pos + 9]]) as usize;

        pos += 10;

        if pos + rdlength > data.len() {
            anyhow::bail!("Resource record data truncated");
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
    pub fn as_srv(&self, full_data: &[u8], offset: usize) -> Result<SrvRecord> {
        SrvRecord::parse(full_data, offset, &self.rdata)
    }

    /// Parse as TXT record
    pub fn as_txt(&self) -> Result<TxtRecord> {
        TxtRecord::parse(&self.rdata)
    }

    /// Parse as A record
    pub fn as_a(&self) -> Result<ARecord> {
        ARecord::parse(&self.rdata)
    }

    /// Parse as AAAA record
    pub fn as_aaaa(&self) -> Result<AaaaRecord> {
        AaaaRecord::parse(&self.rdata)
    }

    /// Get record type enum
    pub fn record_type(&self) -> Option<RecordType> {
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
        assert_eq!(a.addr.to_string(), "192.168.1.100");
    }
}
