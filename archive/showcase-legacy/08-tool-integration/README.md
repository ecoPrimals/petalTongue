# 🔧 Tool Integration - Dynamic Extensions

**Integrate external tools at runtime via capability discovery**

---

## 🎯 What You'll Learn

In **10 minutes**:
- Dynamic tool integration
- Capability-based discovery
- Plugin architecture
- Zero hardcoding

---

## ⏱️ Duration

**10 minutes**

---

## 📋 Prerequisites

- petalTongue built
- Completed: 00-hello through 07-audio-export

---

## 🚀 Run the Demo

```bash
./demo.sh
```

---

## 🎓 Tool Integration Philosophy

### **The Principle**

> **"Discover capabilities at runtime, never hardcode."**

petalTongue doesn't **know** about tools ahead of time:
- ✗ No hardcoded tool list
- ✗ No compiled-in tool ports
- ✗ No assumed tool locations
- ✓ **Runtime discovery via capabilities**

### **How It Works**

```
1. Tool announces: "I can visualize graphs"
2. petalTongue discovers: "Found visualization capability"
3. User activates: "Use this tool"
4. Integration happens: Dynamic, runtime
```

**Zero assumptions!**

---

## 🔍 Built-In Tool Panels

petalTongue includes several tool integrations:

### **1. System Monitor**

```
┌─ System Monitor ────────────┐
│ CPU Usage: [▓▓▓▓▓░░░] 65%  │
│ Memory: [▓▓▓░░░░░] 3.2/8GB │
│ Disk I/O: 45 MB/s           │
│                             │
│ Capability: "system.monitor"│
└─────────────────────────────┘
```

**Uses**: `sysinfo` crate (cross-platform)

### **2. Process Viewer**

```
┌─ Process Viewer ────────────┐
│ PID    Name         CPU  Mem│
│ 1234   petaltongue  23%  45M│
│ 5678   biomeos      12%  32M│
│ 9012   songbird      5%  18M│
│                             │
│ Capability: "process.view"  │
└─────────────────────────────┘
```

**Uses**: `sysinfo` crate (cross-platform)

### **3. Graph Metrics**

```
┌─ Graph Metrics ─────────────┐
│ Nodes: 12 (↑2 from last)   │
│ Edges: 34 (↓1 from last)   │
│ Density: 0.47               │
│ Avg Degree: 2.83            │
│                             │
│ Capability: "graph.metrics" │
└─────────────────────────────┘
```

**Uses**: `petgraph` analysis

### **4. BingoCube Integration**

```
┌─ BingoCube Visualizer ──────┐
│ Status: Connected ✓         │
│ Visualizations: 3 active    │
│ • Graph 3D                  │
│ • Heatmap                   │
│ • Timeline                  │
│                             │
│ Capability: "bingocube.viz" │
└─────────────────────────────┘
```

**Uses**: BingoCube adapters

### **5. ToadStool Bridge**

```
┌─ ToadStool Tools ───────────┐
│ Python Tools Discovered: 2  │
│ • GraphAnalyzer             │
│ • DataExporter              │
│                             │
│ Status: Python 3.11 ✓       │
│                             │
│ Capability: "toadstool.exec"│
└─────────────────────────────┘
```

**Uses**: ToadStool integration

---

## 📊 What This Demonstrates

1. ✅ **Dynamic Discovery** - Find tools at runtime
2. ✅ **Capability-Based** - Match by capability, not name
3. ✅ **Zero Hardcoding** - No compiled-in assumptions
4. ✅ **Plugin Architecture** - Extensible design
5. ✅ **Graceful Degradation** - Missing tools = Hidden panels

---

## 🧮 Technical Implementation

### **ToolPanel Trait**

```rust
pub trait ToolPanel: Send + Sync {
    /// Tool name (for display)
    fn name(&self) -> &str;
    
    /// Capabilities this tool provides
    fn capabilities(&self) -> Vec<String>;
    
    /// Check if tool is available
    fn is_available(&self) -> bool;
    
    /// Render tool UI
    fn render(&mut self, ui: &mut egui::Ui);
    
    /// Handle tool actions
    fn handle_action(&mut self, action: &str) -> Result<()>;
}
```

### **Tool Manager**

```rust
pub struct ToolManager {
    tools: Vec<Box<dyn ToolPanel>>,
}

impl ToolManager {
    pub fn discover_tools() -> Self {
        let mut tools: Vec<Box<dyn ToolPanel>> = vec![];
        
        // Try system monitor
        if let Ok(monitor) = SystemMonitor::try_new() {
            tools.push(Box::new(monitor));
        }
        
        // Try BingoCube
        if let Ok(bingocube) = BingoCubePanel::try_new() {
            tools.push(Box::new(bingocube));
        }
        
        // Try ToadStool
        if let Ok(toadstool) = ToadStoolBridge::try_new() {
            tools.push(Box::new(toadstool));
        }
        
        Self { tools }
    }
    
    pub fn tools_with_capability(&self, cap: &str) -> Vec<&dyn ToolPanel> {
        self.tools
            .iter()
            .filter(|t| t.capabilities().contains(&cap.to_string()))
            .map(|t| t.as_ref())
            .collect()
    }
}
```

### **Runtime Discovery**

```rust
// At startup
let tool_manager = ToolManager::discover_tools();

// Show only available tools
for tool in tool_manager.available_tools() {
    ui.collapsing(tool.name(), |ui| {
        tool.render(ui);
    });
}
```

**No hardcoding!**

---

## 💡 Try This

### **1. View Available Tools**

In the UI:
1. Look for "Tools" panel or sidebar
2. See which tools were discovered
3. Each shows its capability
4. Unavailable tools are hidden

### **2. Use System Monitor**

1. Open "System Monitor" panel
2. Watch real-time CPU/memory
3. See sparkline graphs
4. Observe system load

### **3. Try BingoCube (if available)**

1. Open "BingoCube" panel
2. See visualization status
3. Activate visualization
4. Watch 3D graph rendering

### **4. Explore Capabilities**

Each tool declares:
- What it can do
- What it needs
- Whether it's available

**Transparency!**

---

## 🎯 Design Patterns

### **1. Try, Don't Assume**

```rust
// ✗ BAD: Assume tool exists
let tool = SystemMonitor::new();  // Panics if unavailable

// ✓ GOOD: Try and handle
if let Ok(tool) = SystemMonitor::try_new() {
    // Use tool
} else {
    // Degrade gracefully
}
```

### **2. Capability-Based Matching**

```rust
// ✗ BAD: Match by name
if tool.name() == "BingoCube" {
    // Brittle, breaks if renamed
}

// ✓ GOOD: Match by capability
if tool.capabilities().contains("visualization.3d") {
    // Robust, semantic
}
```

### **3. Graceful Degradation**

```rust
// ✗ BAD: Show error panel
ui.label("ERROR: Tool not found!");

// ✓ GOOD: Simply hide panel
if tool.is_available() {
    ui.collapsing(tool.name(), |ui| {
        tool.render(ui);
    });
}
// If unavailable, nothing shown (clean UX)
```

---

## 🐛 Troubleshooting

### **Expected tool not showing**

Reasons:
- Tool not installed
- Dependencies missing
- Capability detection failed

**This is correct behavior!** Check capability report.

### **Tool shows "unavailable"**

Check:
- System requirements
- Dependencies installed
- Configuration correct

**Honest reporting working as designed.**

---

## 🎯 Success Criteria

You've mastered tool integration when you:
- ✅ Understand runtime discovery
- ✅ Appreciate capability-based design
- ✅ See why zero hardcoding matters
- ✅ Can add new tools via trait
- ✅ Recognize extensible architecture

---

## ➡️ Next Steps

**Congratulations!** You've completed **Phase 1: Local Primal** (9/9 demos).

```bash
cd ../../02-biomeos-integration/
cat README.md
```

**Next Phase**: Learn BiomeOS orchestration and inter-primal coordination.

---

## 📚 Adding New Tools

### **Step 1: Implement ToolPanel**

```rust
pub struct MyTool {
    // Tool state
}

impl ToolPanel for MyTool {
    fn name(&self) -> &str {
        "My Tool"
    }
    
    fn capabilities(&self) -> Vec<String> {
        vec!["my.capability".to_string()]
    }
    
    fn is_available(&self) -> bool {
        // Check if tool can run
        true
    }
    
    fn render(&mut self, ui: &mut egui::Ui) {
        // Draw UI
    }
    
    fn handle_action(&mut self, action: &str) -> Result<()> {
        // Handle actions
        Ok(())
    }
}
```

### **Step 2: Add to Discovery**

```rust
impl ToolManager {
    pub fn discover_tools() -> Self {
        let mut tools = vec![];
        
        // Existing tools...
        
        // Add your tool
        if let Ok(my_tool) = MyTool::try_new() {
            tools.push(Box::new(my_tool));
        }
        
        Self { tools }
    }
}
```

### **Step 3: Done!**

Your tool is now:
- ✓ Discovered at runtime
- ✓ Shown if available
- ✓ Hidden if unavailable
- ✓ Integrated seamlessly

**That's it!**

---

## 🌟 Key Takeaway

**Dynamic tool integration enables extensibility without recompilation.**

Benefits:
- ✅ Zero hardcoded tool knowledge
- ✅ Runtime capability discovery
- ✅ Graceful degradation
- ✅ Easy to add new tools
- ✅ Robust plugin architecture

**This is how sovereign software grows!**

---

*"Discover, don't dictate."* 🌸

