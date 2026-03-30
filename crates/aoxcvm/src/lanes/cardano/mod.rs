// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Cardano-style eUTxO validation lane.

pub mod executor;
pub mod receipt;
pub mod script;
pub mod utxo_store;

pub use executor::CardanoExecutor;
