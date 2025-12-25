//! # petal-tongue-api
//!
//! API client for connecting to `BiomeOS` and other data sources

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Allow some pedantic warnings - addressing in future refactoring
#![allow(clippy::unused_self)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::redundant_closure_for_method_calls)]

pub mod biomeos_client;

pub use biomeos_client::{BiomeOSClient, DiscoveredPrimal, DiscoveryResponse};
