// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Ethereum-style execution lane.

pub mod executor;
pub mod receipt;
pub mod rpc;
pub mod state;

pub use executor::EvmExecutor;
