// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.
//
// Production-oriented native token ledger with replay-hardened transfer support.
// This implementation is designed to remain deterministic, auditable, and
// compatible with a hardened receipt primitive.
//
// Security objectives:
// - strict policy validation
// - bounded replay metadata validation
// - deterministic anti-replay commitment derivation
// - safe arithmetic discipline
// - receipt construction compatible with fail-closed receipt APIs
// - no dead code and no placeholder branches

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::asset::SupplyModel;
use crate::receipts::{Event, HASH_SIZE, Receipt, ReceiptError};

include!("native_token_types.rs");
include!("native_token_ledger.rs");
include!("native_token_tests.rs");
