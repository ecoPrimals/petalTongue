# 🌸 petalTongue Rich TUI

**A pure Rust terminal user interface for managing the ecoPrimals ecosystem**

> "petalTongue renders. Other primals provide capabilities."

---

## 🎯 Overview

The Rich TUI is a comprehensive, terminal-based interface for managing primals, devices, topology, and biomeOS systems. It provides 8 interactive views with real-time updates, graceful degradation, and zero unsafe code.

### Key Features

✅ **8 Interactive Views** - Dashboard, Topology, Devices, Primals, Logs, neuralAPI, NUCLEUS, LiveSpore  
✅ **Pure Rust** - Zero C dependencies, zero unsafe code  
✅ **Capability-Based** - Runtime primal discovery, graceful degradation  
✅ **biomeOS Integration** - neuralAPI, NUCLEUS, liveSpore management  
✅ **Standalone Mode** - Works with or without other primals  
✅ **Keyboard Navigation** - Full keyboard control, vim-style keys  

---

## 🚀 Quick Start

### Run the TUI

```bash
# From petalTongue root
cargo run -p petal-tongue-tui --example simple_demo

# Or build and run
cargo build --release -p petal-tongue-tui
./target/release/petal-tongue-tui
```

### Keyboard Shortcuts

**View Navigation:**
- `1-8` - Switch between views
- `q` or `Ctrl+C` - Quit

**List Navigation:**
- `↑/k` or `↓/j` - Navigate up/down
- `Home` - Jump to top
- `End` - Jump to bottom
- `Page Up/Down` - Scroll page

**Actions:**
- `r` - Refresh data
- `?` - Show help
- `Enter` - Select/activate item

---

## 📊 The 8 Views

### 1. Dashboard (`1`)

**System Overview**

- Active primals count
- Topology edge count
- Recent log count
- Primal health status
- Quick topology summary

**Standalone Mode:** Shows "Standalone Mode" message with helpful tips.

### 2. Topology (`2`)

**ASCII Art Graph Visualization**

- Node boxes with health icons
- Edge connections with types
- Graph statistics (nodes, edges)
- Edge type breakdown

**Leverages:** Songbird (topology data)  
**Standalone Mode:** Shows helpful message about starting Songbird.

### 3. Devices (`3`)

**Device Management**

- Discovered devices list
- Device availability status
- Assignment interface
- Device details

**Leverages:** Songbird (device discovery)  
**Standalone Mode:** Shows helpful message about device discovery.

### 4. Primals (`4`)

**Primal Status Monitoring**

- Primal list with health icons
  - ✅ Healthy
  - ⚠️ Warning
  - ❌ Critical
  - ❓ Unknown
- Detailed primal information
- Capability display
- Selection navigation

**Leverages:** Songbird (primal discovery)  
**Standalone Mode:** Shows helpful message about starting primals.

### 5. Logs (`5`)

**Real-Time Log Streaming**

- Color-coded log levels
  - ❌ Error (Red)
  - ⚠️ Warning (Yellow)
  - ℹ️ Info (Cyan)
  - 🐛 Debug (Magenta)
  - 🔍 Trace (Gray)
- Timestamp and source display
- Ring buffer (1000 max logs)
- Newest first ordering

**Leverages:** Songbird (event stream)  
**Standalone Mode:** Shows system logs only.

### 6. neuralAPI (`6`)

**Graph Orchestration for biomeOS**

- Neural graph definitions
- Execution status tracking
- Node execution details
- Graph management actions

**Leverages:** biomeOS neuralAPI  
**Standalone Mode:** Shows integration information.

### 7. NUCLEUS (`7`)

**Secure Discovery Management**

- Multi-layer discovery
  - Layer 1: Local Filesystem (High Trust)
  - Layer 2: Network/DNS-SD (Medium Trust)
  - Layer 3: External (Low Trust, verify)
- Trust matrix
- Security policies
- Capability verification

**Leverages:** biomeOS NUCLEUS  
**Standalone Mode:** Shows security architecture info.

### 8. LiveSpore (`8`)

**Live Deployment Management**

- Atomic deployment pipeline
  - Tower (BearDog + Songbird)
  - Node (Tower + ToadStool)
  - Nest (Tower + NestGate)
  - NUCLEUS (All atomics)
- Node availability
- Deployment status
- Deployment actions

**Leverages:** biomeOS liveSpore  
**Standalone Mode:** Shows deployment architecture info.

---

## 🏗️ Architecture

### Division of Labor

**petalTongue OWNS:**
- Rendering (TUI framework, ASCII art)
- Layout management
- Event handling
- User interaction

**petalTongue LEVERAGES:**
- **Songbird** → Discovery, topology, events
- **ToadStool** → GPU compute (optional)
- **NestGate** → Preferences (optional)
- **BearDog** → Authentication (optional)
- **biomeOS** → neuralAPI, NUCLEUS, liveSpore

### Graceful Degradation

All primal clients are `Optional<T>`:

```rust
// Tries to leverage Songbird
if let Ok(discovery) = DiscoveryServiceClient::discover().await {
    let topology = songbird.get_topology().await?;
    ui.render_topology(topology)?;
} else {
    // Falls back to standalone mode
    ui.render_standalone_mode()?;
}
```

**Works with 0 primals** → Standalone mode  
**Works with 1-N primals** → Enhanced mode

---

## 🛠️ Development

### Project Structure

```
crates/petal-tongue-tui/
├── src/
│   ├── lib.rs              # Public API
│   ├── state.rs            # TUI state management
│   ├── events.rs           # Event system
│   ├── app.rs              # Main application
│   ├── layout.rs           # Layout utilities
│   ├── widgets/            # Reusable widgets
│   │   ├── header.rs
│   │   ├── footer.rs
│   │   └── status.rs
│   └── views/              # 8 interactive views
│       ├── dashboard.rs
│       ├── topology.rs
│       ├── logs.rs
│       ├── devices.rs
│       ├── primals.rs
│       ├── neural_api.rs
│       ├── nucleus.rs
│       └── livespore.rs
├── examples/
│   └── simple_demo.rs      # Demo application
├── Cargo.toml
└── README.md               # This file
```

### Dependencies

```toml
[dependencies]
ratatui = "0.28"           # TUI framework
crossterm = "0.28"         # Terminal control
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
async-trait = "0.1"
tracing = "0.1"
dashmap = "6.1"            # Concurrent state
chrono = "0.4"             # Timestamps

petal-tongue-core = { path = "../petal-tongue-core" }
```

### Building

```bash
# Check compilation
cargo check -p petal-tongue-tui

# Run tests
cargo test -p petal-tongue-tui

# Build release
cargo build --release -p petal-tongue-tui

# Run example
cargo run -p petal-tongue-tui --example simple_demo
```

### Testing

```bash
# Unit tests (13 tests)
cargo test -p petal-tongue-tui

# Test specific module
cargo test -p petal-tongue-tui state::tests
cargo test -p petal-tongue-tui events::tests
```

---

## 🎨 Customization

### Configuration

Create a `TUIConfig`:

```rust
use petal_tongue_tui::{TUIConfig, RichTUI};
use std::time::Duration;

let config = TUIConfig {
    tick_rate: Duration::from_millis(100),  // Refresh rate
    mouse_support: false,                    // Enable mouse
    standalone: false,                       // Force standalone mode
};

let mut tui = RichTUI::with_config(config).await?;
tui.run().await?;
```

### Adding Custom Views

1. Create a new view module in `src/views/`:

```rust
// src/views/my_view.rs
use ratatui::{Frame, layout::Rect};
use crate::state::TUIState;

pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    // Your rendering logic
}
```

2. Add to `src/views/mod.rs`:

```rust
mod my_view;

pub fn render_my_view(frame: &mut Frame, state: &TUIState) {
    // Wire up your view
    my_view::render(frame, layout.body, state);
}
```

3. Add to `View` enum in `src/state.rs`

---

## 🌸 TRUE PRIMAL Principles

### Zero Hardcoding

✅ Runtime primal discovery  
✅ Environment-based configuration  
✅ No hardcoded endpoints

### Capability-Based

✅ Adapts to available primals  
✅ Progressive enhancement  
✅ Graceful degradation

### Self-Knowledge

✅ Knows rendering domain  
✅ No orchestration logic  
✅ Clean separation of concerns

### Agnostic

✅ No assumptions about other primals  
✅ Works alone or with others  
✅ Discovers at runtime

### Fast AND Safe

✅ **2,490 LOC of zero unsafe code**  
✅ Concurrent state management  
✅ Efficient rendering

---

## 📈 Metrics

- **Total LOC:** 2,490
- **Unsafe Code:** 0 lines
- **Tests:** 13 (all passing)
- **Modules:** 18
- **Views:** 8 (all complete)
- **Widgets:** 3
- **Dependencies:** Pure Rust only

---

## 🤝 Integration with biomeOS

### neuralAPI

The TUI can display and manage neural graphs defined in biomeOS:

```rust
// biomeOS defines graphs
// TUI displays and manages them
```

### NUCLEUS

Secure discovery with trust scoring:

```rust
// NUCLEUS provides discovery layers
// TUI visualizes trust matrix
```

### liveSpore

Live atomic deployments:

```rust
// liveSpore manages deployments
// TUI provides deployment UI
```

---

## 🐛 Troubleshooting

### TUI won't start

**Issue:** Terminal not supported  
**Solution:** Ensure terminal supports ANSI escape codes

**Issue:** Permission denied  
**Solution:** Check socket permissions in `/run/user/$UID/`

### No primals showing

**Issue:** Standalone mode active  
**Solution:** Start Songbird or other primals

**Issue:** Socket paths incorrect  
**Solution:** Set `SONGBIRD_SOCKET` env var

### Rendering issues

**Issue:** Garbled output  
**Solution:** Ensure terminal size is at least 80x24

**Issue:** Colors not showing  
**Solution:** Enable true color in terminal

---

## 📚 Related Documentation

- [Universal UI Vision](../../UNIVERSAL_USER_INTERFACE_EVOLUTION.md)
- [Formal Specification](../../specs/UNIVERSAL_USER_INTERFACE_SPECIFICATION.md)
- [Implementation Tracking](../../UNIVERSAL_UI_TRACKING.md)
- [Socket Configuration](../../SOCKET_CONFIGURATION_COMPLETE.md)

---

## 🌟 Examples

### Simple Demo

```rust
use petal_tongue_tui::{TUIConfig, RichTUI};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create TUI with default config
    let mut tui = RichTUI::new().await?;

    // Run the TUI (blocks until quit)
    tui.run().await?;

    Ok(())
}
```

### Custom Configuration

```rust
use petal_tongue_tui::{TUIConfig, RichTUI};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = TUIConfig {
        tick_rate: Duration::from_millis(50),  // Faster refresh
        mouse_support: true,                    // Enable mouse
        standalone: false,
    };

    let mut tui = RichTUI::with_config(config).await?;
    tui.run().await?;

    Ok(())
}
```

---

## 📝 License

AGPL-3.0 - See LICENSE file

---

## 🌸 Credits

Part of the ecoPrimals ecosystem

**Different orders of the same architecture.** 🍄🐸

---

**Status:** Production Ready 🚀  
**Version:** 1.0.0  
**Last Updated:** January 12, 2026

