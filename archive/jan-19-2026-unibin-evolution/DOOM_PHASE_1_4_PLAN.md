# 🎮📊 Doom Phase 1.4: Live Stats with biomeOS Integration

**Status**: 🚧 In Progress  
**Started**: January 15, 2026 (Evening)  
**Goal**: Validate panel system with live biomeOS stats while playing Doom

---

## 🎯 **Objective**

Make Doom playable with **live biomeOS stats in the background**, validating:
1. ✅ Panel system architecture (from deep debt evolution)
2. ✅ Multi-panel coordination (game + stats)
3. ✅ Neural API integration (proprioception, metrics)
4. ✅ Real-time updates without blocking gameplay
5. ✅ petalTongue as composition layer

**User Quote**: "once its playable with live stats in the background, we can advance to the next stage"

---

## 📋 **What We'll Build**

### **1. Proprioception Overlay Panel** ⭐ Priority 1

**Display SAME DAVE self-awareness while playing:**

```
╔═══════════════════════════════════════════════════════════════╗
║ 🧠 NUCLEUS Proprioception              Health: 100% 💚       ║
╠═══════════════════════════════════════════════════════════════╣
║ Sensory:   3 active sockets detected                         ║
║ Awareness: Knows about 3 primals                             ║
║ Motor:     Can deploy, coordinate, execute                   ║
║ Confidence: 100%                                              ║
╠═══════════════════════════════════════════════════════════════╣
║ Core Systems:                                                 ║
║ ✅ Security (BearDog)   ✅ Discovery (Songbird)              ║
║ ✅ Compute (Toadstool)                                        ║
╚═══════════════════════════════════════════════════════════════╝
```

**API**: `neural_api.get_proprioception()`

**Data**:
```json
{
  "health": { "percentage": 100.0, "status": "healthy" },
  "confidence": 100.0,
  "self_awareness": {
    "knows_about": 3,
    "can_coordinate": true,
    "has_security": true,
    "has_discovery": true,
    "has_compute": true
  },
  "motor": {
    "can_deploy": true,
    "can_execute_graphs": true,
    "can_coordinate_primals": true
  },
  "sensory": {
    "active_sockets": 3,
    "last_scan": "2026-01-15T22:00:00Z"
  }
}
```

### **2. System Metrics Dashboard** ⭐ Priority 2

**Real-time system metrics:**

```
╔═══════════════════════════════════════╗
║ 📊 System Metrics                    ║
╠═══════════════════════════════════════╣
║ CPU:    [████████░░] 16.5%          ║
║ Memory: [██████████] 66.7%          ║
║ Uptime: 1d 2h 34m                   ║
║                                      ║
║ Active Primals: 3                   ║
║ Available Graphs: 5                 ║
║ Active Executions: 0                ║
╚═══════════════════════════════════════╝
```

**API**: `neural_api.get_metrics()`

**Data**:
```json
{
  "system": {
    "cpu_percent": 16.5,
    "memory_used_mb": 32768,
    "memory_total_mb": 49152,
    "memory_percent": 66.7,
    "uptime_seconds": 86400
  },
  "neural_api": {
    "family_id": "nat0",
    "active_primals": 3,
    "graphs_available": 5,
    "active_executions": 0
  }
}
```

### **3. Mini Topology View** ⭐ Priority 3

**Compact primal network view:**

```
╔═══════════════════════════════════════╗
║ 🌐 Biome Topology                    ║
╠═══════════════════════════════════════╣
║                                      ║
║   🔒 BearDog ──┐                    ║
║   🎵 Songbird ─┼─→ Coordinator      ║
║   ⚡ Toadstool ┘                     ║
║                                      ║
║   Status: All systems nominal        ║
║                                      ║
╚═══════════════════════════════════════╝
```

**API**: `neural_api.get_primals()` + topology data

### **4. Doom Game Stats** ⭐ Priority 4

**In-game statistics:**

```
╔═══════════════════════════════════════╗
║ 🎮 Doom Stats                        ║
╠═══════════════════════════════════════╣
║ Map: E1M1 (Hangar)                   ║
║ View: First-Person (3D)              ║
║ Position: (1056, 2304)               ║
║ Angle: 90°                           ║
║                                      ║
║ FPS: 60                              ║
║ Frame Time: 16ms                     ║
╚═══════════════════════════════════════╝
```

**Source**: `DoomInstance` internal state

---

## 🏗️ **Architecture**

### **Panel Layout**

```
╔══════════════════════════════════════════════════════════════════╗
║                         petalTongue                              ║
╠══════════════════════════════════════════════════════════════════╣
║                                                                  ║
║  ┌─────────────────────────────────────────┐  ┌───────────────┐ ║
║  │                                         │  │ 🧠 Proprio    │ ║
║  │          DOOM GAME PANEL                │  │   Health: ✅  │ ║
║  │         (First-Person View)             │  │   Conf: 100%  │ ║
║  │                                         │  ├───────────────┤ ║
║  │                                         │  │ 📊 Metrics    │ ║
║  │   [Your 3D Doom gameplay here]          │  │   CPU: 16.5%  │ ║
║  │                                         │  │   Mem: 66.7%  │ ║
║  │                                         │  ├───────────────┤ ║
║  │                                         │  │ 🌐 Topology   │ ║
║  │                                         │  │   3 primals   │ ║
║  │                                         │  ├───────────────┤ ║
║  │                                         │  │ 🎮 Game Stats │ ║
║  │                                         │  │   E1M1        │ ║
║  │                                         │  │   FPS: 60     │ ║
║  └─────────────────────────────────────────┘  └───────────────┘ ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
```

### **Update Strategy**

- **Doom Panel**: 60 FPS (16ms target)
- **Proprioception**: 5 second polling
- **Metrics**: 1 second updates
- **Topology**: 5 second polling
- **Game Stats**: Every frame (from DoomInstance)

**Key**: Game rendering must not be blocked by stats updates!

---

## 🛠️ **Implementation Plan**

### **Phase A: Doom Stats Panel** (30 min)

Simple internal stats from DoomInstance:

**File**: `crates/doom-core/src/stats_panel.rs`

```rust
pub struct DoomStatsPanel {
    doom: Arc<RwLock<DoomInstance>>,
}

impl PanelInstance for DoomStatsPanel {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let doom = self.doom.read().unwrap();
        
        ui.heading("🎮 Doom Stats");
        ui.separator();
        
        ui.label(format!("Map: E1M1 (Hangar)"));
        ui.label(format!("View: {:?}", doom.current_view_mode));
        ui.label(format!("Position: ({:.0}, {:.0})", doom.player_x, doom.player_y));
        ui.label(format!("Angle: {:.0}°", doom.player_angle.to_degrees()));
        ui.separator();
        ui.label(format!("Frame: {}", doom.frame_count));
    }
}
```

### **Phase B: Metrics Panel** (45 min)

Use existing `NeuralApiProvider`:

**File**: `crates/petal-tongue-ui/src/panels/metrics_panel.rs`

```rust
use petal_tongue_discovery::NeuralApiProvider;

pub struct MetricsPanel {
    provider: Option<NeuralApiProvider>,
    last_metrics: Option<Metrics>,
    last_update: Instant,
    update_interval: Duration,
}

impl MetricsPanel {
    pub fn new() -> Self {
        Self {
            provider: None,
            last_metrics: None,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1),
        }
    }
    
    async fn refresh_metrics(&mut self) {
        if let Some(provider) = &self.provider {
            if let Ok(metrics) = provider.get_metrics().await {
                self.last_metrics = Some(metrics);
                self.last_update = Instant::now();
            }
        }
    }
}

impl PanelInstance for MetricsPanel {
    fn init(&mut self) -> Result<(), PanelError> {
        // Discover Neural API
        let provider = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                NeuralApiProvider::discover(None).await.ok()
            })
        });
        
        self.provider = provider;
        Ok(())
    }
    
    fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Trigger async refresh if needed
        if self.last_update.elapsed() > self.update_interval {
            let provider = self.provider.clone();
            tokio::spawn(async move {
                // Refresh in background
            });
        }
        
        // Render current data
        if let Some(metrics) = &self.last_metrics {
            ui.heading("📊 System Metrics");
            ui.separator();
            
            // CPU bar
            let cpu = metrics.system.cpu_percent;
            ui.label(format!("CPU: {:.1}%", cpu));
            ui.add(egui::ProgressBar::new(cpu / 100.0).show_percentage());
            
            // Memory bar
            let mem = metrics.system.memory_percent;
            ui.label(format!("Memory: {:.1}%", mem));
            ui.add(egui::ProgressBar::new(mem / 100.0).show_percentage());
            
            // Uptime
            let uptime = format_duration(metrics.system.uptime_seconds);
            ui.label(format!("Uptime: {}", uptime));
            
            ui.separator();
            
            // Neural API stats
            ui.label(format!("Active Primals: {}", metrics.neural_api.active_primals));
            ui.label(format!("Available Graphs: {}", metrics.neural_api.graphs_available));
        } else {
            ui.label("⏳ Connecting to Neural API...");
        }
    }
}
```

### **Phase C: Proprioception Panel** (45 min)

Display SAME DAVE self-awareness:

**File**: `crates/petal-tongue-ui/src/panels/proprioception_panel.rs`

```rust
pub struct ProprioceptionPanel {
    provider: Option<NeuralApiProvider>,
    last_proprio: Option<Proprioception>,
    last_update: Instant,
    update_interval: Duration,
}

impl PanelInstance for ProprioceptionPanel {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some(proprio) = &self.last_proprio {
            ui.heading("🧠 NUCLEUS Proprioception");
            
            // Health indicator
            let health_emoji = match proprio.health.status.as_str() {
                "healthy" => "💚",
                "degraded" => "💛",
                _ => "❤️",
            };
            ui.label(format!("Health: {:.0}% {}", 
                proprio.health.percentage, health_emoji));
            
            ui.separator();
            
            // SAME DAVE dimensions
            ui.label("Sensory:");
            ui.label(format!("  {} active sockets", 
                proprio.sensory.active_sockets));
            
            ui.label("Awareness:");
            ui.label(format!("  Knows about {} primals", 
                proprio.self_awareness.knows_about));
            
            ui.label("Motor:");
            if proprio.motor.can_deploy {
                ui.label("  ✅ Can deploy");
            }
            if proprio.motor.can_coordinate_primals {
                ui.label("  ✅ Can coordinate");
            }
            
            ui.label(format!("Confidence: {:.0}%", proprio.confidence));
            
            ui.separator();
            
            // Core systems
            ui.label("Core Systems:");
            if proprio.self_awareness.has_security {
                ui.label("  ✅ Security (BearDog)");
            }
            if proprio.self_awareness.has_discovery {
                ui.label("  ✅ Discovery (Songbird)");
            }
            if proprio.self_awareness.has_compute {
                ui.label("  ✅ Compute (Toadstool)");
            }
        } else {
            ui.label("⏳ Loading proprioception...");
        }
    }
}
```

### **Phase D: New Scenario** (15 min)

Create `sandbox/scenarios/doom-with-stats.json`:

```json
{
  "schema_version": "v2.0.0",
  "scenario_id": "doom-with-live-stats",
  "name": "Doom with Live biomeOS Stats",
  "description": "Play Doom with real-time biomeOS monitoring",
  
  "sensory_config": {
    "required_capabilities": {
      "outputs": ["visual"],
      "inputs": ["keyboard", "pointer"]
    },
    "optional_capabilities": {
      "outputs": ["audio"],
      "inputs": []
    },
    "complexity_hint": "rich"
  },
  
  "ui_config": {
    "mode": "custom",
    "show_left_sidebar": false,
    "show_right_sidebar": true,
    "show_dashboard": false,
    "show_graph_stats": false,
    "enable_audio_sonification": false,
    "auto_refresh": false,
    
    "custom_panels": [
      {
        "id": "doom-game",
        "panel_type": "doom",
        "title": "Doom",
        "position": "center",
        "config": {
          "width": 960,
          "height": 720,
          "mode": "game"
        }
      },
      {
        "id": "doom-stats",
        "panel_type": "doom_stats",
        "title": "Game Stats",
        "position": "right_top",
        "config": {}
      },
      {
        "id": "proprioception",
        "panel_type": "proprioception",
        "title": "System Health",
        "position": "right_middle",
        "config": {
          "update_interval_secs": 5
        }
      },
      {
        "id": "metrics",
        "panel_type": "metrics",
        "title": "System Metrics",
        "position": "right_bottom",
        "config": {
          "update_interval_secs": 1
        }
      }
    ]
  },
  
  "data": {}
}
```

### **Phase E: Panel Registration** (15 min)

Register new panels in `panel_registry.rs`:

```rust
pub fn register_default_panels(registry: &mut PanelRegistry) {
    // Existing Doom panel
    registry.register("doom", Box::new(DoomPanelFactory::new()));
    
    // NEW: Stats panels
    registry.register("doom_stats", Box::new(DoomStatsPanelFactory::new()));
    registry.register("proprioception", Box::new(ProprioceptionPanelFactory::new()));
    registry.register("metrics", Box::new(MetricsPanelFactory::new()));
}
```

---

## 🧪 **Testing Plan**

### **Manual Test**

```bash
# 1. Ensure Neural API is running
cd ../biomeOS
target/release/nucleus serve --family nat0 &

# 2. Ensure primals are running
plasmidBin/primals/beardog-server &
plasmidBin/primals/songbird-orchestrator &
plasmidBin/primals/toadstool &

# 3. Get WAD file (if not already present)
sudo apt install freedoom
cp /usr/share/games/doom/freedoom1.wad .

# 4. Run petalTongue with new scenario
cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/doom-with-stats.json
```

### **Expected Result**

You should see:
- Left/center: Doom game (first-person view, full size)
- Right sidebar with 4 stacked panels:
  1. Game Stats (map, position, FPS)
  2. System Health (proprioception from Neural API)
  3. System Metrics (CPU, memory, primals)
  4. (Optional) Mini topology view

### **Success Criteria**

- [ ] Doom runs at 60 FPS
- [ ] Game input is not blocked by stats updates
- [ ] Stats update in background (1-5 second intervals)
- [ ] Proprioception shows SAME DAVE data
- [ ] Metrics show real CPU/memory
- [ ] All panels render without errors
- [ ] Can play Doom while monitoring system health

---

## 🎯 **Success Metrics**

### **Performance**
- Doom: 60 FPS sustained
- Stats updates: <1ms per panel
- No input lag
- Total memory: <200 MB

### **Functionality**
- All panels render correctly
- Neural API data displays accurately
- Stats update independently
- Game remains playable

### **Architecture Validation**
- ✅ Panel system handles multiple concurrent panels
- ✅ Input focus works (game gets keyboard/mouse)
- ✅ Async updates don't block rendering
- ✅ Lifecycle hooks manage resources
- ✅ Composition over implementation

---

## 📊 **Expected Outcomes**

### **Immediate**
1. Playable Doom with live system stats
2. Validation of panel architecture
3. Proof of biomeOS integration
4. Cool demo for stakeholders! 🎮📊

### **Architectural Insights**
1. How well does panel system scale?
2. Are lifecycle hooks sufficient?
3. Does input focus work correctly?
4. Can we handle 60 FPS + background tasks?

### **Next Steps Unlocked**
After this is working, we can:
1. Add more game panels (e.g., web browser)
2. Implement Phase 1.3 (gameplay features)
3. Move to Phase 2 (petalTongue IS Doom)
4. Build more complex multi-panel layouts

---

## 🔮 **Evolution Opportunities**

As we build, we may discover:

1. **Panel Communication**: Need panels to talk to each other?
2. **Layout Management**: Better panel positioning/resizing?
3. **Resource Priorities**: How to handle CPU contention?
4. **Error Isolation**: What if Neural API goes down mid-game?
5. **State Sync**: Should game stats persist across restarts?

Each discovery makes petalTongue stronger! 🌸

---

## 📝 **Timeline**

- **Phase A** (Doom Stats): 30 min
- **Phase B** (Metrics): 45 min
- **Phase C** (Proprioception): 45 min
- **Phase D** (Scenario): 15 min
- **Phase E** (Registration): 15 min
- **Testing**: 30 min

**Total**: ~3 hours

**Status**: 🚀 Ready to start!

---

**Next**: Implement Phase A - Doom Stats Panel

