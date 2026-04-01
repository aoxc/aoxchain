// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC Multi-VM Runtime (AOVM)
//!
//! This crate implements a deterministic multi-lane execution runtime
//! capable of hosting EVM, Move/Sui-style, WASM and Cardano-style lanes
//! under a shared settlement host.

pub mod compatibility;
pub mod context;
pub mod contracts;
pub mod error;
pub mod gas;
pub mod host;
pub mod hypervm;
pub mod kernel;
pub mod lanes;
pub mod language;
pub mod routing;
pub mod system;
pub mod vm_kind;
