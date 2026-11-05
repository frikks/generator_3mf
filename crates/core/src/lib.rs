//! Core geometry for Gridfinity bins and baseplates.
//!
//! Everything here is plain Rust without platform dependencies, so the same
//! code runs on desktop, Android and WebAssembly.

pub mod export;
pub mod gridfinity;
pub mod mesh;
pub mod solid;

pub use gridfinity::{generate, Kind, Params, Warning};
pub use mesh::Mesh;

/// Crate version, shared by all front-ends.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
