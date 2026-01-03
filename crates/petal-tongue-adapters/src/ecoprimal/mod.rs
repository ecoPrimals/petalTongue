//! ecoPrimals-specific adapters module

pub mod trust;
pub mod family;
pub mod capabilities;

pub use trust::EcoPrimalTrustAdapter;
pub use family::EcoPrimalFamilyAdapter;
pub use capabilities::EcoPrimalCapabilityAdapter;

