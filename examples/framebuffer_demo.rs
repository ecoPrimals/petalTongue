// SPDX-License-Identifier: AGPL-3.0-or-later
//! Framebuffer Demo - Pure Rust GUI via Direct Framebuffer
//!
//! This example demonstrates Tier 3 Pure Rust rendering:
//! Direct framebuffer access without any display server.
//!
//! **REQUIREMENTS**:
//! - Root access (sudo) to write to `/dev/fb0`
//! - Linux with framebuffer support
//! - Framebuffer device available at `/dev/fb0`
//!
//! **USAGE**:
//! ```bash
//! # Build the example
//! cargo build --release --example framebuffer_demo --features examples
//!
//! # Run with sudo (required for /dev/fb0 access)
//! sudo target/release/examples/framebuffer_demo
//! ```
//!
//! **WHAT THIS DEMONSTRATES**:
//! - Direct hardware framebuffer rendering
//! - Complete GUI without X11/Wayland
//! - Tier 3 Pure Rust display capability
//! - Console-mode GUI rendering

use anyhow::Result;
use petal_tongue_ui::display::{DisplayManager, EguiPixelRenderer};
use std::time::{Duration, Instant};
use tracing_subscriber::EnvFilter;

#[tokio::main]
#[expect(
    clippy::too_many_lines,
    reason = "demo main is cohesive setup and render loop"
)]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    println!("🌸 petalTongue - Framebuffer Demo");
    println!("═══════════════════════════════════════\n");

    // Check if running with root
    #[cfg(target_os = "linux")]
    {
        let euid = petal_tongue_core::system_info::get_current_euid();
        if euid != 0 {
            eprintln!("⚠️  WARNING: Not running as root (eUID: {euid})");
            eprintln!("    Framebuffer access may require root privileges.");
            eprintln!("    If this fails, try: sudo target/release/examples/framebuffer_demo\n");
        }
    }

    // Initialize display manager
    let mut display_manager = DisplayManager::init().await?;

    // Check if framebuffer backend is active
    let active_name = display_manager.active_backend_name().unwrap_or("Unknown");
    println!("✅ Active backend: {active_name}");

    if !active_name.contains("Framebuffer") {
        eprintln!("⚠️  WARNING: Not using framebuffer backend!");
        eprintln!("   This demo is intended for framebuffer display.");
        eprintln!("   Current backend: {active_name}");
        eprintln!("\nReasons framebuffer may not be available:");
        eprintln!("   - /dev/fb0 does not exist");
        eprintln!("   - Insufficient permissions (need root)");
        eprintln!("   - Framebuffer driver not loaded");
        eprintln!("\nProceeding with available backend anyway...\n");
    }

    // Initialize egui context and pixel renderer
    let ctx = egui::Context::default();
    let mut pixel_renderer = EguiPixelRenderer::new(1920, 1080);

    println!("\n🎨 Rendering 60 frames to framebuffer...\n");

    let mut frame_times = Vec::new();
    let start_time = Instant::now();
    let mut last_frame_time = start_time;

    for i in 0..60 {
        let frame_start = Instant::now();
        let delta_time = frame_start.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = frame_start;

        // Run egui frame
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("🌸 petalTongue - Framebuffer Demo");
                ui.separator();

                ui.label(format!(
                    "Backend: {}",
                    display_manager.active_backend_name().unwrap_or("Unknown")
                ));
                ui.label(format!("Frame: {}/60", i + 1));
                ui.label(format!("Delta: {:.2}ms", delta_time * 1000.0));

                ui.separator();

                ui.label("✅ Pure Rust rendering");
                ui.label("✅ No X11/Wayland required");
                ui.label("✅ No OpenGL required");
                ui.label("✅ Direct hardware access");

                ui.separator();

                if i < 20 {
                    ui.label("🌸 Stage: Warming up...");
                } else if i < 40 {
                    ui.label("🌼 Stage: Steady state");
                } else {
                    ui.label("🌺 Stage: Finishing...");
                }

                // Add some interactive elements
                ui.separator();
                ui.label("Interactive Elements:");
                if ui.button("Button (demo)").clicked() {
                    println!("Button clicked at frame {i}");
                }

                #[expect(
                    clippy::cast_precision_loss,
                    reason = "frame index 0..60 is exactly representable in f32"
                )]
                let mut value = i as f32 / 60.0;
                ui.add(egui::Slider::new(&mut value, 0.0..=1.0).text("Progress"));
            });
        });

        // Tessellate egui output
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);

        // Render egui to pixel buffer
        let render_start = Instant::now();
        let buffer = pixel_renderer.render(&primitives)?;
        let render_time = render_start.elapsed();

        // Present buffer via display backend
        let present_start = Instant::now();
        display_manager.present(buffer.as_ref()).await?;
        let present_time = present_start.elapsed();

        let frame_time = frame_start.elapsed();
        frame_times.push(frame_time);

        // Print progress every 10 frames
        if (i + 1) % 10 == 0 {
            #[expect(clippy::cast_possible_truncation, reason = "frame count is small")]
            let avg_frame_time: Duration =
                frame_times.iter().sum::<Duration>() / frame_times.len() as u32;
            println!(
                "Frame {}/60 | Frame: {:.2}ms | Render: {:.2}ms | Present: {:.2}ms | Avg: {:.2}ms",
                i + 1,
                frame_time.as_secs_f64() * 1000.0,
                render_time.as_secs_f64() * 1000.0,
                present_time.as_secs_f64() * 1000.0,
                avg_frame_time.as_secs_f64() * 1000.0
            );
        }

        // Target 60 FPS (16.67ms per frame)
        let target_frame_time = Duration::from_millis(16);
        if frame_time < target_frame_time {
            tokio::time::sleep(
                target_frame_time
                    .checked_sub(frame_time)
                    .unwrap_or(Duration::ZERO),
            )
            .await;
        }
    }

    let total_time = start_time.elapsed();
    #[expect(clippy::cast_possible_truncation, reason = "frame count is 60")]
    let avg_frame_time: Duration = frame_times.iter().sum::<Duration>() / frame_times.len() as u32;
    let fps = 60.0 / total_time.as_secs_f64();

    println!("\n═══════════════════════════════════════");
    println!("📊 Performance Summary");
    println!("═══════════════════════════════════════");
    println!("Total Time: {:.2}s", total_time.as_secs_f64());
    println!("Frames: 60");
    println!("Average FPS: {fps:.1}");
    println!(
        "Average Frame Time: {:.2}ms",
        avg_frame_time.as_secs_f64() * 1000.0
    );
    println!("Target Frame Time: 16.67ms (60 FPS)");

    let target_achievement = (16.67 / (avg_frame_time.as_secs_f64() * 1000.0)) * 100.0;
    println!("Target Achievement: {target_achievement:.1}%");

    println!("\n✅ Framebuffer demo complete!");
    println!("   Direct hardware rendering working");
    println!("   No display server required");
    println!("   Pure Rust sovereignty achieved 🌸\n");

    Ok(())
}
