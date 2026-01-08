//! Egui Pixel Renderer Demo
//!
//! Demonstrates the EguiPixelRenderer converting egui UI to RGBA8 pixels.
//! This is a proof-of-concept for the Pure Rust Display System.

use anyhow::Result;
use petal_tongue_ui::display::EguiPixelRenderer;
use std::fs;

fn main() -> Result<()> {
    println!("🎨 Egui Pixel Renderer Demo\n");

    // Create renderer
    let mut renderer = EguiPixelRenderer::new(800, 600);
    println!("✅ Created renderer: 800x600");

    // Create egui context
    let ctx = egui::Context::default();
    println!("✅ Created egui context");

    // Create a simple UI
    let output = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌸 petalTongue");
            ui.label("Pure Rust Pixel Rendering");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.colored_label(egui::Color32::GREEN, "✅ Working");
            });

            ui.add_space(20.0);

            if ui.button("Click Me!").clicked() {
                println!("Button clicked!");
            }

            ui.add_space(20.0);

            ui.label("This UI is rendered to pixels using:");
            ui.label("• egui (UI framework)");
            ui.label("• epaint (tessellation)");
            ui.label("• tiny-skia (rasterization)");
            ui.label("• 100% Pure Rust!");
        });
    });

    println!("✅ Generated UI");

    // Get primitives
    let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
    println!("✅ Tessellated {} primitives", primitives.len());

    // Render to pixel buffer
    let buffer = renderer.render(&primitives)?;
    println!("✅ Rendered to {} bytes", buffer.len());

    // Save as PNG for verification
    let output_path = "/tmp/petaltongue_pixel_render_demo.png";
    save_as_png(&buffer, 800, 600, output_path)?;
    println!("✅ Saved to: {}", output_path);

    println!("\n════════════════════════════════════════════════════════════");
    println!("   🏆 SUCCESS! Egui → Pixels Working!");
    println!("════════════════════════════════════════════════════════════\n");

    println!("The rendered UI has been saved to:");
    println!("  {}", output_path);
    println!("\nYou can view it with:");
    println!("  xdg-open {}", output_path);
    println!("\nThis demonstrates that petalTongue can render its full UI");
    println!("to a pixel buffer without OpenGL or a display server! 🌸");

    Ok(())
}

fn save_as_png(buffer: &[u8], width: u32, height: u32, path: &str) -> Result<()> {
    let file = fs::File::create(path)?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(buffer)?;

    Ok(())
}
