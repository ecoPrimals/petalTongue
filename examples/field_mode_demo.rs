// SPDX-License-Identifier: AGPL-3.0-only
//! Field Mode Demo - No Screen Required!
//!
//! Demonstrates bidirectional UUI with audio + keyboard only.
//! This proves the abstraction: same topology data, different sensors.

use anyhow::Result;
use petal_tongue_core::{Key, SensorEvent};
use petal_tongue_ui::discover_all_sensors;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("\n════════════════════════════════════════════════════════════");
    println!("   🌸 petalTongue Field Mode Demo");
    println!("   No Monitor Required! Audio + Keyboard Only");
    println!("════════════════════════════════════════════════════════════\n");

    // Discover all sensors
    println!("🔍 Discovering sensors...\n");
    let mut registry = discover_all_sensors().await?;
    let stats = registry.stats();

    println!("📊 Sensor Discovery Results:");
    println!("   Total sensors: {}", stats.total);
    println!("   Active sensors: {}", stats.active);
    println!("   Has input: {}", stats.has_input);
    println!("   Has output: {}", stats.has_output);
    println!("   Has bidirectional: {}", stats.has_bidirectional);
    println!();

    // Find keyboard and audio sensors
    let keyboard_available = !registry
        .sensors_by_type(petal_tongue_core::SensorType::Keyboard)
        .is_empty();

    let audio_available = !registry
        .sensors_by_type(petal_tongue_core::SensorType::Audio)
        .is_empty();

    if !keyboard_available {
        println!("❌ No keyboard detected. Cannot run field mode.");
        return Ok(());
    }

    println!("✅ Field mode requirements met!");
    println!(
        "   Keyboard: {}",
        if keyboard_available { "✓" } else { "✗" }
    );
    println!("   Audio: {}", if audio_available { "✓" } else { "✗" });
    println!();

    // Run field interface
    run_field_interface(&mut registry, audio_available).await?;

    println!("\n🌸 Field mode demo complete!");
    Ok(())
}

async fn run_field_interface(
    registry: &mut petal_tongue_core::SensorRegistry,
    audio_available: bool,
) -> Result<()> {
    println!("════════════════════════════════════════════════════════════");
    println!("   🎮 Field Interface Active");
    println!("════════════════════════════════════════════════════════════\n");
    println!("Commands:");
    println!("  [S] - Status (show topology status)");
    println!("  [H] - Health (show system health)");
    println!("  [N] - Next node");
    println!("  [P] - Previous node");
    println!("  [Q] - Quit");
    println!();

    if audio_available {
        println!("🔊 Audio feedback enabled - listen for beeps!");
        // Audio playback requires a discovered audio.synthesis capability provider
    }

    let mut current_node = 0;
    let total_nodes = 3; // Mock data

    loop {
        // Poll all sensors for events
        let events = registry.poll_all().await?;

        for event in events {
            if let SensorEvent::KeyPress { key, .. } = event {
                match key {
                    Key::Char('s' | 'S') => {
                        println!("\n📊 Topology Status:");
                        println!("   Total nodes: {total_nodes}");
                        println!("   Healthy: {}", total_nodes - 1);
                        println!("   Warnings: 1");
                        println!("   Current node: {}/{}", current_node + 1, total_nodes);
                        println!();

                        if audio_available {
                            // Play success beep
                            println!("🔊 *beep*");
                        }
                    }
                    Key::Char('h' | 'H') => {
                        println!("\n💚 System Health:");
                        println!("   Overall: Healthy");
                        println!("   Uptime: 5m 23s");
                        println!("   Memory: 45MB");
                        println!("   CPU: 2%");
                        println!();

                        if audio_available {
                            println!("🔊 *success tone*");
                        }
                    }
                    Key::Char('n' | 'N') => {
                        current_node = (current_node + 1) % total_nodes;
                        println!("→ Next node: {}/{}", current_node + 1, total_nodes);

                        if audio_available {
                            println!("🔊 *click*");
                        }
                    }
                    Key::Char('p' | 'P') => {
                        current_node = if current_node == 0 {
                            total_nodes - 1
                        } else {
                            current_node - 1
                        };
                        println!("← Previous node: {}/{}", current_node + 1, total_nodes);

                        if audio_available {
                            println!("🔊 *click*");
                        }
                    }
                    Key::Char('q' | 'Q') => {
                        println!("\n👋 Exiting field mode...");
                        if audio_available {
                            println!("🔊 *goodbye tone*");
                        }
                        return Ok(());
                    }
                    _ => {
                        println!("❓ Unknown command. Press [S], [H], [N], [P], or [Q].");
                        if audio_available {
                            println!("🔊 *error beep*");
                        }
                    }
                }
            } else {
                // Ignore other events
            }
        }

        // Sleep briefly to avoid busy-waiting
        sleep(Duration::from_millis(10)).await;
    }
}
