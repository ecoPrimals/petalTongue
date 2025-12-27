# ToadStool Bridge Pattern for Python Tools

**Date**: December 26, 2025  
**Status**: 📋 **DESIGN**  
**Next Phase**: After 2-3 more Rust tools

---

## Vision

**petalTongue should NOT hardcode Python knowledge.**  
Instead, **ToadStool** (the compute primal) handles running Python tools as capabilities.

---

## The Pattern

### Primal Separation of Concerns

```
┌─────────────────────────────────────────────────┐
│ petalTongue (UI/Visualization Primal)           │
│ ├─ Knows: Rendering, egui, visual patterns     │
│ ├─ Discovers: Tool capabilities at runtime      │
│ └─ Delegates: Compute to ToadStool              │
└─────────────────────────────────────────────────┘
                    │
                    │ Capability Discovery
                    ▼
┌─────────────────────────────────────────────────┐
│ ToadStool (Compute Primal)                      │
│ ├─ Knows: Process spawning, IPC, Python runtime│
│ ├─ Advertises: "python-tool-runner" capability │
│ └─ Provides: Execute, stream results, lifecycle│
└─────────────────────────────────────────────────┘
                    │
                    │ Spawns/Manages
                    ▼
┌─────────────────────────────────────────────────┐
│ Python Tools (External Scripts/Packages)        │
│ ├─ matplotlib, numpy, pandas, scikit-learn     │
│ ├─ Custom analysis scripts                     │
│ └─ Implements: Tool protocol (stdin/stdout)     │
└─────────────────────────────────────────────────┘
```

---

## Why This Pattern?

### ❌ What We Avoid (Anti-Pattern)

```rust
// DON'T DO THIS in petalTongue
use pyo3::prelude::*;

impl PetalTongueApp {
    fn run_python_tool(&self) {
        Python::with_gil(|py| {
            let tool = py.import("matplotlib")?;
            // ... hardcoded Python knowledge
        })
    }
}
```

**Problems:**
- petalTongue now depends on Python runtime
- Violates single responsibility (UI primal doing compute)
- Can't discover Python tools dynamically
- Breaks when Python isn't installed

### ✅ What We Do Instead (Primal Pattern)

```rust
// petalTongue discovers ToadStool's capabilities
impl PetalTongueApp {
    fn discover_compute_capabilities(&mut self) {
        // Ask BiomeOS: "Who can run python-tools?"
        let compute_primals = self.biomeos_client
            .discover_capability("python-tool-runner")
            .await?;
        
        if let Some(toadstool) = compute_primals.first() {
            // Register ToadStool bridge as a tool provider
            let bridge = ToadStoolBridge::new(toadstool.endpoint);
            self.tool_manager.register_tool_provider(bridge);
        }
    }
}
```

**Benefits:**
- petalTongue stays pure UI/visualization
- ToadStool does what it's meant to do (compute)
- Works even if ToadStool is on another machine
- Python optional - system gracefully degrades

---

## Implementation Design

### Phase 1: ToadStool Bridge (Rust in petalTongue)

**File:** `crates/petal-tongue-ui/src/toadstool_bridge.rs`

```rust
/// Bridge to ToadStool compute primal for running Python tools
///
/// This is NOT a tool itself - it's a tool PROVIDER that discovers
/// Python tools via ToadStool's capabilities.
pub struct ToadStoolBridge {
    toadstool_endpoint: String,
    http_client: reqwest::Client,
    discovered_tools: Vec<ToolMetadata>,
}

impl ToadStoolBridge {
    pub async fn new(endpoint: String) -> Result<Self> {
        let http_client = reqwest::Client::new();
        let mut bridge = Self {
            toadstool_endpoint: endpoint,
            http_client,
            discovered_tools: Vec::new(),
        };
        
        // Discover what Python tools ToadStool can run
        bridge.refresh_available_tools().await?;
        
        Ok(bridge)
    }
    
    /// Ask ToadStool: "What Python tools do you support?"
    async fn refresh_available_tools(&mut self) -> Result<()> {
        let response: Vec<ToolMetadata> = self.http_client
            .get(format!("{}/api/tools/list", self.toadstool_endpoint))
            .send()
            .await?
            .json()
            .await?;
        
        self.discovered_tools = response;
        Ok(())
    }
    
    /// Execute a Python tool via ToadStool
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<ToolOutput> {
        let response = self.http_client
            .post(format!("{}/api/tools/execute", self.toadstool_endpoint))
            .json(&ExecuteRequest {
                tool_name: tool_name.to_string(),
                input,
            })
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response)
    }
}
```

---

### Phase 2: Python Tool Protocol (in ToadStool repo)

**File:** `toadstool/python_tools/protocol.py`

```python
"""
Standard protocol for Python tools that integrate with petalTongue via ToadStool.

Tools implement this interface and ToadStool discovers/runs them.
"""

from abc import ABC, abstractmethod
from typing import Any, Dict, List
import json
import sys

class ToolCapability:
    VISUAL = "visual"
    COMPUTE = "compute"
    EXPORT = "export"
    STREAMING = "streaming"

class ToolPanel(ABC):
    """Python tools implement this to integrate with petalTongue"""
    
    @abstractmethod
    def metadata(self) -> Dict[str, Any]:
        """Return tool metadata"""
        return {
            "name": "MyTool",
            "description": "What this tool does",
            "version": "0.1.0",
            "capabilities": [ToolCapability.VISUAL, ToolCapability.COMPUTE],
            "icon": "📊",
            "source": "https://github.com/org/repo",
        }
    
    @abstractmethod
    def execute(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Execute tool with input data, return results.
        
        Results can include:
        - plot_data: Base64-encoded image
        - table_data: JSON array of rows
        - metrics: Key-value pairs
        - error: Error message if failed
        """
        pass

# Example: Matplotlib plotting tool
class MatplotlibPlotter(ToolPanel):
    def metadata(self):
        return {
            "name": "Matplotlib Plotter",
            "description": "Create scientific plots from data",
            "version": "0.1.0",
            "capabilities": [ToolCapability.VISUAL, ToolCapability.EXPORT],
            "icon": "📈",
            "source": "https://matplotlib.org",
        }
    
    def execute(self, input_data):
        import matplotlib.pyplot as plt
        import io
        import base64
        
        # Get data from input
        x = input_data.get("x", [])
        y = input_data.get("y", [])
        
        # Create plot
        fig, ax = plt.subplots()
        ax.plot(x, y)
        ax.set_title(input_data.get("title", "Plot"))
        
        # Convert to base64 for transport
        buf = io.BytesIO()
        fig.savefig(buf, format='png')
        buf.seek(0)
        img_base64 = base64.b64encode(buf.read()).decode('utf-8')
        
        return {
            "plot_data": img_base64,
            "width": 800,
            "height": 600,
        }

# Standard entry point
if __name__ == "__main__":
    tool = MatplotlibPlotter()
    
    # Read input from stdin
    input_data = json.loads(sys.stdin.read())
    
    # Execute
    result = tool.execute(input_data)
    
    # Write output to stdout
    print(json.dumps(result))
```

---

### Phase 3: ToadStool API (in ToadStool repo)

**Endpoints ToadStool Provides:**

```
GET  /api/tools/list
  → Returns: [{ name, description, version, capabilities, ... }]

POST /api/tools/execute
  → Body: { tool_name: "matplotlib-plotter", input: {...} }
  → Returns: { status: "success", output: {...} }

GET  /api/tools/status/{execution_id}
  → Returns: { status: "running" | "complete" | "error", progress: 0.0-1.0 }

POST /api/tools/cancel/{execution_id}
  → Cancels a running tool execution
```

---

### Phase 4: petalTongue Integration

**How it works in petalTongue:**

1. **Startup Discovery**
   ```rust
   // In app.rs initialization
   if let Ok(toadstool_bridge) = ToadStoolBridge::new("http://localhost:4000").await {
       // ToadStool discovered and provides Python tools
       for tool_meta in toadstool_bridge.discovered_tools() {
           // Wrap each Python tool in a PythonToolPanel
           let python_tool = PythonToolPanel::new(
               tool_meta.clone(),
               toadstool_bridge.clone(),
           );
           app.tools.register_tool(Box::new(python_tool));
       }
   }
   ```

2. **Rendering**
   ```rust
   // When user clicks Python tool toggle
   // PythonToolPanel::render_panel() is called
   
   impl ToolPanel for PythonToolPanel {
       fn render_panel(&mut self, ui: &mut egui::Ui) {
           // Show input controls
           ui.text_edit_singleline(&mut self.input_x);
           ui.text_edit_singleline(&mut self.input_y);
           
           if ui.button("Generate Plot").clicked() {
               // Send to ToadStool
               let result = self.bridge.execute_tool(
                   &self.metadata.name,
                   json!({ "x": self.input_x, "y": self.input_y })
               ).await?;
               
               // Decode base64 image and display
               let img = egui::ColorImage::from_rgba_unmultiplied(...);
               self.cached_image = Some(img);
           }
           
           // Display cached result
           if let Some(img) = &self.cached_image {
               ui.image(img);
           }
       }
   }
   ```

---

## Example Use Cases

### 1. Data Analysis Dashboard
```
User: Uploads CSV via petalTongue UI
  ↓
petalTongue: Discovers ToadStool has "pandas-analysis" tool
  ↓
petalTongue: Sends CSV + query to ToadStool
  ↓
ToadStool: Runs Python pandas script
  ↓
ToadStool: Returns summary stats + plot
  ↓
petalTongue: Displays results in UI
```

### 2. Machine Learning Model Viewer
```
User: Selects "scikit-learn model inspector" tool
  ↓
petalTongue: Requests model from ToadStool
  ↓
ToadStool: Loads model, generates feature importance
  ↓
ToadStool: Returns plot + metrics
  ↓
petalTongue: Renders interactive visualization
```

### 3. Custom Analysis Script
```
User: Writes custom Python script in ToadStool
  ↓
ToadStool: Registers script as new tool
  ↓
ToadStool: Advertises capability to BiomeOS
  ↓
petalTongue: Discovers new tool automatically
  ↓
petalTongue: Shows in tool menu (no restart needed!)
```

---

## Sovereignty & Security

### Trust Boundaries

1. **petalTongue trusts ToadStool** (same user's primals)
2. **ToadStool sandboxes Python** (process isolation)
3. **Python tools are user-controlled** (no remote code execution)

### User Control

- ✅ User sees what tools are available
- ✅ User explicitly clicks to run
- ✅ User can cancel long-running tasks
- ✅ User can inspect Python source
- ✅ Logs show all compute requests

---

## Development Roadmap

### Phase 1: Rust Tools (Current) ✅
- [x] System Monitor
- [ ] 2-3 more pure Rust tools
- [ ] Pattern proven stable

### Phase 2: ToadStool Bridge (Next)
- [ ] Design ToadStool API
- [ ] Implement `ToadStoolBridge` in petalTongue
- [ ] Create Python tool protocol
- [ ] Test with simple Python tool (matplotlib)

### Phase 3: Python Tool Ecosystem
- [ ] pandas data viewer
- [ ] scikit-learn model inspector
- [ ] Custom script runner
- [ ] Tool template repository

### Phase 4: Community Expansion
- [ ] Tool registry (BiomeOS?)
- [ ] Community-contributed tools
- [ ] Tool marketplace concept

---

## Technical Decisions

### IPC Method: HTTP/JSON
**Why not:**
- ❌ Shared memory (complex, unsafe)
- ❌ gRPC (overkill, adds complexity)
- ❌ PyO3 (couples petalTongue to Python)

**Why HTTP/JSON:**
- ✅ Simple, well-understood
- ✅ Works across machines
- ✅ Language-agnostic
- ✅ Easy to debug (curl)
- ✅ Matches existing BiomeOS patterns

### Data Encoding: Base64 for Images
- Handles binary data in JSON
- Widely supported
- Small overhead acceptable for occasional images

### Process Model: One Python Process Per ToadStool
- ToadStool manages Python runtime lifecycle
- Reuses interpreter for multiple tool runs
- petalTongue never spawns processes directly

---

## Success Metrics

### Pattern Validation
- ✅ petalTongue has **zero** Python dependencies
- ✅ Works without Python installed (tools just don't appear)
- ✅ Python tools dynamically discovered
- ✅ No code changes to add new Python tools

### Performance
- Startup time: <500ms for tool discovery
- Execution time: Python-native (no overhead)
- UI responsiveness: Non-blocking async

### Developer Experience
- Python tool template: <50 lines
- Time to add new tool: <30 minutes
- No Rust knowledge required for Python tool authors

---

## Related Documents

- `SYSTEM_MONITOR_COMPLETE.md` - First Rust tool integration
- `EXTERNAL_TOOL_INTEGRATION_SHOWCASE.md` - Overall tool strategy
- `CAPABILITY_BASED_TOOL_PATTERN_COMPLETE.md` - Core pattern docs

---

## Next Steps

1. **Complete 2-3 more Rust tools** (prove pattern scales)
2. **Design ToadStool API spec** (with ToadStool team)
3. **Implement `ToadStoolBridge`** in petalTongue
4. **Create first Python tool** (matplotlib plotter)
5. **Document Python tool template** for community

---

**🔬 ToadStool: The Compute Primal**  
*Bringing Python's ecosystem to petalTongue without compromising primal sovereignty*

