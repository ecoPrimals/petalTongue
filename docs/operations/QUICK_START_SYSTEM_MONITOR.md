# Quick Start: System Monitor Tool Integration

**Goal**: Implement the first external tool to prove the ToolPanel pattern works  
**Tool**: System Monitor using `sysinfo` crate  
**Time**: ~1 hour  
**Difficulty**: ⭐ Easy

---

## Why System Monitor First?

1. **Simple API** - sysinfo is straightforward
2. **Visual impact** - Real-time CPU/memory graphs look great
3. **Universal utility** - Everyone needs system monitoring
4. **No external deps** - Pure Rust, no system libraries
5. **Fast to implement** - Proves the pattern quickly

---

## Step 1: Add Dependency

```toml
# In crates/petal-tongue-ui/Cargo.toml
[dependencies]
sysinfo = "0.30"
```

---

## Step 2: Create Integration Module

```bash
cd crates/petal-tongue-ui/src
touch system_monitor_integration.rs
```

---

## Step 3: Implement the Tool

```rust
//! System Monitor Integration
//!
//! Real-time system resource monitoring using sysinfo.
//! Demonstrates petalTongue integrating with external monitoring tool.

use crate::tool_integration::{ToolCapability, ToolMetadata, ToolPanel};
use sysinfo::{System, SystemExt, CpuExt, ProcessExt};
use std::time::{Duration, Instant};

pub struct SystemMonitorTool {
    show_panel: bool,
    system: System,
    last_refresh: Instant,
    refresh_interval: Duration,
    cpu_history: Vec<f32>,  // Last 60 seconds
    mem_history: Vec<f32>,  // Last 60 seconds
    max_history: usize,
}

impl Default for SystemMonitorTool {
    fn default() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            show_panel: false,
            system,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(1),
            cpu_history: Vec::new(),
            mem_history: Vec::new(),
            max_history: 60,  // 60 seconds of history
        }
    }
}

impl SystemMonitorTool {
    /// Refresh system information
    fn refresh(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_refresh) >= self.refresh_interval {
            self.system.refresh_all();
            self.last_refresh = now;
            
            // Update history
            let cpu_usage = self.system.global_cpu_info().cpu_usage();
            self.cpu_history.push(cpu_usage);
            if self.cpu_history.len() > self.max_history {
                self.cpu_history.remove(0);
            }
            
            let used_mem = self.system.used_memory() as f32;
            let total_mem = self.system.total_memory() as f32;
            let mem_percent = (used_mem / total_mem) * 100.0;
            self.mem_history.push(mem_percent);
            if self.mem_history.len() > self.max_history {
                self.mem_history.remove(0);
            }
        }
    }
    
    /// Render CPU section
    fn render_cpu(&self, ui: &mut egui::Ui) {
        ui.heading("💻 CPU");
        
        // Current usage
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        ui.label(format!("Usage: {:.1}%", cpu_usage));
        
        // Progress bar
        ui.add(egui::ProgressBar::new(cpu_usage / 100.0)
            .text(format!("{:.1}%", cpu_usage)));
        
        // Simple sparkline (last 60 seconds)
        if !self.cpu_history.is_empty() {
            ui.label(format!("History ({} samples)", self.cpu_history.len()));
            self.render_sparkline(ui, &self.cpu_history, 100.0);
        }
        
        ui.add_space(10.0);
    }
    
    /// Render memory section
    fn render_memory(&self, ui: &mut egui::Ui) {
        ui.heading("🧠 Memory");
        
        let used = self.system.used_memory();
        let total = self.system.total_memory();
        let percent = (used as f64 / total as f64) * 100.0;
        
        ui.label(format!("Used: {} / {} GB", 
            used / 1_073_741_824,  // Convert to GB
            total / 1_073_741_824
        ));
        
        ui.add(egui::ProgressBar::new(percent as f32 / 100.0)
            .text(format!("{:.1}%", percent)));
        
        // History sparkline
        if !self.mem_history.is_empty() {
            ui.label(format!("History ({} samples)", self.mem_history.len()));
            self.render_sparkline(ui, &self.mem_history, 100.0);
        }
        
        ui.add_space(10.0);
    }
    
    /// Render disk section
    fn render_disk(&self, ui: &mut egui::Ui) {
        ui.heading("💾 Disk");
        
        for disk in self.system.disks() {
            let name = disk.name().to_string_lossy();
            let used = disk.total_space() - disk.available_space();
            let total = disk.total_space();
            let percent = (used as f64 / total as f64) * 100.0;
            
            ui.label(format!("{}: {:.1}%", name, percent));
            ui.add(egui::ProgressBar::new(percent as f32 / 100.0)
                .text(format!("{} / {} GB", 
                    used / 1_073_741_824,
                    total / 1_073_741_824
                )));
        }
        
        ui.add_space(10.0);
    }
    
    /// Render process list (top 5 by CPU)
    fn render_processes(&self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Top Processes");
        
        let mut processes: Vec<_> = self.system.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
        
        for process in processes.iter().take(5) {
            ui.horizontal(|ui| {
                ui.label(format!("{:20}", process.name()));
                ui.label(format!("CPU: {:.1}%", process.cpu_usage()));
                ui.label(format!("Mem: {} MB", process.memory() / 1_048_576));
            });
        }
    }
    
    /// Render a simple sparkline chart
    fn render_sparkline(&self, ui: &mut egui::Ui, data: &[f32], max_value: f32) {
        use egui::{Color32, Pos2, Stroke};
        
        let height = 50.0;
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), height),
            egui::Sense::hover()
        );
        
        let rect = response.rect;
        let points: Vec<Pos2> = data
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let x = rect.left() + (i as f32 / (data.len() - 1).max(1) as f32) * rect.width();
                let y = rect.bottom() - (value / max_value) * rect.height();
                Pos2::new(x, y)
            })
            .collect();
        
        if points.len() >= 2 {
            painter.add(egui::Shape::line(
                points,
                Stroke::new(2.0, Color32::from_rgb(100, 200, 255))
            ));
        }
    }
}

impl ToolPanel for SystemMonitorTool {
    fn metadata(&self) -> &ToolMetadata {
        static METADATA: std::sync::OnceLock<ToolMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| ToolMetadata {
            name: "System Monitor".to_string(),
            description: "Real-time system resource monitoring".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec![
                ToolCapability::Visual,
                ToolCapability::Custom("RealTime".to_string()),
            ],
            icon: "📡".to_string(),
            source: Some("https://github.com/GuillaumeGomez/sysinfo".to_string()),
        })
    }
    
    fn is_visible(&self) -> bool {
        self.show_panel
    }
    
    fn toggle_visibility(&mut self) {
        self.show_panel = !self.show_panel;
    }
    
    fn render_panel(&mut self, ui: &mut egui::Ui) {
        // Refresh system data
        self.refresh();
        
        // Header
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(egui::RichText::new("📡 System Monitor").size(24.0));
            ui.label(
                egui::RichText::new("Real-time system resource monitoring")
                    .size(14.0)
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(10.0);
        });
        
        ui.separator();
        ui.add_space(10.0);
        
        // Main content in scroll area
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 30, 35))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    self.render_cpu(ui);
                    ui.separator();
                    self.render_memory(ui);
                    ui.separator();
                    self.render_disk(ui);
                    ui.separator();
                    self.render_processes(ui);
                });
        });
        
        // Request continuous repaint for live updates
        ui.ctx().request_repaint();
    }
    
    fn status_message(&self) -> Option<String> {
        let cpu = self.system.global_cpu_info().cpu_usage();
        let mem = (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0;
        Some(format!("CPU: {:.1}% | MEM: {:.1}%", cpu, mem))
    }
}
```

---

## Step 4: Register in lib.rs

```rust
// In crates/petal-tongue-ui/src/lib.rs
pub mod system_monitor_integration;
```

---

## Step 5: Register in app.rs

```rust
// In crates/petal-tongue-ui/src/app.rs
use crate::system_monitor_integration::SystemMonitorTool;

impl PetalTongueApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // ... existing code ...
        
        // Register tools
        app.tools.register_tool(Box::new(BingoCubeIntegration::new()));
        app.tools.register_tool(Box::new(SystemMonitorTool::default()));  // ✅ NEW!
        
        app
    }
}
```

---

## Step 6: Build and Test

```bash
cd /path/to/petalTongue
cargo build -p petal-tongue-ui
cargo run -p petal-tongue-ui
```

---

## Step 7: Verify Integration

1. **Tools Menu**: Click "🔧 Tools" → Should see:
   - 🎲 BingoCube
   - 📡 System Monitor ✨ NEW!

2. **Toggle Tool**: Click "System Monitor" → Should see real-time:
   - CPU usage graph
   - Memory usage graph
   - Disk usage bars
   - Top 5 processes

3. **Status**: Hover over tool → Should see: "CPU: X% | MEM: Y%"

4. **No Hardcoding**: The app doesn't know it's a system monitor - just a ToolPanel!

---

## Success Criteria

✅ Tool appears in Tools menu  
✅ Toggle shows/hides panel  
✅ Real-time updates (every second)  
✅ CPU/Memory graphs display  
✅ Top processes listed  
✅ Status message in menu  
✅ No changes to app.rs logic (just registration)  
✅ Works alongside BingoCube (both can be visible)

---

## Next Steps

After System Monitor works:

1. **Document it** - Add to showcase examples
2. **Create demo video** - Show it working
3. **Blog post** - "How to integrate external tools with petalTongue"
4. **Pick next tool** - Maybe Plotters or Log Viewer
5. **Repeat** - Build momentum with 2-3 more tools

---

## Optional Enhancements

### Add Configuration Panel

```rust
// In SystemMonitorTool
fn render_config(&mut self, ui: &mut egui::Ui) {
    ui.heading("⚙️ Configuration");
    
    ui.horizontal(|ui| {
        ui.label("Refresh Rate:");
        let mut secs = self.refresh_interval.as_secs() as f32;
        if ui.add(egui::Slider::new(&mut secs, 0.1..=10.0).text("seconds")).changed() {
            self.refresh_interval = Duration::from_secs_f32(secs);
        }
    });
    
    ui.horizontal(|ui| {
        ui.label("History Length:");
        ui.add(egui::Slider::new(&mut self.max_history, 10..=300).text("samples"));
    });
}
```

### Export to CSV

```rust
impl SystemMonitorTool {
    fn export_csv(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path("system_monitor_export.csv")?;
        wtr.write_record(&["timestamp", "cpu_percent", "mem_percent"])?;
        
        for (i, (&cpu, &mem)) in self.cpu_history.iter().zip(&self.mem_history).enumerate() {
            wtr.write_record(&[i.to_string(), cpu.to_string(), mem.to_string()])?;
        }
        
        wtr.flush()?;
        Ok(())
    }
}
```

### Alert on High Usage

```rust
impl SystemMonitorTool {
    fn check_alerts(&self) -> Option<String> {
        let cpu = self.system.global_cpu_info().cpu_usage();
        let mem = (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0;
        
        if cpu > 90.0 {
            return Some(format!("⚠️ High CPU usage: {:.1}%", cpu));
        }
        if mem > 90.0 {
            return Some(format!("⚠️ High memory usage: {:.1}%", mem));
        }
        None
    }
}
```

---

## Estimated Timeline

- **Setup**: 10 minutes (dependency, file creation)
- **Implementation**: 30 minutes (code the tool)
- **Registration**: 5 minutes (lib.rs, app.rs)
- **Testing**: 15 minutes (build, run, verify)
- **Total**: ~1 hour

---

**Status**: Ready to implement  
**Difficulty**: ⭐ Easy  
**Impact**: 🔥🔥🔥 Proves the pattern works!

---

*"The first external tool integration proves the pattern. The second makes it a habit. The third makes it a platform."*

