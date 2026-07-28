// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "petaltongue")]
#[command(
    version,
    about = "🌸 petalTongue - Universal UI & Visualization System"
)]
#[command(
    long_about = "ecoBud v1.0: UniBin + Pure Rust\n\nFully concurrent, modern Rust architecture"
)]
pub struct Cli {
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Log format (json, pretty, compact)
    #[arg(long, default_value = "pretty")]
    pub log_format: String,

    /// Family namespace for multi-gate deployments (also settable via `FAMILY_ID` env)
    #[arg(long, env = "FAMILY_ID")]
    pub family_id: Option<String>,

    /// Unix domain socket path (global; subcommand --socket takes precedence)
    #[arg(long, env = "PETALTONGUE_SOCKET")]
    pub socket: Option<String>,

    /// TCP port (global; subcommand --port takes precedence)
    #[arg(long)]
    pub port: Option<u16>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch native desktop display
    #[command(visible_alias = "gui")]
    Ui {
        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,

        /// Disable audio sonification
        #[arg(long)]
        no_audio: bool,
    },

    /// Launch terminal user interface (Pure Rust! ✅)
    Tui {
        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,

        /// Refresh rate in Hz
        #[arg(long, default_value = "60")]
        refresh_rate: u32,
    },

    /// Launch web UI server (Pure Rust backend! ✅)
    ///
    /// When `--docroot` is provided, serves static files from that directory
    /// as a fallback for any path not matched by the API routes. This enables
    /// sovereign static site serving (sporePrint, Zola builds, etc.).
    Web {
        /// TCP port (`UniBin` standard: `--port` binds `0.0.0.0:PORT`)
        #[arg(long)]
        port: Option<u16>,

        /// Bind address override (takes precedence over --port)
        #[arg(long)]
        bind: Option<String>,

        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,

        /// Static file document root for catch-all serving (e.g., Zola build output)
        #[arg(long, env = "PETALTONGUE_DOCROOT")]
        docroot: Option<String>,

        /// Content backend: "filesystem" (default) or "content-provider"
        #[arg(long, env = "PETALTONGUE_WEB_BACKEND", default_value = "filesystem")]
        backend: String,

        /// Also start UDS JSON-RPC IPC server alongside HTTP (NUCLEUS dual-port mode)
        #[arg(long)]
        ipc: bool,

        /// TCP port for IPC JSON-RPC when --ipc is active (optional, UDS always active)
        #[arg(long)]
        ipc_port: Option<u16>,

        /// Number of worker threads (configures tokio runtime)
        #[arg(long, default_value = "4")]
        workers: usize,

        /// Hide code cells when rendering .ipynb notebooks (outputs only)
        #[arg(long, env = "PETALTONGUE_STRIP_SOURCES")]
        strip_sources: bool,

        /// Cache-Control max-age in seconds for static files (0 = no cache header)
        #[arg(long, env = "PETALTONGUE_CACHE_TTL")]
        cache_ttl: Option<u64>,

        /// SPA mode: serve index.html for missing paths (client-side routing)
        #[arg(long, env = "PETALTONGUE_SPA")]
        spa: bool,

        /// CORS allowed origins (comma-separated, or "*" for all)
        #[arg(long, env = "PETALTONGUE_ALLOWED_ORIGINS", value_delimiter = ',')]
        allowed_origins: Vec<String>,
    },

    /// Run headless API server (Pure Rust! ✅)
    Headless {
        /// TCP port (`UniBin` standard: `--port` binds `0.0.0.0:PORT`)
        #[arg(long)]
        port: Option<u16>,

        /// Bind address override (takes precedence over --port)
        #[arg(long)]
        bind: Option<String>,

        /// Number of worker threads
        #[arg(long, default_value = "4")]
        workers: usize,
    },

    /// Run IPC server (Unix socket JSON-RPC) without display
    ///
    /// Socket path priority: subcommand --socket > global --socket > `PETALTONGUE_SOCKET` env > XDG default
    Server {
        /// TCP port for newline-delimited JSON-RPC (optional, UDS always active)
        #[arg(long)]
        port: Option<u16>,

        /// TCP bind host (default: 127.0.0.1; use 0.0.0.0 for Docker/network)
        #[arg(long, env = "PETALTONGUE_IPC_HOST")]
        bind: Option<String>,

        /// Unix domain socket path override (or set `PETALTONGUE_SOCKET` env var)
        #[arg(long, env = "PETALTONGUE_SOCKET")]
        socket: Option<String>,
    },

    /// Launch native desktop display with IPC server (NUCLEUS interactive mode)
    Live {
        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,

        /// Disable audio sonification
        #[arg(long)]
        no_audio: bool,

        /// TCP port for newline-delimited JSON-RPC (optional, UDS always active)
        #[arg(long)]
        port: Option<u16>,

        /// TCP bind host (default: 127.0.0.1; use 0.0.0.0 for Docker/network)
        #[arg(long, env = "PETALTONGUE_IPC_HOST")]
        bind: Option<String>,

        /// Unix domain socket path override (or set `PETALTONGUE_SOCKET` env var)
        #[arg(long, env = "PETALTONGUE_SOCKET")]
        socket: Option<String>,
    },

    /// Show status and system info (Pure Rust! ✅)
    Status {
        /// Show detailed information
        #[arg(long)]
        verbose: bool,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },
}
