// SPDX-License-Identifier: AGPL-3.0-or-later
//! DNS domain name decoding: length-prefixed labels and name compression (RFC 1035).

use crate::errors::{DiscoveryError, DiscoveryResult};

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_name_parser_exceeds_bounds() {
        let data = [4, b't', b'e', b's', b't'];
        let parser = NameParser::new(&data);
        let result = parser.parse_name(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_name_parser_single_label() {
        let data = [4, b't', b'e', b's', b't', 0];
        let parser = NameParser::new(&data);
        let (name, len) = parser.parse_name(0).expect("parse");
        assert_eq!(name, "test");
        assert_eq!(len, 6);
    }
}
