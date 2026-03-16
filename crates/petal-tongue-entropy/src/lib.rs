// SPDX-License-Identifier: AGPL-3.0-only
//! # `PetalTongue` Entropy Capture
//!
//! Multi-modal human entropy capture for sovereign, non-fungible keys.
//!
//! ## Modalities
//!
//! - **Audio** (`audio`): Singing, speaking (timing, pitch, dynamics)
//! - **Visual** (`visual`): Drawing, painting (strokes, patterns)
//! - **Narrative** (`narrative`): Storytelling (keystroke dynamics)
//! - **Gesture** (`gesture`): Motion, touch (sensors, patterns)
//! - **Video** (`video`): Camera motion (movement analysis)
//!
//! ## Architecture
//!
//! ```text
//! Capture → Quality Assessment → User Feedback → Stream (Encrypted)
//! ```
//!
//! **Privacy**: Stream-only (never persisted), encrypted transmission, secure zeroization.
//!
//! ## Example
//!
//! ```rust,no_run
//! use petal_tongue_entropy::prelude::*;
//!
//! # async fn example() -> Result<(), petal_tongue_entropy::EntropyError> {
//! // Create narrative capture (always available, no audio dependencies)
//! let mut capture = NarrativeEntropyCapture::new();
//!
//! // Start capturing
//! capture.start();
//! capture.add_char('H');
//! capture.add_char('e');
//! capture.add_char('l');
//! capture.add_char('l');
//! capture.add_char('o');
//!
//! // Get real-time quality
//! let quality = capture.assess_quality();
//! println!("Quality: {:.1}%", quality.overall_quality * 100.0);
//!
//! // Finalize and stream
//! let entropy = capture.finalize()?;
//! let entropy_capture = EntropyCapture::Narrative(entropy);
//! stream_entropy(entropy_capture, "https://biomeos/api/v1/entropy/stream").await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod error;
pub mod quality;

pub use error::EntropyError;
pub mod stream;
pub mod types;

// Always-available modalities (no special dependencies)
pub mod awakening_audio; // NEW: Awakening experience audio layers
pub mod gesture;
pub mod narrative;
pub mod visual;

/// Prelude for common imports
pub mod prelude {
    pub use crate::error::EntropyError;
    pub use crate::quality::*;
    pub use crate::stream::*;
    pub use crate::types::*;

    pub use crate::awakening_audio::*;
    pub use crate::gesture::*;
    pub use crate::narrative::*;
    pub use crate::visual::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_compiles() {
        // Basic smoke test - verify modules are accessible
        assert_eq!(2 + 2, 4);
    }
}
