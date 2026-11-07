pub mod app;
pub mod i18n;
pub mod preview;
pub mod save;
pub mod theme;

pub use app::GeneratorApp;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: &str) -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);

        let document = web_sys::window()
            .and_then(|win| win.document())
            .ok_or_else(|| JsValue::from_str("No document found"))?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str(&format!("Canvas '{canvas_id}' not found")))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("Element is not an HtmlCanvasElement"))?;

        self.runner
            .start(
                &canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    cc.egui_ctx.set_visuals(egui::Visuals::dark());
                    Ok(Box::new(GeneratorApp::new(cc)))
                }),
            )
            .await
            .map_err(|err| JsValue::from_str(&format!("{err:?}")))?;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WebHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "android")]
#[android_activity::main(flavor = "native_activity")]
fn android_main(app: android_activity::AndroidApp) {
    use std::fs::OpenOptions;

    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("generator_3mf"),
    );
    log::info!("=== ANDROID MAIN STARTED ===");

    let mut options = eframe::NativeOptions::default();
    options.android_app = Some(app);

    let res = eframe::run_native(
        "ГЕНЕРАТОР 3MF",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(GeneratorApp::new(cc)))
        }),
    );

    if let Err(e) = res {
        log::error!("eframe run_native failed: {e:?}");
    }
}