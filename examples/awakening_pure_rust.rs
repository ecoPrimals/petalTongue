// SPDX-License-Identifier: AGPL-3.0-only
//! Awakening Experience via Pure Rust Rendering
//!
//! Demonstrates the complete awakening experience rendered through
//! the Pure Rust display pipeline (no OpenGL required!).
//!
//! This is the full 4-stage awakening journey:
//! 1. Awakening (0-3s) - Flower opening
//! 2. Self-Knowledge (3-6s) - Identity
//! 3. Discovery (6-10s) - Exploring
//! 4. Tutorial (10-12s) - Invitation

use anyhow::Result;
use petal_tongue_ui::awakening_overlay::AwakeningOverlay;
use petal_tongue_ui::display::{DisplayManager, EguiPixelRenderer};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("\n════════════════════════════════════════════════════════════");
    println!("   🌸 petalTongue Awakening Experience");
    println!("   Pure Rust Rendering (No OpenGL!)");
    println!("════════════════════════════════════════════════════════════\n");

    // Initialize display system
    println!("🎨 Initializing display system...");
    let mut display = DisplayManager::init().await?;
    let backend_name = display
        .active_backend_name()
        .unwrap_or("Unknown")
        .to_string();
    println!("✅ Active backend: {backend_name}\n");

    // Get display dimensions
    let (width, height) = display
        .dimensions()
        .ok_or_else(|| anyhow::anyhow!("No active display backend"))?;
    println!("📐 Display dimensions: {width}x{height}\n");

    // Create egui context
    let ctx = egui::Context::default();

    // Create pixel renderer
    let mut renderer = EguiPixelRenderer::new(width, height);
    println!("✅ Pixel renderer ready\n");

    // Create awakening overlay
    let mut awakening = AwakeningOverlay::new();
    awakening.start();
    println!("🌸 Starting awakening experience...\n");

    println!("════════════════════════════════════════════════════════════");
    println!("   🎬 4-Stage Awakening Journey (12 seconds)");
    println!("════════════════════════════════════════════════════════════\n");
    println!("  Stage 1: Awakening (0-3s) - Flower opens");
    println!("  Stage 2: Self-Knowledge (3-6s) - Identity emerges");
    println!("  Stage 3: Discovery (6-10s) - Exploring connections");
    println!("  Stage 4: Tutorial (10-12s) - Ready to guide\n");

    let start = Instant::now();
    let mut frame_count = 0;
    let target_fps = 60;
    let frame_duration = Duration::from_millis(1000 / target_fps);

    // Awakening loop (render until complete)
    while awakening.is_active() {
        let frame_start = Instant::now();

        // Calculate delta time
        let delta_time = frame_duration.as_secs_f32();

        // Update awakening state
        awakening.update(delta_time)?;

        // Create UI
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            awakening.render(ctx);
        });

        // Tessellate
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);

        // Render to pixels
        let buffer = renderer.render(&primitives)?;

        // Present via display backend
        display.present(&buffer).await?;

        frame_count += 1;

        // Progress indicator (every second)
        let elapsed = start.elapsed().as_secs_f32();
        if frame_count % target_fps == 0 {
            let stage = if elapsed < 3.0 {
                "Awakening"
            } else if elapsed < 6.0 {
                "Self-Knowledge"
            } else if elapsed < 10.0 {
                "Discovery"
            } else {
                "Tutorial"
            };
            println!("  [{elapsed:.1}s] Stage: {stage} (Frame {frame_count})");
        }

        // Frame timing
        let frame_time = frame_start.elapsed();
        if frame_time < frame_duration {
            tokio::time::sleep(
                frame_duration
                    .checked_sub(frame_time)
                    .unwrap_or(Duration::ZERO),
            )
            .await;
        }

        // Request repaint
        ctx.request_repaint();
    }

    let total_time = start.elapsed();
    #[expect(clippy::cast_possible_truncation, reason = "frame count is small")]
    let avg_frame_time = total_time / (frame_count as u32);
    let fps = 1.0 / avg_frame_time.as_secs_f64();

    println!("\n════════════════════════════════════════════════════════════");
    println!("   ✅ Awakening Complete!");
    println!("════════════════════════════════════════════════════════════\n");

    println!("📊 Performance Metrics:");
    println!("   Total time: {:.2}s", total_time.as_secs_f32());
    println!("   Frames rendered: {frame_count}");
    println!(
        "   Average frame time: {:.2}ms",
        avg_frame_time.as_secs_f32() * 1000.0
    );
    println!("   Average FPS: {fps:.1}\n");

    println!("🎯 Achievement Unlocked:");
    println!("   ✅ Full awakening experience via Pure Rust!");
    println!("   ✅ Zero OpenGL required");
    println!("   ✅ Zero display server required");
    println!("   ✅ Complete GUI sovereignty");
    println!("   ✅ Backend: {backend_name}\n");

    // Check for tutorial transition
    if awakening.should_transition_to_tutorial() {
        println!("🎓 Ready to transition to tutorial mode\n");
    }

    // Cleanup
    display.shutdown().await?;
    println!("✅ Display system shutdown cleanly\n");

    println!("════════════════════════════════════════════════════════════");
    println!("   🌸 This is the future of sovereign software");
    println!("════════════════════════════════════════════════════════════\n");

    Ok(())
}
