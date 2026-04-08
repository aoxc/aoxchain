// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/src/block/mod.rs
//!
//! Canonical AOVM block-domain module.
//!
//! This module defines the canonical block-domain data structures, invariants,
//! constructors, validation rules, and reporting entry points for AOVM.
//!
//! Design objectives:
//! - Stable and auditable block-domain data structures
//! - Deterministic validation behavior across nodes
//! - Explicit separation between block types and lifecycle semantics
//! - Production-friendly helpers for hashing, reporting, and parent-link checks
//! - Panic-free validation paths for consensus-relevant logic

pub mod assembly;
pub mod error;
pub mod hash;
pub mod report;

pub use assembly::{
    AssemblyError, AssemblyLane, AssemblyLaneCommitment, CanonicalBlockAssemblyPlan,
};
pub use error::BlockError;
pub use hash::{
    HASH_FORMAT_VERSION, HASH_SIZE, ZERO_HASH, calculate_task_root, compute_hash, empty_task_root,
    hash_header, hash_internal_node, hash_task, hash_task_leaf, try_hash_task, try_hash_task_leaf,
};
pub use report::{
    BlockValidationReport, ErrorDescriptor, ValidationEvent, ValidationEventType,
    build_block_validation_report, describe_block_error,
};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

include!("block_types.rs");
include!("block_impl.rs");
include!("block_tests.rs");
