// SPDX-License-Identifier: AGPL-3.0-only
//! JSON-RPC Provider Demo
//!
//! Demonstrates connecting to a JSON-RPC provider (like biomeOS)
//! over Unix sockets.

use petal_tongue_discovery::{JsonRpcProvider, VisualizationDataProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Note: Logging initialization removed (tracing_subscriber not in example deps)
    // Examples are for demonstration, not production use

    println!("🔌 JSON-RPC Provider Demo");
    println!("═══════════════════════════\n");

    // Try to discover JSON-RPC provider
    println!("1. Auto-discovering JSON-RPC providers...\n");

    match JsonRpcProvider::discover().await {
        Ok(provider) => {
            println!("✅ Found JSON-RPC provider!");

            // Get metadata
            let metadata = provider.get_metadata();
            println!("\n📋 Provider Info:");
            println!("   Name: {}", metadata.name);
            println!("   Endpoint: {}", metadata.endpoint);
            println!("   Protocol: {}", metadata.protocol);
            println!("   Capabilities: {:?}", metadata.capabilities);

            // Health check
            println!("\n🏥 Health Check:");
            match provider.health_check().await {
                Ok(status) => println!("   ✅ {status}"),
                Err(e) => println!("   ❌ Error: {e}"),
            }

            // Get primals
            println!("\n🌸 Fetching Primals:");
            match provider.get_primals().await {
                Ok(primals) => {
                    println!("   Found {} primals:", primals.len());
                    for primal in primals {
                        println!("   • {} ({}) - {:?}", primal.name, primal.id, primal.health);
                    }
                }
                Err(e) => println!("   ❌ Error: {e}"),
            }

            // Get topology
            println!("\n🕸️  Fetching Topology:");
            match provider.get_topology().await {
                Ok(topology) => {
                    if topology.is_empty() {
                        println!("   (No topology data - provider may not support it)");
                    } else {
                        println!("   Found {} connections:", topology.len());
                        for edge in topology {
                            println!("   • {} → {}", edge.from, edge.to);
                        }
                    }
                }
                Err(e) => println!("   ❌ Error: {e}"),
            }
        }
        Err(e) => {
            println!("❌ No JSON-RPC providers found!");
            println!("\nTroubleshooting:");
            println!("   1. Start biomeOS device_management_server:");
            println!("      $ cargo run --bin device_management_server");
            println!();
            println!("   2. Or set BIOMEOS_URL:");
            println!(
                "      $ BIOMEOS_URL=unix:///run/user/$UID/biomeos-device-management.sock cargo run"
            );
            println!();
            println!("Error: {e}");
        }
    }

    Ok(())
}
