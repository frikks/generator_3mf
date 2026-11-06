#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use generator_3mf_app::GeneratorApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1020.0, 720.0])
            .with_min_inner_size([480.0, 360.0])
            .with_title("ГЕНЕратор"),
        ..Default::default()
    };

    eframe::run_native(
        "ГЕНЕратор",
        native_options,
        Box::new(|cc| Ok(Box::new(GeneratorApp::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}