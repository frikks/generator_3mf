#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn main() -> eframe::Result<()> {
    use eframe::egui;

    let _ = env_logger::try_init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("ГЕНЕРАТОР 3MF")
            .with_icon(generator_3mf::app::icon()), // <-- добавить эту строку
        ..Default::default()
    };

    eframe::run_native(
        "ГЕНЕРАТОР 3MF",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(generator_3mf::GeneratorApp::new(cc)))
        }),
    )
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
fn main() {}