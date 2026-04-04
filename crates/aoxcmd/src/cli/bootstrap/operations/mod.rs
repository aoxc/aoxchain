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

#[cfg(test)]
use commands_genesis::evaluate_consensus_profile_audit;
use commands_identity::genesis_path;
use helpers_args::{
    is_decimal_string, is_non_zero_decimal_string, parse_optional_text_arg,
    parse_required_or_default_text_arg, parse_required_text_arg,
};
use helpers_core::{
    bootstrap_profile_directory, build_profile_settings, derive_short_fingerprint,
    load_genesis, load_or_default_bootnodes_binding, load_or_default_validators_binding,
    materialize_binding_documents, persist_bootnodes_binding, persist_genesis,
    persist_validators_binding, sync_optional_accounts_binding, upsert_bootnode_binding,
    upsert_validator_account, upsert_validator_binding, validate_binding_files,
    validate_genesis, write_json_pretty,
};

#[cfg(test)]
mod helpers_tests;

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
