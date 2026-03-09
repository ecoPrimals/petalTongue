// SPDX-License-Identifier: AGPL-3.0-only
//! Proprioception - Complete Sensory-Motor Self-Awareness
//!
//! Like humans knowing their body position without light through feedback,
//! the primal knows its complete input/output state through bidirectional
//! verification loops.
//!
//! # SAME DAVE - Neuroanatomy Model
//!
//! This implements the neuroanatomy mnemonic for spinal cord pathways:
//!
//! - **SAME**: **S**ensory **A**fferent, **M**otor **E**fferent
//! - **DAVE**: **D**orsal **A**fferent, **V**entral **E**fferent
//!
//! In our model:
//! - **Sensory (Afferent)**: Input pathways - signals coming TO the primal (keyboard, mouse, etc.)
//! - **Motor (Efferent)**: Output pathways - signals going FROM the primal (display, audio, etc.)
//! - **Bidirectional Loop**: Both pathways working together = proprioception function
//!
//! Just like human proprioception requires both motor commands (efferent) and
//! sensory feedback (afferent) to know body position, primals need both output
//! and input verification to achieve complete self-awareness.

mod tracker;
mod types;

#[cfg(test)]
mod tests;

pub use tracker::ProprioceptionSystem;
pub use types::ProprioceptiveState;

use crate::input_verification::InputModality;
use crate::output_verification::OutputModality;
use tracing::info;

/// Initialize proprioception with common modalities
pub fn initialize_standard_proprioception() -> ProprioceptionSystem {
    let mut system = ProprioceptionSystem::new();

    // Register standard outputs
    system.register_output(OutputModality::Visual);
    system.register_output(OutputModality::Audio);
    system.register_output(OutputModality::Haptic);

    // Register standard inputs
    system.register_input(InputModality::Keyboard);
    system.register_input(InputModality::Pointer);
    system.register_input(InputModality::Audio);

    info!("✅ Standard proprioception initialized");

    system
}
