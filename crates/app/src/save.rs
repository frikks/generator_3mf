/// Сохранение файла в браузере (скачивание через виртуальный blob)
#[cfg(target_arch = "wasm32")]
pub fn save(data: &[u8], filename: &str, mime: &str) -> Result<Option<String>, String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or_else(|| "Window not available".to_string())?;
    let document = window
        .document()
        .ok_or_else(|| "Document not available".to_string())?;

    let uint8_array = js_sys::Uint8Array::new_with_length(data.len() as u32);
    uint8_array.copy_from(data);

    let array = js_sys::Array::new();
    array.push(&uint8_array.buffer());

    let blob_props = web_sys::BlobPropertyBag::new();
    blob_props.set_type(mime);

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &blob_props)
        .map_err(|e| format!("Failed to create blob: {e:?}"))?;

    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create object url: {e:?}"))?;

    let anchor = document
        .create_element("a")
        .map_err(|e| format!("Failed to create anchor: {e:?}"))?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|e| format!("Failed to cast anchor: {e:?}"))?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(Some(filename.to_string()))
}

/// Сохранение на настольных ОС (Windows / Linux / macOS через диалоговое окно)
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn save(data: &[u8], filename: &str, _mime: &str) -> Result<Option<String>, String> {
    let path = rfd::FileDialog::new()
        .set_file_name(filename)
        .save_file();

    if let Some(path) = path {
        std::fs::write(&path, data).map_err(|e| format!("Failed to write file: {e}"))?;
        Ok(Some(path.display().to_string()))
    } else {
        Ok(None)
    }
}

/// Заглушка для Android
#[cfg(target_os = "android")]
pub fn save(_data: &[u8], filename: &str, _mime: &str) -> Result<Option<String>, String> {
    log::info!("Save requested on Android for {}", filename);
    Ok(Some(filename.to_string()))
}