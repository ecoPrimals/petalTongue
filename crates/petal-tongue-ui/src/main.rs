//! Main entry point for petalTongue desktop UI

use petal_tongue_ui::PetalTongueApp;

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    tracing::info!("Starting petalTongue Universal Representation System");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("🌸 petalTongue - Universal Representation System"),
        ..Default::default()
    };
    
    eframe::run_native(
        "petalTongue",
        options,
        Box::new(|cc| Ok(Box::new(PetalTongueApp::new(cc)))),
    )
}

