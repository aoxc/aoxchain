// AOXC MIT License
// Production-oriented protocol message envelope primitive.

mod core;
pub mod interop;
pub mod quantum;

pub use core::*;

#[cfg(test)]
mod tests;
