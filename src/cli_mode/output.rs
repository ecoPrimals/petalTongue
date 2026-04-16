// SPDX-License-Identifier: AGPL-3.0-or-later
//! Human-readable status printing.

use super::types::SystemStatus;

/// Print status in human-readable format
pub fn print_status_text(status: &SystemStatus) {
    println!("🌸 petalTongue ecoBud v{}", status.version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    println!("UniBin:");
    println!(
        "  ✅ {} binary, {} modes",
        status.unibin.binary_count, status.unibin.mode_count
    );
    println!();

    println!("ecoBin:");
    println!(
        "  ✅ {}% Pure Rust ({}/{} modes)",
        status.ecobin.percentage, status.ecobin.pure_rust_modes, status.ecobin.total_modes
    );
    for mode in &status.ecobin.modes {
        let check = if mode.pure_rust { "✅" } else { "⚠️ " };
        println!("     {} {}", check, mode.name);
    }
    println!();

    println!("System:");
    println!("  OS: {}", status.system.os);
    println!("  Arch: {}", status.system.arch);
    if let Some(cpus) = status.system.cpu_count {
        println!("  CPUs: {cpus}");
    }
    if let Some(mem) = status.system.memory_total {
        println!("  Memory: {} GB", mem / 1024 / 1024 / 1024);
    }

    if let Some(detailed) = &status.detailed {
        println!();
        println!("Modes:");
        for mode in &detailed.modes {
            let check = if mode.pure_rust { "✅" } else { "⚠️ " };
            println!("  {} {} - {}", check, mode.name, mode.description);
            println!("     Command: {}", mode.command);
        }

        println!();
        println!("Features:");
        for feature in &detailed.features {
            println!("  ✅ {feature}");
        }

        println!();
        println!("Dependencies:");
        println!("  Total: {}", detailed.dependencies.total);
        println!("  Pure Rust: {}", detailed.dependencies.rust_deps);
        println!(
            "  C deps: {} (only display mode)",
            detailed.dependencies.c_deps
        );
    }
}
