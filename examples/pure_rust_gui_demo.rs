// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure Rust Display Demo
//!
//! Demonstrates the complete Pure Rust rendering pipeline:
//! egui UI → `EguiPixelRenderer` → Display Backend
//!
//! This showcases the display sovereignty achieved by petalTongue.

use anyhow::Result;
use petal_tongue_ui::display::{DisplayManager, EguiPixelRenderer};
use std::time::{Duration, Instant};

#[tokio::main]
#[expect(
    clippy::cast_precision_loss,
    reason = "frame_count and target_frames are small; f32 progress and timing display are acceptable"
)]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌸 Pure Rust Display Demo\n");
    println!("Demonstrating complete display sovereignty:");
    println!("  egui → EguiPixelRenderer → Display Backend\n");

    // Initialize display system (discovers best backend)
    let mut display = DisplayManager::init().await?;
    println!("✅ Display system initialized\n");

    // Get active backend info
    let backend_name = display.active_backend_name().unwrap_or("None").to_string(); // Clone to avoid borrow issues
    println!("Active backend: {backend_name}\n");

    // Create egui context
    let ctx = egui::Context::default();
    println!("✅ Created egui context\n");

    // Get display dimensions
    let (width, height) = display
        .dimensions()
        .ok_or_else(|| anyhow::anyhow!("No active display backend"))?;
    let mut renderer = EguiPixelRenderer::new(width, height);
    println!("✅ Created pixel renderer: {width}x{height}\n");

    println!("════════════════════════════════════════════════════════════");
    println!("   🎨 Rendering 60 Frames (1 second)");
    println!("════════════════════════════════════════════════════════════\n");

    let start = Instant::now();
    let mut frame_count = 0;
    let target_frames = 60;

    while frame_count < target_frames {
        let frame_start = Instant::now();

        // Create UI
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("🌸 petalTongue");
                ui.label("Pure Rust Display Rendering");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Frame:");
                    ui.colored_label(
                        egui::Color32::GREEN,
                        format!("{}/{}", frame_count + 1, target_frames),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Backend:");
                    ui.colored_label(egui::Color32::LIGHT_BLUE, &backend_name);
                });

                ui.add_space(10.0);

                ui.label("✅ No OpenGL required");
                ui.label("✅ No display server required");
                ui.label("✅ 100% Pure Rust");
                ui.label("✅ Complete sovereignty");

                ui.add_space(10.0);

                // Progress bar
                let progress = (frame_count as f32) / (target_frames as f32);
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "progress in [0,1], percentage clamped"
                )]
                ui.add(
                    egui::ProgressBar::new(progress)
                        .text(format!("{}%", (progress * 100.0).clamp(0.0, 100.0) as u32)),
                );
            });
        });

        // Tessellate
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);

        // Render to pixels
        let buffer = renderer.render(&primitives)?;

        // Present via active backend
        display.present(buffer.as_ref()).await?;

        frame_count += 1;

        // Frame timing (target 60 FPS = 16.67ms)
        let frame_time = frame_start.elapsed();
        if frame_time < Duration::from_millis(16) {
            tokio::time::sleep(
                Duration::from_millis(16)
                    .checked_sub(frame_time)
                    .unwrap_or(Duration::ZERO),
            )
            .await;
        }

        // Progress indicator
        if frame_count % 10 == 0 {
            println!(
                "  Frame {}/{} ({:.2}ms)",
                frame_count,
                target_frames,
                frame_time.as_secs_f32() * 1000.0
            );
        }
    }

    let total_time = start.elapsed();
    let avg_frame_time = total_time / target_frames;
    let fps = 1.0 / avg_frame_time.as_secs_f64();

    println!("\n════════════════════════════════════════════════════════════");
    println!("   📊 Performance Results");
    println!("════════════════════════════════════════════════════════════\n");
    println!("Total time: {:.2}s", total_time.as_secs_f32());
    println!("Frames rendered: {target_frames}");
    println!(
        "Average frame time: {:.2}ms",
        avg_frame_time.as_secs_f32() * 1000.0
    );
    println!("Average FPS: {fps:.1}");
    println!("\n✅ Pure Rust GUI rendering complete!\n");

    // Cleanup
    display.shutdown().await?;
    println!("✅ Display system shut down cleanly\n");

    Ok(())
}
