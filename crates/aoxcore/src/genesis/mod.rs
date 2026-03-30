// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC canonical genesis module.
//!
//! This module exposes the production-oriented genesis configuration model and
//! the corresponding loading / persistence workflows.

pub mod config;
pub mod loader;

pub use config::*;
pub use loader::*;
