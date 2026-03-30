// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOX shared low-level utility crate.
//! Provides encoding/time helpers and shared error definitions.

pub mod encoding;
pub mod time;
pub mod types;

pub use types::LibError;
