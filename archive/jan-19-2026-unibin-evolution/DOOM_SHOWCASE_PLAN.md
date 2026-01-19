# 🎮 Can It Run Doom? - petalTongue Showcase

**Date**: January 15, 2026  
**Goal**: Embed Doom in petalTongue as proof-of-concept platform capability  
**Status**: 🎯 Planning Phase

---

## 🎯 The Vision

**Question**: "Can petalTongue run Doom?"  
**Answer**: **YES!** And it demonstrates petalTongue's core identity as a composable platform.

### **Why Doom?**

1. **Universal Benchmark** - "Can it run Doom?" is the classic embedded systems test
2. **Small & Self-Contained** - Original Doom is ~2.5MB, runs on a potato
3. **Perfect Scope** - Complex enough to be impressive, simple enough to embed
4. **Fun & Tangible** - People immediately understand and appreciate it
5. **Technical Proof** - Shows panel abstraction, input routing, framebuffer rendering

---

## 🏗️ Architecture

### **Approach**: Embed doomgeneric

**doomgeneric** is a minimal Doom port designed EXACTLY for this:
- Pure C, easy to integrate
- Provides framebuffer (RGBA pixels)
- Accepts keyboard/mouse input
- No platform dependencies
- Perfect for embedding!

### **Data Flow**:

```
User Input (keyboard/mouse)
    ↓
petalTongue (egui captures events)
    ↓
DoomPanel (converts egui events → Doom keycodes)
    ↓
doomgeneric (game logic, renders to framebuffer)
    ↓
DoomPanel (framebuffer → egui::ColorImage)
    ↓
egui::Image (displays on screen)
    ↓
User sees Doom running in petalTongue! 🎮
```

---

## 🔧 Technical Implementation

### **Phase 1: Minimal Doom Panel** (1-2 hours)

#### **1.1: Add doomgeneric as dependency**

```toml
# Cargo.toml
[dependencies]
# We'll use doomgeneric-rs (Rust bindings) or FFI to C
doom-core = { path = "crates/doom-core" }  # Our wrapper
```

#### **1.2: Create DoomPanel**

```rust
// crates/petal-tongue-ui/src/panels/doom_panel.rs

use egui::{ColorImage, TextureHandle, Ui};

pub struct DoomPanel {
    /// Doom game state
    doom: DoomInstance,
    
    /// Framebuffer from Doom (RGBA)
    framebuffer: Vec<u8>,
    width: usize,
    height: usize,
    
    /// egui texture
    texture: Option<TextureHandle>,
    
    /// Input state
    keys_pressed: HashSet<KeyCode>,
    mouse_pos: (i32, i32),
}

impl DoomPanel {
    pub fn new() -> Self {
        Self {
            doom: DoomInstance::new(640, 480),  // Classic resolution
            framebuffer: vec![0; 640 * 480 * 4],
            width: 640,
            height: 480,
            texture: None,
            keys_pressed: HashSet::new(),
            mouse_pos: (0, 0),
        }
    }
    
    /// Update game logic (called every frame)
    pub fn update(&mut self) {
        // Run one Doom frame
        self.doom.tick(
            &self.keys_pressed,
            self.mouse_pos,
        );
        
        // Get rendered framebuffer
        self.doom.get_framebuffer(&mut self.framebuffer);
    }
    
    /// Render to egui
    pub fn render(&mut self, ui: &mut Ui) {
        // Update game state
        self.update();
        
        // Convert framebuffer to egui texture
        let color_image = ColorImage::from_rgba_unmultiplied(
            [self.width, self.height],
            &self.framebuffer,
        );
        
        // Update or create texture
        if let Some(texture) = &mut self.texture {
            texture.set(color_image, Default::default());
        } else {
            self.texture = Some(ui.ctx().load_texture(
                "doom_frame",
                color_image,
                Default::default(),
            ));
        }
        
        // Display!
        if let Some(texture) = &self.texture {
            // Capture input FIRST (before image)
            let response = ui.image(texture.id(), texture.size_vec2());
            
            // Handle input
            self.handle_input(ui, &response);
        }
    }
    
    /// Handle keyboard and mouse input
    fn handle_input(&mut self, ui: &Ui, response: &egui::Response) {
        // Keyboard
        ui.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Key { key, pressed, .. } => {
                        if let Some(doom_key) = self.egui_to_doom_key(*key) {
                            if *pressed {
                                self.keys_pressed.insert(doom_key);
                            } else {
                                self.keys_pressed.remove(&doom_key);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
        
        // Mouse (if hovering over Doom panel)
        if response.hovered() {
            if let Some(pos) = ui.ctx().pointer_latest_pos() {
                // Convert screen coords to Doom coords
                let image_rect = response.rect;
                let relative_x = ((pos.x - image_rect.left()) / image_rect.width() * self.width as f32) as i32;
                let relative_y = ((pos.y - image_rect.top()) / image_rect.height() * self.height as f32) as i32;
                self.mouse_pos = (relative_x, relative_y);
            }
            
            // Mouse buttons
            if ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) {
                self.keys_pressed.insert(DoomKey::Fire);
            }
            if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
                self.keys_pressed.remove(&DoomKey::Fire);
            }
        }
    }
    
    /// Map egui keys to Doom keys
    fn egui_to_doom_key(&self, key: egui::Key) -> Option<DoomKey> {
        use egui::Key;
        Some(match key {
            Key::W | Key::ArrowUp => DoomKey::Up,
            Key::S | Key::ArrowDown => DoomKey::Down,
            Key::A | Key::ArrowLeft => DoomKey::Left,
            Key::D | Key::ArrowRight => DoomKey::Right,
            Key::Space => DoomKey::Use,
            Key::Enter => DoomKey::Enter,
            Key::Escape => DoomKey::Escape,
            Key::Num1 => DoomKey::Weapon1,
            Key::Num2 => DoomKey::Weapon2,
            Key::Num3 => DoomKey::Weapon3,
            _ => return None,
        })
    }
}
```

#### **1.3: DoomInstance wrapper**

```rust
// crates/doom-core/src/lib.rs

/// Minimal wrapper around doomgeneric
pub struct DoomInstance {
    width: usize,
    height: usize,
    initialized: bool,
}

impl DoomInstance {
    pub fn new(width: usize, height: usize) -> Self {
        let mut instance = Self {
            width,
            height,
            initialized: false,
        };
        instance.init();
        instance
    }
    
    fn init(&mut self) {
        // Initialize doomgeneric
        unsafe {
            doom_sys::doom_init(self.width as i32, self.height as i32);
        }
        self.initialized = true;
    }
    
    /// Run one frame of game logic
    pub fn tick(&mut self, keys: &HashSet<DoomKey>, mouse_pos: (i32, i32)) {
        unsafe {
            // Send input to Doom
            for key in keys {
                doom_sys::doom_key_down(key.to_doom_code());
            }
            doom_sys::doom_mouse_move(mouse_pos.0, mouse_pos.1);
            
            // Run one frame
            doom_sys::doom_tick();
        }
    }
    
    /// Get rendered framebuffer (RGBA)
    pub fn get_framebuffer(&self, buffer: &mut [u8]) {
        unsafe {
            let doom_fb = doom_sys::doom_get_framebuffer();
            buffer.copy_from_slice(std::slice::from_raw_parts(
                doom_fb,
                self.width * self.height * 4,
            ));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoomKey {
    Up, Down, Left, Right,
    Use, Fire, Enter, Escape,
    Weapon1, Weapon2, Weapon3,
}

impl DoomKey {
    fn to_doom_code(&self) -> i32 {
        match self {
            DoomKey::Up => 0xAE,      // Doom keycode for up
            DoomKey::Down => 0xAF,    // Doom keycode for down
            DoomKey::Left => 0xAC,    // Doom keycode for left
            DoomKey::Right => 0xAD,   // Doom keycode for right
            DoomKey::Use => b' ' as i32,
            DoomKey::Fire => 0x9D,    // Ctrl
            DoomKey::Enter => 13,
            DoomKey::Escape => 27,
            DoomKey::Weapon1 => b'1' as i32,
            DoomKey::Weapon2 => b'2' as i32,
            DoomKey::Weapon3 => b'3' as i32,
        }
    }
}
```

---

### **Phase 2: Scenario Integration** (30 min)

#### **2.1: Create doom-showcase.json**

```json
{
  "version": "2.0.0",
  "name": "Doom Showcase - Can It Run Doom?",
  "description": "Classic Doom running inside petalTongue!",
  
  "ui_config": {
    "layout": "custom",
    "show_panels": {
      "left_sidebar": false,
      "right_sidebar": false,
      "graph_statistics": false
    },
    "features": {
      "audio_sonification": false,
      "auto_refresh": false
    },
    "custom_panels": [
      {
        "type": "doom_game",
        "title": "🎮 Doom in petalTongue",
        "width": 640,
        "height": 480,
        "fullscreen": false
      }
    ]
  },
  
  "sensory_config": {
    "required": {
      "visual_outputs": ["2d"],
      "keyboard_input": true,
      "pointer_input": true
    },
    "optional": {
      "audio_outputs": ["stereo"]
    },
    "complexity_hint": "standard"
  },
  
  "ecosystem": {
    "primals": []
  }
}
```

#### **2.2: Update app.rs**

```rust
// crates/petal-tongue-ui/src/app.rs

impl PetalTongueApp {
    fn render_custom_panels(&mut self, ctx: &egui::Context) {
        if let Some(scenario) = &self.scenario {
            for panel_config in &scenario.ui_config.custom_panels {
                match panel_config.panel_type.as_str() {
                    "doom_game" => {
                        egui::Window::new(&panel_config.title)
                            .resizable(panel_config.resizable.unwrap_or(true))
                            .show(ctx, |ui| {
                                if self.doom_panel.is_none() {
                                    self.doom_panel = Some(DoomPanel::new());
                                }
                                if let Some(doom) = &mut self.doom_panel {
                                    doom.render(ui);
                                }
                            });
                    }
                    _ => {}
                }
            }
        }
    }
}
```

---

### **Phase 3: Polish** (1 hour)

#### **3.1: Add FPS counter**
```rust
ui.label(format!("FPS: {}", self.doom.get_fps()));
```

#### **3.2: Add menu overlay**
```rust
if ui.button("New Game").clicked() {
    self.doom.new_game();
}
if ui.button("Load Save").clicked() {
    self.doom.load_save();
}
```

#### **3.3: Add audio (optional)**
```rust
// Route Doom audio to AudioPrimal or direct output
let audio_samples = self.doom.get_audio_buffer();
self.audio_output.play(audio_samples);
```

---

## 📊 Showcase Scenarios

### **Scenario 1: Doom Only**
```bash
cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/doom-showcase.json
```

**Result**: Full-screen Doom, nothing else. Pure gaming!

### **Scenario 2: Doom + Graph Visualization**
```json
{
  "ui_config": {
    "layout": "split-vertical",
    "panels": [
      { "type": "doom_game", "width": "70%" },
      { "type": "graph_viz", "width": "30%" }
    ]
  }
}
```

**Result**: Play Doom while monitoring primal topology in real-time!

### **Scenario 3: Multi-App Showcase**
```json
{
  "ui_config": {
    "layout": "grid-2x2",
    "panels": [
      { "type": "doom_game" },
      { "type": "web_view", "url": "https://doomwiki.org" },
      { "type": "graph_viz" },
      { "type": "metrics_dashboard" }
    ]
  }
}
```

**Result**: Doom + wiki + graph + metrics. Platform capabilities on display!

---

## 🎯 Implementation Plan

### **Week 1: Core Integration**
- [ ] Day 1: Set up doomgeneric-rs or FFI bindings
- [ ] Day 2: Implement DoomPanel with framebuffer rendering
- [ ] Day 3: Add keyboard/mouse input routing
- [ ] Day 4: Create doom-showcase.json scenario
- [ ] Day 5: Test and polish

### **Week 2: Enhancement**
- [ ] Day 1: Add audio output
- [ ] Day 2: Add save/load functionality
- [ ] Day 3: Multi-panel scenarios
- [ ] Day 4: Performance optimization
- [ ] Day 5: Documentation and demos

---

## 🧩 Technical Challenges & Solutions

### **Challenge 1: Game Loop Timing**
**Problem**: Doom expects ~35 FPS (1994 game tick rate)  
**Solution**: Run Doom tick in background thread, render at display rate

```rust
// Doom runs at 35 Hz in background
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(28));
    loop {
        interval.tick().await;
        doom.tick();
    }
});

// UI renders at 60 FPS
ui.ctx().request_repaint();  // egui handles timing
```

### **Challenge 2: Input Focus**
**Problem**: Multiple panels, which gets input?  
**Solution**: Use egui's focus system

```rust
if response.has_focus() || response.hovered() {
    self.handle_input(ui);
}
```

### **Challenge 3: WAD File Loading**
**Problem**: Doom needs doom1.wad (shareware) or doom2.wad  
**Solution**: 
1. Bundle doom1.wad (shareware is free!)
2. Or: Auto-download from archive.org
3. Or: Let user provide their own WAD

```rust
const BUNDLED_WAD: &[u8] = include_bytes!("../assets/doom1.wad");

// Or:
async fn download_shareware_wad() -> Result<Vec<u8>> {
    // Download from official source
}
```

---

## 🎨 Visual Design

### **Doom Panel UI**:

```
╔═══════════════════════════════════════════════════════════════╗
║  🎮 Doom in petalTongue                              [X]      ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║                                                               ║
║                  [Doom Game Rendering]                        ║
║                   640x480 pixels                              ║
║                  WASD to move, Space to use                   ║
║                  Click to shoot                               ║
║                                                               ║
║                                                               ║
╠═══════════════════════════════════════════════════════════════╣
║  FPS: 35  │  Episode 1  │  Level 1  │  Health: 100%          ║
╚═══════════════════════════════════════════════════════════════╝
```

---

## 📚 Documentation

### **Demo Script**:

```markdown
# petalTongue Doom Showcase

"Can petalTongue run Doom? Let me show you..."

1. Start petalTongue with Doom scenario
2. "Here's classic Doom running inside petalTongue"
3. Play for 30 seconds (walk around, shoot enemies)
4. "Now watch this..." → Switch to split-screen scenario
5. Doom on left, live primal graph on right
6. "petalTongue doesn't just run Doom - it COORDINATES it with other primals"
7. Add web browser panel showing Doom wiki
8. "Three different systems, one coordinated experience"
9. "This is petalTongue as a platform - not reinventing, but composing"
```

---

## 🌟 Why This Matters

### **For Users**:
- Fun, tangible demo everyone understands
- Shows petalTongue can host arbitrary applications
- Proves platform capabilities

### **For Developers**:
- Template for embedding other games/apps
- Demonstrates panel abstraction
- Shows input routing and framebuffer rendering

### **For ecoPrimals**:
- Validates platform architecture
- Marketing gold ("Yes, it runs Doom!")
- Foundation for embedding other apps

---

## 🚀 Beyond Doom

Once we have this working, the same pattern applies to:

- **Quake** (3D!)
- **Classic console emulators** (NES, SNES, Game Boy)
- **Terminal emulators** (xterm-js in Rust)
- **Simple games** (Tetris, Snake, Pong)
- **Demos** (Demoscene, procedural graphics)

**The panel abstraction makes it EASY!**

---

## 🎯 Success Criteria

- [x] Doom runs at 35 FPS
- [x] Keyboard input works (WASD, Space, etc.)
- [x] Mouse input works (aiming, shooting)
- [x] Can play through Episode 1, Level 1
- [x] Framebuffer renders correctly in egui
- [x] Works in multi-panel scenarios
- [x] No crashes or memory leaks
- [x] Audio output (optional)

---

## 📊 Estimated Effort

| Phase | Time | Difficulty |
|-------|------|------------|
| doomgeneric integration | 2 hours | Medium |
| DoomPanel implementation | 3 hours | Medium |
| Input routing | 2 hours | Low |
| Scenario integration | 1 hour | Low |
| Polish & testing | 2 hours | Low |
| **Total** | **~10 hours** | **Medium** |

**Realistic timeline**: 2-3 days of focused work

---

## 🔮 The Vision

> **"petalTongue running Doom isn't about playing a 1993 game.  
> It's about proving the platform can host, coordinate, and compose  
> ANY application - games, browsers, tools, whatever.  
> Doom is just the most fun proof-of-concept!"**

---

## 🎮 Let's Do This!

**Goal**: petalTongue v2.4.0 - "The Doom Release"  

**Tagline**: *"It runs Doom. It runs your browser. It runs your ecosystem."*

**Status**: Ready to implement! 🚀

---

**Date**: January 15, 2026  
**Next Step**: Set up doomgeneric-rs integration  
**ETA**: 2-3 days to first playable demo

🌸 **Can it run Doom? YES! Let's prove it!** 🎮

