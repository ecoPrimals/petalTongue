// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::manual_async_fn)] // explicit `impl Future + Send` on `GUIModality`
//! # Modality System
//!
//! Defines the trait and types for output modalities.

use crate::error::Result;
use std::sync::Arc;

use crate::engine::UniversalRenderingEngine;
use crate::event::EngineEvent;

/// Modality Tier
///
/// Three-tier system for progressive enhancement:
/// - Tier 1: Always available (zero dependencies)
/// - Tier 2: Default available (minimal dependencies)
/// - Tier 3: Enhancement (optional capabilities)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModalityTier {
    /// Tier 1: Zero dependencies, always works
    /// Examples: Terminal, SVG export, JSON export
    AlwaysAvailable = 1,

    /// Tier 2: Minimal dependencies, usually available
    /// Examples: Audio output, PNG export
    DefaultAvailable = 2,

    /// Tier 3: Optional enhancements
    /// Examples: Egui, VR, GPU acceleration
    Enhancement = 3,
}

/// Accessibility features supported by a modality
#[derive(Debug, Clone, Default)]
#[expect(clippy::struct_excessive_bools)]
pub struct AccessibilityFeatures {
    /// Screen reader compatible
    pub screen_reader: bool,

    /// Keyboard-only navigation
    pub keyboard_only: bool,

    /// High contrast modes
    pub high_contrast: bool,

    /// For blind users (audio representation)
    pub blind_users: bool,

    /// Audio descriptions
    pub audio_description: bool,

    /// Spatial audio
    pub spatial_audio: bool,

    /// ARIA labels (web)
    pub aria_labels: bool,

    /// Semantic markup
    pub semantic_markup: bool,

    /// WCAG compliant
    pub wcag_compliant: bool,

    /// Gesture control
    pub gesture_control: bool,
}

/// What a modality can do
#[derive(Debug, Clone, Default)]
#[expect(clippy::struct_excessive_bools)]
pub struct ModalityCapabilities {
    /// Can handle user input (interactive)
    pub interactive: bool,

    /// Can present real-time updates
    pub realtime: bool,

    /// Can export to files
    pub export: bool,

    /// Supports animations
    pub animation: bool,

    /// Supports 3D rendering
    pub three_d: bool,

    /// Supports audio output
    pub audio: bool,

    /// Supports haptic feedback
    pub haptic: bool,

    /// Maximum graph size (None = unlimited)
    pub max_nodes: Option<usize>,

    /// Accessibility features
    pub accessibility: AccessibilityFeatures,
}

/// Default no-op modality used when no concrete GUI modality is selected.
pub struct NullModality;

/// Universal output modality
///
/// Each modality provides a different representation of the same
/// topology data. Modalities are discovered at runtime and can
/// run simultaneously.
pub trait GUIModality: Sized + Send + Sync {
    /// Get modality name (e.g., "terminal", "soundscape", "egui")
    fn name(&self) -> &'static str;

    /// Check if this modality is available in current environment
    fn is_available(&self) -> bool;

    /// Get modality tier (1, 2, or 3)
    fn tier(&self) -> ModalityTier;

    /// Initialize modality
    fn initialize(
        &mut self,
        engine: Arc<UniversalRenderingEngine<Self>>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Start rendering (blocking or returns handle)
    fn render(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Handle events from other modalities
    fn handle_event(
        &mut self,
        event: EngineEvent,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Shutdown gracefully
    fn shutdown(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get modality-specific capabilities
    fn capabilities(&self) -> ModalityCapabilities;
}

impl GUIModality for NullModality {
    fn name(&self) -> &'static str {
        "null"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn tier(&self) -> ModalityTier {
        ModalityTier::AlwaysAvailable
    }

    fn initialize(
        &mut self,
        _engine: Arc<UniversalRenderingEngine<Self>>,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    fn render(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    fn handle_event(
        &mut self,
        _event: EngineEvent,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    fn shutdown(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    fn capabilities(&self) -> ModalityCapabilities {
        ModalityCapabilities::default()
    }
}

/// Modality Registry
///
/// Manages available modalities and selects best for environment.
pub struct ModalityRegistry<M: GUIModality = NullModality> {
    modalities: indexmap::IndexMap<String, M>,
}

impl<M: GUIModality> ModalityRegistry<M> {
    /// Create new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            modalities: indexmap::IndexMap::new(),
        }
    }

    /// Register a modality
    pub fn register(&mut self, modality: M) {
        let name = modality.name().to_string();
        self.modalities.insert(name, modality);
    }

    /// Number of registered modalities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.modalities.len()
    }

    /// Returns true if no modalities are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.modalities.is_empty()
    }

    /// Check if a modality is registered
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.modalities.contains_key(name)
    }

    /// Get a modality by name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&M> {
        self.modalities.get(name)
    }

    /// Get mutable modality
    pub fn get_mut(&mut self, name: &str) -> Option<&mut M> {
        self.modalities.get_mut(name)
    }

    /// Get all available modalities (filtered by `is_available`)
    #[must_use]
    pub fn available(&self) -> Vec<&'static str> {
        self.modalities
            .values()
            .filter(|m| m.is_available())
            .map(GUIModality::name)
            .collect()
    }

    /// Get all modalities by tier
    #[must_use]
    pub fn by_tier(&self, tier: ModalityTier) -> Vec<&'static str> {
        self.modalities
            .values()
            .filter(|m| m.tier() == tier && m.is_available())
            .map(GUIModality::name)
            .collect()
    }

    /// Auto-select best modality for environment
    ///
    /// Returns a `'static` name string (from each modality's [`GUIModality::name`]) so callers
    /// may use the result after releasing any lock on this registry.
    #[must_use]
    pub fn auto_select(&self) -> Option<&'static str> {
        // Try in order of preference:
        // 1. Tier 3 (Enhancement) - interactive display
        // 2. Tier 2 (Default) - audio or terminal
        // 3. Tier 1 (Always) - terminal fallback

        for tier in [
            ModalityTier::Enhancement,
            ModalityTier::DefaultAvailable,
            ModalityTier::AlwaysAvailable,
        ] {
            let available = self.by_tier(tier);
            if !available.is_empty() {
                // Prefer interactive modalities
                for name in &available {
                    if let Some(modality) = self.get(name)
                        && modality.capabilities().interactive
                    {
                        return Some(*name);
                    }
                }
                // Otherwise return first available
                return available.first().copied();
            }
        }

        None
    }
}

impl Default for ModalityRegistry<NullModality> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{ModalityCapabilities, ModalityTier, *};

    // Mock modality for testing
    struct MockModality {
        name: &'static str,
        tier: ModalityTier,
        available: bool,
    }

    impl GUIModality for MockModality {
        fn name(&self) -> &'static str {
            self.name
        }

        fn is_available(&self) -> bool {
            self.available
        }

        fn tier(&self) -> ModalityTier {
            self.tier
        }

        fn initialize(
            &mut self,
            _engine: Arc<UniversalRenderingEngine<Self>>,
        ) -> impl std::future::Future<Output = Result<()>> + Send {
            async { Ok(()) }
        }

        fn render(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
            async { Ok(()) }
        }

        fn handle_event(
            &mut self,
            _event: EngineEvent,
        ) -> impl std::future::Future<Output = Result<()>> + Send {
            async { Ok(()) }
        }

        fn shutdown(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
            async { Ok(()) }
        }

        fn capabilities(&self) -> ModalityCapabilities {
            ModalityCapabilities::default()
        }
    }

    #[test]
    fn test_modality_registration() {
        let mut registry = ModalityRegistry::<MockModality>::new();

        registry.register(MockModality {
            name: "terminal",
            tier: ModalityTier::AlwaysAvailable,
            available: true,
        });

        assert!(registry.has("terminal"));
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_auto_select() {
        let mut registry = ModalityRegistry::<MockModality>::new();

        // Register Tier 1
        registry.register(MockModality {
            name: "terminal",
            tier: ModalityTier::AlwaysAvailable,
            available: true,
        });

        // Register Tier 3
        registry.register(MockModality {
            name: "egui",
            tier: ModalityTier::Enhancement,
            available: true,
        });

        // Should prefer Tier 3
        let selected = registry.auto_select();
        assert_eq!(selected, Some("egui"));
    }

    #[test]
    fn test_tier_filtering() {
        let mut registry = ModalityRegistry::<MockModality>::new();

        registry.register(MockModality {
            name: "terminal",
            tier: ModalityTier::AlwaysAvailable,
            available: true,
        });

        registry.register(MockModality {
            name: "svg",
            tier: ModalityTier::AlwaysAvailable,
            available: true,
        });

        let tier1 = registry.by_tier(ModalityTier::AlwaysAvailable);
        assert_eq!(tier1.len(), 2);
    }
}
