// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOX HyperVM foundation layer.
//!
//! This module provides a forward-compatible skeleton for a deterministic,
//! multi-lane execution kernel with post-quantum-ready signature policies.

pub mod kernel;
pub mod lane;
pub mod pq;
pub mod scheduler;

pub use kernel::{ExecutionEnvelope, HyperVm, HyperVmConfig, HyperVmError, HyperVmResult};
pub use lane::{LaneDescriptor, LaneExecutor, LaneId, LaneRegistry};
pub use pq::{HybridSignature, HybridSigner, SignaturePolicy};
pub use scheduler::{DeterministicScheduler, SchedulingDecision};
