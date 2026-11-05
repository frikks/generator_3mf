#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn main() -> eframe::Result {
    generator_3mf::run_native()
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
fn main() {}
