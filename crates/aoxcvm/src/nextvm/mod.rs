//! AOXCVM-NEXT bootstrap module.
//!
//! This module is the first step for a ground-up VM track that focuses on:
//! - deterministic execution,
//! - capability-gated host interaction,
//! - crypto agility with post-quantum migration hooks.

pub mod crypto;
pub mod error;
pub mod host;
pub mod opcode;
pub mod state;
pub mod vm;

pub use crypto::{CryptoProfile, SignatureEnvelope};
pub use error::NextVmError;
pub use host::{HostAdapter, HostCallRequest, NullHost};
pub use opcode::{Instruction, Opcode};
pub use state::{Capability, StateStore};
pub use vm::{ExecutionOutcome, VmConfig, VmExecution};
