//! Simple TUI Demo
//!
//! Demonstrates the Rich TUI in action.
//!
//! Run with: cargo run --example simple_demo

use petal_tongue_tui::{TUIConfig, launch_with_config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create config
    let config = TUIConfig {
        tick_rate: std::time::Duration::from_millis(100),
        mouse_support: false,
        standalone: false, // Try to discover primals
    };

    // Launch TUI
    println!("🌸 Launching petalTongue Rich TUI...");
    println!("Press 'q' to quit, '1-8' to switch views, '?' for help");

    launch_with_config(config).await?;

    println!("👋 Thanks for using petalTongue!");

    Ok(())
}

