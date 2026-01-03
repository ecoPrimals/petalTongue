//! Human Entropy Capture UI
//!
//! Provides a user-friendly interface for multi-modal entropy capture.

#[cfg(feature = "audio")]
use petal_tongue_entropy::audio::AudioEntropyCapture;
use petal_tongue_entropy::prelude::*;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Human entropy capture window
///
/// Provides UI for capturing human entropy through multiple modalities:
/// - Audio (singing, speaking)
/// - Visual (drawing)
/// - Narrative (storytelling)
/// - Gesture (motion)
/// - Video (camera)
pub struct HumanEntropyWindow {
    /// Currently selected modality
    modality: EntropyModality,
    
    /// Capture state
    state: CaptureWindowState,
    
    /// Audio capturer (if audio feature enabled)
    #[cfg(feature = "audio")]
    audio_capturer: Option<Arc<Mutex<AudioEntropyCapture>>>,
    
    /// Narrative capturer (always available)
    narrative_capturer: Option<NarrativeEntropyCapture>,
    
    /// Current quality metrics (real-time feedback)
    current_quality: Option<f64>,
    
    /// Capture start time
    capture_start: Option<Instant>,
    
    /// Waveform buffer for visualization (audio)
    waveform_buffer: Vec<f32>,
    
    /// Status message
    status_message: String,
}

/// Capture modality selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntropyModality {
    Audio,
    Visual,
    Narrative,
    Gesture,
    Video,
}

impl EntropyModality {
    fn name(&self) -> &'static str {
        match self {
            Self::Audio => "🎵 Audio (Sing a Song)",
            Self::Visual => "🎨 Visual (Draw/Paint)",
            Self::Narrative => "📝 Narrative (Tell a Story)",
            Self::Gesture => "✋ Gesture (Motion)",
            Self::Video => "📹 Video (Camera)",
        }
    }
    
    fn description(&self) -> &'static str {
        match self {
            Self::Audio => "Capture entropy from your voice - sing a song, tell a story, or just speak naturally.",
            Self::Visual => "Draw or paint something unique. Your strokes, timing, and patterns create entropy.",
            Self::Narrative => "Type a story, poem, or thoughts. Your keystroke timing and style create entropy.",
            Self::Gesture => "Use motion sensors or touch patterns to create entropy from your natural movements.",
            Self::Video => "Use your camera to capture entropy from your movements and environment.",
        }
    }
    
    fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "audio")]
            Self::Audio => true,
            #[cfg(not(feature = "audio"))]
            Self::Audio => false,
            
            Self::Narrative => true, // Always available
            
            Self::Visual => false, // TODO: Phase 3
            Self::Gesture => false, // TODO: Phase 5
            Self::Video => false, // TODO: Phase 6
        }
    }
}

/// Capture window state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureWindowState {
    /// Idle, ready to start
    Idle,
    
    /// Actively capturing
    Recording,
    
    /// Stopped, ready to finalize or discard
    Stopped,
    
    /// Processing/streaming
    Processing,
}

impl HumanEntropyWindow {
    /// Create a new human entropy capture window
    pub fn new() -> Self {
        info!("Creating human entropy capture window");
        
        Self {
            modality: EntropyModality::Audio, // Default to audio
            state: CaptureWindowState::Idle,
            
            #[cfg(feature = "audio")]
            audio_capturer: None,
            
            narrative_capturer: None,
            current_quality: None,
            capture_start: None,
            waveform_buffer: Vec::new(),
            status_message: "Ready to capture human entropy".to_string(),
        }
    }
    
    /// Render the window
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        
        egui::Window::new("🌸 Human Entropy Capture")
            .default_width(600.0)
            .default_height(500.0)
            .open(&mut open)
            .show(ctx, |ui| {
                self.render_ui(ui);
            });
        
        open
    }
    
    fn render_ui(&mut self, ui: &mut egui::Ui) {
        // Header
        ui.heading("Capture Human Entropy for Sovereign Keys");
        ui.add_space(10.0);
        
        // Privacy notice
        ui.horizontal(|ui| {
            ui.label("🔒");
            ui.label("Privacy: Stream-only (never persisted), encrypted transmission, secure zeroization");
        });
        ui.add_space(10.0);
        
        // Modality selector
        self.render_modality_selector(ui);
        ui.add_space(10.0);
        
        // Capture area
        match self.state {
            CaptureWindowState::Idle => {
                self.render_idle_state(ui);
            }
            CaptureWindowState::Recording => {
                self.render_recording_state(ui);
            }
            CaptureWindowState::Stopped => {
                self.render_stopped_state(ui);
            }
            CaptureWindowState::Processing => {
                self.render_processing_state(ui);
            }
        }
        
        ui.add_space(10.0);
        
        // Status bar
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Status:");
            ui.label(&self.status_message);
        });
    }
    
    fn render_modality_selector(&mut self, ui: &mut egui::Ui) {
        ui.label("Choose Modality:");
        
        ui.horizontal(|ui| {
            for modality in [
                EntropyModality::Audio,
                EntropyModality::Visual,
                EntropyModality::Narrative,
                EntropyModality::Gesture,
                EntropyModality::Video,
            ] {
                let enabled = modality.is_available() && self.state == CaptureWindowState::Idle;
                
                ui.add_enabled_ui(enabled, |ui| {
                    if ui.selectable_label(self.modality == modality, modality.name()).clicked() {
                        self.modality = modality;
                        self.status_message = format!("Selected: {}", modality.name());
                    }
                });
                
                if !modality.is_available() && ui.is_enabled() {
                    ui.label("(unavailable)");
                }
            }
        });
        
        ui.add_space(5.0);
        ui.label(self.modality.description());
    }
    
    fn render_idle_state(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            if ui.button("🎙️ Start Capture").clicked() {
                self.start_capture();
            }
            
            ui.add_space(10.0);
            ui.label("Click to begin capturing entropy");
        });
    }
    
    fn render_recording_state(&mut self, ui: &mut egui::Ui) {
        // Show duration
        if let Some(start) = self.capture_start {
            let duration = Instant::now().duration_since(start);
            ui.horizontal(|ui| {
                ui.label("Recording:");
                ui.label(format!("{:.1}s", duration.as_secs_f64()));
            });
        }
        
        ui.add_space(10.0);
        
        // Show real-time quality (if available)
        if let Some(quality) = self.current_quality {
            ui.horizontal(|ui| {
                ui.label("Quality:");
                
                let color = if quality > 0.7 {
                    egui::Color32::GREEN
                } else if quality > 0.4 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };
                
                ui.colored_label(color, format!("{:.1}%", quality * 100.0));
            });
            
            // Quality bar
            let progress_bar = egui::ProgressBar::new(quality as f32)
                .desired_width(400.0)
                .show_percentage();
            ui.add(progress_bar);
        }
        
        ui.add_space(10.0);
        
        // Waveform visualization (audio only)
        #[cfg(feature = "audio")]
        if self.modality == EntropyModality::Audio {
            self.render_waveform(ui);
        }
        
        ui.add_space(20.0);
        
        // Stop button
        ui.vertical_centered(|ui| {
            if ui.button("⏹️ Stop Capture").clicked() {
                self.stop_capture();
            }
        });
    }
    
    fn render_stopped_state(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            if let Some(quality) = self.current_quality {
                ui.label(format!("Capture complete! Quality: {:.1}%", quality * 100.0));
            } else {
                ui.label("Capture complete!");
            }
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("✅ Send to BearDog").clicked() {
                    self.finalize_and_stream();
                }
                
                if ui.button("🗑️ Discard").clicked() {
                    self.discard();
                }
            });
        });
    }
    
    fn render_processing_state(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.spinner();
            ui.label("Processing and streaming entropy...");
            ui.add_space(10.0);
            ui.label("🔒 Encrypted transmission to BearDog");
        });
    }
    
    #[cfg(feature = "audio")]
    fn render_waveform(&mut self, ui: &mut egui::Ui) {
        use egui::*;
        
        ui.label("Waveform:");
        
        let (_response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), 100.0),
            Sense::hover(),
        );
        
        let rect = painter.clip_rect();
        
        // Draw waveform
        if !self.waveform_buffer.is_empty() {
            let points: Vec<Pos2> = self
                .waveform_buffer
                .iter()
                .enumerate()
                .map(|(i, &sample)| {
                    let x = rect.min.x + (i as f32 / self.waveform_buffer.len() as f32) * rect.width();
                    let y = rect.center().y - (sample * rect.height() * 0.4);
                    Pos2::new(x, y)
                })
                .collect();
            
            if points.len() > 1 {
                painter.add(Shape::line(points, Stroke::new(2.0, Color32::LIGHT_BLUE)));
            }
        }
        
        // Draw center line
        painter.add(Shape::line_segment(
            [
                Pos2::new(rect.min.x, rect.center().y),
                Pos2::new(rect.max.x, rect.center().y),
            ],
            Stroke::new(1.0, Color32::DARK_GRAY),
        ));
    }
    
    fn start_capture(&mut self) {
        info!("Starting entropy capture: {:?}", self.modality);
        
        match self.modality {
            #[cfg(feature = "audio")]
            EntropyModality::Audio => {
                match AudioEntropyCapture::new() {
                    Ok(mut capturer) => {
                        if let Err(e) = capturer.start() {
                            tracing::error!("Failed to start audio capture: {}", e);
                            self.status_message = format!("Error: {}", e);
                            return;
                        }
                        
                        self.audio_capturer = Some(Arc::new(Mutex::new(capturer)));
                        self.state = CaptureWindowState::Recording;
                        self.capture_start = Some(Instant::now());
                        self.status_message = "Recording audio...".to_string();
                    }
                    Err(e) => {
                        tracing::error!("Failed to create audio capturer: {}", e);
                        self.status_message = format!("Error: {}", e);
                    }
                }
            }
            
            EntropyModality::Narrative => {
                let mut capturer = NarrativeEntropyCapture::new();
                capturer.start();
                self.narrative_capturer = Some(capturer);
                self.state = CaptureWindowState::Recording;
                self.capture_start = Some(Instant::now());
                self.status_message = "Type your story...".to_string();
            }
            
            _ => {
                self.status_message = "Modality not yet implemented".to_string();
            }
        }
    }
    
    fn stop_capture(&mut self) {
        info!("Stopping entropy capture");
        
        match self.modality {
            #[cfg(feature = "audio")]
            EntropyModality::Audio => {
                if let Some(capturer) = &self.audio_capturer {
                    let mut cap = capturer.lock().unwrap();
                    if let Err(e) = cap.stop() {
                        tracing::error!("Failed to stop audio capture: {}", e);
                        self.status_message = format!("Error stopping: {}", e);
                        return;
                    }
                    
                    self.current_quality = Some(cap.assess_quality().overall_quality);
                }
            }
            
            EntropyModality::Narrative => {
                if let Some(capturer) = &self.narrative_capturer {
                    self.current_quality = Some(capturer.assess_quality().overall_quality);
                }
            }
            
            _ => {}
        }
        
        self.state = CaptureWindowState::Stopped;
        self.status_message = "Capture stopped. Ready to send or discard.".to_string();
    }
    
    fn finalize_and_stream(&mut self) {
        info!("Finalizing and streaming entropy");
        self.state = CaptureWindowState::Processing;
        self.status_message = "Streaming to BearDog...".to_string();
        
        // TODO: Actual streaming implementation
        // For now, just simulate completion
        
        // In real implementation:
        // 1. Finalize capturer to get entropy data
        // 2. Stream to biomeOS/BearDog via encrypted channel
        // 3. Show success/failure
        
        warn!("Streaming not yet implemented - entropy would be zeroized here");
        
        // Reset to idle after "streaming"
        self.reset();
        self.status_message = "Entropy sent successfully! (simulated)".to_string();
    }
    
    fn discard(&mut self) {
        info!("Discarding captured entropy");
        self.reset();
        self.status_message = "Entropy discarded and zeroized.".to_string();
    }
    
    fn reset(&mut self) {
        #[cfg(feature = "audio")]
        {
            self.audio_capturer = None;
        }
        
        self.narrative_capturer = None;
        self.current_quality = None;
        self.capture_start = None;
        self.waveform_buffer.clear();
        self.state = CaptureWindowState::Idle;
    }
    
    /// Update function (called each frame)
    pub fn update(&mut self) {
        if self.state != CaptureWindowState::Recording {
            return;
        }
        
        // Update quality metrics
        match self.modality {
            #[cfg(feature = "audio")]
            EntropyModality::Audio => {
                if let Some(capturer) = &self.audio_capturer {
                    let cap = capturer.lock().unwrap();
                    let quality = cap.assess_quality();
                    self.current_quality = Some(quality.overall_quality);
                    
                    // TODO: Update waveform buffer for visualization
                }
            }
            
            EntropyModality::Narrative => {
                if let Some(capturer) = &self.narrative_capturer {
                    let quality = capturer.assess_quality();
                    self.current_quality = Some(quality.overall_quality);
                }
            }
            
            _ => {}
        }
    }
}

impl Default for HumanEntropyWindow {
    fn default() -> Self {
        Self::new()
    }
}

