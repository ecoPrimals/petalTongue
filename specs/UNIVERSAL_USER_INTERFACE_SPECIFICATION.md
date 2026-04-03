# Universal User Interface Specification
## petalTongue - The Translation Layer for ANY Universe and ANY User

**Version**: 1.0.0  
**Date**: January 12, 2026  
**Status**: Formal Specification  
**Primal**: petalTongue  
**Domain**: Rendering & Interface Translation

---

## 1. Executive Summary

### 1.1 Purpose

Define the architecture and implementation of petalTongue's **Universal User Interface (UUI)** system - a translation layer that enables ANY computational universe to interact with ANY user type.

### 1.2 Scope

**petalTongue's Domain** (What We Own):
- Interface rendering (visual, audio, terminal, API)
- User capability detection
- Universe capability detection
- Interface selection and adaptation
- Multi-modal coordination
- Translation between modalities

**Other Primals' Domains** (What We Leverage):
- **ToadStool**: Compute acceleration (GPU, fractal)
- **Songbird**: Discovery and communication
- **NestGate**: Preference persistence
- **BearDog**: User authentication
- **Squirrel**: AI assistance

### 1.3 Core Principle

> **"petalTongue renders. Other primals provide capabilities."**

**Division of Labor**:
```
┌─────────────────────────────────────────────────────┐
│         petalTongue (Rendering Layer)               │
│   - Interface rendering (GUI, TUI, Audio, API)      │
│   - User/Universe detection                         │
│   - Translation between modalities                  │
└────────────┬────────────────────────────────────────┘
             │ Leverages capabilities from:
    ┌────────┼────────┬────────┬────────┐
    │        │        │        │        │
┌───▼──┐ ┌──▼───┐ ┌──▼───┐ ┌──▼───┐ ┌──▼────┐
│Toad  │ │Song  │ │Nest  │ │Bear  │ │Squirrel│
│Stool │ │bird  │ │Gate  │ │Dog   │ │       │
└──────┘ └──────┘ └──────┘ └──────┘ └────────┘
Compute  Discovery Persist Auth     AI
```

---

## 2. Two-Dimensional Universality

### 2.1 Dimension 1: Universe (Computational Environment)

**What Can Vary:**

| Aspect | Options | Detection Method |
|--------|---------|------------------|
| **Substrate** | Traditional OS, Cloud, Fractal, Edge, Exotic | Environment vars, /proc, network |
| **Display** | Full GUI, Terminal, None | DISPLAY, WAYLAND_DISPLAY, TERM |
| **Audio** | Speakers, Headphones, None | Audio device enumeration |
| **Input** | Keyboard, Mouse, Touch, Voice, API | Input device discovery |
| **Compute** | GPU, CPU, Fractal (ToadStool) | GPU detection, ToadStool discovery |
| **Network** | Full, Limited, Airgapped | Network interface discovery |

**petalTongue's Role**: Detect and adapt  
**ToadStool's Role**: Provide compute when available  
**Songbird's Role**: Provide network discovery

### 2.2 Dimension 2: User (Intelligence Interface)

**Who Can Use petalTongue:**

| User Type | Interface Needs | Detection Method |
|-----------|----------------|------------------|
| **Human (Sighted)** | Visual GUI | Default assumption |
| **Human (Blind)** | Audio + Screen Reader | Accessibility prefs (NestGate) |
| **Human (Mobility-Limited)** | Voice + Simplified UI | Accessibility prefs (NestGate) |
| **AI Agent** | JSON/GraphQL API | AI_AGENT_ID env var |
| **Non-Human** | Custom Protocol | NONHUMAN_PROTOCOL env var |
| **Hybrid** | Collaborative UI | Multiple interface modes |

**petalTongue's Role**: Render appropriate interface  
**NestGate's Role**: Store/load user preferences  
**BearDog's Role**: Authenticate users  
**Squirrel's Role**: AI agent intelligence

---

## 3. Architecture Specification

### 3.1 Core Components

#### 3.1.1 Universe Detector

**Responsibility**: Detect computational environment capabilities

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
    
    async fn detect_compute() -> Result<ComputeCapability> {
        // Try to discover ToadStool for fractal compute
        if let Ok(toadstool) = ToadStoolClient::discover().await {
            return Ok(ComputeCapability::Fractal { client: toadstool });
        }
        
        // Fall back to local GPU or CPU
        if Self::has_gpu() {
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
```

**Leverages**:
- ✅ ToadStool (if available) for compute capability detection
- ✅ Songbird (if available) for network capability detection

#### 3.1.2 User Detector

**Responsibility**: Detect user type and capabilities

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
    
    async fn load_preferences() -> Result<UserPreferences> {
        // Try to load from NestGate
        if let Ok(nestgate) = NestGateClient::discover().await {
            return nestgate.load_preferences("petaltongue").await;
        }
        
        // Fall back to defaults
        Ok(UserPreferences::default())
    }
}

pub struct User {
    pub user_type: UserType,
    pub abilities: UserAbilities,
    pub preferences: UserPreferences,
    pub context: UserContext,
}
```

**Leverages**:
- ✅ NestGate (if available) for preference persistence
- ✅ BearDog (if available) for user authentication

#### 3.1.3 Interface Selector

**Responsibility**: Select optimal interface(s) for (Universe, User) pair

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
                // Leverage Squirrel for AI context if available
                if let Ok(squirrel) = SquirrelClient::discover().await {
                    interfaces.push(Interface::AIEnhancedAPI { squirrel });
                }
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
}
```

**Leverages**:
- ✅ Squirrel (if available) for AI-enhanced interfaces

#### 3.1.4 Interface Implementations

**petalTongue Owns These**:

```rust
pub enum Interface {
    // Visual Interfaces (petalTongue renders)
    EguiGUI,           // Native desktop GUI
    RichTUI,           // Enhanced terminal UI (ratatui)
    TerminalGUI,       // Basic terminal UI
    WebGUI,            // Browser-based GUI
    
    // Audio Interfaces (petalTongue renders)
    AudioGUI,          // Screen reader compatible
    AudioSonification, // Pure audio representation
    AudioOnlyGUI,      // Voice-controlled
    
    // API Interfaces (petalTongue provides)
    JSONAPI,           // JSON-RPC 2.0
    GraphQLAPI,        // Rich queries
    RestAPI,           // HTTP fallback
    
    // Non-Human Interfaces (petalTongue translates)
    DolphinAcoustic,   // Click patterns
    FungalChemical,    // Chemical gradients
    CustomProtocol(String),
    
    // Hybrid Interfaces (petalTongue coordinates)
    HybridCollaborative {
        human_view: Box<Vec<Interface>>,
        ai_view: Box<Vec<Interface>>,
    },
}
```

**Each Interface Leverages Other Primals**:

```rust
impl EguiGUI {
    async fn render(&mut self) -> Result<()> {
        // Use ToadStool for GPU acceleration if available
        if let Some(toadstool) = self.toadstool_client.as_ref() {
            self.renderer.set_compute_backend(toadstool);
        }
        
        // Load preferences from NestGate if available
        if let Some(nestgate) = self.nestgate_client.as_ref() {
            self.settings = nestgate.load_ui_settings().await?;
        }
        
        // Render UI (petalTongue's job)
        self.render_frame()?;
        
        Ok(())
    }
}

impl RichTUI {
    async fn render(&mut self) -> Result<()> {
        // Discover topology via discovery service if available
        if let Some(discovery_client) = self.discovery_client.as_ref() {
            self.topology = discovery_client.get_topology().await?;
        }
        
        // Render TUI (petalTongue's job)
        self.render_frame()?;
        
        Ok(())
    }
}
```

---

## 4. Primal Division of Labor

### 4.1 petalTongue's Domain (Rendering & Translation)

**What petalTongue Does**:
- ✅ Render visual interfaces (GUI, TUI, Web)
- ✅ Render audio interfaces (Soundscape, Voice)
- ✅ Provide API interfaces (JSON-RPC, GraphQL, REST)
- ✅ Detect universe capabilities
- ✅ Detect user capabilities
- ✅ Select optimal interface(s)
- ✅ Translate between modalities
- ✅ Coordinate multi-modal rendering

**What petalTongue Does NOT Do**:
- ❌ Compute acceleration (ToadStool's job)
- ❌ Service discovery (Songbird's job)
- ❌ Data persistence (NestGate's job)
- ❌ User authentication (BearDog's job)
- ❌ AI reasoning (Squirrel's job)

### 4.2 ToadStool's Domain (Compute)

**What ToadStool Provides to petalTongue**:
- GPU acceleration for rendering
- Fractal compute for complex visualizations
- Resource metrics

**How petalTongue Uses ToadStool**:
```rust
// Optional, graceful degradation if unavailable
if let Ok(toadstool) = ToadStoolClient::discover().await {
    renderer.set_gpu_backend(toadstool);
} else {
    renderer.use_cpu_fallback();
}
```

### 4.3 Songbird's Domain (Discovery & Communication)

**What Songbird Provides to petalTongue**:
- Primal discovery
- Topology data
- Real-time event streaming
- Network capability detection

**How petalTongue Uses Songbird**:
```rust
// Optional, graceful degradation if unavailable
if let Ok(discovery_client) = DiscoveryServiceClient::discover().await {
    let topology = discovery_client.get_topology().await?;
    ui.render_topology(topology)?;
} else {
    ui.render_standalone_mode()?;
}
```

### 4.4 NestGate's Domain (Persistence)

**What NestGate Provides to petalTongue**:
- User preferences storage
- UI settings persistence
- Session state saving
- Configuration templates

**How petalTongue Uses NestGate**:
```rust
// Optional, graceful degradation if unavailable
if let Ok(nestgate) = NestGateClient::discover().await {
    let prefs = nestgate.load_preferences("petaltongue").await?;
    ui.apply_preferences(prefs)?;
} else {
    ui.use_defaults()?;
}
```

### 4.5 BearDog's Domain (Security & Authentication)

**What BearDog Provides to petalTongue**:
- User authentication
- Authorization checks
- Encrypted configuration
- Audit logging

**How petalTongue Uses BearDog**:
```rust
// Optional, but recommended for multi-user scenarios
if let Ok(beardog) = BearDogClient::discover().await {
    let user = beardog.authenticate(credentials).await?;
    ui.show_user_specific_view(user)?;
} else {
    ui.show_unrestricted_view()?;
}
```

### 4.6 Squirrel's Domain (AI Assistance)

**What Squirrel Provides to petalTongue**:
- AI-powered suggestions
- Natural language queries
- Context-aware help
- Intelligent defaults

**How petalTongue Uses Squirrel**:
```rust
// Optional, enhances UX if available
if let Ok(squirrel) = SquirrelClient::discover().await {
    let suggestion = squirrel.suggest_next_action(context).await?;
    ui.show_suggestion(suggestion)?;
}
```

---

## 5. Rich TUI Specification (Immediate Priority)

### 5.1 Purpose

Provide a pure Rust, terminal-based UI for biomeOS management:
- neuralAPI (graph orchestration)
- NUCLEUS (secure discovery)
- liveSpore (live deployments)

### 5.2 Technology Stack

**Core**:
- `ratatui` (TUI framework)
- `crossterm` (terminal control)
- `tokio` (async runtime)

**Integration**:
- `petal-tongue-discovery` (primal discovery)
- `petal-tongue-ipc` (JSON-RPC communication)

### 5.3 Eight Interactive Views

#### 5.3.1 Dashboard View

**Purpose**: System overview

**Data Sources**:
- Songbird: Active primals, topology summary
- ToadStool: Resource metrics (optional)
- NUCLEUS: Security status

**Rendering**: petalTongue (ASCII art, tables, charts)

#### 5.3.2 Topology View

**Purpose**: Visual graph of primal connections

**Data Sources**:
- Songbird: Topology graph
- NUCLEUS: Trust relationships

**Rendering**: petalTongue (ASCII art graph layout)

**Leverages**:
- ToadStool (if available): Force-directed layout computation

#### 5.3.3 Devices View

**Purpose**: Device management

**Data Sources**:
- Songbird: Device discovery
- BearDog: Device authorization (optional)

**Rendering**: petalTongue (device tree, drag-and-drop simulation)

#### 5.3.4 Primals View

**Purpose**: Primal status and health

**Data Sources**:
- Songbird: Primal registry
- Individual primals: Health endpoints

**Rendering**: petalTongue (status cards, metrics)

#### 5.3.5 Logs View

**Purpose**: Real-time log streaming

**Data Sources**:
- Songbird: Event stream
- Individual primals: Log endpoints

**Rendering**: petalTongue (scrolling log panel)

#### 5.3.6 neuralAPI View

**Purpose**: Graph orchestration management

**Data Sources**:
- biomeOS: Graph definitions
- Songbird: Execution status

**Rendering**: petalTongue (graph editor, execution timeline)

#### 5.3.7 NUCLEUS View

**Purpose**: Secure discovery management

**Data Sources**:
- NUCLEUS: Discovery layers, trust scores
- BearDog: Security policies

**Rendering**: petalTongue (trust matrix, discovery timeline)

#### 5.3.8 LiveSpore View

**Purpose**: Live deployment management

**Data Sources**:
- liveSpore: Deployment status
- Songbird: Node availability

**Rendering**: petalTongue (deployment pipeline, node map)

### 5.4 Graceful Degradation

**If Songbird unavailable**:
- Show "Standalone Mode" message
- Render local-only information

**If ToadStool unavailable**:
- Use CPU for layout computation
- May be slower, but still functional

**If NestGate unavailable**:
- Use default settings
- No persistence between sessions

**If BearDog unavailable**:
- Show unrestricted view
- Warning about no authentication

---

## 6. Implementation Phases

### 6.1 Phase 1: Foundation ✅ COMPLETE

**Deliverables**:
- [x] Vision document
- [x] Architecture specification (this document)
- [x] TUI crate structure
- [x] Cargo.toml with dependencies

**Status**: Complete

### 6.2 Phase 2: Core Modules (Week 1)

**Deliverables**:
- [ ] `state.rs` - TUI state management
- [ ] `app.rs` - Main TUI application
- [ ] `events.rs` - Event handling
- [ ] `layout.rs` - Layout management
- [ ] Add to workspace
- [ ] Test compilation

**Leverages**: None (pure petalTongue code)

### 6.3 Phase 3: Basic Views (Week 2)

**Deliverables**:
- [ ] Dashboard view (overview)
- [ ] Topology view (ASCII graph)
- [ ] Logs view (scrolling logs)

**Leverages**:
- Songbird: Topology data, event stream
- ToadStool (optional): Layout computation

### 6.4 Phase 4: Management Views (Week 3)

**Deliverables**:
- [ ] Devices view
- [ ] Primals view
- [ ] neuralAPI view
- [ ] NUCLEUS view
- [ ] LiveSpore view

**Leverages**:
- Songbird: Discovery data
- BearDog (optional): Authorization
- NestGate (optional): Preferences

### 6.5 Phase 5: Real-Time Integration (Week 4)

**Deliverables**:
- [ ] WebSocket client for live updates
- [ ] JSON-RPC command execution
- [ ] Event streaming
- [ ] Auto-refresh

**Leverages**:
- Songbird: Real-time event stream

### 6.6 Phase 6: Polish & Production (Week 5)

**Deliverables**:
- [ ] Keyboard shortcuts
- [ ] Mouse support (optional)
- [ ] Error handling
- [ ] Loading states
- [ ] Help system
- [ ] Testing
- [ ] Documentation

**Leverages**:
- Squirrel (optional): Context-aware help

---

## 7. Success Criteria

### 7.1 Functional Requirements

- ✅ Runs in any terminal (SSH, serial, local)
- ✅ Works with 0 other primals (standalone mode)
- ✅ Leverages other primals when available (enhanced mode)
- ✅ Real-time updates (<100ms latency)
- ✅ Keyboard navigation (100% keyboard accessible)
- ✅ Mouse support (optional, if terminal supports it)
- ✅ Graceful degradation (always shows something useful)

### 7.2 Quality Requirements

- ✅ Zero unsafe code
- ✅ Pure Rust
- ✅ Zero required external dependencies
- ✅ Comprehensive error handling
- ✅ Extensive testing (unit, integration, E2E)
- ✅ Clear documentation

### 7.3 TRUE PRIMAL Requirements

- ✅ Zero hardcoding (runtime discovery)
- ✅ Capability-based (adapts to available primals)
- ✅ Self-knowledge (knows own domain: rendering)
- ✅ Agnostic (no assumptions about other primals)
- ✅ Graceful degradation (works alone or with others)

---

## 8. Primal Coordination Protocol

### 8.1 Discovery Pattern

**All primal clients are optional**:

```rust
pub struct TUIClients {
    songbird: Option<DiscoveryServiceClient>,
    toadstool: Option<ToadStoolClient>,
    nestgate: Option<NestGateClient>,
    beardog: Option<BearDogClient>,
    squirrel: Option<SquirrelClient>,
}

impl TUIClients {
    /// Discover all available primals
    pub async fn discover() -> Self {
        Self {
            songbird: DiscoveryServiceClient::discover().await.ok(),
            toadstool: ToadStoolClient::discover().await.ok(),
            nestgate: NestGateClient::discover().await.ok(),
            beardog: BearDogClient::discover().await.ok(),
            squirrel: SquirrelClient::discover().await.ok(),
        }
    }
}
```

### 8.2 Capability Check Pattern

**Always check before using**:

```rust
impl RichTUI {
    async fn render_topology(&mut self) -> Result<()> {
        // Try to get topology from Songbird
        let topology = if let Some(songbird) = &self.clients.songbird {
            songbird.get_topology().await.ok()
        } else {
            None
        };
        
        // Render what we have
        match topology {
            Some(topo) => self.render_full_topology(topo)?,
            None => self.render_standalone_message()?,
        }
        
        Ok(())
    }
}
```

### 8.3 Graceful Degradation Pattern

**Always provide fallback**:

```rust
impl RichTUI {
    async fn accelerate_layout(&mut self, graph: &Graph) -> Result<Layout> {
        // Try ToadStool for GPU acceleration
        if let Some(toadstool) = &self.clients.toadstool {
            if let Ok(layout) = toadstool.compute_layout(graph).await {
                return Ok(layout);
            }
        }
        
        // Fall back to CPU computation
        Ok(self.compute_layout_cpu(graph)?)
    }
}
```

---

## 9. Non-Functional Requirements

### 9.1 Performance

- UI updates: <16ms (60 FPS)
- Network requests: <100ms timeout
- Startup time: <1 second
- Memory usage: <50MB

### 9.2 Scalability

- Support 100+ nodes in topology view
- Support 1000+ log lines in logs view
- Support 10+ simultaneous primals

### 9.3 Reliability

- No panics (all errors handled gracefully)
- No data loss (auto-save every 30 seconds to NestGate if available)
- Recovery from network failures
- Resilient to primal crashes

### 9.4 Usability

- Intuitive keyboard shortcuts
- Consistent navigation across views
- Clear visual feedback
- Helpful error messages
- Context-sensitive help

---

## 10. Testing Strategy

### 10.1 Unit Tests

- All state management functions
- All layout algorithms
- All rendering functions
- All client interactions (mocked)

### 10.2 Integration Tests

- TUI with Songbird (real connection)
- TUI with ToadStool (real connection)
- TUI with NestGate (real connection)
- TUI standalone (no primals)

### 10.3 E2E Tests

- Full user workflows
- Multi-view navigation
- Real-time updates
- Error scenarios

### 10.4 Chaos Tests

- Random primal disconnections
- Network failures
- Slow responses
- Malformed data

---

## 11. Documentation Requirements

### 11.1 User Documentation

- Quick start guide
- Keyboard shortcuts reference
- View-by-view user guide
- Troubleshooting guide

### 11.2 Developer Documentation

- Architecture overview
- API reference
- Extension guide (adding new views)
- Testing guide

### 11.3 Integration Documentation

- How to integrate with Songbird
- How to integrate with ToadStool
- How to integrate with NestGate
- How to integrate with BearDog
- How to integrate with Squirrel

---

## 12. Future Enhancements

### 12.1 Phase 7: Universe Detection (Future)

- Automatic substrate detection
- Automatic capability detection
- Dynamic interface selection

### 12.2 Phase 8: User Detection (Future)

- User type detection
- Accessibility detection
- Preference learning

### 12.3 Phase 9: AI Agent API (Future)

- JSON-RPC API for AI agents
- GraphQL API for flexible queries
- Context-aware responses

### 12.4 Phase 10: Non-Human Interfaces (Research)

- Dolphin acoustic interface
- Fungal chemical interface
- Custom protocol framework

---

## 13. Appendix A: Primal Capability Matrix

| Capability | petalTongue | ToadStool | Songbird | NestGate | BearDog | Squirrel |
|------------|-------------|-----------|----------|----------|---------|----------|
| **Rendering** | ✅ Owner | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Compute** | ❌ | ✅ Owner | ❌ | ❌ | ❌ | ❌ |
| **Discovery** | ❌ | ❌ | ✅ Owner | ❌ | ❌ | ❌ |
| **Persistence** | ❌ | ❌ | ❌ | ✅ Owner | ❌ | ❌ |
| **Security** | ❌ | ❌ | ❌ | ❌ | ✅ Owner | ❌ |
| **AI** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Owner |

**Key Principle**: Each primal owns its domain, leverages others' capabilities.

---

## 14. Appendix B: JSON-RPC API Specification

### 14.1 petalTongue → Songbird

```json
// Get topology
{
  "jsonrpc": "2.0",
  "method": "songbird.get_topology",
  "params": {},
  "id": 1
}

// Subscribe to events
{
  "jsonrpc": "2.0",
  "method": "songbird.subscribe_events",
  "params": {
    "event_types": ["primal_status", "topology_change"]
  },
  "id": 2
}
```

### 14.2 petalTongue → ToadStool

```json
// Request GPU acceleration
{
  "jsonrpc": "2.0",
  "method": "toadstool.compute_layout",
  "params": {
    "graph": { /* graph data */ },
    "algorithm": "force_directed"
  },
  "id": 3
}
```

### 14.3 petalTongue → NestGate

```json
// Load preferences
{
  "jsonrpc": "2.0",
  "method": "nestgate.load_preferences",
  "params": {
    "primal": "petaltongue",
    "user_id": "optional"
  },
  "id": 4
}

// Save preferences
{
  "jsonrpc": "2.0",
  "method": "nestgate.save_preferences",
  "params": {
    "primal": "petaltongue",
    "preferences": { /* settings */ }
  },
  "id": 5
}
```

---

**Status**: Formal specification complete, ready for implementation! 🌸

**petalTongue**: The rendering layer. Leverages others' capabilities. 🚀

