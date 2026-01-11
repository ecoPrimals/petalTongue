# 🏗️ biomeOS UI Integration - Evolved Architecture

**Date**: January 11, 2026  
**Version**: 1.0.0  
**Status**: Architectural Design  
**Priority**: HIGH (Critical Path)

---

## 🌸 EVOLUTION PRINCIPLES

This specification evolves the initial gap analysis by applying:

1. **TRUE PRIMAL Architecture**: Zero hardcoding, capability-based, graceful degradation
2. **Modern Idiomatic Rust**: Async/await, strong typing, zero unsafe
3. **Component Reusability**: Leverage existing battle-tested components
4. **Progressive Enhancement**: Core functionality without dependencies
5. **Accessibility First**: Universal design patterns
6. **Test-Driven**: Comprehensive coverage from day one
7. **Documentation as Code**: Living documentation that stays current

---

## 🎯 ARCHITECTURAL VISION

### Discord-like UI for Device/Niche Management

**User Experience Goal**: "As intuitive as Discord servers, as powerful as Kubernetes"

**Core Metaphor**:
- **Servers** (Discord) = **Niches** (ecoPrimals)
- **Channels** (Discord) = **Primals** (ecoPrimals)
- **Users** (Discord) = **Devices** (ecoPrimals)
- **Roles** (Discord) = **Capabilities** (ecoPrimals)

### Visual Layout

```
┌─────────────────────────────────────────────────────────────┐
│ petalTongue - Device & Niche Management                    │
├───────────┬─────────────────────────────────┬───────────────┤
│           │                                 │               │
│  Niches   │     Main Canvas                │   Devices     │
│  ┌──────┐ │                                 │  ┌─────────┐  │
│  │ Nest │ │   ┌─────────────────────┐      │  │ GPU-0   │  │
│  │ Tower│ │   │  Niche Designer     │      │  │ CPU-1   │  │
│  │ Node │ │   │                     │      │  │ SSD-2   │  │
│  └──────┘ │   │   Drag & Drop       │      │  │ NET-3   │  │
│           │   │   Primals + Devices │      │  └─────────┘  │
│  Primals  │   └─────────────────────┘      │               │
│  ┌──────┐ │                                 │   Status      │
│  │bearDog│ │   OR                           │  ┌─────────┐  │
│  │songBrd│ │                                 │  │ Health  │  │
│  │toadStl│ │   ┌─────────────────────┐      │  │ Load    │  │
│  └──────┘ │   │  Device Panel       │      │  │ Trust   │  │
│           │   │                     │      │  └─────────┘  │
│           │   │  Device Tree        │      │               │
│           │   │  + Assignments      │      │               │
│           │   └─────────────────────┘      │               │
└───────────┴─────────────────────────────────┴───────────────┘
```

---

## 🏛️ COMPONENT ARCHITECTURE

### 1. Data Layer (Foundation)

**TRUE PRIMAL Compliant Data Providers**

```rust
// Core trait (already exists!)
pub trait VisualizationDataProvider: Send + Sync {
    async fn discover_devices(&self) -> Result<Vec<Device>>;
    async fn discover_primals(&self) -> Result<Vec<Primal>>;
    async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>>;
    async fn subscribe_events(&self) -> Result<EventStream>;
}

// biomeOS implementation (NEW!)
pub struct BiomeOSProvider {
    client: BiomeOSClient,
    event_stream: Arc<tokio::sync::RwLock<Option<EventStream>>>,
    cache: Arc<tokio::sync::RwLock<ProviderCache>>,
}

impl BiomeOSProvider {
    /// Discover biomeOS via capability
    pub async fn discover() -> Result<Option<Self>> {
        // TRUE PRIMAL: Discover by capability, not by name
        let providers = discover_visualization_providers().await?;
        
        for provider in providers {
            if provider.has_capability("device.management").await? {
                return Ok(Some(Self::connect(provider).await?));
            }
        }
        
        Ok(None) // Graceful: works without biomeOS
    }
    
    /// Connect to discovered provider
    async fn connect(provider: PrimalInfo) -> Result<Self> {
        // Use Unix socket or capability-based discovery
        let client = BiomeOSClient::connect(&provider.socket_path).await?;
        
        Ok(Self {
            client,
            event_stream: Arc::new(tokio::sync::RwLock::new(None)),
            cache: Arc::new(tokio::sync::RwLock::new(ProviderCache::new())),
        })
    }
}

// Mock provider for testing & graceful degradation (NEW!)
pub struct MockProvider {
    devices: Vec<Device>,
    primals: Vec<Primal>,
}

impl MockProvider {
    pub fn with_demo_data() -> Self {
        Self {
            devices: vec![
                Device::new("gpu-0", DeviceType::GPU),
                Device::new("cpu-1", DeviceType::CPU),
                Device::new("ssd-2", DeviceType::Storage),
            ],
            primals: vec![
                Primal::new("beardog", vec!["security"]),
                Primal::new("songbird", vec!["discovery"]),
                Primal::new("toadstool", vec!["compute"]),
            ],
        }
    }
}
```

**Key Design Decisions**:
- ✅ **Graceful Degradation**: Mock provider when biomeOS unavailable
- ✅ **Capability Discovery**: Find by "device.management", not "biomeOS"
- ✅ **Caching**: Local cache for offline mode
- ✅ **Event Streaming**: Real-time updates via WebSocket
- ✅ **Async Throughout**: Zero blocking operations

---

### 2. UI Component Layer (Modular)

**Reusable, Composable, Testable**

```rust
// Device Panel (NEW!)
pub struct DevicePanel {
    provider: Arc<dyn VisualizationDataProvider>,
    devices: Vec<Device>,
    selected: Option<DeviceId>,
    filter: DeviceFilter,
}

impl DevicePanel {
    pub fn new(provider: Arc<dyn VisualizationDataProvider>) -> Self {
        Self {
            provider,
            devices: Vec::new(),
            selected: None,
            filter: DeviceFilter::All,
        }
    }
    
    /// Render UI (egui)
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Filter bar
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.radio_value(&mut self.filter, DeviceFilter::All, "All");
            ui.radio_value(&mut self.filter, DeviceFilter::Available, "Available");
            ui.radio_value(&mut self.filter, DeviceFilter::Assigned, "Assigned");
        });
        
        // Device tree
        egui::ScrollArea::vertical().show(ui, |ui| {
            for device in self.filtered_devices() {
                self.device_card(ui, device);
            }
        });
    }
    
    /// Individual device card
    fn device_card(&mut self, ui: &mut egui::Ui, device: &Device) {
        let is_selected = self.selected == Some(device.id);
        
        let response = egui::Frame::none()
            .fill(if is_selected { 
                ui.visuals().selection.bg_fill 
            } else { 
                ui.visuals().faint_bg_color 
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Device icon
                    ui.label(device.icon());
                    
                    // Device name
                    ui.label(&device.name);
                    
                    // Status indicator
                    self.status_indicator(ui, device);
                    
                    // Resource usage
                    self.resource_bar(ui, device);
                });
            })
            .response;
        
        // Drag source
        if response.dragged() {
            egui::DragAndDrop::set_payload(ui.ctx(), device.clone());
        }
        
        // Selection
        if response.clicked() {
            self.selected = Some(device.id);
        }
    }
    
    /// Status indicator (color-coded)
    fn status_indicator(&self, ui: &mut egui::Ui, device: &Device) {
        let (color, text) = match device.status {
            DeviceStatus::Online => (egui::Color32::GREEN, "●"),
            DeviceStatus::Offline => (egui::Color32::GRAY, "●"),
            DeviceStatus::Busy => (egui::Color32::YELLOW, "●"),
            DeviceStatus::Error => (egui::Color32::RED, "●"),
        };
        
        ui.colored_label(color, text);
    }
    
    /// Resource usage bar
    fn resource_bar(&self, ui: &mut egui::Ui, device: &Device) {
        let usage = device.resource_usage();
        
        let bar_color = if usage > 0.9 {
            egui::Color32::RED
        } else if usage > 0.7 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::GREEN
        };
        
        ui.add(egui::ProgressBar::new(usage as f32)
            .fill(bar_color)
            .show_percentage());
    }
    
    /// Update devices from provider
    pub async fn refresh(&mut self) -> Result<()> {
        self.devices = self.provider.discover_devices().await?;
        Ok(())
    }
}

// Primal Panel (NEW!)
pub struct PrimalPanel {
    provider: Arc<dyn VisualizationDataProvider>,
    primals: Vec<Primal>,
    selected: Option<PrimalId>,
}

impl PrimalPanel {
    pub fn new(provider: Arc<dyn VisualizationDataProvider>) -> Self {
        Self {
            provider,
            primals: Vec::new(),
            selected: None,
        }
    }
    
    /// Render UI (egui)
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for primal in &self.primals {
                self.primal_card(ui, primal);
            }
        });
    }
    
    /// Individual primal card
    fn primal_card(&mut self, ui: &mut egui::Ui, primal: &Primal) {
        let is_selected = self.selected == Some(primal.id);
        
        let response = egui::Frame::none()
            .fill(if is_selected { 
                ui.visuals().selection.bg_fill 
            } else { 
                ui.visuals().faint_bg_color 
            })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Primal name
                    ui.heading(&primal.name);
                    
                    // Health status
                    self.health_indicator(ui, primal);
                    
                    // Capabilities
                    ui.label(format!("Capabilities: {}", 
                        primal.capabilities.join(", ")));
                    
                    // Assigned devices
                    ui.label(format!("Devices: {}", 
                        primal.assigned_devices.len()));
                    
                    // Load
                    ui.add(egui::ProgressBar::new(primal.load as f32)
                        .text(format!("Load: {:.0}%", primal.load * 100.0)));
                });
            })
            .response;
        
        // Drop target
        if let Some(device) = egui::DragAndDrop::payload::<Device>(ui.ctx()) {
            if response.hovered() {
                ui.painter().rect_stroke(
                    response.rect,
                    4.0,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                );
                
                if response.clicked() {
                    // Assign device to primal
                    self.assign_device(device.clone(), primal.id);
                }
            }
        }
        
        // Selection
        if response.clicked() {
            self.selected = Some(primal.id);
        }
    }
    
    /// Health indicator
    fn health_indicator(&self, ui: &mut egui::Ui, primal: &Primal) {
        let (color, text) = match primal.health {
            Health::Healthy => (egui::Color32::GREEN, "✓ Healthy"),
            Health::Degraded => (egui::Color32::YELLOW, "⚠ Degraded"),
            Health::Offline => (egui::Color32::RED, "✗ Offline"),
        };
        
        ui.colored_label(color, text);
    }
    
    /// Assign device to primal
    fn assign_device(&self, device: Device, primal_id: PrimalId) {
        // Send to provider
        tokio::spawn(async move {
            // Call biomeOS assign_device method
            // This is capability-based, not hardcoded
        });
    }
}

// Niche Designer (NEW! - evolved from graph_editor)
pub struct NicheDesigner {
    canvas: NicheCanvas,
    templates: Vec<NicheTemplate>,
    selected_template: Option<TemplateId>,
    current_niche: Niche,
}

impl NicheDesigner {
    pub fn new() -> Self {
        Self {
            canvas: NicheCanvas::new(),
            templates: NicheTemplate::builtin(),
            selected_template: None,
            current_niche: Niche::empty(),
        }
    }
    
    /// Render UI (egui)
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Template selector
            ui.label("Template:");
            for template in &self.templates {
                if ui.button(&template.name).clicked() {
                    self.apply_template(template);
                }
            }
        });
        
        // Canvas
        self.canvas.ui(ui, &mut self.current_niche);
        
        // Deploy button
        if ui.button("🚀 Deploy Niche").clicked() {
            self.deploy();
        }
    }
    
    /// Apply template
    fn apply_template(&mut self, template: &NicheTemplate) {
        self.current_niche = template.instantiate();
    }
    
    /// Deploy niche
    fn deploy(&self) {
        // Validate
        if let Err(e) = self.current_niche.validate() {
            // Show error
            return;
        }
        
        // Call biomeOS deploy
        tokio::spawn(async move {
            // Send to provider
        });
    }
}
```

**Key Design Decisions**:
- ✅ **Component Isolation**: Each panel is self-contained
- ✅ **Drag-and-Drop**: Native egui drag-and-drop
- ✅ **Color Coding**: Visual status indicators
- ✅ **Progressive Enhancement**: Works with mock data
- ✅ **Accessibility**: Keyboard navigation, screen reader support
- ✅ **Real-time Updates**: Async refresh from provider

---

### 3. Integration Layer (Wiring)

**Event-Driven Architecture**

```rust
// Event types (NEW!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIEvent {
    DeviceDiscovered(Device),
    DeviceRemoved(DeviceId),
    DeviceStatusChanged(DeviceId, DeviceStatus),
    PrimalDiscovered(Primal),
    PrimalRemoved(PrimalId),
    PrimalHealthChanged(PrimalId, Health),
    DeviceAssigned(DeviceId, PrimalId),
    DeviceUnassigned(DeviceId, PrimalId),
    NicheDeployed(Niche),
    AISuggestion(Suggestion),
}

// Event handler (NEW!)
pub struct UIEventHandler {
    device_panel: Arc<tokio::sync::RwLock<DevicePanel>>,
    primal_panel: Arc<tokio::sync::RwLock<PrimalPanel>>,
    niche_designer: Arc<tokio::sync::RwLock<NicheDesigner>>,
}

impl UIEventHandler {
    pub async fn handle_event(&self, event: UIEvent) {
        match event {
            UIEvent::DeviceDiscovered(device) => {
                let mut panel = self.device_panel.write().await;
                panel.add_device(device);
            }
            UIEvent::PrimalDiscovered(primal) => {
                let mut panel = self.primal_panel.write().await;
                panel.add_primal(primal);
            }
            UIEvent::DeviceAssigned(device_id, primal_id) => {
                // Update both panels
                let mut device_panel = self.device_panel.write().await;
                let mut primal_panel = self.primal_panel.write().await;
                device_panel.mark_assigned(device_id);
                primal_panel.add_device_to_primal(device_id, primal_id);
            }
            // ... handle all events
        }
    }
    
    /// Start event listener
    pub async fn start_event_loop(
        &self,
        provider: Arc<dyn VisualizationDataProvider>,
    ) -> Result<()> {
        let mut event_stream = provider.subscribe_events().await?;
        
        while let Some(event) = event_stream.next().await {
            self.handle_event(event).await;
        }
        
        Ok(())
    }
}
```

**Key Design Decisions**:
- ✅ **Event-Driven**: Reactive updates
- ✅ **Async Throughout**: Non-blocking event handling
- ✅ **Centralized Dispatch**: Single event handler
- ✅ **Type-Safe Events**: Strong typing for all events

---

## 🧪 TESTING STRATEGY

### Unit Tests (Per Component)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_device_panel_discovery() {
        let provider = Arc::new(MockProvider::with_demo_data());
        let mut panel = DevicePanel::new(provider);
        
        panel.refresh().await.unwrap();
        
        assert_eq!(panel.devices.len(), 3);
        assert_eq!(panel.devices[0].name, "gpu-0");
    }
    
    #[tokio::test]
    async fn test_device_assignment() {
        let provider = Arc::new(MockProvider::with_demo_data());
        let mut device_panel = DevicePanel::new(provider.clone());
        let mut primal_panel = PrimalPanel::new(provider);
        
        device_panel.refresh().await.unwrap();
        primal_panel.refresh().await.unwrap();
        
        let device = device_panel.devices[0].clone();
        let primal_id = primal_panel.primals[0].id;
        
        primal_panel.assign_device(device, primal_id);
        
        assert_eq!(primal_panel.primals[0].assigned_devices.len(), 1);
    }
    
    #[tokio::test]
    async fn test_niche_template_application() {
        let mut designer = NicheDesigner::new();
        let nest_template = &designer.templates[0]; // Nest template
        
        designer.apply_template(nest_template);
        
        assert!(designer.current_niche.has_required_primals());
    }
}
```

### Integration Tests (End-to-End)

```rust
#[tokio::test]
async fn test_full_niche_deployment_workflow() {
    // 1. Start mock biomeOS
    let biomeos = MockBiomeOS::start().await;
    
    // 2. Discover provider
    let provider = BiomeOSProvider::discover().await.unwrap().unwrap();
    
    // 3. Create UI components
    let device_panel = DevicePanel::new(Arc::new(provider));
    let primal_panel = PrimalPanel::new(Arc::new(provider));
    let niche_designer = NicheDesigner::new();
    
    // 4. Refresh data
    device_panel.refresh().await.unwrap();
    primal_panel.refresh().await.unwrap();
    
    // 5. Apply template
    niche_designer.apply_template(&NicheTemplate::nest());
    
    // 6. Deploy
    niche_designer.deploy();
    
    // 7. Verify deployment
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(biomeos.has_deployed_niche("nest-1").await);
}
```

### Chaos Tests (Fault Tolerance)

```rust
#[tokio::test]
async fn test_biomeos_disconnect_graceful_degradation() {
    let provider = BiomeOSProvider::discover().await.unwrap().unwrap();
    let mut panel = DevicePanel::new(Arc::new(provider));
    
    panel.refresh().await.unwrap();
    assert_eq!(panel.devices.len(), 3);
    
    // Kill biomeOS
    kill_biomeos().await;
    
    // Should still have cached data
    assert_eq!(panel.devices.len(), 3);
    
    // Refresh should fail gracefully
    assert!(panel.refresh().await.is_err());
    
    // UI should still be interactive with cached data
    panel.ui(&mut mock_egui_context());
}
```

**Testing Coverage Goals**:
- ✅ Unit tests: 90%+ per component
- ✅ Integration tests: Full workflows
- ✅ Chaos tests: Fault tolerance
- ✅ Performance tests: <16ms frame time

---

## 📊 IMPLEMENTATION PHASES (EVOLVED)

### Phase 1: Foundation & Data Layer (Week 1) - 2-3 days

**Goal**: Solid data foundation with graceful degradation

**Tasks**:
1. ✅ Create `BiomeOSProvider` with capability discovery
2. ✅ Create `MockProvider` for testing/offline mode
3. ✅ Implement event streaming (WebSocket)
4. ✅ Add caching for offline mode
5. ✅ Write 20+ unit tests

**Deliverables**:
- `biomeos_integration.rs` (~400 lines, up from 300)
- `mock_provider.rs` (~200 lines, NEW!)
- `event_types.rs` (~150 lines, NEW!)
- 20+ unit tests
- 5+ integration tests

**Success Criteria**:
- ✅ Can discover biomeOS by capability
- ✅ Gracefully falls back to mock mode
- ✅ Events stream in real-time
- ✅ Offline mode works with cache
- ✅ All tests passing

---

### Phase 2: Device Management UI (Week 1-2) - 3-4 days

**Goal**: Beautiful, functional device panel

**Tasks**:
1. ✅ Create `DevicePanel` with tree view
2. ✅ Create `DeviceCard` with status indicators
3. ✅ Implement drag-and-drop
4. ✅ Add filtering (All, Available, Assigned)
5. ✅ Add real-time updates
6. ✅ Add keyboard navigation
7. ✅ Write 15+ unit tests

**Deliverables**:
- `device_panel.rs` (~500 lines, up from 400)
- `device_card.rs` (~200 lines, up from 150)
- `device_filter.rs` (~100 lines, NEW!)
- 15+ unit tests
- 3+ integration tests

**Success Criteria**:
- ✅ Displays all devices
- ✅ Real-time status updates
- ✅ Drag-and-drop works
- ✅ Keyboard accessible
- ✅ <16ms render time

---

### Phase 3: Primal Status UI (Week 2) - 2-3 days

**Goal**: Live primal monitoring

**Tasks**:
1. ✅ Create `PrimalPanel` with card view
2. ✅ Create `PrimalCard` with health indicators
3. ✅ Show assigned devices per primal
4. ✅ Implement drop target for device assignment
5. ✅ Add real-time health updates
6. ✅ Write 15+ unit tests

**Deliverables**:
- `primal_panel.rs` (~500 lines, up from 400)
- `primal_card.rs` (~250 lines, up from 200)
- `primal_health.rs` (~100 lines, NEW!)
- 15+ unit tests
- 3+ integration tests

**Success Criteria**:
- ✅ Displays all primals
- ✅ Real-time health updates
- ✅ Drop target works
- ✅ Shows assigned devices
- ✅ <16ms render time

---

### Phase 4: Niche Designer (Week 2-3) - 4-5 days

**Goal**: Visual niche composition

**Tasks**:
1. ✅ Copy `graph_editor/canvas.rs` as base
2. ✅ Adapt for niche design
3. ✅ Create 3 templates (Nest, Tower, Node)
4. ✅ Add primal drag-and-drop
5. ✅ Add device drag-and-drop
6. ✅ Add validation
7. ✅ Add deploy button
8. ✅ Write 20+ unit tests

**Deliverables**:
- `niche_designer.rs` (~700 lines, up from 600)
- `niche_canvas.rs` (~500 lines, up from 400)
- `niche_templates.rs` (~300 lines, up from 200)
- `niche_validation.rs` (~200 lines, NEW!)
- 20+ unit tests
- 5+ integration tests

**Success Criteria**:
- ✅ Templates work
- ✅ Drag-and-drop works
- ✅ Validation works
- ✅ Deploy works
- ✅ <16ms render time

---

### Phase 5: Integration & Polish (Week 3) - 3-4 days

**Goal**: Production-ready system

**Tasks**:
1. ✅ Wire all components together
2. ✅ Add AI suggestion display
3. ✅ Add conflict resolution
4. ✅ Add error handling
5. ✅ Add loading states
6. ✅ Add keyboard shortcuts
7. ✅ Write 10+ E2E tests
8. ✅ Write 5+ chaos tests
9. ✅ Performance optimization
10. ✅ Documentation

**Deliverables**:
- `ui_event_handler.rs` (~300 lines, NEW!)
- `ai_suggestions.rs` (~200 lines, NEW!)
- Updated `app.rs` (wire everything)
- 10+ E2E tests
- 5+ chaos tests
- User guide (500+ lines)
- API documentation

**Success Criteria**:
- ✅ All components integrated
- ✅ All tests passing
- ✅ Graceful degradation works
- ✅ <16ms frame time
- ✅ Comprehensive documentation

---

## 📈 REVISED EFFORT ESTIMATE

| Phase | Component | Lines | Tests | Time | Priority |
|-------|-----------|-------|-------|------|----------|
| 1 | Foundation | ~750 | 25+ | 2-3 days | ⭐⭐⭐ |
| 2 | Device UI | ~800 | 18+ | 3-4 days | ⭐⭐⭐ |
| 3 | Primal UI | ~850 | 18+ | 2-3 days | ⭐⭐ |
| 4 | Niche Designer | ~1,700 | 25+ | 4-5 days | ⭐⭐ |
| 5 | Integration | ~500 | 15+ | 3-4 days | ⭐⭐⭐ |
| **Total** | **All** | **~4,600** | **101+** | **14-19 days** | **3-4 weeks** |

**Comparison to Original**:
- Lines: 3,050 → 4,600 (+50% for robustness)
- Tests: 0 → 101+ (comprehensive coverage)
- Time: 2.5-3.5 weeks → 3-4 weeks (realistic with testing)

---

## 🌸 TRUE PRIMAL COMPLIANCE

### Zero Hardcoding ✅

```rust
// ❌ BAD: Hardcoded primal name
let biomeos = connect_to_biomeos().await?;

// ✅ GOOD: Capability-based discovery
let provider = discover_visualization_providers().await?
    .into_iter()
    .find(|p| p.has_capability("device.management"))
    .ok_or("No device management provider found")?;
```

### Graceful Degradation ✅

```rust
// Try biomeOS first
if let Some(provider) = BiomeOSProvider::discover().await? {
    return Ok(provider);
}

// Fall back to mock mode
info!("biomeOS not found, using mock data");
Ok(MockProvider::with_demo_data())
```

### Self-Knowledge ✅

```rust
// petalTongue discovers its own capabilities
let my_capabilities = vec![
    "ui.render",
    "ui.device_management",
    "ui.niche_designer",
];

// Announces to ecosystem
announce_capabilities(my_capabilities).await?;
```

### Runtime Discovery ✅

```rust
// Discover all primals at runtime
let primals = provider.discover_primals().await?;

// No compile-time dependencies on specific primals
for primal in primals {
    info!("Discovered primal with capabilities: {:?}", primal.capabilities);
}
```

---

## 🎯 SUCCESS CRITERIA

### Functional ✅

- [ ] Can discover devices in real-time
- [ ] Can discover primals in real-time
- [ ] Can assign devices to primals (drag-and-drop)
- [ ] Can design niches (drag-and-drop)
- [ ] Can deploy niches
- [ ] Can see AI suggestions
- [ ] Works offline with mock data

### Performance ✅

- [ ] <16ms frame time (60 FPS)
- [ ] <500ms event latency
- [ ] <100ms interaction latency
- [ ] Supports 50+ devices
- [ ] Supports 20+ primals

### Quality ✅

- [ ] 90%+ test coverage
- [ ] Zero unsafe code
- [ ] Zero hardcoded primals
- [ ] Graceful degradation works
- [ ] Comprehensive documentation

### Accessibility ✅

- [ ] Keyboard navigation works
- [ ] Screen reader support
- [ ] Color-blind safe palette
- [ ] High contrast mode
- [ ] Adjustable font sizes

---

## 📚 DOCUMENTATION DELIVERABLES

### Technical Documentation

1. **Architecture Specification** (this document)
2. **API Reference** (auto-generated from code)
3. **Integration Guide** (for biomeOS team)
4. **Component Guide** (for UI developers)

### User Documentation

1. **User Guide** (how to use device/niche management)
2. **Quick Start** (5-minute tutorial)
3. **Video Tutorial** (optional, future)

### Developer Documentation

1. **Contributing Guide** (how to add new components)
2. **Testing Guide** (how to write tests)
3. **Troubleshooting Guide** (common issues)

---

## 🚀 DEPLOYMENT STRATEGY

### Alpha (Week 1-2)

**Target**: Internal testing

**Features**:
- Basic device panel
- Basic primal panel
- Mock data only

**Goal**: Validate UX

### Beta (Week 3)

**Target**: biomeOS team

**Features**:
- All panels complete
- Live data from biomeOS
- Basic niche designer

**Goal**: Integration testing

### Production (Week 4)

**Target**: Full ecosystem

**Features**:
- All features complete
- Polish & optimization
- Comprehensive docs

**Goal**: Production deployment

---

## 📞 NEXT STEPS

### Immediate (Today)

1. **Review this spec** with team
2. **Get approval** for evolved approach
3. **Set up tracking** document
4. **Create feature branch**: `feature/biomeos-ui-integration`

### Week 1 (Starts Tomorrow)

1. **Phase 1**: Foundation & Data Layer
2. **Daily standups**: Progress tracking
3. **Continuous integration**: Tests must pass

### Week 2-3

1. **Phase 2-4**: UI components
2. **Weekly demos**: Show progress to biomeOS team
3. **Integration testing**: With live biomeOS

### Week 4

1. **Phase 5**: Integration & polish
2. **Documentation sprint**: Complete all docs
3. **Final review**: Production readiness
4. **Deployment**: Ship it! 🚀

---

## 🎊 EVOLUTION SUMMARY

**What Changed from Initial Analysis**:

1. **+50% Code** (3,050 → 4,600 lines)
   - Added mock provider for testing
   - Added comprehensive error handling
   - Added caching for offline mode
   - Added event system
   - Added validation

2. **+101 Tests** (0 → 101+)
   - Unit tests for all components
   - Integration tests for workflows
   - Chaos tests for fault tolerance
   - Performance tests

3. **+TRUE PRIMAL** (Gap → Compliant)
   - Capability-based discovery
   - Zero hardcoding
   - Graceful degradation
   - Self-knowledge

4. **+Accessibility** (None → Full)
   - Keyboard navigation
   - Screen reader support
   - Color-blind safe
   - High contrast mode

5. **+Documentation** (None → Comprehensive)
   - Technical specs
   - User guides
   - API reference
   - Integration guides

**Result**: More robust, more testable, more maintainable, more accessible, TRUE PRIMAL compliant!

---

**Created**: January 11, 2026  
**Status**: Architectural Design Complete  
**Next**: Create tracking document & begin implementation  
**Timeline**: 3-4 weeks (realistic with quality & testing)

🌸 **Ready to evolve petalTongue with biomeOS integration!** 🤝✨

