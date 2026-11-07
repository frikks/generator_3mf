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

        self.runner
            .start(
                canvas_id,
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
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("generator_3mf"),
    );
    log::info!("Starting Android generator_3mf...");

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
        log::error!("eframe run_native terminated with error: {e:?}");
    }

    // Блокируем завершение функции android_main, чтобы не дропался AndroidApp
    loop {
        std::thread::park();
    }
}