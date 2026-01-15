# 🌸 Panel System v2.0 - Living Specification

**Version**: 2.0.0  
**Date**: January 15, 2026  
**Status**: ✅ Foundation Complete (Phases 1-4)

---

## 🎯 **OVERVIEW**

The Panel System enables petalTongue to embed **any application** as a panel:
- Games (Doom, etc.)
- Web browsers
- Video players
- Terminals
- IDEs
- Custom tools

---

## 🧬 **ARCHITECTURE**

### **Core Components**

1. **PanelFactory** - Creates panel instances
2. **PanelRegistry** - Manages available panel types
3. **PanelInstance** - Runtime panel interface
4. **FocusManager** - Input routing
5. **Lifecycle Hooks** - Resource management

---

## 📋 **PanelInstance Trait**

```rust
pub trait PanelInstance: Send {
    // === Core Methods ===
    fn render(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> &str;
    fn update(&mut self) {}
    
    // === Input Focus (Phase 3) ===
    fn wants_keyboard_input(&self) -> bool { false }
    fn wants_mouse_input(&self) -> bool { false }
    fn wants_exclusive_input(&self) -> bool { false }
    fn input_priority(&self) -> u8 { 5 }
    fn on_keyboard_event(&mut self, ctx: &Context) -> InputAction { Ignored }
    fn on_mouse_event(&mut self, ctx: &Context) -> InputAction { Ignored }
    
    // === Lifecycle (Phase 4) ===
    fn on_open(&mut self) -> Result<()> { Ok(()) }
    fn on_close(&mut self) -> Result<()> { Ok(()) }
    fn on_pause(&mut self) {}
    fn on_resume(&mut self) {}
    fn on_error(&mut self, error: &Error) -> PanelAction { Continue }
    
    // === State Persistence ===
    fn can_save_state(&self) -> bool { false }
    fn can_restore_state(&self) -> bool { false }
    fn save_state(&self) -> Result<Value> { Ok(Null) }
    fn restore_state(&mut self, state: Value) -> Result<()> { Ok(()) }
    
    // === Queries ===
    fn is_closable(&self) -> bool { true }
    fn is_pausable(&self) -> bool { true }
}
```

---

## 🎮 **EXAMPLE: Doom Panel**

```rust
impl PanelInstance for DoomPanel {
    fn title(&self) -> &str { &self.title }
    
    fn render(&mut self, ui: &mut egui::Ui) {
        // Render game framebuffer
    }
    
    // Input
    fn wants_keyboard_input(&self) -> bool { true }
    fn wants_exclusive_input(&self) -> bool { true }
    fn input_priority(&self) -> u8 { 10 }
    
    // Lifecycle
    fn on_open(&mut self) -> Result<()> {
        self.load_wad()?;
        Ok(())
    }
    
    fn on_close(&mut self) -> Result<()> {
        self.save_progress()?;
        Ok(())
    }
    
    fn on_pause(&mut self) {
        self.paused = true;
    }
    
    // State
    fn can_save_state(&self) -> bool { true }
    fn save_state(&self) -> Result<Value> {
        Ok(json!({"level": self.level}))
    }
}
```

---

## 📊 **SCENARIO INTEGRATION**

Panels are defined in scenario JSON:

```json
{
  "ui_config": {
    "custom_panels": [
      {
        "type": "doom_game",
        "title": "Doom",
        "width": 640,
        "height": 480,
        "config": {
          "wad_file": "doom1.wad",
          "show_debug": false
        }
      }
    ]
  }
}
```

---

## ✅ **PHASES IMPLEMENTED**

### **Phase 1: Validation (Complete)**
- Scenario validation
- Panel config validation  
- Rich error messages
- 17/17 tests passing

### **Phase 2: Error Messages (Complete)**
- ScenarioError enum
- Context-rich errors
- Fix suggestions
- 3/3 tests passing

### **Phase 3: Input Focus (Complete)**
- FocusManager
- Priority-based routing
- Exclusive input mode
- 7/7 tests passing

### **Phase 4: Lifecycle (Complete)**
- Resource management hooks
- Error isolation
- State persistence
- 10 new trait methods

---

## 🚀 **FUTURE PHASES**

### **Phase 5: Performance Budgets** (Planned)
- Per-panel FPS targets
- CPU time allocation
- Adaptive quality
- Frame timing

### **Phase 6: Panel Composition** (Planned)
- Split layouts
- Tabs
- Grids
- Multi-window

### **Phase 7: Hot Reloading** (Planned)
- Watch scenario files
- Live panel updates
- No restart needed

---

## 🧬 **TRUE PRIMAL COMPLIANCE**

**Zero Hardcoding**: ✅
- Panel types registered dynamically
- Capabilities discovered at runtime

**Live Evolution**: ✅
- New panel types can be added
- Scenarios hot-reloadable (Phase 7)

**Self-Knowledge Only**: ✅
- Panels declare capabilities
- No global panel list

**Graceful Degradation**: ✅
- Panel errors isolated
- Validation catches mistakes

---

**Status**: Foundation Complete ✅  
**Next**: Real Doom Integration or More Panel Types  
**Version**: 2.0.0  

🌸 Panel system ready for production! 🌸

