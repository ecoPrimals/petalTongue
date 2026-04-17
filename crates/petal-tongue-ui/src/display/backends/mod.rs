// SPDX-License-Identifier: AGPL-3.0-or-later
//! Display backend implementations

#[cfg(feature = "discovered-display")]
pub mod discovered_display;
#[cfg(feature = "discovered-display")]
pub mod discovered_display_v2;
pub mod external;
pub mod framebuffer;
pub mod software;
