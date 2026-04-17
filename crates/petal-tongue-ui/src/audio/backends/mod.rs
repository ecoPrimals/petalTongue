// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio Backend Implementations
//!
//! Each backend implements the `AudioBackend` trait.
//! Backends are discovered at runtime - NO hardcoding!
//!
//! # Feature-gated stub backends
//!
//! - **`audio-socket`**: [`SocketBackend`] — discovers PipeWire/PulseAudio sockets but does not
//!   implement their wire protocols (see module docs in `socket.rs`).
//! - **`audio-direct`**: [`DirectBackend`] — discovers `/dev/snd` (etc.) but does not implement
//!   ALSA `ioctl` playback (see `direct.rs`).
//!
//! Without these features, [`crate::audio::AudioManager`] only registers software synthesis and
//! silent fallback.

#[cfg(feature = "audio-direct")]
mod direct;
mod network;
mod silent;
#[cfg(feature = "audio-socket")]
mod socket;
mod software;

// Re-exports
#[cfg(feature = "audio-direct")]
pub use direct::DirectBackend;
pub use network::NetworkBackend;
pub use silent::SilentBackend;
#[cfg(feature = "audio-socket")]
pub use socket::SocketBackend;
pub use software::SoftwareBackend;
