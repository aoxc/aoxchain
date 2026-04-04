// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::BTreeSet;
use std::fmt;

use crate::identity::{
    hd_path::{HdPath, HdPathError},
    key_engine::{DERIVED_ENTROPY_LEN, KeyEngine, KeyEngineError},
    keyfile::{KeyfileEnvelope, KeyfileError, validate_envelope},
};

include!("key_bundle_types.rs");
include!("key_bundle_impl.rs");
include!("key_bundle_tests.rs");
