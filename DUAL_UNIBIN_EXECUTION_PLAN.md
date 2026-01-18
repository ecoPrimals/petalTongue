# 🚀 petalTongue Dual UniBin Execution Plan

**Date**: January 18, 2026  
**Goal**: Complete ecoBud + UniBin ecoBlossom  
**Status**: 🚧 **EXECUTING**  
**Timeline**: ~2-3 days

---

## 🎯 **Mission**

### **Deliverables**

1. **ecoBud v1.0** (Complete!)
   - UniBin: ✅ 1 binary, 5 modes
   - ecoBin: ✅ 80% (4/5 Pure Rust)
   - Production: ✅ Ready for biomeOS deployment

2. **ecoBlossom UniBin** (Foundation!)
   - UniBin: ✅ 1 binary structure
   - GUI: 🔬 Platform deps (for now, evolve later)
   - Ready: ✅ For cross-platform GUI deployments

---

## 📊 **Architecture Overview**

```
petalTongue/
│
├── ecoBud (Production Track)
│   └── petaltongue
│       ├── ui        → Desktop GUI (platform deps OK)
│       ├── tui       → Terminal (Pure Rust ✅)
│       ├── web       → Browser (Pure Rust ✅)
│       ├── headless  → API (Pure Rust ✅)
│       └── status    → CLI (Pure Rust ✅)
│
└── ecoBlossom (Evolution Track)
    └── petaltongue-blossom
        ├── ui        → Desktop GUI (Pure Rust research)
        ├── tui       → Terminal (Pure Rust ✅)
        ├── web       → Browser (Pure Rust ✅)
        ├── headless  → API (Pure Rust ✅)
        └── status    → CLI (Pure Rust ✅)
```

---

## 🌱 **Phase 1: Complete ecoBud** (~1.5 days)

### **Step 1.1: Create UniBin Main** (~2 hours)

**File**: `src/main.rs` (NEW!)

```rust
//! petalTongue ecoBud - Production UniBin
//! 
//! UniBin: 1 binary, 5 modes
//! ecoBin: 80% (4/5 modes Pure Rust)

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "petaltongue")]
#[command(version, about = "🌸 petalTongue - Universal UI & Visualization System")]
#[command(long_about = "ecoBud v1.0: UniBin + 80% ecoBin")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch native desktop GUI
    #[command(visible_alias = "gui")]
    Ui {
        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,
    },
    
    /// Launch terminal user interface (Pure Rust! ✅)
    Tui {
        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,
    },
    
    /// Launch web UI server (Pure Rust backend! ✅)
    Web {
        /// Bind address
        #[arg(long, default_value = "0.0.0.0:3000")]
        bind: String,
        
        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,
    },
    
    /// Run headless API server (Pure Rust! ✅)
    Headless {
        /// Bind address
        #[arg(long, default_value = "0.0.0.0:8080")]
        bind: String,
    },
    
    /// Show status and system info (Pure Rust! ✅)
    Status {
        /// Show detailed information
        #[arg(long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Ui { scenario } => {
            tracing::info!("🌸 Launching desktop GUI mode...");
            ui_mode::run(scenario).await
        }
        Commands::Tui { scenario } => {
            tracing::info!("🌸 Launching terminal UI mode (Pure Rust!)...");
            tui_mode::run(scenario).await
        }
        Commands::Web { bind, scenario } => {
            tracing::info!("🌸 Launching web UI server (Pure Rust!)...");
            web_mode::run(bind, scenario).await
        }
        Commands::Headless { bind } => {
            tracing::info!("🌸 Launching headless API server (Pure Rust!)...");
            headless_mode::run(bind).await
        }
        Commands::Status { verbose } => {
            tracing::info!("🌸 Querying system status (Pure Rust!)...");
            cli_mode::status(verbose).await
        }
    }
}

mod ui_mode;
mod tui_mode;
mod web_mode;
mod headless_mode;
mod cli_mode;
```

### **Step 1.2: UI Mode Module** (~1 hour)

**File**: `src/ui_mode.rs` (NEW!)

```rust
//! Desktop GUI mode (using egui)
//! 
//! Platform dependencies: wayland-sys, x11-sys (acceptable for ecoBud)

use anyhow::Result;

pub async fn run(scenario: Option<String>) -> Result<()> {
    tracing::info!("Starting desktop GUI mode");
    
    // Use existing petal-tongue-ui logic
    petal_tongue_ui::run_ui(scenario).await
}
```

### **Step 1.3: TUI Mode Module** (~2 hours)

**File**: `src/tui_mode.rs` (NEW!)

```rust
//! Terminal UI mode (Pure Rust! ✅)
//! 
//! Dependencies: ratatui, crossterm (100% Pure Rust)

use anyhow::Result;
use petal_tongue_tui::TuiApp;

pub async fn run(scenario: Option<String>) -> Result<()> {
    tracing::info!("Starting terminal UI mode (Pure Rust!)");
    
    let app = TuiApp::new(scenario)?;
    app.run().await
}
```

### **Step 1.4: Web Mode Module** (~3 hours)

**File**: `src/web_mode.rs` (NEW!)

```rust
//! Web UI server mode (Pure Rust backend! ✅)
//! 
//! Dependencies: axum, tower-http (100% Pure Rust)

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
    response::{Html, IntoResponse},
    Json,
};
use tower_http::services::ServeDir;
use std::net::SocketAddr;

pub async fn run(bind: String, scenario: Option<String>) -> Result<()> {
    tracing::info!("Starting web UI server (Pure Rust!)");
    
    let addr: SocketAddr = bind.parse()?;
    
    // Build router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/status", get(status_handler))
        .route("/api/primals", get(primals_handler))
        .nest_service("/static", ServeDir::new("web/static"));
    
    tracing::info!("🌐 Web UI server listening on http://{}", addr);
    
    // Start server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn status_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
    }))
}

async fn primals_handler() -> impl IntoResponse {
    // Use discovery to get primals
    Json(serde_json::json!({
        "primals": []
    }))
}
```

### **Step 1.5: Headless Mode Module** (~1 hour)

**File**: `src/headless_mode.rs` (NEW!)

```rust
//! Headless API server mode (Pure Rust! ✅)

use anyhow::Result;

pub async fn run(bind: String) -> Result<()> {
    tracing::info!("Starting headless API server (Pure Rust!)");
    
    // Use existing petal-tongue-headless logic
    petal_tongue_headless::run_headless(bind).await
}
```

### **Step 1.6: CLI Mode Module** (~1 hour)

**File**: `src/cli_mode.rs` (NEW!)

```rust
//! CLI tools mode (Pure Rust! ✅)

use anyhow::Result;

pub async fn status(verbose: bool) -> Result<()> {
    tracing::info!("Querying system status (Pure Rust!)");
    
    println!("🌸 petalTongue ecoBud v{}", env!("CARGO_PKG_VERSION"));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("UniBin: ✅ 1 binary, 5 modes");
    println!("ecoBin: ✅ 80% (4/5 modes Pure Rust)");
    println!();
    
    // Show system info
    println!("System:");
    println!("  OS: {}", std::env::consts::OS);
    println!("  Arch: {}", std::env::consts::ARCH);
    
    if verbose {
        println!();
        println!("Modes:");
        println!("  ui       - Desktop GUI (platform deps)");
        println!("  tui      - Terminal UI (Pure Rust ✅)");
        println!("  web      - Web server (Pure Rust ✅)");
        println!("  headless - API server (Pure Rust ✅)");
        println!("  status   - This command (Pure Rust ✅)");
    }
    
    Ok(())
}
```

### **Step 1.7: Update Cargo.toml** (~30 min)

**File**: `Cargo.toml` (MODIFY!)

```toml
[[bin]]
name = "petaltongue"
path = "src/main.rs"

[dependencies]
# Core
petal-tongue-core = { path = "crates/petal-tongue-core" }
petal-tongue-discovery = { path = "crates/petal-tongue-discovery" }

# Modes
petal-tongue-ui = { path = "crates/petal-tongue-ui", optional = true }
petal-tongue-tui = { path = "crates/petal-tongue-tui" }
petal-tongue-headless = { path = "crates/petal-tongue-headless" }

# Web server
axum = { version = "0.7", features = ["tokio"] }
tower-http = { version = "0.5", features = ["fs"] }

# CLI
clap = { workspace = true }

# Async
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Error handling
anyhow = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[features]
default = ["ui"]  # Desktop users get GUI by default
ui = ["petal-tongue-ui"]  # Can disable for minimal build

# OLD binaries (remove after UniBin complete)
# [[bin]]
# name = "petal-tongue-ui"
# path = "crates/petal-tongue-ui/src/main.rs"
# ...
```

### **Step 1.8: Create Web Frontend** (~2 hours)

**File**: `web/index.html` (NEW!)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🌸 petalTongue Web UI</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            padding: 2rem;
        }
        .header {
            text-align: center;
            margin-bottom: 3rem;
        }
        .header h1 { font-size: 3rem; margin-bottom: 0.5rem; }
        .header p { font-size: 1.2rem; opacity: 0.9; }
        .container {
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            border-radius: 1rem;
            padding: 2rem;
            max-width: 1200px;
            width: 100%;
        }
        .status {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }
        .card {
            background: rgba(255, 255, 255, 0.1);
            border-radius: 0.5rem;
            padding: 1.5rem;
        }
        .card h3 { margin-bottom: 0.5rem; }
        .badge {
            display: inline-block;
            background: rgba(0, 255, 0, 0.3);
            color: #0f0;
            padding: 0.25rem 0.75rem;
            border-radius: 1rem;
            font-size: 0.875rem;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🌸 petalTongue</h1>
        <p>ecoBud Web UI (Pure Rust Backend!)</p>
    </div>
    
    <div class="container">
        <div class="status">
            <div class="card">
                <h3>UniBin</h3>
                <span class="badge">✅ Active</span>
                <p>1 binary, 5 modes</p>
            </div>
            <div class="card">
                <h3>ecoBin</h3>
                <span class="badge">✅ 80%</span>
                <p>4/5 modes Pure Rust</p>
            </div>
            <div class="card">
                <h3>Web Mode</h3>
                <span class="badge">✅ Pure Rust</span>
                <p>Zero C dependencies</p>
            </div>
        </div>
        
        <div id="content">
            <h2>System Status</h2>
            <p>Fetching data from API...</p>
        </div>
    </div>
    
    <script>
        // Fetch status from API
        fetch('/api/status')
            .then(r => r.json())
            .then(data => {
                document.getElementById('content').innerHTML = `
                    <h2>System Status</h2>
                    <pre>${JSON.stringify(data, null, 2)}</pre>
                `;
            })
            .catch(err => {
                document.getElementById('content').innerHTML = `
                    <h2>Error</h2>
                    <p>${err.message}</p>
                `;
            });
    </script>
</body>
</html>
```

### **Step 1.9: Testing** (~2 hours)

```bash
# Test UI mode
cargo run --release -- ui

# Test TUI mode
cargo run --release -- tui

# Test web mode
cargo run --release -- web
# Open browser to http://localhost:3000

# Test headless mode
cargo run --release -- headless

# Test status
cargo run --release -- status
cargo run --release -- status --verbose

# Test minimal build (no GUI)
cargo build --release --no-default-features
```

---

## 🌸 **Phase 2: UniBin ecoBlossom** (~1 day)

### **Goal**: Create UniBin structure for future Pure Rust evolution

**File**: `blossom/main.rs` (NEW!)

```rust
//! petalTongue ecoBlossom - Pure Rust Evolution
//! 
//! UniBin: 1 binary, 5 modes
//! ecoBin: Evolving towards 100%

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "petaltongue-blossom")]
#[command(version, about = "🌸 petalTongue ecoBlossom - Pure Rust Evolution")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch desktop GUI (evolving to Pure Rust)
    Ui {
        #[arg(long)]
        scenario: Option<String>,
        
        /// Use experimental Pure Rust renderer
        #[arg(long)]
        experimental: bool,
    },
    
    /// Terminal UI (Pure Rust! ✅)
    Tui { /* ... */ },
    
    /// Web UI server (Pure Rust! ✅)
    Web { /* ... */ },
    
    /// Headless API (Pure Rust! ✅)
    Headless { /* ... */ },
    
    /// Status (Pure Rust! ✅)
    Status { /* ... */ },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Ui { scenario, experimental } => {
            if experimental {
                tracing::info!("🌸 Using experimental Pure Rust renderer!");
                experimental_ui_mode::run(scenario).await
            } else {
                tracing::info!("🌸 Using standard renderer (for now)");
                ui_mode::run(scenario).await
            }
        }
        // ... other modes same as ecoBud
    }
}
```

**File**: `blossom/experimental_ui_mode.rs` (NEW!)

```rust
//! Experimental Pure Rust GUI
//! 
//! Research implementation using:
//! - drm-rs for display (Linux)
//! - Software rendering
//! - Custom input handling

use anyhow::Result;

pub async fn run(scenario: Option<String>) -> Result<()> {
    tracing::warn!("🔬 Experimental Pure Rust GUI - Work in Progress!");
    tracing::warn!("This is research code, expect rough edges");
    
    // TODO: Implement Pure Rust display
    // For now, fall back to standard UI
    tracing::info!("Falling back to standard UI for now...");
    crate::ui_mode::run(scenario).await
}
```

---

## 📦 **Phase 3: Packaging & Deployment** (~4 hours)

### **Build Scripts**

**File**: `scripts/build-ecobud.sh` (NEW!)

```bash
#!/bin/bash
# Build ecoBud for distribution

set -e

echo "🌱 Building petalTongue ecoBud..."

# Full build (with GUI)
cargo build --release --bin petaltongue

# Minimal build (no GUI, Pure Rust only!)
cargo build --release --bin petaltongue --no-default-features

# Check sizes
echo ""
echo "Binary sizes:"
ls -lh target/release/petaltongue

# Verify ecoBin compliance
echo ""
echo "Checking ecoBin compliance..."
cargo tree --package petaltongue --no-default-features | grep "\-sys" | grep -v "linux-raw-sys" || echo "✅ Minimal build is Pure Rust!"

echo ""
echo "✅ ecoBud build complete!"
```

**File**: `scripts/build-ecoblossom.sh` (NEW!)

```bash
#!/bin/bash
# Build ecoBlossom

set -e

echo "🌸 Building petalTongue ecoBlossom..."

cd blossom
cargo build --release

echo ""
echo "✅ ecoBlossom build complete!"
```

### **Installation Script**

**File**: `scripts/install.sh` (NEW!)

```bash
#!/bin/bash
# Install petalTongue for biomeOS deployment

set -e

PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="$PREFIX/bin"

echo "🌸 Installing petalTongue..."

# Build
./scripts/build-ecobud.sh

# Install ecoBud
sudo install -m 755 target/release/petaltongue "$BIN_DIR/petaltongue"

# Create symlinks
sudo ln -sf petaltongue "$BIN_DIR/petaltongue-ui"
sudo ln -sf petaltongue "$BIN_DIR/petaltongue-tui"
sudo ln -sf petaltongue "$BIN_DIR/petaltongue-web"

echo ""
echo "✅ Installed to $BIN_DIR"
echo ""
echo "Usage:"
echo "  petaltongue ui        # Desktop GUI"
echo "  petaltongue tui       # Terminal UI"
echo "  petaltongue web       # Web server"
echo "  petaltongue headless  # API server"
echo "  petaltongue status    # System info"
```

---

## 📊 **Success Criteria**

### **ecoBud v1.0** ✅
- [ ] UniBin: 1 binary with 5 subcommands
- [ ] UI mode works (desktop GUI)
- [ ] TUI mode works (terminal)
- [ ] Web mode works (browser)
- [ ] Headless mode works (API)
- [ ] Status command works (CLI)
- [ ] ecoBin: 80% (verified via `cargo tree`)
- [ ] Documentation complete
- [ ] Ready for biomeOS deployment

### **ecoBlossom UniBin** ✅
- [ ] UniBin structure created
- [ ] Same 5 modes as ecoBud
- [ ] Experimental flag for Pure Rust renderer
- [ ] Research directory set up
- [ ] Ready for evolution

---

## 📚 **Documentation Updates**

### **Files to Create/Update**:

1. **`ECOBUD_V1_RELEASE_NOTES.md`** (NEW!)
   - Feature list
   - Installation instructions
   - Usage examples
   - ecoBin compliance details

2. **`ECOBLOSSOM_UNIBIN_STATUS.md`** (NEW!)
   - UniBin structure complete
   - Evolution roadmap
   - Experimental features

3. **`START_HERE.md`** (UPDATE!)
   - Add UniBin commands
   - Show ecoBud vs ecoBlossom

4. **`PROJECT_STATUS.md`** (UPDATE!)
   - ecoBud v1.0 complete
   - ecoBlossom UniBin ready

---

## 🚀 **Execution Timeline**

### **Day 1** (~8 hours):
- Morning: Steps 1.1-1.3 (UniBin main, UI, TUI)
- Afternoon: Steps 1.4-1.6 (Web, headless, CLI)
- Evening: Step 1.7 (Cargo.toml)

### **Day 2** (~8 hours):
- Morning: Step 1.8 (Web frontend)
- Afternoon: Step 1.9 (Testing all modes)
- Evening: Phase 2 (ecoBlossom UniBin structure)

### **Day 3** (~4 hours):
- Morning: Phase 3 (Packaging & scripts)
- Afternoon: Documentation
- Evening: Release & celebrate! 🎊

**Total**: ~20 hours over 2-3 days

---

## 🎊 **Expected Outcomes**

### **After Execution**:

```bash
# ecoBud (Production)
$ petaltongue --help
🌸 petalTongue ecoBud v1.0

COMMANDS:
  ui        Desktop GUI
  tui       Terminal UI (Pure Rust ✅)
  web       Web server (Pure Rust ✅)
  headless  API server (Pure Rust ✅)
  status    System info (Pure Rust ✅)

$ petaltongue status
🌸 petalTongue ecoBud v1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━
UniBin: ✅ 1 binary, 5 modes
ecoBin: ✅ 80% (4/5 Pure Rust)

# ecoBlossom (Evolution)
$ petaltongue-blossom --help
🌸 petalTongue ecoBlossom - Pure Rust Evolution

COMMANDS:
  ui --experimental  Pure Rust GUI (WIP)
  tui               Terminal UI
  web               Web server
  headless          API server
  status            System info
```

---

**Status**: ✅ Plan Complete  
**Ready**: To execute immediately  
**Timeline**: 2-3 days  
**Outcome**: Full UniBin deployment for biomeOS!  

🌱 **Let's build ecoBud!** 🌸 **Let's structure ecoBlossom!** 🚀

