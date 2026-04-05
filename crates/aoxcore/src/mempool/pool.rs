// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC production-grade FIFO mempool.
//!
//! This module implements a bounded, deterministic, single-threaded FIFO
//! mempool with explicit admission controls, duplicate suppression, expiry
//! policy, and operator-facing telemetry snapshots.
//!
//! Design objectives:
//! - deterministic FIFO collection order,
//! - explicit and validated resource bounds,
//! - duplicate suppression by canonical transaction identifier,
//! - explicit expiry and eviction behavior,
//! - telemetry-friendly rejection and lifecycle statistics,
//! - panic-free accounting and state transitions.
//!
//! Security rationale:
//! The mempool is not a consensus object by itself, but weak admission policy,
//! imprecise accounting, or ambiguous eviction rules can still become an
//! operational denial-of-service surface. All relevant bounds are therefore
//! explicit, validated, and enforced before insertion.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::{Duration, Instant};

include!("pool_types.rs");
include!("pool_impl.rs");
include!("pool_tests.rs");
