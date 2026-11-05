//! Saving exported files on each platform.

/// Result of a save: `Ok(Some(location))`, `Ok(None)` if the user cancelled.
pub type SaveResult = Result<Option<String>, String>;

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn save(bytes: &[u8], file_name: &str, _mime: &str) -> SaveResult {
    let ext = file_name.rsplit('.').next().unwrap_or("bin").to_string();
    let path = rfd::FileDialog::new()
        .set_file_name(file_name)
        .add_filter(ext.to_uppercase(), &[ext.as_str()])
        .save_file();
    match path {
        None => Ok(None),
        Some(p) => {
            std::fs::write(&p, bytes).map_err(|e| e.to_string())?;
            Ok(Some(p.display().to_string()))
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn save(bytes: &[u8], file_name: &str, mime: &str) -> SaveResult {
    use wasm_bindgen::JsCast;
    let err = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&array.buffer());
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = web_sys::Blob::new_with_buffer_source_sequence_and_options(&parts, &opts).map_err(err)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(err)?;
    let a: web_sys::HtmlAnchorElement = document.create_element("a").map_err(err)?.dyn_into().map_err(|_| "anchor")?;
    a.set_href(&url);
    a.set_download(file_name);
    a.click();
    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(Some(file_name.to_string()))
}

#[cfg(target_os = "android")]
pub fn save(bytes: &[u8], file_name: &str, mime: &str) -> SaveResult {
    android::save(bytes, file_name, mime).map(Some)
}

#[cfg(target_os = "android")]
mod android {
    use jni::objects::{JObject, JValue};
    use jni::JNIEnv;

    fn check<T>(env: &mut JNIEnv, r: jni::errors::Result<T>) -> Result<T, String> {
        match r {
            Ok(v) => Ok(v),
            Err(e) => {
                if env.exception_check().unwrap_or(false) {
                    let _ = env.exception_describe();
                    let _ = env.exception_clear();
                }
                Err(e.to_string())
            }
        }
    }

    pub fn save(bytes: &[u8], file_name: &str, mime: &str) -> Result<String, String> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| e.to_string())?;
        let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
        let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
        let sdk = {
            let r = env.get_static_field("android/os/Build$VERSION", "SDK_INT", "I");
            check(&mut env, r)?.i().map_err(|e| e.to_string())?
        };
        if sdk >= 29 {
            if let Ok(p) = media_store(&mut env, &activity, bytes, file_name, mime) {
                return Ok(p);
            }
        }
        // Fallback: the app's own external files folder.
        let dir = {
            let r = env.call_method(&activity, "getExternalFilesDir", "(Ljava/lang/String;)Ljava/io/File;", &[JValue::Object(&JObject::null())]);
            check(&mut env, r)?.l().map_err(|e| e.to_string())?
        };
        if dir.is_null() {
            return Err("storage unavailable".into());
        }
        let path = {
            let r = env.call_method(&dir, "getAbsolutePath", "()Ljava/lang/String;", &[]);
            check(&mut env, r)?.l().map_err(|e| e.to_string())?
        };
        let path: String = env.get_string(&path.into()).map_err(|e| e.to_string())?.into();
        let full = format!("{path}/{file_name}");
        std::fs::write(&full, bytes).map_err(|e| e.to_string())?;
        Ok(full)
    }

    fn media_store(env: &mut JNIEnv, activity: &JObject, bytes: &[u8], file_name: &str, mime: &str) -> Result<String, String> {
        let resolver = {
            let r = env.call_method(activity, "getContentResolver", "()Landroid/content/ContentResolver;", &[]);
            check(env, r)?.l().map_err(|e| e.to_string())?
        };
        let values = {
            let r = env.new_object("android/content/ContentValues", "()V", &[]);
            check(env, r)?
        };
        for (k, v) in [("_display_name", file_name), ("mime_type", mime), ("relative_path", "Download/")] {
            let k = env.new_string(k).map_err(|e| e.to_string())?;
            let v = env.new_string(v).map_err(|e| e.to_string())?;
            let r = env.call_method(&values, "put", "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(&k), JValue::Object(&v)]);
            check(env, r)?;
        }
        let collection = {
            let r = env.get_static_field("android/provider/MediaStore$Downloads", "EXTERNAL_CONTENT_URI", "Landroid/net/Uri;");
            check(env, r)?.l().map_err(|e| e.to_string())?
        };
        let uri = {
            let r = env.call_method(
                &resolver,
                "insert",
                "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
                &[JValue::Object(&collection), JValue::Object(&values)],
            );
            check(env, r)?.l().map_err(|e| e.to_string())?
        };
        if uri.is_null() {
            return Err("insert failed".into());
        }
        let stream = {
            let r = env.call_method(&resolver, "openOutputStream", "(Landroid/net/Uri;)Ljava/io/OutputStream;", &[JValue::Object(&uri)]);
            check(env, r)?.l().map_err(|e| e.to_string())?
        };
        if stream.is_null() {
            return Err("no output stream".into());
        }
        let array = env.byte_array_from_slice(bytes).map_err(|e| e.to_string())?;
        let r = env.call_method(&stream, "write", "([B)V", &[JValue::Object(&array)]);
        check(env, r)?;
        let r = env.call_method(&stream, "close", "()V", &[]);
        check(env, r)?;
        Ok(format!("Download/{file_name}"))
    }
}
