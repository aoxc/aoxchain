// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use super::types::*;
use crate::{
    cli::{AOXC_RELEASE_NAME, TESTNET_FIXTURE_MEMBERS},
    cli_support::{arg_value, emit_serialized, has_flag, output_format, text_envelope},
    config::{
        loader::{init_default, load, persist},
        settings::Settings,
    },
    data_home::{ScopedHomeOverride, ensure_layout, read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    keys::manager::{
        bootstrap_operator_key, consensus_public_key_hex, inspect_operator_key,
        operator_fingerprint, rotate_operator_key, verify_operator_key,
    },
    node::lifecycle::bootstrap_state,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

const DEFAULT_VALIDATOR_GENESIS_BALANCE: &str = "50000000";

mod commands_bootstrap;
mod commands_genesis;
mod commands_identity;
mod helpers_args;
mod helpers_core;

pub use commands_bootstrap::{
    cmd_config_init, cmd_config_print, cmd_config_validate, cmd_dual_profile_bootstrap,
    cmd_production_bootstrap,
};
pub use commands_genesis::{
    cmd_consensus_profile_audit, cmd_genesis_add_account, cmd_genesis_add_validator,
    cmd_genesis_hash, cmd_genesis_init, cmd_genesis_inspect, cmd_genesis_security_audit,
    cmd_genesis_template_advanced, cmd_genesis_validate, consensus_profile_gate_status,
};
pub use commands_identity::{
    cmd_address_create, cmd_key_bootstrap, cmd_key_rotate, cmd_keys_inspect,
    cmd_keys_show_fingerprint, cmd_keys_verify, cmd_testnet_fixture_init, genesis_ready,
};

use commands_genesis::evaluate_consensus_profile_audit;
use commands_identity::genesis_path;
use helpers_args::*;
use helpers_core::*;

#[cfg(test)]
include!("helpers_tests.rs");
