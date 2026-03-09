// SPDX-License-Identifier: AGPL-3.0-only
//! tarpc client tests

use std::time::Duration;

use super::TarpcClient;

#[test]
fn test_endpoint_parsing_valid() {
    let addr = TarpcClient::parse_endpoint("tarpc://localhost:9001").unwrap();
    assert_eq!(addr.port(), 9001);
}

#[test]
fn test_endpoint_parsing_with_ip() {
    let addr = TarpcClient::parse_endpoint("tarpc://127.0.0.1:9002").unwrap();
    assert_eq!(addr.port(), 9002);
}

#[test]
fn test_endpoint_parsing_invalid_no_prefix() {
    let result = TarpcClient::parse_endpoint("localhost:9001");
    assert!(result.is_err());
}

#[test]
fn test_endpoint_parsing_invalid_address() {
    let result = TarpcClient::parse_endpoint("tarpc://invalid");
    assert!(result.is_err());
}

#[test]
fn test_client_creation() {
    let client = TarpcClient::new("tarpc://localhost:9001").unwrap();
    assert_eq!(client.endpoint(), "tarpc://localhost:9001");
    assert_eq!(client.addr().port(), 9001);
}

#[test]
fn test_with_timeout_builder() {
    let client = TarpcClient::new("tarpc://localhost:9001")
        .unwrap()
        .with_timeout(Duration::from_secs(10));

    assert_eq!(client.timeout(), Duration::from_secs(10));
}

#[test]
fn test_debug_impl() {
    let client = TarpcClient::new("tarpc://localhost:9001").unwrap();
    let debug_str = format!("{client:?}");
    assert!(debug_str.contains("TarpcClient"));
    assert!(debug_str.contains("localhost:9001"));
}
