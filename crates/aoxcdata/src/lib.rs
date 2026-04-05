// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

pub mod contracts;
pub mod store;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions, create_dir_all};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

include!("data_types.rs");
include!("data_store.rs");
include!("data_tests.rs");
