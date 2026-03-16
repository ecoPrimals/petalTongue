// SPDX-License-Identifier: AGPL-3.0-or-later
//! Keyboard sensor - Discrete input device
//!
//! Discovers keyboard capabilities and provides key press events.

use crate::error::SensorError;
use async_trait::async_trait;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use petal_tongue_core::{Key, Modifiers, Sensor, SensorCapabilities, SensorEvent, SensorType};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

/// Keyboard sensor implementation
pub struct KeyboardSensor {
    capabilities: SensorCapabilities,
    input_type: InputType,
    last_key_press: Option<Instant>,
}

impl KeyboardSensor {
    /// Create new keyboard sensor
    #[must_use]
    pub const fn new(input_type: InputType) -> Self {
        let capabilities = SensorCapabilities {
            sensor_type: SensorType::Keyboard,
            input: true,
            output: false, // Keyboard is input only
            spatial: false,
            temporal: true, // Timing of key presses
            continuous: false,
            discrete: true, // Individual key events
            bidirectional: false,
        };

        Self {
            capabilities,
            input_type,
            last_key_press: None,
        }
    }
}

#[async_trait]
impl Sensor for KeyboardSensor {
    fn capabilities(&self) -> &SensorCapabilities {
        &self.capabilities
    }

    fn is_available(&self) -> bool {
        // Keyboard is available if terminal stdin is a TTY
        std::io::stdin().is_terminal()
    }

    async fn poll_events(&mut self) -> anyhow::Result<Vec<SensorEvent>> {
        let mut events = Vec::new();

        // Non-blocking poll with very short timeout
        match self.input_type {
            InputType::Terminal => {
                if event::poll(Duration::from_millis(1))
                    .map_err(|e| SensorError::Crossterm(e.to_string()))?
                    && let Event::Key(key_event) =
                        event::read().map_err(|e| SensorError::Crossterm(e.to_string()))?
                {
                    let timestamp = Instant::now();
                    self.last_key_press = Some(timestamp);

                    let key = map_keycode(key_event.code);
                    let modifiers = map_modifiers(key_event.modifiers);

                    events.push(SensorEvent::KeyPress {
                        key,
                        modifiers,
                        timestamp,
                    });
                }
            }
        }

        Ok(events)
    }

    fn last_activity(&self) -> Option<Instant> {
        self.last_key_press
    }

    fn name(&self) -> &str {
        match self.input_type {
            InputType::Terminal => "Terminal Keyboard",
        }
    }
}

/// Input type (discovered at runtime)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    /// Terminal/console input
    Terminal,
}

/// Map crossterm keycode to our Key enum
const fn map_keycode(code: KeyCode) -> Key {
    match code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Esc => Key::Escape,
        KeyCode::Enter => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::F(n) => Key::F(n),
        _ => Key::Unknown,
    }
}

/// Map crossterm modifiers to our Modifiers struct
const fn map_modifiers(mods: KeyModifiers) -> Modifiers {
    Modifiers {
        ctrl: mods.contains(KeyModifiers::CONTROL),
        alt: mods.contains(KeyModifiers::ALT),
        shift: mods.contains(KeyModifiers::SHIFT),
        meta: mods.contains(KeyModifiers::SUPER),
    }
}

/// Discover keyboard capabilities
#[expect(clippy::unused_async, reason = "async for trait compatibility")]
pub async fn discover() -> Option<KeyboardSensor> {
    // Check if stdin is a terminal
    if std::io::stdin().is_terminal() {
        tracing::debug!("Discovered terminal keyboard");
        return Some(KeyboardSensor::new(InputType::Terminal));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_keyboard_sensor_creation() {
        let sensor = KeyboardSensor::new(InputType::Terminal);
        assert_eq!(sensor.capabilities().sensor_type, SensorType::Keyboard);
        assert!(sensor.capabilities().input);
        assert!(!sensor.capabilities().output);
    }

    #[test]
    fn test_keycode_mapping() {
        assert_eq!(map_keycode(KeyCode::Char('a')), Key::Char('a'));
        assert_eq!(map_keycode(KeyCode::Esc), Key::Escape);
        assert_eq!(map_keycode(KeyCode::Up), Key::Up);
    }

    #[test]
    fn test_modifier_mapping() {
        let mods = map_modifiers(KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        assert!(mods.ctrl);
        assert!(mods.shift);
        assert!(!mods.alt);
    }

    #[test]
    fn test_modifier_mapping_alt_meta() {
        let mods = map_modifiers(KeyModifiers::ALT);
        assert!(mods.alt);
        assert!(!mods.ctrl);
        let mods = map_modifiers(KeyModifiers::SUPER);
        assert!(mods.meta);
    }

    #[test]
    fn test_keycode_mapping_full() {
        assert_eq!(map_keycode(KeyCode::Enter), Key::Enter);
        assert_eq!(map_keycode(KeyCode::Tab), Key::Tab);
        assert_eq!(map_keycode(KeyCode::Backspace), Key::Backspace);
        assert_eq!(map_keycode(KeyCode::Delete), Key::Delete);
        assert_eq!(map_keycode(KeyCode::Down), Key::Down);
        assert_eq!(map_keycode(KeyCode::Left), Key::Left);
        assert_eq!(map_keycode(KeyCode::Right), Key::Right);
        assert_eq!(map_keycode(KeyCode::F(1)), Key::F(1));
        assert_eq!(map_keycode(KeyCode::Insert), Key::Unknown);
    }

    #[test]
    fn test_keyboard_sensor_name() {
        let sensor = KeyboardSensor::new(InputType::Terminal);
        assert_eq!(sensor.name(), "Terminal Keyboard");
    }
}
