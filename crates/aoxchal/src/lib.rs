//! AOXChain hardware-adjacent utility layer.
//!
//! This crate intentionally exposes a narrow, deterministic, and testable API
//! surface for CPU capability inspection and bounded in-memory region handling.
//! The current scope is deliberately conservative. It is designed to support
//! higher-level policy, cryptographic path selection, and controlled buffer
//! management without introducing speculative or partially-defined behavior.

pub mod cpu_opt;
pub mod mem_manager;

pub use cpu_opt::CpuCapabilities;
pub use mem_manager::{MemoryRegion, MemoryRegionError};
