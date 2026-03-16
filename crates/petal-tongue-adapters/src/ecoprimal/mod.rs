// SPDX-License-Identifier: AGPL-3.0-or-later
//! ecoPrimals-specific adapters module

pub mod capabilities;
pub mod family;
pub mod trust;

pub use capabilities::EcoPrimalCapabilityAdapter;
pub use family::EcoPrimalFamilyAdapter;
pub use trust::EcoPrimalTrustAdapter;
