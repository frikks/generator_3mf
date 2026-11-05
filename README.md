# generator_3mf

Gridfinity bin and baseplate generator written in pure Rust, with 3MF and STL export.
Генератор контейнеров и оснований Gridfinity на чистом Rust с экспортом в 3MF и STL.

One code base, three builds / Один код — три сборки:

| Build | File | Notes |
| --- | --- | --- |
| Windows | `generator_3mf-0.1.0-windows-x64.exe` | Single portable executable |
| Android | `generator_3mf-0.1.0-android.apk` | Android 7.0+, arm64 / armv7. Files are saved to `Download/` |
| Web | `generator_3mf-0.1.0.html` | One self-contained file: open it in a browser, works offline |

Download them from [Releases](../../releases).

## Features / Возможности

- **Bins / Контейнеры**: 1–8 × 1–8 units of 42 mm, height 2–18u (7 mm), up to 8 × 8 compartments.
  - stacking lip / бортик для штабелирования
  - magnet pockets Ø6.5 × 2.4 mm / гнёзда под магниты
  - scoop / скос для выемки
  - label ledge / полка для этикетки
  - M3 screw holes / отверстия под винты M3
  - wall 0.8–2.4 mm / толщина стенки
- **Baseplates / Основания**: 1–8 × 1–8 units, optional floor with magnet pockets.
- **Export / Экспорт**: 3MF (millimetres, colour) and binary STL. Every mesh is closed and manifold, so slicers open it without repair.
- **Preview / Просмотр**: live 3D view, rotate and zoom with mouse or touch.
- **Themes / Темы**: Classic, Light, Dark, AMOLED.
- **Languages / Языки**: English, Русский.

## Geometry / Геометрия

Dimensions follow the open Gridfinity standard: 42 mm grid, 7 mm height units, 0.5 mm clearance,
foot profile 0.8 + 1.8 + 2.15 mm with 45° chamfers, magnets 13 mm from the cell centre.
Parts print flat without supports.

## Project layout / Структура

```
crates/core   geometry, mesh repair, 3MF/STL writers (no platform dependencies)
crates/app    egui interface for desktop, Android and web
xtask         build helper for the single-file HTML
web           HTML template
```

## Building / Сборка

```sh
# Desktop (Windows / Linux / macOS)
cargo run --release -p generator_3mf

# Tests: every generated mesh is checked for open edges
cargo test --release -p generator_3mf_core

# Web: single self-contained HTML in dist/
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130
cargo run -p xtask -- web

# Android: needs Android SDK + NDK (ANDROID_HOME, ANDROID_NDK_ROOT)
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-apk
export CARGO_APK_RELEASE_KEYSTORE=~/.android/debug.keystore CARGO_APK_RELEASE_KEYSTORE_PASSWORD=android
cargo apk build -p generator_3mf --release --lib
```

Pushing to `main` builds all three variants in GitHub Actions and publishes the release for the version in `Cargo.toml`.

## License

MIT
