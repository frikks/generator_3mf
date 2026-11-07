//! Build helper: `cargo run -p xtask -- web` produces a single, self-contained
//! `dist/generator_3mf-<version>.html` that works from disk without a server.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let task = std::env::args().nth(1).unwrap_or_default();
    match task.as_str() {
        "web" => web(),
        _ => {
            eprintln!("usage: cargo run -p xtask -- web");
            std::process::exit(2);
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn run(cmd: &mut Command) {
    eprintln!("> {cmd:?}");
    let ok = cmd.status().map(|s| s.success()).unwrap_or(false);
    if !ok {
        eprintln!("command failed");
        std::process::exit(1);
    }
}

fn get_version() -> String {
    let root = root();
    let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml")).expect("read root Cargo.toml");
    for line in cargo_toml.lines() {
        let line = line.trim();
        if line.starts_with("version") {
            if let Some(v) = line.split('"').nth(1) {
                return v.to_string();
            }
        }
    }
    "0.1.0".to_string()
}

fn web() {
    let root = root();
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    run(Command::new(&cargo)
        .current_dir(&root)
        .args(["build", "-p", "generator_3mf_app", "--lib", "--release", "--target", "wasm32-unknown-unknown"]));
    let out = root.join("target/web");
    run(Command::new("wasm-bindgen")
        .current_dir(&root)
        .arg("target/wasm32-unknown-unknown/release/generator_3mf.wasm")
        .args(["--out-dir", out.to_str().unwrap(), "--target", "web", "--no-typescript"]));

    let js = std::fs::read_to_string(out.join("generator_3mf.js")).expect("read js");
    let wasm = std::fs::read(out.join("generator_3mf_bg.wasm")).expect("read wasm");
    if js.contains("from './snippets") || js.contains("from \"./snippets") {
        eprintln!("JS snippets are not supported by the single-file build");
        std::process::exit(1);
    }
    let js: String = js
        .lines()
        .filter(|l| !(l.starts_with("export {") || l.starts_with("export default")))
        .map(|l| l.strip_prefix("export ").unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n");
    let template = std::fs::read_to_string(root.join("web/index.html")).expect("read template");
    let version = get_version();
    let boot = format!(
        "{js}\nconst __b64 = \"{}\";\nconst __bin = Uint8Array.from(atob(__b64), c => c.charCodeAt(0));\nawait __wbg_init({{ module_or_path: __bin }});\n",
        base64(&wasm)
    );
    let html = template.replace("{{VERSION}}", &version).replace("{{SCRIPT}}", &boot.replace("</script", "<\\/script"));
    let dist = root.join("dist");
    std::fs::create_dir_all(&dist).unwrap();
    let path = dist.join(format!("generator_3mf-{version}.html"));
    std::fs::write(&path, html).unwrap();
    eprintln!("wrote {} ({} KiB)", path.display(), std::fs::metadata(&path).unwrap().len() / 1024);
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let b = [c[0], *c.get(1).unwrap_or(&0), *c.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        s.push(T[(n >> 18) as usize & 63] as char);
        s.push(T[(n >> 12) as usize & 63] as char);
        s.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        s.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    s
}