# 🎨 Panel System Evolution Specification

**Date**: January 15, 2026  
**Status**: Draft (will evolve as we implement Doom)  
**Driven By**: Real use cases (Doom first, then web, video, etc.)

---

## 🎯 Philosophy

This spec will EVOLVE as we implement Doom. We're not designing everything upfront - we're letting real implementation expose what we need.

**Current**: Documented gaps we expect to hit  
**Future**: Solutions we discover along the way

---

## 📐 Core Panel Abstraction

### **Current State** (v2.3.0):
```rust
// We have: Ad-hoc rendering in app.rs
// Problem: No clear panel abstraction
```

### **Target State** (v2.4.0+):
```rust
/// A panel is any embeddable UI component
pub trait Panel: Send + Sync {
    /// Unique identifier
    fn id(&self) -> &str;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// Panel lifecycle
    fn lifecycle(&mut self) -> &mut dyn PanelLifecycle;
    
    /// Render to egui
    fn render(&mut self, ui: &mut egui::Ui);
    
    /// Handle input (if focused)
    fn handle_input(&mut self, input: &InputEvent) -> InputResponse;
    
    /// Resource requirements
    fn resources(&self) -> ResourceRequirements;
    
    /// Serializable state
    fn save_state(&self) -> Result<Vec<u8>>;
    fn load_state(&mut self, data: &[u8]) -> Result<()>;
}
```

---

## 🔄 Panel Lifecycle

### **Discovery** (from Doom implementation):

Doom needs:
- `init()` - Load WAD, initialize engine
- `update(dt)` - Game tick (35 Hz)
- `render()` - Display (60 Hz)
- `pause()` - User tabs away
- `resume()` - User tabs back
- `cleanup()` - Free resources

### **Design**:
```rust
pub trait PanelLifecycle {
    /// Called once when panel is created
    fn init(&mut self) -> Result<()>;
    
    /// Called every frame (variable dt)
    fn update(&mut self, dt: Duration) -> Result<()>;
    
    /// Panel is being paused (backgrounded)
    fn pause(&mut self);
    
    /// Panel is being resumed (foregrounded)
    fn resume(&mut self);
    
    /// Panel is being destroyed
    fn cleanup(&mut self);
    
    /// Current lifecycle state
    fn state(&self) -> LifecycleState;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Uninitialized,
    Initializing,
    Running,
    Paused,
    Stopped,
    Error,
}
```

---

## 🎮 Input System

### **Discovery** (from Doom implementation):

Problem: Doom + Graph both want keyboard  
Solution: Focus-based routing

### **Design**:
```rust
pub struct InputRouter {
    /// Currently focused panel
    focused: Option<PanelId>,
    
    /// Modal stack (modals get priority)
    modals: Vec<PanelId>,
    
    /// Input event queue
    queue: VecDeque<InputEvent>,
}

impl InputRouter {
    /// Route input to appropriate panel
    pub fn route(&mut self, event: InputEvent) -> Option<PanelId> {
        // 1. Check modal stack first
        if let Some(modal) = self.modals.last() {
            return Some(*modal);
        }
        
        // 2. Check focused panel
        if let Some(focused) = self.focused {
            return Some(focused);
        }
        
        // 3. Broadcast (for things like global hotkeys)
        None
    }
    
    /// Request focus (panel wants input)
    pub fn request_focus(&mut self, panel: PanelId) {
        self.focused = Some(panel);
    }
    
    /// Push modal (gets all input until popped)
    pub fn push_modal(&mut self, panel: PanelId) {
        self.modals.push(panel);
    }
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Keyboard { key: KeyCode, pressed: bool, modifiers: Modifiers },
    Mouse { button: MouseButton, pressed: bool, position: Vec2 },
    MouseMove { delta: Vec2, position: Vec2 },
    Scroll { delta: Vec2 },
    Touch { phase: TouchPhase, position: Vec2, id: u64 },
    GameController { button: ControllerButton, value: f32 },
}

#[derive(Debug, Clone, Copy)]
pub enum InputResponse {
    Handled,      // Consumed, don't propagate
    Ignored,      // Didn't use, pass to next panel
    RequestFocus, // Want to be focused
}
```

---

## ⚡ Performance & Resource Management

### **Discovery** (from Doom implementation):

Problem: Doom wants 70% GPU, Graph wants 30%  
Solution: Explicit budgets

### **Design**:
```rust
pub struct ResourceCoordinator {
    /// GPU time budget per frame (100% = 16.67ms at 60 FPS)
    gpu_budget: ResourceBudget<f32>,
    
    /// Audio channels (finite resource)
    audio_channels: ResourceBudget<usize>,
    
    /// Memory budget (soft limit)
    memory_budget: ResourceBudget<usize>,
    
    /// Panel allocations
    allocations: HashMap<PanelId, ResourceAllocation>,
}

pub struct ResourceBudget<T> {
    total: T,
    allocated: T,
    requests: Vec<(PanelId, T)>,
}

impl ResourceCoordinator {
    /// Request GPU time budget
    pub fn request_gpu(&mut self, panel: PanelId, percent: f32) -> Result<()> {
        if self.gpu_budget.can_allocate(percent) {
            self.gpu_budget.allocate(panel, percent);
            Ok(())
        } else {
            Err(Error::ResourceExhausted)
        }
    }
    
    /// Measure actual usage vs budget
    pub fn measure(&mut self, panel: PanelId, used: Duration) {
        let budget = self.allocations[&panel].gpu_time;
        if used > budget {
            tracing::warn!(
                "Panel {panel:?} exceeded budget: {used:?} > {budget:?}"
            );
        }
    }
}
```

---

## 🎵 Audio Mixing

### **Discovery** (from Doom implementation):

Problem: Doom produces PCM, Graph does sonification  
Solution: Multi-channel mixer

### **Design**:
```rust
pub struct AudioMixer {
    /// Sample rate (44100 Hz typical)
    sample_rate: u32,
    
    /// Active channels
    channels: Vec<AudioChannel>,
    
    /// Output buffer
    output: Vec<f32>,
    
    /// Master volume
    master_volume: f32,
}

pub struct AudioChannel {
    id: ChannelId,
    owner: PanelId,
    source: Box<dyn AudioSource>,
    volume: f32,
    priority: u8,
    state: ChannelState,
}

pub trait AudioSource: Send {
    /// Fill buffer with samples
    fn fill(&mut self, buffer: &mut [f32]);
    
    /// Is this source finished?
    fn is_finished(&self) -> bool;
}

impl AudioMixer {
    /// Create a channel for a panel
    pub fn create_channel(&mut self, panel: PanelId, priority: u8) -> ChannelId {
        let channel = AudioChannel {
            id: ChannelId::new(),
            owner: panel,
            source: Box::new(SilenceSource),
            volume: 1.0,
            priority,
            state: ChannelState::Stopped,
        };
        self.channels.push(channel);
        channel.id
    }
    
    /// Mix all channels
    pub fn mix(&mut self, output: &mut [f32]) {
        // Clear output
        output.fill(0.0);
        
        // Mix each channel
        for channel in &mut self.channels {
            if channel.state == ChannelState::Playing {
                let mut temp = vec![0.0; output.len()];
                channel.source.fill(&mut temp);
                
                // Add to output with volume
                for (i, sample) in temp.iter().enumerate() {
                    output[i] += sample * channel.volume * self.master_volume;
                }
            }
        }
        
        // Clipping protection
        for sample in output.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}
```

---

## 📦 Asset Management

### **Discovery** (from Doom implementation):

Problem: Loading 2.5 MB WAD blocks UI  
Solution: Async asset loading with cache

### **Design**:
```rust
pub struct AssetManager {
    /// LRU cache of loaded assets
    cache: LruCache<AssetId, Arc<dyn Asset>>,
    
    /// Background loader
    loader: AssetLoader,
    
    /// Load queue
    queue: VecDeque<AssetRequest>,
}

pub trait Asset: Send + Sync {
    fn size(&self) -> usize;
    fn asset_type(&self) -> &str;
}

impl AssetManager {
    /// Load asset (async, returns immediately with handle)
    pub async fn load<T: Asset>(&self, path: &str) -> AssetHandle<T> {
        // Check cache
        if let Some(asset) = self.cache.get(&path.into()) {
            return AssetHandle::ready(asset.clone());
        }
        
        // Queue for loading
        let (tx, rx) = oneshot::channel();
        self.queue.push_back(AssetRequest {
            path: path.to_string(),
            sender: tx,
        });
        
        AssetHandle::loading(rx)
    }
    
    /// Preload assets (for faster panel startup)
    pub fn preload(&mut self, paths: &[&str]) {
        for path in paths {
            self.load::<GenericAsset>(path);
        }
    }
}

pub enum AssetHandle<T> {
    Loading(Receiver<Arc<T>>),
    Ready(Arc<T>),
    Failed(Error),
}
```

---

## 🐛 Error Boundaries

### **Discovery** (from Doom implementation):

Problem: Doom crash shouldn't kill entire app  
Solution: Panel sandboxing

### **Design**:
```rust
pub struct PanelSandbox {
    panel: Box<dyn Panel>,
    error_state: Option<PanelError>,
    recovery_strategy: RecoveryStrategy,
}

impl PanelSandbox {
    pub fn update(&mut self, dt: Duration) {
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.panel.lifecycle().update(dt)
        }));
        
        match result {
            Ok(Ok(())) => {
                // Success, clear any previous error
                self.error_state = None;
            }
            Ok(Err(e)) => {
                // Panel returned error
                self.handle_error(PanelError::Runtime(e));
            }
            Err(panic) => {
                // Panel panicked
                self.handle_error(PanelError::Panic(format!("{:?}", panic)));
            }
        }
    }
    
    fn handle_error(&mut self, error: PanelError) {
        tracing::error!("Panel error: {:?}", error);
        self.error_state = Some(error);
        
        match self.recovery_strategy {
            RecoveryStrategy::Restart => self.restart(),
            RecoveryStrategy::ShowError => { /* Display error UI */ }
            RecoveryStrategy::Remove => { /* Remove panel */ }
        }
    }
}

#[derive(Debug)]
pub enum PanelError {
    Runtime(Error),
    Panic(String),
}

pub enum RecoveryStrategy {
    Restart,    // Try to restart panel
    ShowError,  // Display error, keep panel
    Remove,     // Remove panel from UI
}
```

---

## 📊 Panel Registry & Discovery

### **Design** (anticipating multiple panel types):

```rust
pub struct PanelRegistry {
    /// Registered panel types
    factories: HashMap<String, Box<dyn PanelFactory>>,
    
    /// Active panels
    panels: HashMap<PanelId, PanelSandbox>,
}

pub trait PanelFactory: Send + Sync {
    /// Create a new instance of this panel type
    fn create(&self, config: &PanelConfig) -> Result<Box<dyn Panel>>;
    
    /// Panel type identifier
    fn panel_type(&self) -> &str;
    
    /// Required capabilities
    fn required_capabilities(&self) -> Vec<String>;
}

impl PanelRegistry {
    /// Register a panel type
    pub fn register<F>(&mut self, factory: F)
    where
        F: PanelFactory + 'static
    {
        self.factories.insert(
            factory.panel_type().to_string(),
            Box::new(factory),
        );
    }
    
    /// Create panel from scenario config
    pub fn create_from_config(&mut self, config: &PanelConfig) -> Result<PanelId> {
        let factory = self.factories.get(&config.panel_type)
            .ok_or(Error::UnknownPanelType)?;
        
        let panel = factory.create(config)?;
        let id = PanelId::new();
        
        self.panels.insert(id, PanelSandbox::new(panel));
        Ok(id)
    }
}

// Example registration:
registry.register(DoomPanelFactory);
registry.register(GraphPanelFactory);
registry.register(WebPanelFactory);
```

---

## 🎯 Integration with Scenarios

### **Extended Scenario Schema**:

```json
{
  "version": "2.0.0",
  "ui_config": {
    "panels": [
      {
        "id": "doom_game",
        "type": "doom",
        "config": {
          "wad": "doom1.wad",
          "resolution": [640, 480],
          "audio": true
        },
        "resources": {
          "gpu_percent": 70,
          "audio_channels": 8,
          "memory_mb": 64
        },
        "lifecycle": {
          "init_timeout": "5s",
          "pause_on_unfocus": true
        }
      },
      {
        "id": "primal_graph",
        "type": "graph_viz",
        "resources": {
          "gpu_percent": 30
        }
      }
    ],
    "layout": {
      "type": "split-horizontal",
      "splits": [
        { "panel": "doom_game", "size": "70%" },
        { "panel": "primal_graph", "size": "30%" }
      ]
    }
  }
}
```

---

## 📝 Evolution Tracking

As we implement, we'll document:

- **Gaps Discovered**: What we hit that we didn't expect
- **Solutions Implemented**: How we solved each gap
- **Lessons Learned**: What we'd do differently next time
- **API Changes**: How the panel API evolved

---

**Status**: Living document (will evolve with implementation)  
**Next Update**: After Day 1 of Doom implementation  
**Tracking**: DOOM_AS_EVOLUTION_TEST.md for detailed progress

🌸 **Let the implementation teach us!** 🚀

