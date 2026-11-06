pub mod app;
pub mod i18n;
pub mod preview;
pub mod save;
pub mod theme;

pub use app::GeneratorApp;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start_web() -> Result<(), JsValue> {
    use wasm_bindgen::JsCast;

    // Выводим читаемые паники Rust в консоль браузера
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window found")
            .document()
            .expect("No document found");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .or_else(|| document.get_element_by_id("canvas"))
            .or_else(|| {
                document
                    .query_selector("canvas")
                    .ok()
                    .flatten()
            })
            .expect("Could not find any <canvas> element on page")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Target element is not a canvas");

        let runner = eframe::WebRunner::new();
        if let Err(err) = runner
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(GeneratorApp::new(cc)))),
            )
            .await
        {
            log::error!("Failed to start eframe runner: {err:?}");
        }
    });

    Ok(())
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let mut options = eframe::NativeOptions::default();
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app);
    }));

    let _ = eframe::run_native(
        "Generator 3MF",
        options,
        Box::new(|cc| Ok(Box::new(GeneratorApp::new(cc)))),
    );
}