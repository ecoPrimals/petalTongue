// SPDX-License-Identifier: AGPL-3.0-only
//! Simple TUI Demo
//!
//! Demonstrates the Rich TUI in action.
//!
//! Run with: cargo run --example `simple_demo`

use petal_tongue_tui::launch;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Show welcome message
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║   🌸 petalTongue Rich TUI - Demo                         ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("📚 Keyboard Shortcuts:");
    println!("  [1-8]  Switch views");
    println!("  [↑/k ↓/j]  Navigate");
    println!("  [r]  Refresh");
    println!("  [?]  Help");
    println!("  [q]  Quit");
    println!();
    println!("🌸 Press any key to start...");
    println!();

    // Wait a moment
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Launch TUI
    launch().await?;

    // Show goodbye message
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║   👋 Thanks for using petalTongue!                       ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("Different orders of the same architecture. 🍄🐸🌸");
    println!();

    Ok(())
}
