pub fn save_file(data: &[u8], default_name: &str, _ext: &str) -> Result<Option<String>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        use eframe::wasm_bindgen::JsCast;
        let window = web_sys::window().ok_or_else(|| "No window found".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "No document found".to_string())?;

        let array = js_sys::Array::new();
        let uint8_array = js_sys::Uint8Array::new_with_length(data.len() as u32);
        uint8_array.copy_from(data);
        array.push(&uint8_array.buffer());

        let property_bag = web_sys::BlobPropertyBag::new();
        property_bag.set_type("application/vnd.ms-package.3dmanufacturing-3dmodel+xml");

        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &property_bag)
            .map_err(|e| format!("{e:?}"))?;
        let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|e| format!("{e:?}"))?;

        let element = document.create_element("a").map_err(|e| format!("{e:?}"))?;
        let anchor = element
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| "Failed to cast element to HtmlAnchorElement".to_string())?;

        anchor.set_href(&url);
        anchor.set_download(default_name);
        anchor.click();

        let _ = web_sys::Url::revoke_object_url(&url);
        Ok(Some(default_name.to_string()))
    }

    #[cfg(target_os = "android")]
    {
        let download_dir = std::path::PathBuf::from("/sdcard/Download");
        let target_path = download_dir.join(default_name);

        match std::fs::write(&target_path, data) {
            Ok(_) => {
                let path_str = target_path.display().to_string();
                log::info!("Successfully saved file to: {path_str}");
                Ok(Some(path_str))
            }
            Err(e) => {
                let err_msg = format!("Failed to save file to {}: {e}", target_path.display());
                log::error!("{err_msg}");
                Err(err_msg)
            }
        }
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        let mut dialog = rfd::FileDialog::new().set_file_name(default_name);
        if !_ext.is_empty() {
            dialog = dialog.add_filter("3MF Model", &[_ext]);
        }

        if let Some(path) = dialog.save_file() {
            std::fs::write(&path, data).map_err(|e| e.to_string())?;
            Ok(Some(path.display().to_string()))
        } else {
            Ok(None)
        }
    }
}

pub fn save(data: &[u8], default_name: &str, ext: &str) -> Result<Option<String>, String> {
    save_file(data, default_name, ext)
}

pub fn save_file_dialog(default_name: &str, data: &[u8]) -> Result<Option<String>, String> {
    save_file(data, default_name, "3mf")
}
