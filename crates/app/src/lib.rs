//! ГЕНЕратор: desktop, Android and web front-end.

mod app;
mod i18n;
mod preview;
mod save;
mod theme;

pub use app::GeneratorApp;

pub const APP_ID: &str = "generator_3mf";

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn run_native() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("ГЕНЕратор")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([420.0, 560.0])
            .with_icon(app::icon()),
        ..Default::default()
    };
    eframe::run_native(APP_ID, options, Box::new(|cc| Ok(Box::new(GeneratorApp::new(cc)))))
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let options = eframe::NativeOptions {
        persistence_path: app.internal_data_path().map(|p| p.join("state.ron")),
        android_app: Some(app),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(APP_ID, options, Box::new(|cc| Ok(Box::new(GeneratorApp::new(cc))))) {
        log::error!("generator_3mf failed: {e}");
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::GeneratorApp;
    use eframe::wasm_bindgen::{self, prelude::*, JsCast};

    #[wasm_bindgen(start)]
    pub fn start() {
        eframe::WebLogger::init(log::LevelFilter::Warn).ok();
        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window().expect("window").document().expect("document");
            let canvas = document
                .get_element_by_id("app")
                .expect("canvas #app")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("canvas element");
            let result = eframe::WebRunner::new()
                .start(canvas, eframe::WebOptions::default(), Box::new(|cc| Ok(Box::new(GeneratorApp::new(cc)))))
                .await;
            if let Some(loading) = document.get_element_by_id("loading") {
                match result {
                    Ok(()) => loading.remove(),
                    Err(e) => loading.set_inner_html(&format!("Failed to start: {e:?}")),
                }
            }
        });
    }
}
