// SPDX-License-Identifier: AGPL-3.0-or-later
//! Output Verification - Universal awareness of output reaching end state
//!
//! This generalizes display verification to ALL outputs:
//! - Visual (display, AR overlay, VR headset)
//! - Audio (speakers, headphones, bone conduction)
//! - Haptic (vibration, force feedback, future neural)
//! - Any future output modality
//!
//! Key insight: Like human proprioception, we need feedback to know
//! if our outputs are actually reaching the user.

use petal_tongue_core::rendering_awareness::VisibilityState;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Output modality type (agnostic - no specific vendors)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutputModality {
    /// Visual output (any display technology)
    Visual,
    /// Audio output (any audio technology)
    Audio,
    /// Haptic output (any tactile feedback technology)
    Haptic,
    /// Olfactory output (scent, future tech)
    Olfactory,
    /// Thermal output (temperature feedback)
    Thermal,
    /// Generic output (unknown/future modality)
    Generic(String),
}

/// Output topology - where is the output going?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputTopology {
    /// Direct to user (output device directly at user)
    Direct,

    /// Forwarded through intermediary (remote, network, proxy)
    Forwarded,

    /// Nested in another system (compositor, mixer, virtual device)
    Nested,

    /// Virtual (no physical output, memory/file only)
    Virtual,

    /// Unknown topology
    Unknown,
}

/// Output confirmation method - how can we know it reached the user?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputConfirmation {
    /// User interaction confirms they can perceive the output
    UserInteraction,

    /// Device acknowledgment (hardware confirmed receipt)
    DeviceAck,

    /// Echo/reflection detected (output bounced back as input)
    Echo,

    /// Indirect evidence (system metrics suggest delivery)
    IndirectEvidence,

    /// No confirmation possible
    Unconfirmable,
}

/// Output verification result for a single modality
#[derive(Debug, Clone)]
pub struct OutputVerification {
    /// Which output modality is this for?
    pub modality: OutputModality,

    /// Is output device available?
    pub device_available: bool,

    /// Is output being sent?
    pub output_active: bool,

    /// Output topology (direct, forwarded, nested, virtual)
    pub topology: OutputTopology,

    /// Can we confirm output reached user?
    pub reaches_user: bool,

    /// How was this confirmed (or why can't we confirm)?
    pub confirmation_method: OutputConfirmation,

    /// Evidence collected about output path
    pub evidence: Vec<String>,

    /// Current state (Confirmed, Probable, Uncertain, Unknown)
    pub state: VisibilityState,

    /// Last time we confirmed output reached user
    pub last_confirmed: Option<Instant>,

    /// Human-readable status
    pub status_message: String,

    /// Suggested action if there's an issue
    pub suggested_action: Option<String>,
}

impl OutputVerification {
    /// Create a new unverified output
    #[must_use]
    pub fn unverified(modality: OutputModality) -> Self {
        Self {
            modality,
            device_available: false,
            output_active: false,
            topology: OutputTopology::Unknown,
            reaches_user: false,
            confirmation_method: OutputConfirmation::Unconfirmable,
            evidence: vec![],
            state: VisibilityState::Unknown,
            last_confirmed: None,
            status_message: "Output not yet verified".to_string(),
            suggested_action: None,
        }
    }

    /// Update with user interaction (confirms user can perceive output)
    pub fn confirm_via_interaction(&mut self) {
        self.reaches_user = true;
        self.confirmation_method = OutputConfirmation::UserInteraction;
        self.state = VisibilityState::Confirmed;
        self.last_confirmed = Some(Instant::now());
        self.status_message = format!(
            "{:?} output confirmed via user interaction - user can perceive output",
            self.modality
        );
        self.suggested_action = None;
        self.evidence
            .push("User interaction confirms perception".to_string());
    }

    /// Update with device acknowledgment
    pub fn confirm_via_device_ack(&mut self, device_info: String) {
        self.reaches_user = true; // Device ack suggests it's working
        self.confirmation_method = OutputConfirmation::DeviceAck;
        self.state = VisibilityState::Probable; // Not as strong as user interaction
        self.last_confirmed = Some(Instant::now());
        self.status_message = format!(
            "{:?} output acknowledged by device: {}",
            self.modality, device_info
        );
        self.evidence
            .push(format!("Device acknowledgment: {device_info}"));
    }

    /// Update with echo/reflection (output came back as input)
    pub fn confirm_via_echo(&mut self, echo_info: String) {
        self.reaches_user = true; // Echo proves round-trip
        self.confirmation_method = OutputConfirmation::Echo;
        self.state = VisibilityState::Confirmed;
        self.last_confirmed = Some(Instant::now());
        self.status_message = format!(
            "{:?} output confirmed via echo/reflection: {}",
            self.modality, echo_info
        );
        self.evidence.push(format!("Echo detected: {echo_info}"));
    }

    /// Check if confirmation is stale (no recent confirmation)
    #[must_use]
    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.last_confirmed.is_none_or(|t| t.elapsed() > max_age)
    }
}

/// Universal output verification system
pub struct OutputVerificationSystem {
    /// Verifications for each active output modality
    verifications: std::collections::HashMap<OutputModality, OutputVerification>,

    /// Last time we updated verifications
    last_update: Instant,
}

impl OutputVerificationSystem {
    /// Create a new output verification system
    #[must_use]
    pub fn new() -> Self {
        Self {
            verifications: std::collections::HashMap::new(),
            last_update: Instant::now(),
        }
    }

    /// Register an output modality for verification
    pub fn register_output(&mut self, modality: OutputModality) {
        if !self.verifications.contains_key(&modality) {
            info!("📤 Registering output modality: {:?}", modality);
            self.verifications
                .insert(modality.clone(), OutputVerification::unverified(modality));
        }
    }

    /// Update output verification based on user interaction
    /// This is the KEY: user interaction confirms they can perceive the output!
    pub fn confirm_via_interaction(&mut self, modality: &OutputModality) {
        if let Some(verification) = self.verifications.get_mut(modality) {
            verification.confirm_via_interaction();
            debug!("✅ {:?} output confirmed via user interaction", modality);
        }
    }

    /// Update output verification based on device acknowledgment
    pub fn confirm_via_device(&mut self, modality: &OutputModality, device_info: String) {
        if let Some(verification) = self.verifications.get_mut(modality) {
            verification.confirm_via_device_ack(device_info);
            debug!("✅ {:?} output acknowledged by device", modality);
        }
    }

    /// Update output verification based on echo/reflection
    pub fn confirm_via_echo(&mut self, modality: &OutputModality, echo_info: String) {
        if let Some(verification) = self.verifications.get_mut(modality) {
            verification.confirm_via_echo(echo_info);
            debug!("✅ {:?} output confirmed via echo", modality);
        }
    }

    /// Get verification for a specific modality
    #[must_use]
    pub fn get_verification(&self, modality: &OutputModality) -> Option<&OutputVerification> {
        self.verifications.get(modality)
    }

    /// Get all output verifications
    #[must_use]
    pub fn get_all_verifications(&self) -> Vec<&OutputVerification> {
        self.verifications.values().collect()
    }

    /// Check if any outputs are unconfirmed
    #[must_use]
    pub fn has_unconfirmed_outputs(&self) -> bool {
        self.verifications.values().any(|v| !v.reaches_user)
    }

    /// Get status summary
    #[must_use]
    pub fn get_status_summary(&self) -> String {
        let total = self.verifications.len();
        let confirmed = self
            .verifications
            .values()
            .filter(|v| v.reaches_user)
            .count();
        let stale = self
            .verifications
            .values()
            .filter(|v| v.is_stale(Duration::from_secs(30)))
            .count();

        format!("Outputs: {confirmed}/{total} confirmed, {stale} stale")
    }

    /// Perform periodic verification update
    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);

        // Only update every 5 seconds
        if elapsed < Duration::from_secs(5) {
            return;
        }

        self.last_update = now;

        // Check for stale confirmations
        for (modality, verification) in &mut self.verifications {
            if verification.is_stale(Duration::from_secs(300)) {
                warn!(
                    "⚠️  {:?} output confirmation is stale (no recent confirmation)",
                    modality
                );
                verification.state = VisibilityState::Uncertain;
                verification.status_message = format!(
                    "{modality:?} output: No recent confirmation, cannot verify user perception"
                );
                verification.suggested_action = Some(
                    "Interact with the application to confirm you can perceive this output"
                        .to_string(),
                );
            }
        }
    }
}

impl Default for OutputVerificationSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect visual output topology (using existing display verification logic)
#[must_use]
pub fn detect_visual_topology() -> (OutputTopology, Vec<String>) {
    // Import from display_verification
    let (topo, evidence) = crate::display_verification::detect_display_topology();

    let output_topo = match topo {
        crate::display_verification::DisplayTopology::DirectLocal => OutputTopology::Direct,
        crate::display_verification::DisplayTopology::Forwarded => OutputTopology::Forwarded,
        crate::display_verification::DisplayTopology::Nested => OutputTopology::Nested,
        crate::display_verification::DisplayTopology::Virtual => OutputTopology::Virtual,
        crate::display_verification::DisplayTopology::Unknown => OutputTopology::Unknown,
    };

    (output_topo, evidence)
}

/// Detect audio output topology (agnostic)
#[must_use]
pub fn detect_audio_topology() -> (OutputTopology, Vec<String>) {
    let mut evidence = Vec::new();

    // EVOLVED: Audio Canvas - direct device detection!
    // Check /dev/snd for audio devices (Linux)
    if let Ok(devices) = crate::audio_canvas::AudioCanvas::discover() {
        if devices.is_empty() {
            evidence.push("No audio devices found in /dev/snd".to_string());
        } else {
            evidence.push(format!(
                "Audio Canvas: {} device(s) found in /dev/snd (100% pure Rust!)",
                devices.len()
            ));

            // Check for virtual/network audio by device names
            for device in &devices {
                if let Some(name) = device.to_str() {
                    let device_lower = name.to_lowercase();
                    if device_lower.contains("network")
                        || device_lower.contains("tunnel")
                        || device_lower.contains("remote")
                    {
                        evidence.push("Network audio device detected".to_string());
                        return (OutputTopology::Forwarded, evidence);
                    }
                }
            }

            return (OutputTopology::Direct, evidence);
        }
    } else {
        evidence.push("Failed to scan /dev/snd for audio devices".to_string());
    }

    // Check for ALSA as fallback indicator
    if std::path::Path::new("/proc/asound/cards").exists() {
        evidence.push("ALSA audio devices present (but using Audio Canvas!)".to_string());
    }

    // Default: unknown if no devices found
    if evidence.is_empty() {
        evidence.push("No audio output detected".to_string());
    }

    (OutputTopology::Unknown, evidence)
}

/// Detect haptic output topology (agnostic)
#[must_use]
pub fn detect_haptic_topology() -> (OutputTopology, Vec<String>) {
    let mut evidence = Vec::new();

    // Check for input devices that support force feedback
    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        let ff_devices: Vec<_> = entries
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_name().to_string_lossy().starts_with("event"))
            .collect();

        if !ff_devices.is_empty() {
            evidence.push(format!(
                "Found {} input devices (may support haptic)",
                ff_devices.len()
            ));
        }
    }

    // Gamepad rumble support?
    if std::path::Path::new("/sys/class/input").exists() {
        evidence.push("Input subsystem available".to_string());
    }

    if evidence.is_empty() {
        evidence.push("No haptic output detected".to_string());
        (OutputTopology::Unknown, evidence)
    } else {
        (OutputTopology::Direct, evidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_output_modality_variants() {
        assert_eq!(OutputModality::Visual, OutputModality::Visual);
        assert_eq!(OutputModality::Audio, OutputModality::Audio);
        assert_eq!(OutputModality::Haptic, OutputModality::Haptic);
        assert_eq!(OutputModality::Olfactory, OutputModality::Olfactory);
        assert_eq!(OutputModality::Thermal, OutputModality::Thermal);
        assert_eq!(
            OutputModality::Generic("custom".to_string()),
            OutputModality::Generic("custom".to_string())
        );
        assert_ne!(OutputModality::Visual, OutputModality::Audio);
    }

    #[test]
    fn test_output_topology_variants() {
        assert_eq!(OutputTopology::Direct, OutputTopology::Direct);
        assert_eq!(OutputTopology::Forwarded, OutputTopology::Forwarded);
        assert_eq!(OutputTopology::Nested, OutputTopology::Nested);
        assert_eq!(OutputTopology::Virtual, OutputTopology::Virtual);
        assert_eq!(OutputTopology::Unknown, OutputTopology::Unknown);
    }

    #[test]
    fn test_output_confirmation_variants() {
        assert_eq!(
            OutputConfirmation::UserInteraction,
            OutputConfirmation::UserInteraction
        );
        assert_eq!(OutputConfirmation::DeviceAck, OutputConfirmation::DeviceAck);
        assert_eq!(OutputConfirmation::Echo, OutputConfirmation::Echo);
        assert_eq!(
            OutputConfirmation::IndirectEvidence,
            OutputConfirmation::IndirectEvidence
        );
        assert_eq!(
            OutputConfirmation::Unconfirmable,
            OutputConfirmation::Unconfirmable
        );
    }

    #[test]
    fn test_output_verification_unverified() {
        let v = OutputVerification::unverified(OutputModality::Visual);
        assert!(!v.device_available);
        assert!(!v.output_active);
        assert_eq!(v.topology, OutputTopology::Unknown);
        assert!(!v.reaches_user);
        assert_eq!(v.confirmation_method, OutputConfirmation::Unconfirmable);
        assert!(v.evidence.is_empty());
        assert!(v.last_confirmed.is_none());
        assert!(v.status_message.contains("not yet verified"));
        assert!(v.suggested_action.is_none());
    }

    #[test]
    fn test_confirm_via_interaction() {
        let mut v = OutputVerification::unverified(OutputModality::Audio);
        v.confirm_via_interaction();
        assert!(v.reaches_user);
        assert_eq!(v.confirmation_method, OutputConfirmation::UserInteraction);
        assert!(v.last_confirmed.is_some());
        assert!(v.status_message.contains("user interaction"));
        assert!(v.suggested_action.is_none());
        assert!(!v.evidence.is_empty());
    }

    #[test]
    fn test_confirm_via_device_ack() {
        let mut v = OutputVerification::unverified(OutputModality::Visual);
        v.confirm_via_device_ack("HDMI-1".to_string());
        assert!(v.reaches_user);
        assert_eq!(v.confirmation_method, OutputConfirmation::DeviceAck);
        assert!(v.last_confirmed.is_some());
        assert!(v.status_message.contains("HDMI-1"));
        assert!(!v.evidence.is_empty());
    }

    #[test]
    fn test_confirm_via_echo() {
        let mut v = OutputVerification::unverified(OutputModality::Audio);
        v.confirm_via_echo("mic feedback".to_string());
        assert!(v.reaches_user);
        assert_eq!(v.confirmation_method, OutputConfirmation::Echo);
        assert!(v.last_confirmed.is_some());
        assert!(v.status_message.contains("echo"));
        assert!(v.evidence.iter().any(|e| e.contains("mic feedback")));
    }

    #[test]
    fn test_is_stale_none_confirmed() {
        let v = OutputVerification::unverified(OutputModality::Visual);
        assert!(v.is_stale(Duration::from_secs(30)));
    }

    #[test]
    fn test_is_stale_recently_confirmed() {
        let mut v = OutputVerification::unverified(OutputModality::Visual);
        v.confirm_via_interaction();
        assert!(!v.is_stale(Duration::from_secs(300)));
    }

    #[test]
    fn test_output_verification_system_new() {
        let sys = OutputVerificationSystem::new();
        assert!(sys.get_all_verifications().is_empty());
        assert!(!sys.has_unconfirmed_outputs());
    }

    #[test]
    fn test_output_verification_system_default() {
        let sys = OutputVerificationSystem::default();
        assert!(sys.get_all_verifications().is_empty());
    }

    #[test]
    fn test_register_output() {
        let mut sys = OutputVerificationSystem::new();
        sys.register_output(OutputModality::Visual);
        sys.register_output(OutputModality::Audio);
        assert_eq!(sys.get_all_verifications().len(), 2);
        assert!(sys.has_unconfirmed_outputs());
        assert!(sys.get_verification(&OutputModality::Visual).is_some());
        assert!(sys.get_verification(&OutputModality::Audio).is_some());
        assert!(sys.get_verification(&OutputModality::Haptic).is_none());
    }

    #[test]
    fn test_register_output_idempotent() {
        let mut sys = OutputVerificationSystem::new();
        sys.register_output(OutputModality::Visual);
        sys.register_output(OutputModality::Visual);
        assert_eq!(sys.get_all_verifications().len(), 1);
    }

    #[test]
    fn test_confirm_via_interaction_system() {
        let mut sys = OutputVerificationSystem::new();
        sys.register_output(OutputModality::Visual);
        sys.confirm_via_interaction(&OutputModality::Visual);
        assert!(!sys.has_unconfirmed_outputs());
        let v = sys.get_verification(&OutputModality::Visual).unwrap();
        assert!(v.reaches_user);
    }

    #[test]
    fn test_confirm_via_device_system() {
        let mut sys = OutputVerificationSystem::new();
        sys.register_output(OutputModality::Audio);
        sys.confirm_via_device(&OutputModality::Audio, "speaker-0".to_string());
        let v = sys.get_verification(&OutputModality::Audio).unwrap();
        assert!(v.reaches_user);
        assert!(v.status_message.contains("speaker-0"));
    }

    #[test]
    fn test_confirm_via_echo_system() {
        let mut sys = OutputVerificationSystem::new();
        sys.register_output(OutputModality::Haptic);
        sys.confirm_via_echo(&OutputModality::Haptic, "feedback".to_string());
        let v = sys.get_verification(&OutputModality::Haptic).unwrap();
        assert!(v.reaches_user);
    }

    #[test]
    fn test_get_status_summary() {
        let mut sys = OutputVerificationSystem::new();
        sys.register_output(OutputModality::Visual);
        sys.register_output(OutputModality::Audio);
        sys.confirm_via_interaction(&OutputModality::Visual);
        let summary = sys.get_status_summary();
        assert!(summary.contains("Outputs:"));
        assert!(summary.contains('1'));
        assert!(summary.contains('2'));
    }

    #[test]
    fn test_detect_visual_topology() {
        let (topo, evidence) = detect_visual_topology();
        assert!(
            matches!(
                topo,
                OutputTopology::Direct
                    | OutputTopology::Forwarded
                    | OutputTopology::Nested
                    | OutputTopology::Virtual
                    | OutputTopology::Unknown
            ),
            "topo = {topo:?}"
        );
        assert!(!evidence.is_empty() || topo == OutputTopology::Unknown);
    }

    #[test]
    fn test_detect_audio_topology() {
        let (topo, evidence) = detect_audio_topology();
        assert!(
            matches!(
                topo,
                OutputTopology::Direct | OutputTopology::Forwarded | OutputTopology::Unknown
            ),
            "topo = {topo:?}"
        );
        assert!(!evidence.is_empty());
    }

    #[test]
    fn test_detect_haptic_topology() {
        let (topo, evidence) = detect_haptic_topology();
        assert!(
            matches!(topo, OutputTopology::Direct | OutputTopology::Unknown),
            "topo = {topo:?}"
        );
        assert!(!evidence.is_empty());
    }

    #[test]
    fn test_update_early_return_when_elapsed_under_5_secs() {
        let mut sys = OutputVerificationSystem::new();
        sys.register_output(OutputModality::Visual);
        sys.update();
        let v = sys.get_verification(&OutputModality::Visual).unwrap();
        assert_eq!(
            v.state,
            petal_tongue_core::rendering_awareness::VisibilityState::Unknown
        );
    }
}
