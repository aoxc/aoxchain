// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC identity module public surface.
//!
//! This module exposes the canonical identity-layer building blocks used across
//! the AOXC stack, including:
//! - actor identity derivation,
//! - certificate issuance and validation,
//! - registry and revocation state,
//! - deterministic seed lifecycle,
//! - deterministic key derivation,
//! - encrypted key custody,
//! - operational key bundles,
//! - threshold-signature policy envelopes,
//! - pseudo-ZKP integration envelopes.
//!
//! Design objectives:
//! - maintain a stable public surface for downstream crates,
//! - keep root-seed lifecycle separate from deterministic derivation,
//! - avoid unnecessary leakage of internal-only helper APIs,
//! - preserve compatibility with existing AOXC consumers while exposing the
//!   hardened production-oriented interfaces required by higher layers.

pub mod actor_id;
pub mod ca;
pub mod certificate;
pub mod ed25519_keys;
pub mod gate;
pub mod handshake;
pub mod hd_path;
pub mod hexa_quorum;
pub mod key_bundle;
pub mod key_engine;
pub mod keyfile;
pub mod passport;
pub mod pq_keys;
pub mod registry;
pub mod revocation;
pub mod seed;
pub mod threshold_sig;
pub mod zkp_engine;
pub mod mnemonic;

pub use ed25519_keys::{
    AOXC_ED25519_PUBLIC_KEY_LEN,
    AOXC_ED25519_SEED_LEN,
    derive_ed25519_seed,
    derive_ed25519_signing_key,
    derive_ed25519_verifying_key,
    encode_ed25519_public_key_hex,
    fingerprint_ed25519_public_key,
};

pub use key_bundle::{
    AOXC_PUBLIC_KEY_ENCODING,
    CryptoProfile,
    NodeKeyBundleError,
    NodeKeyBundleV1,
    NodeKeyRecord,
    NodeKeyRole,
};

pub use key_engine::{
    AOXC_HD_BIP44_PURPOSE,
    AOXC_HD_PURPOSE,
    DERIVED_ENTROPY_LEN,
    KeyEngine,
    KeyEngineError,
    MASTER_SEED_LEN,
    ROLE_SEED_LEN,
    derive_role_seed_from_material,
};

pub use seed::{
    AOXC_SEED_VERSION,
    GeneratedSeed,
    RECOVERY_SEED_LEN,
    SEED_FINGERPRINT_LEN,
    SeedError,
    SeedKind,
    SeedMetadata,
    generate_seed,
    generate_seed_with_additional_entropy,
};

pub use mnemonic::{
    AOXC_MNEMONIC_LANGUAGE,
    AOXC_MNEMONIC_VERSION,
    AOXC_MNEMONIC_WORD_COUNT,
    MnemonicBackup,
    MnemonicError,
    MnemonicMetadata,
    encode_recovery_seed_as_phrase,
    generate_seed_and_mnemonic,
    generate_seed_and_mnemonic_with_additional_entropy,
    restore_seed_from_phrase,
};
