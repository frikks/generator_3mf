// ============================================================================
// Desktop (Windows / Linux / macOS): системный диалог rfd
// ============================================================================
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn save(bytes: &[u8], filename: &str, _mime: &str) -> Result<Option<String>, String> {
    let dialog = rfd::FileDialog::new()
        .set_file_name(filename)
        .add_filter("3MF Model", &["3mf"]);

    if let Some(path) = dialog.save_file() {
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        Ok(Some(path.display().to_string()))
    } else {
        Ok(None)
    }
}

// ============================================================================
// WebAssembly: скачивание через браузерный Blob
// ============================================================================
#[cfg(target_arch = "wasm32")]
pub fn save(bytes: &[u8], filename: &str, mime: &str) -> Result<Option<String>, String> {
    use wasm_bindgen::JsCast;

    let err = |e: wasm_bindgen::JsValue| format!("{e:?}");

    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&array.buffer());

    let mut opts = web_sys::BlobPropertyBag::new();
    opts.type_(mime);

    let blob = web_sys::Blob::new_with_buffer_source_sequence_and_options(&parts, &opts)
        .map_err(err)?;

    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(err)?;

    let window = web_sys::window().ok_or_else(|| "no window found".to_string())?;
    let document = window
        .document()
        .ok_or_else(|| "no document found".to_string())?;

    let anchor = document
        .create_element("a")
        .map_err(err)?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "failed to cast to HtmlAnchorElement".to_string())?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    let _ = web_sys::Url::revoke_object_url(&url);

    Ok(Some(filename.to_string()))
}

// ============================================================================
// Android: сохранение в Downloads или локальную папку
// ============================================================================
#[cfg(target_os = "android")]
pub fn save(bytes: &[u8], filename: &str, _mime: &str) -> Result<Option<String>, String> {
    let candidates = [
        std::path::PathBuf::from("/sdcard/Download"),
        std::path::PathBuf::from("/storage/emulated/0/Download"),
    ];

    for dir in &candidates {
        if dir.exists() {
            let path = dir.join(filename);
            if std::fs::write(&path, bytes).is_ok() {
                return Ok(Some(path.display().to_string()));
            }
        }
    }

    let fallback = std::path::PathBuf::from(format!(
        "/data/data/com.example.generator_3mf/files/{}",
        filename
    ));
    if let Some(parent) = fallback.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&fallback, bytes).map_err(|e| format!("Failed to save on Android: {e}"))?;

    Ok(Some(fallback.display().to_string()))
}