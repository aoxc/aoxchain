// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC keyforge command module registry.
//!
//! This module tree defines the operator-facing command surfaces for:
//! - identity derivation,
//! - certificate and passport inspection workflows,
//! - key custody and keyfile operations,
//! - registry and revocation management,
//! - quorum evaluation,
//! - ZKP setup artifact initialization.
//!
//! Design goals:
//! - keep module discovery predictable,
//! - preserve a stable command-surface layout,
//! - centralize shared file and serialization helpers.

/// Shared CLI type definitions and command contracts.
pub mod cli;

/// Shared filesystem and serialization helpers.
///
/// Security note:
/// this module is part of the command-plane trust boundary because it governs
/// file reads, atomic writes, and persistence semantics for operator artifacts.
pub mod util;

/// Identity and key management command handlers.
pub mod cmd_actor_id;
pub mod cmd_key;
pub mod cmd_keyfile;

/// Certificate and passport command handlers.
pub mod cmd_cert;
pub mod cmd_passport;

/// Registry and revocation command handlers.
pub mod cmd_registry;
pub mod cmd_revoke;

/// Operational evaluation and setup command handlers.
pub mod cmd_quorum;
pub mod cmd_zkp_setup;
