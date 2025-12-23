//! # petal-tongue-api
//!
//! API client for connecting to BiomeOS and other data sources

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod biomeos_client;

pub use biomeos_client::{BiomeOSClient, DiscoveredPrimal, DiscoveryResponse};
