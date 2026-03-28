// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC mnemonic backup layer.
//!
//! This module provides a human-backup surface on top of `seed.rs`.
//!
//! Responsibility split:
//! - `seed.rs`      => root secret generation / restore / custody handoff,
//! - `mnemonic.rs`  => human-readable 24-word backup and restore,
//! - `key_engine.rs` => deterministic derivation from the internal master seed.
//!
//! Security posture:
//! - mnemonic handling is intentionally separate from seed generation,
//! - the mnemonic maps to the AOXC recovery seed, not directly to a derived wallet address,
//! - BIP39 English is used instead of a custom word system,
//! - phrase memory is zeroized on drop,
//! - fixed English policy avoids cross-language ambiguity.
//!
//! Important note:
//! This module does not make the system "post-quantum" by itself.
//! It provides a safe human backup format for the recovery seed.

use bip39::{Language, Mnemonic};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroize;

use super::seed::{
    GeneratedSeed, RECOVERY_SEED_LEN, SEED_FINGERPRINT_LEN, SeedError, SeedKind,
};

/// Current AOXC mnemonic schema version.
pub const AOXC_MNEMONIC_VERSION: u8 = 1;

/// Canonical mnemonic language used by AOXC.
///
/// Fixed language policy reduces ambiguity and operational mistakes.
pub const AOXC_MNEMONIC_LANGUAGE: &str = "english";

/// Canonical AOXC mnemonic word count.
///
/// A 32-byte recovery seed maps to a 24-word BIP39 mnemonic.
pub const AOXC_MNEMONIC_WORD_COUNT: usize = 24;

/// Public mnemonic metadata safe to log or serialize.
///
/// This structure intentionally excludes the mnemonic phrase itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MnemonicMetadata {
    pub version: u8,
    pub kind: SeedKind,
    pub language: String,
    pub word_count: u8,
    pub fingerprint: String,
}

/// Secret mnemonic backup object.
///
/// Security note:
/// the contained phrase is highly sensitive and is zeroized on drop.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct MnemonicBackup {
    #[zeroize(skip)]
    metadata: MnemonicMetadata,
    phrase: String,
}

impl fmt::Debug for MnemonicBackup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MnemonicBackup")
            .field("metadata", &self.metadata)
            .field("phrase", &"<redacted>")
            .finish()
    }
}

/// Canonical mnemonic error surface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MnemonicError {
    EmptyPhrase,
    InvalidPhrase,
    InvalidWordCount,
    InvalidEntropyLength,
    FingerprintMismatch,
    Seed(SeedError),
}

impl MnemonicError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyPhrase => "MNEMONIC_EMPTY_PHRASE",
            Self::InvalidPhrase => "MNEMONIC_INVALID_PHRASE",
            Self::InvalidWordCount => "MNEMONIC_INVALID_WORD_COUNT",
            Self::InvalidEntropyLength => "MNEMONIC_INVALID_ENTROPY_LENGTH",
            Self::FingerprintMismatch => "MNEMONIC_FINGERPRINT_MISMATCH",
            Self::Seed(_) => "MNEMONIC_SEED_ERROR",
        }
    }
}

impl fmt::Display for MnemonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPhrase => write!(f, "mnemonic restore failed: phrase is empty"),
            Self::InvalidPhrase => write!(f, "mnemonic restore failed: phrase is invalid"),
            Self::InvalidWordCount => write!(f, "mnemonic restore failed: word count is invalid"),
            Self::InvalidEntropyLength => {
                write!(f, "mnemonic restore failed: decoded entropy length is invalid")
            }
            Self::FingerprintMismatch => {
                write!(f, "mnemonic validation failed: fingerprint mismatch")
            }
            Self::Seed(error) => write!(f, "mnemonic restore failed: {}", error),
        }
    }
}

impl std::error::Error for MnemonicError {}

impl From<SeedError> for MnemonicError {
    fn from(value: SeedError) -> Self {
        Self::Seed(value)
    }
}

impl MnemonicBackup {
    /// Builds a mnemonic backup from an already-generated seed.
    ///
    /// This is the preferred path for first-wallet or first-node bootstrap.
    pub fn from_generated_seed(seed: &GeneratedSeed) -> Result<Self, MnemonicError> {
        let phrase = encode_recovery_seed_as_phrase(seed.recovery_seed())?;

        Ok(Self {
            metadata: MnemonicMetadata {
                version: AOXC_MNEMONIC_VERSION,
                kind: seed.metadata().kind,
                language: AOXC_MNEMONIC_LANGUAGE.to_string(),
                word_count: AOXC_MNEMONIC_WORD_COUNT as u8,
                fingerprint: seed.metadata().fingerprint.clone(),
            },
            phrase,
        })
    }

    /// Builds a mnemonic backup directly from a recovery seed.
    pub fn from_recovery_seed(
        kind: SeedKind,
        recovery_seed: &[u8; RECOVERY_SEED_LEN],
    ) -> Result<Self, MnemonicError> {
        let seed = GeneratedSeed::from_recovery_seed(kind, *recovery_seed)?;
        Self::from_generated_seed(&seed)
    }

    /// Builds a mnemonic backup from an existing phrase.
    ///
    /// The phrase is parsed, canonicalized, restored into a seed, and then
    /// re-emitted in canonical AOXC form.
    pub fn from_phrase(kind: SeedKind, phrase: &str) -> Result<Self, MnemonicError> {
        let seed = restore_seed_from_phrase(kind, phrase)?;
        Self::from_generated_seed(&seed)
    }

    /// Returns public mnemonic metadata safe to log or serialize.
    #[must_use]
    pub fn metadata(&self) -> &MnemonicMetadata {
        &self.metadata
    }

    /// Returns the sensitive mnemonic phrase.
    ///
    /// Operational note:
    /// callers should display this once, avoid logging it, and avoid persisting
    /// it unless the user explicitly requests an offline backup artifact.
    #[must_use]
    pub fn phrase(&self) -> &str {
        &self.phrase
    }

    /// Validates the mnemonic backup object.
    ///
    /// Validation policy:
    /// - phrase must be canonical English BIP39,
    /// - word count must be 24,
    /// - restored seed fingerprint must match metadata.
    pub fn validate(&self) -> Result<(), MnemonicError> {
        if self.metadata.version != AOXC_MNEMONIC_VERSION {
            return Err(MnemonicError::InvalidPhrase);
        }

        if self.metadata.language != AOXC_MNEMONIC_LANGUAGE {
            return Err(MnemonicError::InvalidPhrase);
        }

        if self.metadata.word_count as usize != AOXC_MNEMONIC_WORD_COUNT {
            return Err(MnemonicError::InvalidWordCount);
        }

        let restored = restore_seed_from_phrase(self.metadata.kind, &self.phrase)?;

        if restored.metadata().fingerprint != self.metadata.fingerprint {
            return Err(MnemonicError::FingerprintMismatch);
        }

        Ok(())
    }

    /// Restores the AOXC seed from this mnemonic backup.
    pub fn restore_seed(&self) -> Result<GeneratedSeed, MnemonicError> {
        restore_seed_from_phrase(self.metadata.kind, &self.phrase)
    }
}

/// Generates a new AOXC seed and immediately returns its mnemonic backup.
///
/// This is the preferred first-wallet bootstrap path.
pub fn generate_seed_and_mnemonic(
    kind: SeedKind,
) -> Result<(GeneratedSeed, MnemonicBackup), MnemonicError> {
    let seed = GeneratedSeed::generate(kind)?;
    let mnemonic = MnemonicBackup::from_generated_seed(&seed)?;
    Ok((seed, mnemonic))
}

/// Generates a new AOXC seed, mixes additional entropy, and returns its mnemonic backup.
pub fn generate_seed_and_mnemonic_with_additional_entropy(
    kind: SeedKind,
    additional_entropy: &[u8],
) -> Result<(GeneratedSeed, MnemonicBackup), MnemonicError> {
    let seed = GeneratedSeed::generate_with_additional_entropy(kind, additional_entropy)?;
    let mnemonic = MnemonicBackup::from_generated_seed(&seed)?;
    Ok((seed, mnemonic))
}

/// Restores an AOXC seed from a mnemonic phrase.
pub fn restore_seed_from_phrase(
    kind: SeedKind,
    phrase: &str,
) -> Result<GeneratedSeed, MnemonicError> {
    validate_phrase_surface(phrase)?;

    let mnemonic = Mnemonic::parse_in(Language::English, phrase)
        .map_err(|_| MnemonicError::InvalidPhrase)?;

    if mnemonic.word_count() != AOXC_MNEMONIC_WORD_COUNT {
        return Err(MnemonicError::InvalidWordCount);
    }

    let mut entropy = mnemonic.to_entropy();
    if entropy.len() != RECOVERY_SEED_LEN {
        entropy.zeroize();
        return Err(MnemonicError::InvalidEntropyLength);
    }

    let mut recovery_seed = [0u8; RECOVERY_SEED_LEN];
    recovery_seed.copy_from_slice(&entropy);
    entropy.zeroize();

    GeneratedSeed::from_recovery_seed(kind, recovery_seed).map_err(MnemonicError::from)
}

/// Encodes a recovery seed into a canonical 24-word English BIP39 mnemonic.
pub fn encode_recovery_seed_as_phrase(
    recovery_seed: &[u8; RECOVERY_SEED_LEN],
) -> Result<String, MnemonicError> {
    let mnemonic = Mnemonic::from_entropy_in(Language::English, recovery_seed)
        .map_err(|_| MnemonicError::InvalidEntropyLength)?;

    if mnemonic.word_count() != AOXC_MNEMONIC_WORD_COUNT {
        return Err(MnemonicError::InvalidWordCount);
    }

    Ok(mnemonic.to_string())
}

/// Validates the outer phrase surface before invoking BIP39 parsing.
///
/// Policy:
/// - phrase must not be blank,
/// - surrounding whitespace is rejected,
/// - canonical separator is a single ASCII space,
/// - word count must be exactly 24.
fn validate_phrase_surface(phrase: &str) -> Result<(), MnemonicError> {
    if phrase.is_empty() || phrase.trim().is_empty() {
        return Err(MnemonicError::EmptyPhrase);
    }

    if phrase != phrase.trim() {
        return Err(MnemonicError::InvalidPhrase);
    }

    let words: Vec<&str> = phrase.split(' ').collect();

    if words.len() != AOXC_MNEMONIC_WORD_COUNT {
        return Err(MnemonicError::InvalidWordCount);
    }

    if words.iter().any(|word| word.is_empty()) {
        return Err(MnemonicError::InvalidPhrase);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{generate_seed_and_mnemonic, restore_seed_from_phrase};

    #[test]
    fn generated_mnemonic_is_24_words() {
        let (_seed, mnemonic) =
            generate_seed_and_mnemonic(SeedKind::WalletRoot).expect("generation must succeed");

        assert_eq!(
            mnemonic.phrase().split(' ').count(),
            AOXC_MNEMONIC_WORD_COUNT
        );
        assert_eq!(mnemonic.metadata().word_count as usize, AOXC_MNEMONIC_WORD_COUNT);
    }

    #[test]
    fn mnemonic_restore_roundtrip_is_deterministic() {
        let (seed, mnemonic) =
            generate_seed_and_mnemonic(SeedKind::WalletRoot).expect("generation must succeed");

        let restored = mnemonic.restore_seed().expect("restore must succeed");

        assert_eq!(seed.recovery_seed(), restored.recovery_seed());
        assert_eq!(seed.master_seed(), restored.master_seed());
        assert_eq!(seed.metadata().fingerprint, restored.metadata().fingerprint);
    }

    #[test]
    fn from_phrase_rebuilds_canonical_backup() {
        let (_seed, mnemonic) =
            generate_seed_and_mnemonic(SeedKind::NodeRoot).expect("generation must succeed");

        let rebuilt = MnemonicBackup::from_phrase(SeedKind::NodeRoot, mnemonic.phrase())
            .expect("canonical rebuild must succeed");

        assert_eq!(mnemonic.phrase(), rebuilt.phrase());
        assert_eq!(mnemonic.metadata(), rebuilt.metadata());
    }

    #[test]
    fn invalid_phrase_word_count_is_rejected() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        let error = restore_seed_from_phrase(SeedKind::WalletRoot, phrase)
            .expect_err("restore must fail");

        assert_eq!(error, MnemonicError::InvalidWordCount);
    }

    #[test]
    fn invalid_phrase_surface_is_rejected() {
        let phrase = " abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ";
        let error = restore_seed_from_phrase(SeedKind::WalletRoot, phrase)
            .expect_err("restore must fail");

        assert_eq!(error, MnemonicError::InvalidPhrase);
    }

    #[test]
    fn mnemonic_validation_detects_tampering() {
        let (_seed, mnemonic) =
            generate_seed_and_mnemonic(SeedKind::WalletRoot).expect("generation must succeed");

        let mut tampered = mnemonic.clone();
        tampered.phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art".to_string();

        let error = tampered.validate().expect_err("validation must fail");
        assert!(
            matches!(
                error,
                MnemonicError::InvalidPhrase
                    | MnemonicError::FingerprintMismatch
                    | MnemonicError::InvalidEntropyLength
            ),
            "unexpected mnemonic validation error: {error}"
        );
    }

    #[test]
    fn different_seed_kinds_restore_to_different_master_seeds() {
        let (wallet_seed, mnemonic) =
            generate_seed_and_mnemonic(SeedKind::WalletRoot).expect("generation must succeed");

        let node_seed = restore_seed_from_phrase(SeedKind::NodeRoot, mnemonic.phrase())
            .expect("restore must succeed");

        assert_eq!(wallet_seed.recovery_seed(), node_seed.recovery_seed());
        assert_ne!(wallet_seed.master_seed(), node_seed.master_seed());
        assert_ne!(wallet_seed.metadata().fingerprint, node_seed.metadata().fingerprint);
    }

    #[test]
    fn fingerprint_length_matches_seed_policy() {
        let (_seed, mnemonic) =
            generate_seed_and_mnemonic(SeedKind::WalletRoot).expect("generation must succeed");

        assert_eq!(mnemonic.metadata().fingerprint.len(), SEED_FINGERPRINT_LEN * 2);
    }
}
