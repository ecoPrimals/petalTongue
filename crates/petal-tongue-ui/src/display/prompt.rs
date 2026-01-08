//! External Display Server Prompt
//!
//! Prompts user to start display server if not available.
//! Used for Tier 4 (External Display) fallback.

use anyhow::Result;
use std::env;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use tracing::info;

/// Prompt user to start display server with sudo
///
/// Returns:
/// - `Ok(true)` if display server became available
/// - `Ok(false)` if user declined or timeout
/// - `Err(_)` on IO error
pub fn prompt_for_display_server() -> Result<bool> {
    // Check if display is already available
    if is_display_available() {
        return Ok(true);
    }

    // Check if we're in non-interactive mode
    if is_non_interactive() {
        info!("Non-interactive mode detected, skipping display server prompt");
        return Ok(false);
    }

    // Show prompt
    print_prompt();

    // Wait for user input
    print!("\nPress Enter to continue with Pure Rust rendering...\n");
    print!("(or start a display server in another terminal)\n\n");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Give user time to start display server
    info!("⏳ Checking for display server (5 seconds)...");
    for i in 1..=5 {
        thread::sleep(Duration::from_secs(1));
        if is_display_available() {
            info!("✅ Display server detected!");
            return Ok(true);
        }
        info!("⏳ {}...", 6 - i);
    }

    info!("📦 No display server found. Continuing with Pure Rust rendering.");
    Ok(false)
}

/// Check if display server is available
fn is_display_available() -> bool {
    env::var("DISPLAY").is_ok()
        || env::var("WAYLAND_DISPLAY").is_ok()
        || cfg!(target_os = "windows")
        || cfg!(target_os = "macos")
}

/// Check if running in non-interactive mode
fn is_non_interactive() -> bool {
    // Check common CI/non-interactive indicators
    env::var("CI").is_ok()
        || env::var("HEADLESS").is_ok()
        || env::var("PETALTONGUE_HEADLESS").is_ok()
        || env::var("NON_INTERACTIVE").is_ok()
        || !atty::is(atty::Stream::Stdin)
}

/// Print the display server prompt
fn print_prompt() {
    println!();
    println!("════════════════════════════════════════════════════════════");
    println!("   🪟 No Display Server Detected");
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!("petalTongue can run in multiple display modes:");
    println!();
    println!("  1. ✅ Pure Rust (recommended)");
    println!("     - Software rendering (no GPU needed)");
    println!("     - Works everywhere");
    println!("     - Continues automatically");
    println!();
    println!("  2. 🪟 Traditional GUI (benchmark)");
    println!("     - Requires X11 or Wayland");
    println!("     - Better performance (if available)");
    println!("     - You can start manually with:");
    println!();
    println!("       sudo systemctl start display-manager");
    println!("       # or");
    println!("       startx");
    println!();
    println!("════════════════════════════════════════════════════════════");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_detection() {
        let available = is_display_available();
        info!("Display available: {}", available);
    }

    #[test]
    fn test_non_interactive_detection() {
        let non_interactive = is_non_interactive();
        info!("Non-interactive: {}", non_interactive);
    }
}

