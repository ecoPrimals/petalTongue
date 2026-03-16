// SPDX-License-Identifier: AGPL-3.0-or-later
//! Universal User Interface (UUI) Glossary
//!
//! Canonical terminology for petalTongue's universal interface philosophy.
//! petalTongue is not a GUI — it is a translation layer between any computational
//! universe and any user type, across any modality.
//!
//! # Core Philosophy
//!
//! > "A graphical interface is simply the interconnection of information
//! >  and how it is represented."
//!
//! > "petalTongue doesn't have ONE interface — it IS the interface."
//!
//! > "One engine, infinite representations."
//!
//! # Two-Dimensional Universality
//!
//! 1. **Universe** — computational environment (substrate, display, input, audio, compute, network)
//! 2. **User** — intelligence interface (human, AI agent, non-human, hybrid)
//!
//! # SAME DAVE Model
//!
//! Neuroanatomy mnemonic for bidirectional UUI architecture:
//!
//! - **SAME**: **S**ensory **A**fferent, **M**otor **E**fferent
//! - **DAVE**: **D**orsal **A**fferent, **V**entral **E**fferent
//!
//! Sensory = input TO the primal. Motor = output FROM the primal.
//! Just like human proprioception requires both motor commands (efferent)
//! and sensory feedback (afferent) to know body position, primals need
//! both output and input verification to achieve complete self-awareness.
//!
//! # Terminology Evolution
//!
//! | Legacy Term | UUI Term | Reason |
//! |-------------|----------|--------|
//! | GUI | Display / Interface / Modality | Visual output is one modality among many |
//! | Screen | Display / Surface | Output is not always a screen |
//! | Click | Activate / Select | Input is not always a mouse |
//! | Visible | Perceivable | Perception includes audio, haptic, braille |
//! | See / Show | Perceive / Present | Not all users perceive visually |
//! | Draw / Paint | Render / Compose | Backends choose how to materialize |
//! | Window | Surface / Panel | Not all substrates have windows |
//! | Pixel | Display unit | Braille has dots, audio has samples |
//!
//! # User Types
//!
//! petalTongue serves any intelligence that can interface:
//!
//! - **Human (sighted)** — visual display modality (default)
//! - **Human (blind)** — audio sonification + braille + screen reader
//! - **Human (mobility-limited)** — voice + simplified interface
//! - **AI Agent** — JSON-RPC / GraphQL API modality
//! - **Non-human (dolphin)** — acoustic click patterns
//! - **Non-human (fungal)** — chemical gradient interface
//! - **Hybrid** — collaborative human + AI views
//!
//! # Modality Tiers
//!
//! | Tier | Modalities | Availability |
//! |------|-----------|--------------|
//! | 1 (Always) | Terminal, SVG, JSON, Text Description | Zero dependencies |
//! | 2 (Default) | Audio Sonification, Braille, Haptic | Platform sensors |
//! | 3 (Enhanced) | Egui Display, Web, VR | Optional backends |

/// Canonical name for this primal's role in the ecosystem.
pub const PRIMAL_ROLE: &str = "Universal User Interface";

/// What petalTongue IS — not a GUI, not a TUI, but a UUI.
pub const INTERFACE_PHILOSOPHY: &str =
    "petalTongue doesn't have ONE interface — it IS the interface.";

/// The UUI design principle.
pub const DESIGN_PRINCIPLE: &str = "One engine, infinite representations.";

/// Output channel types — the ways petalTongue presents information.
///
/// These are the modality categories, not specific implementations.
pub mod modality_names {
    /// Visual display output (egui, framebuffer, web canvas)
    pub const VISUAL: &str = "visual";
    /// Audio sonification output (speakers, headphones)
    pub const AUDIO: &str = "audio";
    /// Terminal text output (ratatui, plain text)
    pub const TERMINAL: &str = "terminal";
    /// Braille tactile output
    pub const BRAILLE: &str = "braille";
    /// Haptic feedback output (vibration, force)
    pub const HAPTIC: &str = "haptic";
    /// Text description output (screen readers, alt text)
    pub const DESCRIPTION: &str = "description";
    /// SVG vector export
    pub const SVG: &str = "svg";
    /// JSON machine-readable export
    pub const JSON_API: &str = "json-api";
    /// Acoustic patterns (non-human)
    pub const ACOUSTIC: &str = "acoustic";
    /// Chemical gradients (non-human)
    pub const CHEMICAL: &str = "chemical";
}

/// User type categories for interface adaptation.
pub mod user_types {
    /// Human with full visual perception
    pub const HUMAN_SIGHTED: &str = "human.sighted";
    /// Human without visual perception
    pub const HUMAN_BLIND: &str = "human.blind";
    /// Human with limited mobility
    pub const HUMAN_MOBILITY_LIMITED: &str = "human.mobility-limited";
    /// Human with limited hearing
    pub const HUMAN_DEAF: &str = "human.deaf";
    /// AI agent interfacing via API
    pub const AI_AGENT: &str = "ai.agent";
    /// Non-human: cetacean (dolphin, whale)
    pub const NONHUMAN_CETACEAN: &str = "nonhuman.cetacean";
    /// Non-human: fungal network
    pub const NONHUMAN_FUNGAL: &str = "nonhuman.fungal";
    /// Hybrid human + AI collaborative
    pub const HYBRID_COLLABORATIVE: &str = "hybrid.collaborative";
}

/// SAME DAVE bidirectional pathway names.
pub mod same_dave {
    /// Sensory Afferent — input signals coming TO the primal
    pub const SENSORY_AFFERENT: &str = "sensory.afferent";
    /// Motor Efferent — output signals going FROM the primal
    pub const MOTOR_EFFERENT: &str = "motor.efferent";
    /// Dorsal pathway — afferent (back → center)
    pub const DORSAL: &str = "dorsal";
    /// Ventral pathway — efferent (center → front)
    pub const VENTRAL: &str = "ventral";
}
