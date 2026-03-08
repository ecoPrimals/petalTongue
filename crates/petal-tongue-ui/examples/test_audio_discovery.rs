// SPDX-License-Identifier: AGPL-3.0-only
//! Test audio discovery

use petal_tongue_ui::audio_discovery::AudioDiscovery;

fn main() {
    println!("\n🎵 Testing Audio Discovery...\n");

    let discovery = AudioDiscovery::discover();

    println!("📊 Discovery Results:");
    println!("  - Sockets found: {}", discovery.sockets.len());
    for socket in &discovery.sockets {
        println!(
            "    • {:?}: {} (accessible: {})",
            socket.backend_type,
            socket.path.display(),
            socket.accessible
        );
    }

    println!(
        "  - Direct ALSA devices: {}",
        discovery.direct_devices.len()
    );
    for device in &discovery.direct_devices {
        println!("    • {}", device.display());
    }

    println!("\n🎯 Preferred Backend: {:?}", discovery.preferred);
    println!("📝 Status: {}", discovery.status_message());
    println!("✅ Audio Available: {}\n", discovery.is_available());
}
