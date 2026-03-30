// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Deterministic WASM execution lane.

pub mod executor;
pub mod receipt;
pub mod state;

pub use executor::WasmExecutor;
