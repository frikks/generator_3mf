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

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("No window found");
        let document = window.document().expect("No document found");

        let loading_ids = ["loading", "loading_text", "loader", "loading-screen"];
        for id in &loading_ids {
            if let Some(elem) = document.get_element_by_id(id) {
                elem.remove();
            }
        }
        if let Ok(Some(elem)) = document.query_selector(".loading") {
            elem.remove();
        }
        if let Ok(Some(elem)) = document.query_selector("#loading") {
            elem.remove();
        }

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .or_else(|| document.get_element_by_id("canvas"))
            .or_else(|| document.query_selector("canvas").ok().flatten())
            .expect("Could not find any <canvas> element on page")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Target element is not a canvas");

        let _ = canvas.style().set_property("pointer-events", "auto");
        let _ = canvas.focus();

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
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );

    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("CRITICAL PANIC: {panic_info:?}");
    }));

    let mut options = eframe::NativeOptions::default();
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app);
    }));

    let _ = eframe::run_native(
        "Generator 3MF",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            Ok(Box::new(GeneratorApp::new(cc)))
        }),
    );
}