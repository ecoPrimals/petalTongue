// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure Rust Display System Demo
//!
//! Demonstrates the four-tier display architecture.

use petal_tongue_ui::display::prompt::prompt_for_display_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\n🌸 petalTongue Pure Rust Display System Demo\n");

    // Check display availability
    let has_display = std::env::var("DISPLAY").is_ok()
        || std::env::var("WAYLAND_DISPLAY").is_ok()
        || cfg!(target_os = "windows")
        || cfg!(target_os = "macos");

    if has_display {
        println!("✅ Display server detected:");
        if let Ok(display) = std::env::var("DISPLAY") {
            println!("   DISPLAY={display}");
        }
        if let Ok(display) = std::env::var("WAYLAND_DISPLAY") {
            println!("   WAYLAND_DISPLAY={display}");
        }
        println!();
        println!("🎨 Available backends:");
        println!("   1. External Display (active)");
        println!("   2. Discovered Display (ecosystem IPC)");
        println!("   3. Software Rendering");
        println!("   4. Framebuffer Direct");
    } else {
        println!("🪟 No display server detected\n");
        println!("🎨 Pure Rust display backends available:");
        println!("   1. Discovered Display (ecosystem IPC via capability.call)");
        println!("   2. Software Rendering (pure Rust, works everywhere)");
        println!("   3. Framebuffer Direct (Linux console mode)");
        println!();

        // Show the prompt
        println!("Demonstrating display server prompt...\n");
        match prompt_for_display_server().await {
            Ok(true) => println!("\nDisplay server became available!"),
            Ok(false) => println!("\nContinuing with Pure Rust rendering"),
            Err(e) => println!("\nPrompt error: {e}"),
        }
    }

    println!("\n════════════════════════════════════════════════════════════");
    println!("   ✅ Display System Architecture Complete!");
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!("Display Backend Tiers:");
    println!("  1. Discovered (ecosystem primal via capability.call IPC)");
    println!("  2. Software (pure Rust, works everywhere)");
    println!("  3. Framebuffer (Linux console mode)");
    println!("  4. External (traditional GUI via eframe)");
    println!();
    println!("All backends support the awakening experience!");
    println!("════════════════════════════════════════════════════════════\n");

    Ok(())
}
