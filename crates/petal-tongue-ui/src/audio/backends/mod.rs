// SPDX-License-Identifier: AGPL-3.0-only
//! Audio Backend Implementations
//!
//! Each backend implements the `AudioBackend` trait.
//! Backends are discovered at runtime - NO hardcoding!

mod direct;
mod silent;
mod socket;
mod software;

// Re-exports
pub use direct::DirectBackend;
pub use silent::SilentBackend;
pub use socket::SocketBackend;
pub use software::SoftwareBackend;
