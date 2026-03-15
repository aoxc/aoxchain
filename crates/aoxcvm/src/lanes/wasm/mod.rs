//! Deterministic WASM execution lane.

pub mod executor;
pub mod receipt;
pub mod state;

pub use executor::WasmExecutor;
