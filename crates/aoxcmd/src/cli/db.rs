// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    cli_support::{arg_value, emit_serialized, output_format},
    data_home::{ensure_layout, resolve_home},
    error::{AppError, ErrorCode},
};
use aoxcdata::{BlockEnvelope, HybridDataStore, IndexBackend};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

include!("db_support.rs");
include!("db_commands.rs");
include!("db_tests/tests.rs");
