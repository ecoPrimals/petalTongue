# 🌸 Universal User Interface - Evolution Vision

**Date**: January 12, 2026  
**Status**: Vision & Architecture  
**Priority**: Strategic Evolution  
**Goal**: Evolve petalTongue to THE universal interface layer for ecoPrimals

---

## 🎯 **The Vision: Universal User Interface**

**petalTongue becomes the translation layer between ANY universe and ANY user.**

### **Core Insight:**

Traditional UIs assume:
- **ONE universe**: Desktop OS with keyboard, mouse, screen
- **ONE user**: Sighted human with hands

**Universal UI supports:**
- **∞ universes**: OS, cloud, fractal compute, edge devices, spacecraft, ocean sensors
- **∞ users**: Humans (all abilities), AI agents, dolphins, fungi, distributed intelligence

---

## 🌍 **The Two Dimensions of Universality**

### **Dimension 1: Universe (Computational Environment)**

**What Can Change:**

1. **Substrate**:
   - Traditional OS (Linux, Windows, Mac)
   - Cloud/Fractal (distributed compute)
   - Edge devices (IoT, embedded)
   - Exotic environments (spacecraft, underwater, biological computing)

2. **Input/Output Capabilities**:
   - Full desktop (keyboard, mouse, display)
   - TUI only (SSH, serial, headless)
   - API only (programmatic access)
   - Hybrid (partial capabilities)

3. **Resources**:
   - Full compute (desktop, server)
   - Limited compute (Raspberry Pi)
   - Minimal compute (microcontroller)
   - Shared compute (cloud fractal)

**petalTongue's Adaptation:**
```rust
// Runtime capability detection
let universe = UniverseDetector::detect().await?;

match universe.display_capability {
    DisplayCapability::Full => render_egui(),
    DisplayCapability::Terminal => render_tui(),
    DisplayCapability::None => render_api_only(),
}

match universe.compute_capability {
    ComputeCapability::Full => enable_gpu_acceleration(),
    ComputeCapability::Limited => use_cpu_only(),
    ComputeCapability::Minimal => use_remote_compute(),
}
```

---

### **Dimension 2: User (Intelligence Interface)**

**Who Can Use petalTongue:**

1. **Humans** (Primary, Diverse Abilities):
   - Sighted users → Visual GUI (egui)
   - Blind users → Audio GUI (soundscape + screen reader)
   - Mobility-limited → Voice control + simplified UI
   - Cognitive diversity → Adaptive complexity levels
   - Different languages/cultures → Localization

2. **AI Agents** (Agentic Users):
   - LLMs (GPT, Claude, etc.) → JSON/API interface
   - Specialized agents → Task-specific views
   - Distributed intelligence → Coordinated multi-agent access

3. **Non-Human Intelligence** (Future):
   - Dolphins → Acoustic interface (click patterns)
   - Fungi → Chemical gradient interface (mycelial networks)
   - Distributed systems → Emergent collective intelligence

4. **Hybrid** (Human + AI Collaboration):
   - AI assistant augmenting human interaction
   - Human supervising AI workflows
   - Collaborative problem-solving

**petalTongue's Adaptation:**
```rust
// Runtime user detection
let user = UserDetector::detect().await?;

match user.interface_type {
    InterfaceType::HumanVisual => render_gui_with_accessibility(),
    InterfaceType::HumanAudio => render_soundscape(),
    InterfaceType::AIAgent => render_api_with_context(),
    InterfaceType::NonHuman(protocol) => render_protocol_specific(protocol),
    InterfaceType::Hybrid { human, ai } => render_collaborative(human, ai),
}
```

---

## 🏗️ **Architecture: Universal Adaptation Layer**

### **Current Multi-Modal Architecture** (Foundation)

```
┌────────────────────────────────────────────┐
│      petalTongue Core Engine               │
│   (Topology State, Event Bus, Discovery)   │
└────────────────┬───────────────────────────┘
                 │
    ┌────────────┼────────────┐
    │            │            │
┌───▼────┐  ┌───▼────┐  ┌───▼────┐
│Terminal│  │  Egui  │  │ Audio  │
│  GUI   │  │  GUI   │  │  GUI   │
└────────┘  └────────┘  └────────┘
 (Tier 1)    (Tier 3)    (Tier 2)
```

**Status**: ✅ Working! 72% complete

---

### **Evolved Universal Interface Architecture** (Target)

```
┌─────────────────────────────────────────────────────────────┐
│            Universal Adaptation Layer                       │
│  (Universe Detection + User Detection + Capability Mapping) │
└──────────────┬──────────────────────────────────────────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
┌───▼────┐ ┌──▼───┐ ┌────▼────┐
│Universe│ │User  │ │Interface│
│Detector│ │Detect│ │Selector │
└───┬────┘ └──┬───┘ └────┬────┘
    │         │          │
    └─────────┴──────────┘
              │
    ┌─────────┴─────────┐
    │  Interface Matrix  │ ← Maps (Universe × User) → Interface
    └─────────┬──────────┘
              │
    ┌─────────┴─────────────────────────────────┐
    │           │           │           │         │
┌───▼────┐ ┌──▼──┐ ┌──▼────┐ ┌──▼──┐ ┌──▼────┐
│Terminal│ │Egui │ │Audio  │ │JSON │ │Custom │
│  TUI   │ │ GUI │ │scape  │ │ API │ │Protocol│
└────────┘ └─────┘ └───────┘ └─────┘ └────────┘
 Human      Human    Blind    AI      Non-Human
 Terminal   Desktop  Human    Agent   (Dolphin)
```

---

## 🔬 **Universe Detection System**

### **1. Computational Environment Detection**

```rust
pub struct UniverseDetector;

impl UniverseDetector {
    /// Detect current computational universe
    pub async fn detect() -> Result<Universe> {
        Ok(Universe {
            substrate: Self::detect_substrate()?,
            display: Self::detect_display()?,
            input: Self::detect_input()?,
            audio: Self::detect_audio()?,
            compute: Self::detect_compute().await?,
            network: Self::detect_network().await?,
        })
    }
    
    fn detect_substrate() -> Result<Substrate> {
        // OS vs Cloud vs Fractal vs Edge
        if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            Ok(Substrate::CloudNative)
        } else if std::env::var("TOADSTOOL_FRACTAL_ID").is_ok() {
            Ok(Substrate::FractalCompute)
        } else if Path::new("/proc/device-tree/model").exists() {
            Ok(Substrate::EdgeDevice)
        } else {
            Ok(Substrate::TraditionalOS)
        }
    }
    
    fn detect_display() -> Result<DisplayCapability> {
        // Full GUI vs TUI vs None
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            Ok(DisplayCapability::Full)
        } else if std::env::var("TERM").is_ok() {
            Ok(DisplayCapability::Terminal)
        } else {
            Ok(DisplayCapability::None)
        }
    }
    
    async fn detect_compute() -> Result<ComputeCapability> {
        // Check for ToadStool, GPU, CPU-only
        if let Ok(toadstool) = ToadStoolClient::discover().await {
            Ok(ComputeCapability::Fractal { client: toadstool })
        } else if has_gpu() {
            Ok(ComputeCapability::GPU)
        } else {
            Ok(ComputeCapability::CPU)
        }
    }
}

pub struct Universe {
    pub substrate: Substrate,
    pub display: DisplayCapability,
    pub input: InputCapability,
    pub audio: AudioCapability,
    pub compute: ComputeCapability,
    pub network: NetworkCapability,
}

pub enum Substrate {
    TraditionalOS,      // Linux, Windows, Mac
    CloudNative,        // Kubernetes, Docker
    FractalCompute,     // ToadStool fractal
    EdgeDevice,         // Raspberry Pi, embedded
    ExoticEnvironment,  // Spacecraft, underwater, etc.
}

pub enum DisplayCapability {
    Full,         // X11/Wayland display
    Terminal,     // TUI only
    None,         // Headless/API-only
    Remote,       // VNC/RDP
}
```

---

## 👤 **User Detection System**

### **2. User Interface Type Detection**

```rust
pub struct UserDetector;

impl UserDetector {
    /// Detect user type and capabilities
    pub async fn detect() -> Result<User> {
        Ok(User {
            user_type: Self::detect_type()?,
            abilities: Self::detect_abilities()?,
            preferences: Self::load_preferences().await?,
            context: Self::detect_context()?,
        })
    }
    
    fn detect_type() -> Result<UserType> {
        // Check environment for hints
        if let Ok(agent_id) = std::env::var("AI_AGENT_ID") {
            Ok(UserType::AIAgent { id: agent_id })
        } else if let Ok(protocol) = std::env::var("NONHUMAN_PROTOCOL") {
            Ok(UserType::NonHuman { protocol })
        } else {
            Ok(UserType::Human {
                abilities: HumanAbilities::detect(),
            })
        }
    }
    
    async fn load_preferences() -> Result<UserPreferences> {
        // Check for stored preferences (NestGate)
        if let Ok(nestgate) = NestGateClient::discover().await {
            nestgate.load_preferences("petaltongue").await
        } else {
            Ok(UserPreferences::default())
        }
    }
}

pub struct User {
    pub user_type: UserType,
    pub abilities: UserAbilities,
    pub preferences: UserPreferences,
    pub context: UserContext,
}

pub enum UserType {
    Human {
        abilities: HumanAbilities,
    },
    AIAgent {
        id: String,
    },
    NonHuman {
        protocol: String,  // "dolphin-acoustic", "fungal-chemical", etc.
    },
    Hybrid {
        human: Box<User>,
        ai: Box<User>,
    },
}

pub struct HumanAbilities {
    pub vision: VisionCapability,     // Full, Low, None
    pub hearing: HearingCapability,    // Full, Partial, None
    pub mobility: MobilityCapability,  // Full, Limited, VoiceOnly
    pub cognition: CognitionLevel,     // Expert, Intermediate, Simple
    pub language: Vec<String>,         // ["en", "es", "zh"]
}
```

---

## 🎨 **Interface Selection Matrix**

### **3. (Universe × User) → Interface Mapping**

```rust
pub struct InterfaceSelector;

impl InterfaceSelector {
    /// Select optimal interface(s) for (Universe, User) pair
    pub fn select(universe: &Universe, user: &User) -> Result<Vec<Interface>> {
        let mut interfaces = Vec::new();
        
        // Primary interface based on user type
        match &user.user_type {
            UserType::Human { abilities } => {
                interfaces.extend(Self::select_human_interface(universe, abilities)?);
            }
            UserType::AIAgent { id } => {
                interfaces.push(Interface::JSONAPI);
                interfaces.push(Interface::GraphQLAPI);  // Richer queries
            }
            UserType::NonHuman { protocol } => {
                interfaces.push(Self::select_protocol_interface(protocol)?);
            }
            UserType::Hybrid { human, ai } => {
                // Collaborative interface
                interfaces.push(Interface::HybridCollaborative {
                    human_view: Box::new(Self::select(universe, human)?),
                    ai_view: Box::new(Self::select(universe, ai)?),
                });
            }
        }
        
        Ok(interfaces)
    }
    
    fn select_human_interface(
        universe: &Universe,
        abilities: &HumanAbilities,
    ) -> Result<Vec<Interface>> {
        let mut interfaces = Vec::new();
        
        // Vision-based selection
        match (universe.display, abilities.vision) {
            (DisplayCapability::Full, VisionCapability::Full) => {
                interfaces.push(Interface::EguiGUI);
            }
            (DisplayCapability::Full, VisionCapability::Low) => {
                interfaces.push(Interface::HighContrastGUI);
            }
            (DisplayCapability::Full, VisionCapability::None) => {
                interfaces.push(Interface::AudioGUI);  // Screen reader + soundscape
                interfaces.push(Interface::BrailleGUI);  // If hardware available
            }
            (DisplayCapability::Terminal, _) => {
                interfaces.push(Interface::RichTUI);  // New! Enhanced TUI
            }
            (DisplayCapability::None, _) => {
                interfaces.push(Interface::JSONAPI);
                if abilities.hearing != HearingCapability::None {
                    interfaces.push(Interface::AudioOnlyGUI);
                }
            }
            _ => {
                // Fallback to most accessible
                interfaces.push(Interface::TerminalGUI);
            }
        }
        
        // Add complementary interfaces
        if abilities.hearing == HearingCapability::Full {
            interfaces.push(Interface::AudioSonification);  // Always add audio
        }
        
        Ok(interfaces)
    }
    
    fn select_protocol_interface(protocol: &str) -> Result<Interface> {
        match protocol {
            "dolphin-acoustic" => Ok(Interface::DolphinAcoustic),
            "fungal-chemical" => Ok(Interface::FungalChemical),
            "api-json" => Ok(Interface::JSONAPI),
            _ => Err(anyhow!("Unknown protocol: {}", protocol)),
        }
    }
}

pub enum Interface {
    // Human Visual
    EguiGUI,
    HighContrastGUI,
    RichTUI,           // ⭐ NEW! Enhanced TUI for biomeOS
    TerminalGUI,
    
    // Human Audio
    AudioGUI,          // Screen reader compatible
    AudioSonification, // Pure audio representation
    AudioOnlyGUI,      // Voice-controlled
    
    // Human Tactile
    BrailleGUI,        // If hardware available
    
    // AI Agent
    JSONAPI,
    GraphQLAPI,
    RestAPI,
    
    // Non-Human
    DolphinAcoustic,   // Click patterns
    FungalChemical,    // Chemical gradients
    CustomProtocol(String),
    
    // Hybrid
    HybridCollaborative {
        human_view: Box<Vec<Interface>>,
        ai_view: Box<Vec<Interface>>,
    },
}
```

---

## 🚀 **Key Evolution: Rich TUI for biomeOS**

### **biomeOS Use Case: Pure Rust TUI**

**Requirements**:
- Can run as PopOS-like UI (independent OS)
- Can run on top of existing OS (SSH, headless)
- Full neuralAPI, NUCLEUS, liveSpore management
- Interactive, real-time, beautiful

**Solution: Rich TUI using `ratatui`**

```rust
// crates/petal-tongue-tui/src/lib.rs

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

pub struct RichTUI {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    state: TUIState,
}

pub struct TUIState {
    /// Active view
    view: View,
    
    /// Topology graph (live)
    graph: Arc<RwLock<GraphEngine>>,
    
    /// Selected node
    selected: Option<String>,
    
    /// Log messages
    logs: Vec<LogMessage>,
    
    /// Status
    status: SystemStatus,
}

pub enum View {
    Dashboard,      // Overview of all primals
    Topology,       // Graph visualization (ASCII art)
    Devices,        // Device management (biomeOS)
    Primals,        // Primal status
    Logs,           // System logs
    neuralAPI,      // Graph orchestration
    NUCLEUS,        // Secure discovery
    LiveSpore,      // Live deployment
}

impl RichTUI {
    /// Launch rich TUI
    pub async fn launch() -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let mut tui = Self {
            terminal,
            state: TUIState::new(),
        };
        
        tui.run().await?;
        
        // Cleanup
        disable_raw_mode()?;
        tui.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        
        Ok(())
    }
    
    async fn run(&mut self) -> Result<()> {
        loop {
            // Render
            self.terminal.draw(|f| self.render(f))?;
            
            // Handle input
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('1') => self.state.view = View::Dashboard,
                        KeyCode::Char('2') => self.state.view = View::Topology,
                        KeyCode::Char('3') => self.state.view = View::Devices,
                        KeyCode::Char('4') => self.state.view = View::Primals,
                        KeyCode::Char('5') => self.state.view = View::Logs,
                        KeyCode::Char('6') => self.state.view = View::neuralAPI,
                        KeyCode::Char('7') => self.state.view = View::NUCLEUS,
                        KeyCode::Char('8') => self.state.view = View::LiveSpore,
                        _ => {}
                    }
                }
            }
            
            // Update state (async)
            self.state.update().await?;
        }
        
        Ok(())
    }
    
    fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Header
                Constraint::Min(0),      // Main content
                Constraint::Length(3),   // Footer/status
            ])
            .split(f.size());
        
        // Header
        let header = Paragraph::new("🌸 petalTongue - Universal Interface (TUI Mode)")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);
        
        // Main content (based on view)
        match self.state.view {
            View::Dashboard => self.render_dashboard(f, chunks[1]),
            View::Topology => self.render_topology(f, chunks[1]),
            View::Devices => self.render_devices(f, chunks[1]),
            View::Primals => self.render_primals(f, chunks[1]),
            View::Logs => self.render_logs(f, chunks[1]),
            View::neuralAPI => self.render_neural_api(f, chunks[1]),
            View::NUCLEUS => self.render_nucleus(f, chunks[1]),
            View::LiveSpore => self.render_livespore(f, chunks[1]),
        }
        
        // Footer
        let footer = Paragraph::new("[1-8] Views | [q] Quit")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    }
}
```

---

## 🧬 **Non-Human Interface Examples**

### **Dolphin Acoustic Interface**

```rust
// Example: Dolphin translator interface

pub struct DolphinAcousticInterface {
    /// Audio input/output
    audio_io: AudioIO,
    
    /// Click pattern recognizer
    recognizer: ClickPatternRecognizer,
    
    /// Topology state
    graph: Arc<RwLock<GraphEngine>>,
}

impl DolphinAcousticInterface {
    /// Render topology as acoustic patterns
    pub async fn render(&mut self) -> Result<()> {
        let graph = self.graph.read().await;
        
        // Convert graph to acoustic representation
        // - Node count → frequency modulation
        // - Edge connections → click patterns
        // - Health status → amplitude
        
        for node in graph.nodes() {
            let pattern = self.node_to_click_pattern(&node)?;
            self.audio_io.emit_clicks(pattern).await?;
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Ok(())
    }
    
    /// Listen for dolphin input (click patterns)
    pub async fn listen(&mut self) -> Result<DolphinCommand> {
        let audio_data = self.audio_io.record().await?;
        let pattern = self.recognizer.recognize(&audio_data)?;
        
        // Map click patterns to commands
        match pattern {
            ClickPattern::Single => Ok(DolphinCommand::Select),
            ClickPattern::Double => Ok(DolphinCommand::Confirm),
            ClickPattern::LongBurst => Ok(DolphinCommand::Navigate),
            _ => Ok(DolphinCommand::Unknown),
        }
    }
}
```

---

## 📊 **Implementation Roadmap**

### **Phase 1: Universe Detection** (1-2 weeks)

**Goal**: Runtime environment detection

- [x] Substrate detection (OS, cloud, fractal, edge)
- [x] Display capability detection (Full, Terminal, None)
- [x] Input capability detection (Keyboard, Voice, API)
- [x] Audio capability detection (Speakers, Headphones, None)
- [x] Compute capability detection (GPU, CPU, Fractal)
- [x] Network capability detection (Full, Limited, Airgapped)

**Status**: Mostly complete! (Display detection already exists)

---

### **Phase 2: User Detection** (1-2 weeks)

**Goal**: User type and ability detection

- [ ] User type detection (Human, AI, Non-Human, Hybrid)
- [ ] Human abilities detection (Vision, Hearing, Mobility, Cognition)
- [ ] AI agent identification (Agent ID, capabilities)
- [ ] Preference loading (NestGate integration)
- [ ] Accessibility configuration

**Status**: Not started

---

### **Phase 3: Interface Selection** (1-2 weeks)

**Goal**: Dynamic interface mapping

- [ ] Interface selection matrix
- [ ] Multi-interface coordination
- [ ] Adaptive complexity levels
- [ ] Graceful degradation

**Status**: Not started

---

### **Phase 4: Rich TUI for biomeOS** (2-3 weeks) ⭐ **PRIORITY**

**Goal**: Production TUI for neuralAPI, NUCLEUS, liveSpore

- [ ] Create `petal-tongue-tui` crate
- [ ] Integrate `ratatui` for rich terminal UI
- [ ] Implement 8 views (Dashboard, Topology, Devices, Primals, Logs, neuralAPI, NUCLEUS, LiveSpore)
- [ ] Real-time updates via WebSocket/JSON-RPC
- [ ] Keyboard navigation
- [ ] Mouse support (if terminal supports it)
- [ ] ASCII art topology visualization
- [ ] Interactive device assignment
- [ ] Live log streaming

**Status**: Not started (HIGH PRIORITY for biomeOS)

---

### **Phase 5: API Interfaces for AI Agents** (2-3 weeks)

**Goal**: AI-friendly programmatic access

- [ ] Enhanced JSON API (context-aware)
- [ ] GraphQL API (flexible queries)
- [ ] WebSocket streaming
- [ ] Agent authentication (BearDog)
- [ ] Agent preferences (NestGate)
- [ ] Multi-agent coordination

**Status**: Not started

---

### **Phase 6: Accessibility Enhancements** (2-3 weeks)

**Goal**: True accessibility for all abilities

- [ ] Screen reader optimization
- [ ] High contrast themes
- [ ] Large text mode
- [ ] Voice control
- [ ] Simplified UI mode
- [ ] Multilingual support

**Status**: Partial (color palette system exists)

---

### **Phase 7: Non-Human Interfaces** (Exploratory)

**Goal**: Proof-of-concept for non-human intelligence

- [ ] Dolphin acoustic interface (click patterns)
- [ ] API extension framework
- [ ] Protocol plugin system

**Status**: Not started (Future research)

---

## 🎊 **Success Criteria**

### **Universe Universality**:
- ✅ Runs on any OS (Linux, Windows, Mac)
- [ ] Runs in cloud (Kubernetes, Docker)
- [ ] Runs in fractal compute (ToadStool)
- [ ] Runs on edge devices (Raspberry Pi)
- [ ] Runs headless (API-only)
- [ ] Runs in TUI (SSH, serial)

### **User Universality**:
- [ ] Accessible to sighted humans (egui)
- [ ] Accessible to blind humans (audio + screen reader)
- [ ] Accessible to mobility-limited humans (voice + simple UI)
- [ ] Accessible to AI agents (JSON/GraphQL API)
- [ ] Extensible for non-human interfaces (plugin system)

### **Quality**:
- [ ] Zero unsafe code
- [ ] Pure Rust
- [ ] Graceful degradation
- [ ] Runtime discovery
- [ ] TRUE PRIMAL principles

---

## 🌸 **TRUE PRIMAL Alignment**

1. **Zero Hardcoding**: ✅
   - Runtime universe detection
   - Runtime user detection
   - Dynamic interface selection

2. **Capability-Based**: ✅
   - Discover available interfaces at runtime
   - Select optimal interface for (universe, user) pair
   - Graceful degradation

3. **Self-Knowledge**: ✅
   - Knows own capabilities
   - Knows own interfaces
   - Adapts to environment

4. **Agnostic**: ✅
   - No assumptions about universe
   - No assumptions about user
   - Universal translation layer

5. **Graceful Degradation**: ✅
   - Works in any environment
   - Works for any user
   - Always provides SOME interface

---

## 🚀 **Immediate Next Steps**

### **For biomeOS** (This Week):

1. **Create `petal-tongue-tui` crate**
   - Integrate `ratatui`
   - Implement basic dashboard view
   - Wire to existing topology data

2. **Implement 8 TUI Views**:
   - Dashboard (overview)
   - Topology (ASCII graph)
   - Devices (device management)
   - Primals (primal status)
   - Logs (real-time logs)
   - neuralAPI (graph orchestration)
   - NUCLEUS (secure discovery)
   - LiveSpore (live deployment)

3. **Real-Time Integration**:
   - WebSocket for live updates
   - JSON-RPC for commands
   - Keyboard shortcuts
   - Interactive selection

---

**Status**: Vision documented, architecture defined, ready to evolve! 🌸

**petalTongue**: From "a UI for topology" → "THE universal interface for ANY universe and ANY user" 🚀

