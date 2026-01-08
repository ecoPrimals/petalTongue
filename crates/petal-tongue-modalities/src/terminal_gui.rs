//! # Terminal GUI Modality
//!
//! ASCII-based terminal visualization (Tier 1: Always Available).

use anyhow::Result;
use async_trait::async_trait;
use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{Write, stdout};
use std::sync::Arc;

use petal_tongue_core::{
    engine::UniversalRenderingEngine,
    event::EngineEvent,
    modality::{AccessibilityFeatures, GUIModality, ModalityCapabilities, ModalityTier},
};

/// Terminal GUI Modality
///
/// Renders topology as ASCII art in the terminal.
///
/// **Tier**: 1 (Always Available)
/// **Dependencies**: Zero (uses crossterm for terminal control)
/// **Interactive**: Yes
/// **Real-time**: Yes
pub struct TerminalGUI {
    /// Reference to engine
    engine: Option<Arc<UniversalRenderingEngine>>,

    /// Current frame number
    frame: u64,

    /// Running state
    running: bool,
}

impl TerminalGUI {
    /// Create new terminal GUI modality
    pub fn new() -> Self {
        Self {
            engine: None,
            frame: 0,
            running: false,
        }
    }

    /// Clear terminal screen
    fn clear_screen(&self) -> Result<()> {
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        Ok(())
    }

    /// Render header
    fn render_header(&self) -> Result<()> {
        let mut stdout = stdout();

        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("═".repeat(80)),
            Print("\n"),
            Print("   🌸 petalTongue - Universal Rendering Engine (Terminal Mode)"),
            Print("\n"),
            Print("═".repeat(80)),
            Print("\n"),
            ResetColor
        )?;

        stdout.flush()?;
        Ok(())
    }

    /// Render topology (placeholder)
    fn render_topology(&self) -> Result<()> {
        let mut stdout = stdout();

        queue!(
            stdout,
            cursor::MoveTo(0, 4),
            SetForegroundColor(Color::Green),
            Print("   Topology Visualization:\n\n"),
            ResetColor,
            Print("   ┌─────────┐\n"),
            Print("   │ Node 1  │\n"),
            Print("   └────┬────┘\n"),
            Print("        │\n"),
            Print("        ▼\n"),
            Print("   ┌─────────┐\n"),
            Print("   │ Node 2  │\n"),
            Print("   └─────────┘\n"),
        )?;

        stdout.flush()?;
        Ok(())
    }

    /// Render footer
    fn render_footer(&self) -> Result<()> {
        let mut stdout = stdout();

        queue!(
            stdout,
            cursor::MoveTo(0, 20),
            SetForegroundColor(Color::DarkGrey),
            Print("─".repeat(80)),
            Print("\n"),
            Print(format!("   Frame: {} | Press 'q' to quit", self.frame)),
            Print("\n"),
            ResetColor
        )?;

        stdout.flush()?;
        Ok(())
    }

    /// Render awakening stage
    fn render_awakening(&self, stage: &str, message: &str) -> Result<()> {
        let mut stdout = stdout();

        queue!(
            stdout,
            cursor::MoveTo(0, 10),
            SetForegroundColor(Color::Yellow),
            Print(format!("   🌸 {}: {}", stage, message)),
            Print("\n"),
            ResetColor
        )?;

        stdout.flush()?;
        Ok(())
    }
}

impl Default for TerminalGUI {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GUIModality for TerminalGUI {
    fn name(&self) -> &'static str {
        "terminal"
    }

    fn is_available(&self) -> bool {
        // Terminal is always available
        true
    }

    fn tier(&self) -> ModalityTier {
        ModalityTier::AlwaysAvailable
    }

    async fn initialize(&mut self, engine: Arc<UniversalRenderingEngine>) -> Result<()> {
        tracing::info!("🖥️  Initializing Terminal GUI");

        self.engine = Some(engine);

        // Setup terminal
        terminal::enable_raw_mode()?;
        self.clear_screen()?;

        Ok(())
    }

    async fn render(&mut self) -> Result<()> {
        tracing::info!("🖥️  Starting Terminal GUI rendering");

        self.running = true;

        // Subscribe to events
        let events = self.engine.as_ref().unwrap().events();
        let mut rx = events.subscribe().await;

        // Main render loop
        while self.running {
            // Clear and render
            self.clear_screen()?;
            self.render_header()?;
            self.render_topology()?;
            self.render_footer()?;

            self.frame += 1;

            // Check for events (non-blocking)
            if let Ok(event) = rx.try_recv() {
                self.handle_event(event).await?;
            }

            // Check for quit key (simplified)
            // In real implementation, would use crossterm::event

            // Sleep for frame time (30 FPS = ~33ms)
            tokio::time::sleep(std::time::Duration::from_millis(33)).await;

            // For demo, run for limited time
            if self.frame > 300 {
                break;
            }
        }

        tracing::info!("🖥️  Terminal GUI rendering complete");

        Ok(())
    }

    async fn handle_event(&mut self, event: EngineEvent) -> Result<()> {
        match event {
            EngineEvent::StateUpdate { key, value } => {
                if key == "awakening_text" {
                    if let Some(message) = value.get("message").and_then(|v| v.as_str()) {
                        if let Some(stage) = value.get("stage").and_then(|v| v.as_str()) {
                            self.render_awakening(stage, message)?;
                        }
                    }
                }
            }
            EngineEvent::Shutdown => {
                self.running = false;
            }
            _ => {
                // Handle other events
                tracing::debug!("Terminal GUI received event: {:?}", event);
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("🖥️  Shutting down Terminal GUI");

        self.running = false;

        // Restore terminal
        terminal::disable_raw_mode()?;
        self.clear_screen()?;

        Ok(())
    }

    fn capabilities(&self) -> ModalityCapabilities {
        ModalityCapabilities {
            interactive: true,
            realtime: true,
            export: false,
            animation: true,
            three_d: false,
            audio: false,
            haptic: false,
            max_nodes: Some(100), // Terminal has space limits
            accessibility: AccessibilityFeatures {
                screen_reader: true,
                keyboard_only: true,
                high_contrast: true,
                blind_users: false, // Use SoundscapeGUI instead
                audio_description: false,
                spatial_audio: false,
                aria_labels: false,
                semantic_markup: false,
                wcag_compliant: true,
                gesture_control: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_gui_creation() {
        let gui = TerminalGUI::new();
        assert_eq!(gui.name(), "terminal");
        assert_eq!(gui.tier(), ModalityTier::AlwaysAvailable);
        assert!(gui.is_available());
    }

    #[test]
    fn test_terminal_gui_capabilities() {
        let gui = TerminalGUI::new();
        let caps = gui.capabilities();

        assert!(caps.interactive);
        assert!(caps.realtime);
        assert!(!caps.export);
        assert!(caps.animation);
        assert_eq!(caps.max_nodes, Some(100));
    }

    #[test]
    fn test_terminal_gui_accessibility() {
        let gui = TerminalGUI::new();
        let caps = gui.capabilities();

        assert!(caps.accessibility.screen_reader);
        assert!(caps.accessibility.keyboard_only);
        assert!(caps.accessibility.wcag_compliant);
    }
}
