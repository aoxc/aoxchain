// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC canonical genesis configuration model.
//!
//! This module defines a production-oriented genesis configuration surface for
//! AOXC-native networks.
//!
//! Design objectives:
//! - deterministic identity derivation,
//! - canonical and reproducible genesis fingerprinting,
//! - strict validation of policy-bearing fields,
//! - future-compatible network profile governance,
//! - quantum-aware configuration posture,
//! - explicit protection against non-canonical hashing inputs.
//!
//! Security rationale:
//! This module must not derive consensus-sensitive fingerprints from debug
//! formatting, unstable serialization order, or runtime-dependent defaults.
//! All fingerprintable data is encoded through explicit canonical framing.

use core::fmt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

include!("constants_network.rs");
include!("chain_identity.rs");
include!("genesis_config.rs");
include!("errors.rs");
include!("entities.rs");
include!("canonical_encoder.rs");
include!("tests.rs");
