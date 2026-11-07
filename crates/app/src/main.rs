#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use generator_3mf_app::GeneratorApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("Generator 3MF"),
        ..Default::default()
    };

    eframe::run_native(
        "Generator 3MF",
        native_options,
        Box::new(|cc| Ok(Box::new(GeneratorApp::new(cc)))),
    )
}