// SPDX-License-Identifier: AGPL-3.0-or-later
//! DNS packet parsing utilities for mDNS.
//!
//! Simplified DNS parser focused on service discovery (RFC 1035, RFC 6762, RFC 6763).

mod header;
mod name;
mod record;

pub use header::DnsHeader;
pub use name::NameParser;
pub use record::*;
