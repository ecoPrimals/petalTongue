# Sensory Input v1: Computer Peripherals

**Date**: January 8, 2026  
**Phase**: v0.3.1 - Basic Sensory Implementation  
**Approach**: Discover → Understand → Interact → Abstract → Evolve

---

## Philosophy: Progressive Sensor Discovery

> **"petalTongue starts knowing nothing about input devices. It discovers what exists, understands their capabilities, learns minimal interactions, then builds to complex ones."**

Like an infant learning to sense the world:
1. **Discover**: What sensors exist?
2. **Understand**: What can each do?
3. **Interact**: Start with minimal viable interaction
4. **Enhance**: Build to richer interactions
5. **Abstract**: Generalize patterns
6. **Evolve**: Add new sensor types

---

## Core Principle: Capability-Based Sensor Discovery

```rust
// petalTongue knows NOTHING about specific devices
// It discovers capabilities at runtime

pub enum SensorCapability {
    // Spatial input
    PointingDevice,      // Can provide X,Y coordinates
    DiscreteInput,       // Can provide button/key presses
    ContinuousInput,     // Can provide continuous values (sliders, etc)
    
    // Audio input
    AudioCapture,        // Can capture sound
    VoiceInput,         // Can understand voice
    
    // Visual feedback
    DisplayOutput,       // Can show pixels
    VisualVerification, // Can confirm what's displayed
    
    // Haptic
    TactileOutput,      // Can provide touch feedback
    
    // Future
    MotionCapture,      // Can track movement
    BiometricSensor,    // Can read biological signals
    EnvironmentalSensor, // Can read environment
}
```

---

## Part 1: Screen (Display + Verification)

### What Is a Screen?

**From petalTongue's perspective**:
- An output device that MAY provide visual feedback
- Unknown if user can actually see it
- Need to verify through interaction

### Discovery Process

```rust
pub struct ScreenDiscovery;

impl ScreenDiscovery {
    pub async fn discover() -> Option<ScreenCapabilities> {
        // Try multiple detection methods
        
        // Method 1: Environment variables
        if std::env::var("DISPLAY").is_ok() || 
           std::env::var("WAYLAND_DISPLAY").is_ok() {
            Some(Self::probe_x11_wayland().await)
        }
        // Method 2: Framebuffer
        else if Path::new("/dev/fb0").exists() {
            Some(Self::probe_framebuffer().await)
        }
        // Method 3: Terminal
        else if atty::is(atty::Stream::Stdout) {
            Some(Self::probe_terminal().await)
        }
        else {
            None
        }
    }
    
    async fn probe_terminal() -> ScreenCapabilities {
        let (width, height) = term_size::dimensions()
            .unwrap_or((80, 24));
        
        ScreenCapabilities {
            display_type: DisplayType::Terminal,
            width_chars: width,
            height_chars: height,
            color_depth: ColorDepth::ANSI256,
            can_verify: true, // Terminal always exists if we're here
            refresh_rate: None,
        }
    }
}

pub struct ScreenCapabilities {
    pub display_type: DisplayType,
    pub width_chars: usize,
    pub height_chars: usize,
    pub width_pixels: Option<u32>,
    pub height_pixels: Option<u32>,
    pub color_depth: ColorDepth,
    pub can_verify: bool,
    pub refresh_rate: Option<u32>,
}

pub enum DisplayType {
    Terminal,       // Text-based
    Framebuffer,    // Direct pixel access
    Window,         // Windowed GUI
    Unknown,
}
```

### Minimal Interaction: Confirmation

```rust
pub struct ScreenSensor {
    capabilities: ScreenCapabilities,
    frames_sent: u64,
    last_interaction: Option<Instant>,
}

impl ScreenSensor {
    // Minimal: Can we confirm ANYTHING is displayed?
    pub async fn verify_minimal(&mut self) -> bool {
        match self.capabilities.display_type {
            DisplayType::Terminal => {
                // Terminal: If we can write, assume visible
                print!("\x1b[?6n"); // Query cursor position
                // If we get response, terminal is active
                true
            }
            DisplayType::Window => {
                // Window: Check if window manager responds
                // Send heartbeat, wait for ack
                self.send_heartbeat().await.is_ok()
            }
            _ => false
        }
    }
    
    // Enhanced: User interaction proves visibility
    pub fn confirm_via_interaction(&mut self) {
        self.last_interaction = Some(Instant::now());
    }
    
    pub fn is_confirmed_visible(&self) -> bool {
        self.last_interaction
            .map(|t| t.elapsed() < Duration::from_secs(30))
            .unwrap_or(false)
    }
}
```

### Complex Interaction: Adaptive Rendering

```rust
impl ScreenSensor {
    pub fn adaptive_render_strategy(&self) -> RenderStrategy {
        match self.capabilities.display_type {
            DisplayType::Terminal => {
                RenderStrategy::ASCII {
                    width: self.capabilities.width_chars,
                    height: self.capabilities.height_chars,
                    colors: match self.capabilities.color_depth {
                        ColorDepth::ANSI256 => 256,
                        ColorDepth::TrueColor => 16_777_216,
                        _ => 16,
                    }
                }
            }
            DisplayType::Framebuffer => {
                RenderStrategy::DirectPixels {
                    width: self.capabilities.width_pixels.unwrap_or(1920),
                    height: self.capabilities.height_pixels.unwrap_or(1080),
                }
            }
            _ => RenderStrategy::Fallback,
        }
    }
}
```

---

## Part 2: Keyboard (Discrete Input)

### What Is a Keyboard?

**From petalTongue's perspective**:
- A discrete input device
- Provides button press events
- Has unknown layout and capabilities

### Discovery Process

```rust
pub struct KeyboardDiscovery;

impl KeyboardDiscovery {
    pub async fn discover() -> Option<KeyboardCapabilities> {
        // Try detection methods in order
        
        // Method 1: stdin is terminal
        if atty::is(atty::Stream::Stdin) {
            Some(Self::probe_terminal_input().await)
        }
        // Method 2: Raw device access
        else if Self::can_access_event_devices() {
            Some(Self::probe_event_devices().await)
        }
        else {
            None
        }
    }
    
    async fn probe_terminal_input() -> KeyboardCapabilities {
        // Test what the terminal can do
        let supports_raw_mode = crossterm::terminal::enable_raw_mode().is_ok();
        if supports_raw_mode {
            let _ = crossterm::terminal::disable_raw_mode();
        }
        
        KeyboardCapabilities {
            input_type: InputType::Terminal,
            supports_raw_mode,
            supports_unicode: true,
            supports_modifiers: true,
            layout: KeyboardLayout::Unknown,
        }
    }
}

pub struct KeyboardCapabilities {
    pub input_type: InputType,
    pub supports_raw_mode: bool,
    pub supports_unicode: bool,
    pub supports_modifiers: bool,
    pub layout: KeyboardLayout,
}

pub enum InputType {
    Terminal,      // stdin from terminal
    RawDevice,     // /dev/input/eventX
    WindowEvent,   // GUI framework events
}
```

### Minimal Interaction: Single Key Press

```rust
pub struct KeyboardSensor {
    capabilities: KeyboardCapabilities,
    events: VecDeque<KeyEvent>,
}

impl KeyboardSensor {
    // Minimal: Can we detect ANY key press?
    pub async fn wait_for_any_key(&mut self, timeout: Duration) -> Option<KeyEvent> {
        match self.capabilities.input_type {
            InputType::Terminal => {
                use crossterm::event::{self, Event, KeyCode};
                
                if event::poll(timeout).ok()? {
                    if let Event::Key(key_event) = event::read().ok()? {
                        return Some(KeyEvent {
                            code: Self::map_keycode(key_event.code),
                            modifiers: Modifiers::empty(),
                            timestamp: Instant::now(),
                        });
                    }
                }
                None
            }
            _ => None
        }
    }
    
    // Enhanced: Track interaction patterns
    pub fn record_interaction(&mut self, event: KeyEvent) {
        self.events.push_back(event);
        // Keep last 100 events
        if self.events.len() > 100 {
            self.events.pop_front();
        }
    }
    
    pub fn interaction_rate(&self) -> f32 {
        if self.events.is_empty() {
            return 0.0;
        }
        
        let duration = self.events.back().unwrap().timestamp
            .duration_since(self.events.front().unwrap().timestamp);
        
        self.events.len() as f32 / duration.as_secs_f32()
    }
}
```

### Complex Interaction: Command Recognition

```rust
impl KeyboardSensor {
    pub fn detect_command_sequence(&self) -> Option<Command> {
        // Look for command patterns in recent events
        let recent: Vec<_> = self.events.iter()
            .rev()
            .take(10)
            .collect();
        
        // Example: Ctrl+C
        if let Some(last) = recent.first() {
            if last.code == KeyCode::Char('c') && 
               last.modifiers.contains(Modifiers::CONTROL) {
                return Some(Command::Exit);
            }
        }
        
        // Example: Arrow key navigation
        match recent.get(0).map(|e| &e.code) {
            Some(KeyCode::Up) => Some(Command::NavigateUp),
            Some(KeyCode::Down) => Some(Command::NavigateDown),
            Some(KeyCode::Enter) => Some(Command::Select),
            _ => None,
        }
    }
}
```

---

## Part 3: Mouse (Pointing Device)

### What Is a Mouse?

**From petalTongue's perspective**:
- A spatial input device
- Provides continuous position updates
- May have buttons for discrete actions

### Discovery Process

```rust
pub struct MouseDiscovery;

impl MouseDiscovery {
    pub async fn discover() -> Option<MouseCapabilities> {
        // Try detection methods
        
        // Method 1: Terminal mouse support
        if Self::probe_terminal_mouse().await {
            Some(MouseCapabilities {
                input_type: PointerType::TerminalMouse,
                has_buttons: true,
                button_count: 3,
                has_scroll: true,
                coordinates: CoordinateSystem::CharacterGrid,
            })
        }
        // Method 2: Window system mouse
        else if Self::probe_window_mouse().await {
            Some(MouseCapabilities {
                input_type: PointerType::WindowMouse,
                has_buttons: true,
                button_count: 5,
                has_scroll: true,
                coordinates: CoordinateSystem::Pixels,
            })
        }
        else {
            None
        }
    }
    
    async fn probe_terminal_mouse() -> bool {
        // Check if terminal supports mouse events
        // Try enabling and see if it works
        print!("\x1b[?1000h"); // Enable mouse tracking
        std::thread::sleep(Duration::from_millis(10));
        print!("\x1b[?1000l"); // Disable mouse tracking
        true // If no error, assume it works
    }
}

pub struct MouseCapabilities {
    pub input_type: PointerType,
    pub has_buttons: bool,
    pub button_count: u8,
    pub has_scroll: bool,
    pub coordinates: CoordinateSystem,
}

pub enum PointerType {
    TerminalMouse,  // Terminal mouse events
    WindowMouse,    // GUI mouse events
    Touchpad,       // Gesture-capable
    Touchscreen,    // Direct pointing
}
```

### Minimal Interaction: Click Detection

```rust
pub struct MouseSensor {
    capabilities: MouseCapabilities,
    last_position: Option<(f32, f32)>,
    last_click: Option<Instant>,
}

impl MouseSensor {
    // Minimal: Can we detect a click ANYWHERE?
    pub async fn wait_for_any_click(&mut self, timeout: Duration) -> Option<ClickEvent> {
        match self.capabilities.input_type {
            PointerType::TerminalMouse => {
                use crossterm::event::{self, Event, MouseEventKind};
                
                if event::poll(timeout).ok()? {
                    if let Event::Mouse(mouse_event) = event::read().ok()? {
                        if matches!(mouse_event.kind, MouseEventKind::Down(_)) {
                            return Some(ClickEvent {
                                x: mouse_event.column as f32,
                                y: mouse_event.row as f32,
                                button: MouseButton::Left,
                                timestamp: Instant::now(),
                            });
                        }
                    }
                }
                None
            }
            _ => None
        }
    }
    
    // Enhanced: Track movement patterns
    pub fn record_movement(&mut self, x: f32, y: f32) {
        self.last_position = Some((x, y));
    }
    
    pub fn is_user_active(&self) -> bool {
        self.last_click
            .map(|t| t.elapsed() < Duration::from_secs(10))
            .unwrap_or(false)
    }
}
```

### Complex Interaction: Spatial Selection

```rust
impl MouseSensor {
    pub fn detect_spatial_interaction(&self, 
        click: &ClickEvent, 
        ui_elements: &[UIElement]
    ) -> Option<UIInteraction> {
        // Find what was clicked
        for element in ui_elements {
            if element.bounds.contains(click.x, click.y) {
                return Some(UIInteraction {
                    element: element.id.clone(),
                    action: InteractionType::Click,
                    position: (click.x, click.y),
                });
            }
        }
        None
    }
    
    pub fn detect_gesture(&self, history: &[ClickEvent]) -> Option<Gesture> {
        if history.len() < 2 {
            return None;
        }
        
        // Detect double-click
        if history.len() >= 2 {
            let time_diff = history[1].timestamp
                .duration_since(history[0].timestamp);
            if time_diff < Duration::from_millis(300) {
                return Some(Gesture::DoubleClick);
            }
        }
        
        None
    }
}
```

---

## Part 4: Audio (Bidirectional I/O)

### What Is Audio?

**From petalTongue's perspective**:
- BOTH output (speaker) AND input (microphone)
- Can convey information without screen
- Can receive voice commands without keyboard

### Discovery Process

```rust
pub struct AudioDiscovery;

impl AudioDiscovery {
    pub async fn discover() -> AudioCapabilities {
        let output = Self::probe_audio_output().await;
        let input = Self::probe_audio_input().await;
        
        AudioCapabilities {
            has_output: output.is_some(),
            output_device: output,
            has_input: input.is_some(),
            input_device: input,
            can_do_voice: false, // Requires additional processing
        }
    }
    
    async fn probe_audio_output() -> Option<AudioDevice> {
        // Try to open audio output
        match rodio::OutputStream::try_default() {
            Ok(_) => Some(AudioDevice {
                device_type: AudioDeviceType::Output,
                sample_rate: 44100,
                channels: 2,
            }),
            Err(_) => None,
        }
    }
    
    async fn probe_audio_input() -> Option<AudioDevice> {
        // Try to open audio input
        // This requires microphone access
        #[cfg(feature = "audio-input")]
        {
            match cpal::default_host().default_input_device() {
                Some(device) => Some(AudioDevice {
                    device_type: AudioDeviceType::Input,
                    sample_rate: 44100,
                    channels: 1,
                }),
                None => None,
            }
        }
        #[cfg(not(feature = "audio-input"))]
        {
            None
        }
    }
}

pub struct AudioCapabilities {
    pub has_output: bool,
    pub output_device: Option<AudioDevice>,
    pub has_input: bool,
    pub input_device: Option<AudioDevice>,
    pub can_do_voice: bool,
}
```

### Minimal Interaction: Status Beeps

```rust
pub struct AudioSensor {
    capabilities: AudioCapabilities,
}

impl AudioSensor {
    // Minimal OUTPUT: Simple status tones
    pub async fn beep_status(&self, status: Status) {
        if !self.capabilities.has_output {
            return;
        }
        
        let frequency = match status {
            Status::Success => 800.0,  // Higher tone = good
            Status::Warning => 600.0,  // Mid tone = warning
            Status::Error => 400.0,    // Lower tone = bad
        };
        
        self.play_tone(frequency, Duration::from_millis(200)).await;
    }
    
    // Minimal INPUT: Detect any sound
    pub async fn wait_for_sound(&mut self, timeout: Duration) -> bool {
        if !self.capabilities.has_input {
            return false;
        }
        
        // Listen for amplitude above threshold
        // If ANY sound detected, return true
        // This proves microphone works
        self.detect_amplitude_threshold(0.1, timeout).await
    }
}
```

### Complex Interaction: Voice Commands

```rust
impl AudioSensor {
    // Enhanced OUTPUT: Spoken feedback
    pub async fn speak(&self, message: &str) {
        if !self.capabilities.has_output {
            return;
        }
        
        // Use text-to-speech (future)
        // For now, just beep patterns
        for word in message.split_whitespace() {
            self.beep_morse_code(word).await;
        }
    }
    
    // Enhanced INPUT: Voice recognition
    pub async fn listen_for_command(&mut self, 
        timeout: Duration
    ) -> Option<VoiceCommand> {
        if !self.capabilities.has_input || !self.capabilities.can_do_voice {
            return None;
        }
        
        // Capture audio
        let audio_data = self.record_audio(timeout).await?;
        
        // Process for commands (future: use Whisper or similar)
        // For now, detect simple patterns
        self.detect_voice_pattern(&audio_data)
    }
}
```

---

## Field Scenario: No Monitor

### Use Case

**Situation**: Working in the field without a monitor
**Available**: Laptop with keyboard and speakers/microphone
**Goal**: Interact with petalTongue topology

### Implementation

```rust
pub struct FieldMode {
    keyboard: KeyboardSensor,
    audio: AudioSensor,
}

impl FieldMode {
    pub async fn run_field_interface(&mut self) -> Result<()> {
        // No screen, but we have audio + keyboard
        
        println!("🌸 petalTongue Field Mode");
        println!("Commands: [S]tatus, [N]ext, [P]rev, [Q]uit");
        
        loop {
            // Audio feedback
            self.audio.beep_status(Status::Ready).await;
            
            // Wait for key
            if let Some(key) = self.keyboard.wait_for_any_key(
                Duration::from_secs(60)
            ).await {
                match key.code {
                    KeyCode::Char('s') => {
                        let status = self.get_topology_status().await;
                        self.audio.speak(&format!(
                            "{} nodes, {} healthy, {} warnings",
                            status.total,
                            status.healthy,
                            status.warnings
                        )).await;
                    }
                    KeyCode::Char('n') => {
                        self.select_next_node().await;
                        self.audio.beep_status(Status::Success).await;
                    }
                    KeyCode::Char('q') => {
                        break;
                    }
                    _ => {
                        self.audio.beep_status(Status::Warning).await;
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

**This proves the abstraction works**: Same topology data, different sensory channels!

---

## Sensor Abstraction Layer

### Unified Interface

```rust
pub trait Sensor {
    fn capabilities(&self) -> SensorCapabilities;
    fn is_available(&self) -> bool;
    async fn poll_events(&mut self) -> Vec<SensorEvent>;
    fn get_last_activity(&self) -> Option<Instant>;
}

pub struct SensorCapabilities {
    pub sensor_type: SensorType,
    pub input: bool,
    pub output: bool,
    pub spatial: bool,
    pub temporal: bool,
    pub continuous: bool,
    pub discrete: bool,
}

pub enum SensorType {
    Screen,
    Keyboard,
    Mouse,
    Audio,
    // Future
    Camera,
    Microphone,
    Accelerometer,
    GPS,
    HeartRateSensor,
    TemperatureSensor,
    // Anything!
}

pub enum SensorEvent {
    // Spatial
    Position { x: f32, y: f32 },
    Click { x: f32, y: f32, button: u8 },
    
    // Discrete
    KeyPress { key: String },
    ButtonPress { button: u8 },
    
    // Continuous
    AudioLevel { amplitude: f32 },
    Temperature { celsius: f32 },
    
    // Confirmation
    Heartbeat,
    Alive,
}
```

### Discovery Registry

```rust
pub struct SensorRegistry {
    sensors: Vec<Box<dyn Sensor>>,
}

impl SensorRegistry {
    pub async fn discover_all() -> Self {
        let mut sensors: Vec<Box<dyn Sensor>> = Vec::new();
        
        // Try each sensor type
        if let Some(screen) = ScreenDiscovery::discover().await {
            sensors.push(Box::new(ScreenSensor::new(screen)));
        }
        
        if let Some(keyboard) = KeyboardDiscovery::discover().await {
            sensors.push(Box::new(KeyboardSensor::new(keyboard)));
        }
        
        if let Some(mouse) = MouseDiscovery::discover().await {
            sensors.push(Box::new(MouseSensor::new(mouse)));
        }
        
        let audio = AudioDiscovery::discover().await;
        if audio.has_output || audio.has_input {
            sensors.push(Box::new(AudioSensor::new(audio)));
        }
        
        Self { sensors }
    }
    
    pub fn has_capability(&self, cap: SensorCapability) -> bool {
        self.sensors.iter().any(|s| 
            s.capabilities().matches(cap)
        )
    }
    
    pub async fn poll_all(&mut self) -> Vec<SensorEvent> {
        let mut events = Vec::new();
        for sensor in &mut self.sensors {
            events.extend(sensor.poll_events().await);
        }
        events
    }
}
```

---

## Evolution Path

### Phase 1: Basic Peripherals (v0.3.1) ← START HERE

**Sensors**:
- ✅ Screen (with verification)
- ✅ Keyboard (discrete input)
- ✅ Mouse (spatial input)
- ✅ Audio (bidirectional)

**Capabilities**:
- Discover each sensor
- Understand minimal interaction
- Confirm sensor is working
- Basic event handling

### Phase 2: Enhanced Interactions (v0.3.2)

**Enhancements**:
- Gesture recognition (mouse)
- Command sequences (keyboard)
- Voice commands (audio)
- Adaptive rendering (screen)

**Capabilities**:
- Pattern detection
- Context awareness
- Multi-sensor fusion
- Predictive input

### Phase 3: Advanced Sensors (v0.4.0)

**New Sensors**:
- Camera (motion detection, QR codes, visual input)
- Accelerometer (device orientation)
- GPS (location awareness)
- Network (discover other primals via mDNS)

**Capabilities**:
- Spatial awareness
- Environmental context
- Multi-device coordination

### Phase 4: Biometric Sensors (v0.5.0)

**Human Sensors**:
- Heart rate monitor
- Skin conductance
- Eye tracking
- Brain-computer interface (future)

**Capabilities**:
- User state awareness
- Adaptive UX based on stress/attention
- Health monitoring integration

### Phase 5: Environmental Sensors (v0.6.0)

**Nature Sensors**:
- Temperature
- Humidity
- Air quality
- Soil moisture
- Light levels
- Sound levels

**Capabilities**:
- "petalTongue can feel them all"
- Environmental monitoring
- Ecosystem awareness
- Integration with biomeOS sensors

---

## Implementation Priority

### Week 1: Screen + Keyboard (Essential)

```rust
// Can display + receive commands = minimal viable interface
let screen = ScreenDiscovery::discover().await.unwrap();
let keyboard = KeyboardDiscovery::discover().await.unwrap();

// Render something
screen.render(&data);

// Wait for confirmation
if let Some(key) = keyboard.wait_for_any_key(Duration::from_secs(5)).await {
    // User saw it and pressed a key!
    screen.confirm_visible = true;
}
```

### Week 2: Mouse + Audio (Enhanced)

```rust
// Add spatial selection
let mouse = MouseDiscovery::discover().await;

// Add audio feedback
let audio = AudioDiscovery::discover().await;
if audio.has_output {
    audio.beep_status(Status::Ready).await;
}
```

### Week 3: Sensor Abstraction

```rust
// Unified interface
let registry = SensorRegistry::discover_all().await;

// Work with any sensor
for event in registry.poll_all().await {
    handle_sensor_event(event);
}
```

### Week 4: Field Mode + Testing

```rust
// Prove it works without screen
let field_mode = FieldMode::new(keyboard, audio);
field_mode.run_field_interface().await?;
```

---

## Testing Strategy

### Sensor Discovery Tests

```rust
#[tokio::test]
async fn test_discover_all_sensors() {
    let registry = SensorRegistry::discover_all().await;
    
    // At minimum, should find SOMETHING
    assert!(!registry.sensors.is_empty());
    
    // Log what we found
    for sensor in &registry.sensors {
        println!("Found: {:?}", sensor.capabilities().sensor_type);
    }
}
```

### Minimal Interaction Tests

```rust
#[tokio::test]
async fn test_screen_verification() {
    let mut screen = ScreenSensor::new(
        ScreenDiscovery::discover().await.unwrap()
    );
    
    // Can we verify ANYTHING about the screen?
    assert!(screen.verify_minimal().await);
}

#[tokio::test]
async fn test_keyboard_any_key() {
    let mut keyboard = KeyboardSensor::new(
        KeyboardDiscovery::discover().await.unwrap()
    );
    
    println!("Press any key within 5 seconds...");
    
    let key = keyboard.wait_for_any_key(Duration::from_secs(5)).await;
    assert!(key.is_some());
}
```

### Field Mode Tests

```rust
#[tokio::test]
async fn test_field_mode() {
    // Verify we can work without screen
    let keyboard = KeyboardDiscovery::discover().await;
    let audio = AudioDiscovery::discover().await;
    
    // If we have BOTH keyboard and audio, field mode is viable
    assert!(keyboard.is_some());
    assert!(audio.has_output || audio.has_input);
}
```

---

## Success Criteria

### v0.3.1 Complete When:

- ✅ Can discover screen, keyboard, mouse, audio
- ✅ Each sensor knows its capabilities
- ✅ Minimal interaction works for each
- ✅ Screen visibility can be verified
- ✅ User interaction is tracked
- ✅ Field mode works (no screen required)
- ✅ Sensor abstraction layer complete
- ✅ Tests pass for all sensors

---

## Future Vision

### "petalTongue can feel them all"

```rust
// Year 2027...
pub enum SensorType {
    // Computer peripherals ✅
    Screen, Keyboard, Mouse, Audio,
    
    // Human sensors ✅
    HeartRate, SkinConductance, EyeTracking, BrainSignals,
    
    // Environmental sensors ✅
    Temperature, Humidity, AirQuality, SoilMoisture,
    LightLevel, SoundLevel, Pressure, WindSpeed,
    
    // Biological sensors ✅
    PlantElectricity, MycelialNetwork, BirdSongs, 
    InsectActivity, SoilMicrobiome,
    
    // Network sensors ✅
    PrimalHeartbeat, ServiceHealth, NetworkLatency,
    
    // Abstract sensors ✅
    UserAttention, SystemHealth, EcosystemBalance,
    CommunityMood, TemporalRhythm,
}
```

**petalTongue becomes aware of everything it can sense,
understands each capability, and adapts its interactions
accordingly. True Universal User Interface.** 🌸

---

**Status**: Specification Complete  
**Priority**: High (foundation for bidirectional UUI)  
**Effort**: 4 weeks (phased implementation)  
**Impact**: Complete sensory awareness

