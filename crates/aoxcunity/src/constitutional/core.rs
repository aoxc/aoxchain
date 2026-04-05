// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::seal::QuorumCertificate;
use crate::validator::ValidatorId;

const EXECUTION_CERTIFICATE_DOMAIN_V1: &[u8] = b"AOXC_EXECUTION_CERTIFICATE_V1";
const LEGITIMACY_CERTIFICATE_DOMAIN_V1: &[u8] = b"AOXC_LEGITIMACY_CERTIFICATE_V1";
const CONTINUITY_CERTIFICATE_DOMAIN_V1: &[u8] = b"AOXC_CONTINUITY_CERTIFICATE_V1";
const CONSTITUTIONAL_SEAL_DOMAIN_V1: &[u8] = b"AOXC_CONSTITUTIONAL_SEAL_V1";

/// Canonical classification of finality maturity for a single block context.
///
/// This enum intentionally separates mechanical finality from legitimacy-aware
/// and constitution-aware finality. The objective is to allow higher layers to
/// express policy decisions without weakening the deterministic core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstitutionalFinalityStage {
    /// The block does not yet satisfy any finality predicate recognized by this module.
    None,
    /// The block has an execution certificate derived from quorum-finalized commit evidence.
    ExecutionFinal,
    /// The block has execution finality plus an authority-backed legitimacy certificate.
    LegitimatelyFinal,
    /// The block has execution finality plus continuity evidence.
    ContinuousFinal,
    /// The block satisfies execution, legitimacy, and continuity requirements and can
    /// therefore be represented by a constitutional seal.
    ConstitutionallyFinal,
}

/// Explicit validation error model for constitutional artifacts.
///
/// The intent is to preserve auditability and to avoid silent rejection paths
/// when higher-level orchestration wants to reason about why seal composition failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstitutionalValidationError {
    EmptySignerSet,
    ZeroObservedPower,
    InvalidTimeoutRound,
    ExecutionLegitimacyBlockMismatch,
    ExecutionContinuityBlockMismatch,
    ExecutionContinuityHeightMismatch,
    ExecutionContinuityRoundMismatch,
    ExecutionLegitimacyEpochMismatch,
    ExecutionContinuityEpochMismatch,
}

impl core::fmt::Display for ConstitutionalValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::EmptySignerSet => {
                "constitutional validation failed: signer set must not be empty"
            }
            Self::ZeroObservedPower => {
                "constitutional validation failed: observed power must be non-zero"
            }
            Self::InvalidTimeoutRound => {
                "constitutional validation failed: timeout round must be greater than the observed round"
            }
            Self::ExecutionLegitimacyBlockMismatch => {
                "constitutional validation failed: execution and legitimacy block hash mismatch"
            }
            Self::ExecutionContinuityBlockMismatch => {
                "constitutional validation failed: execution and continuity block hash mismatch"
            }
            Self::ExecutionContinuityHeightMismatch => {
                "constitutional validation failed: execution and continuity height mismatch"
            }
            Self::ExecutionContinuityRoundMismatch => {
                "constitutional validation failed: execution and continuity round mismatch"
            }
            Self::ExecutionLegitimacyEpochMismatch => {
                "constitutional validation failed: execution epoch does not match legitimacy authority epoch"
            }
            Self::ExecutionContinuityEpochMismatch => {
                "constitutional validation failed: execution epoch does not match continuity epoch"
            }
        };

        f.write_str(message)
    }
}

impl std::error::Error for ConstitutionalValidationError {}

/// Minimal deterministic evaluation report describing which constitutional
/// ingredients are present for a given block context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalEligibilityReport {
    pub has_execution: bool,
    pub has_legitimacy: bool,
    pub has_continuity: bool,
    pub stage: ConstitutionalFinalityStage,
}

impl ConstitutionalEligibilityReport {
    #[must_use]
    pub fn from_inputs(execution: bool, legitimacy: bool, continuity: bool) -> Self {
        let stage = match (execution, legitimacy, continuity) {
            (false, _, _) => ConstitutionalFinalityStage::None,
            (true, false, false) => ConstitutionalFinalityStage::ExecutionFinal,
            (true, true, false) => ConstitutionalFinalityStage::LegitimatelyFinal,
            (true, false, true) => ConstitutionalFinalityStage::ContinuousFinal,
            (true, true, true) => ConstitutionalFinalityStage::ConstitutionallyFinal,
        };

        Self {
            has_execution: execution,
            has_legitimacy: legitimacy,
            has_continuity: continuity,
            stage,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionCertificate {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub validator_set_hash: [u8; 32],
    pub quorum_certificate: QuorumCertificate,
    pub certificate_hash: [u8; 32],
}

impl ExecutionCertificate {
    /// Constructs a deterministic execution certificate from a finalized quorum certificate.
    ///
    /// Security posture:
    /// - block, height, and round are inherited directly from the quorum certificate,
    /// - epoch and validator-set identity are bound into the derived certificate hash,
    /// - certificate formation is side-effect free and fully deterministic.
    #[must_use]
    pub fn new(
        epoch: u64,
        validator_set_hash: [u8; 32],
        quorum_certificate: QuorumCertificate,
    ) -> Self {
        let certificate_hash = compute_execution_certificate_hash(
            quorum_certificate.block_hash,
            quorum_certificate.height,
            quorum_certificate.round,
            epoch,
            validator_set_hash,
            quorum_certificate.certificate_hash,
        );

        Self {
            block_hash: quorum_certificate.block_hash,
            height: quorum_certificate.height,
            round: quorum_certificate.round,
            epoch,
            validator_set_hash,
            quorum_certificate,
            certificate_hash,
        }
    }

    /// Returns the finality stage represented by this artifact in isolation.
    #[must_use]
    pub const fn finality_stage(&self) -> ConstitutionalFinalityStage {
        ConstitutionalFinalityStage::ExecutionFinal
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegitimacyCertificate {
    pub block_hash: [u8; 32],
    pub authority_epoch: u64,
    pub constitution_root: [u8; 32],
    pub validator_authority_commitment: [u8; 32],
    pub transition_proof_root: [u8; 32],
    pub signers: Vec<ValidatorId>,
    pub certificate_hash: [u8; 32],
}

impl LegitimacyCertificate {
    /// Constructs a legitimacy certificate with deterministic signer ordering.
    ///
    /// This artifact is intended to bind a block to its authority and transition context.
    /// The signer set is normalized before hashing in order to keep replay and
    /// cross-node verification stable.
    #[must_use]
    pub fn new(
        block_hash: [u8; 32],
        authority_epoch: u64,
        constitution_root: [u8; 32],
        validator_authority_commitment: [u8; 32],
        transition_proof_root: [u8; 32],
        mut signers: Vec<ValidatorId>,
    ) -> Self {
        signers.sort();
        signers.dedup();
        let certificate_hash = compute_legitimacy_certificate_hash(
            block_hash,
            authority_epoch,
            constitution_root,
            validator_authority_commitment,
            transition_proof_root,
            &signers,
        );

        Self {
            block_hash,
            authority_epoch,
            constitution_root,
            validator_authority_commitment,
            transition_proof_root,
            signers,
            certificate_hash,
        }
    }

    /// Validates local structural assumptions for legitimacy evidence.
    pub fn validate(&self) -> Result<(), ConstitutionalValidationError> {
        if self.signers.is_empty() {
            return Err(ConstitutionalValidationError::EmptySignerSet);
        }

        Ok(())
    }

    /// Returns `true` when this legitimacy certificate is structurally and contextually
    /// compatible with the supplied execution certificate.
    #[must_use]
    pub fn is_compatible_with_execution(&self, execution: &ExecutionCertificate) -> bool {
        self.block_hash == execution.block_hash && self.authority_epoch == execution.epoch
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuityCertificate {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub timeout_round: u64,
    pub signers: Vec<ValidatorId>,
    pub observed_power: u64,
    pub certificate_hash: [u8; 32],
}

impl ContinuityCertificate {
    /// Constructs a continuity certificate with deterministic signer ordering.
    ///
    /// Continuity evidence is intended to prove that the network observed enough
    /// timeout support to justify controlled forward motion without violating
    /// deterministic replay semantics.
    #[must_use]
    pub fn new(
        block_hash: [u8; 32],
        height: u64,
        round: u64,
        epoch: u64,
        timeout_round: u64,
        observed_power: u64,
        mut signers: Vec<ValidatorId>,
    ) -> Self {
        signers.sort();
        signers.dedup();
        let certificate_hash = compute_continuity_certificate_hash(
            block_hash,
            height,
            round,
            epoch,
            timeout_round,
            observed_power,
            &signers,
        );

        Self {
            block_hash,
            height,
            round,
            epoch,
            timeout_round,
            signers,
            observed_power,
            certificate_hash,
        }
    }

    /// Validates local structural assumptions for continuity evidence.
    pub fn validate(&self) -> Result<(), ConstitutionalValidationError> {
        if self.signers.is_empty() {
            return Err(ConstitutionalValidationError::EmptySignerSet);
        }
        if self.observed_power == 0 {
            return Err(ConstitutionalValidationError::ZeroObservedPower);
        }
        if self.timeout_round <= self.round {
            return Err(ConstitutionalValidationError::InvalidTimeoutRound);
        }

        Ok(())
    }

    /// Returns `true` when this continuity certificate is structurally and contextually
    /// compatible with the supplied execution certificate.
    #[must_use]
    pub fn is_compatible_with_execution(&self, execution: &ExecutionCertificate) -> bool {
        self.block_hash == execution.block_hash
            && self.height == execution.height
            && self.round == execution.round
            && self.epoch == execution.epoch
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalSeal {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub execution_certificate_hash: [u8; 32],
    pub legitimacy_certificate_hash: [u8; 32],
    pub continuity_certificate_hash: [u8; 32],
    pub seal_hash: [u8; 32],
}

impl ConstitutionalSeal {
    /// Backward-compatible composition helper.
    ///
    /// This method intentionally preserves the existing `Option`-based API so
    /// existing callers do not need to change immediately. Newer code paths
    /// should prefer `compose_strict` when they need explicit rejection reasons.
    #[must_use]
    pub fn compose(
        execution: &ExecutionCertificate,
        legitimacy: &LegitimacyCertificate,
        continuity: &ContinuityCertificate,
    ) -> Option<Self> {
        Self::compose_strict(execution, legitimacy, continuity).ok()
    }

    /// Strict constitutional seal composition with explicit validation errors.
    ///
    /// Composition succeeds only if:
    /// - legitimacy evidence is structurally valid,
    /// - continuity evidence is structurally valid,
    /// - block identity matches across all three inputs,
    /// - height and round match between execution and continuity,
    /// - epoch / authority epoch alignment is exact.
    pub fn compose_strict(
        execution: &ExecutionCertificate,
        legitimacy: &LegitimacyCertificate,
        continuity: &ContinuityCertificate,
    ) -> Result<Self, ConstitutionalValidationError> {
        legitimacy.validate()?;
        continuity.validate()?;

        if execution.block_hash != legitimacy.block_hash {
            return Err(ConstitutionalValidationError::ExecutionLegitimacyBlockMismatch);
        }
        if execution.block_hash != continuity.block_hash {
            return Err(ConstitutionalValidationError::ExecutionContinuityBlockMismatch);
        }
        if execution.height != continuity.height {
            return Err(ConstitutionalValidationError::ExecutionContinuityHeightMismatch);
        }
        if execution.round != continuity.round {
            return Err(ConstitutionalValidationError::ExecutionContinuityRoundMismatch);
        }
        if execution.epoch != legitimacy.authority_epoch {
            return Err(ConstitutionalValidationError::ExecutionLegitimacyEpochMismatch);
        }
        if execution.epoch != continuity.epoch {
            return Err(ConstitutionalValidationError::ExecutionContinuityEpochMismatch);
        }

        let seal_hash = compute_constitutional_seal_hash(
            execution.block_hash,
            execution.height,
            execution.round,
            execution.epoch,
            execution.certificate_hash,
            legitimacy.certificate_hash,
            continuity.certificate_hash,
        );

        Ok(Self {
            block_hash: execution.block_hash,
            height: execution.height,
            round: execution.round,
            epoch: execution.epoch,
            execution_certificate_hash: execution.certificate_hash,
            legitimacy_certificate_hash: legitimacy.certificate_hash,
            continuity_certificate_hash: continuity.certificate_hash,
            seal_hash,
        })
    }

    /// Evaluates what finality stage is currently justified by the supplied
    /// constitutional ingredients.
    #[must_use]
    pub fn evaluate_stage(
        execution: Option<&ExecutionCertificate>,
        legitimacy: Option<&LegitimacyCertificate>,
        continuity: Option<&ContinuityCertificate>,
    ) -> ConstitutionalEligibilityReport {
        ConstitutionalEligibilityReport::from_inputs(
            execution.is_some(),
            legitimacy.is_some(),
            continuity.is_some(),
        )
    }
}

fn compute_execution_certificate_hash(
    block_hash: [u8; 32],
    height: u64,
    round: u64,
    epoch: u64,
    validator_set_hash: [u8; 32],
    quorum_certificate_hash: [u8; 32],
) -> [u8; 32] {
    compute_hash(
        EXECUTION_CERTIFICATE_DOMAIN_V1,
        &[
            &block_hash,
            &height.to_le_bytes(),
            &round.to_le_bytes(),
            &epoch.to_le_bytes(),
            &validator_set_hash,
            &quorum_certificate_hash,
        ],
    )
}

fn compute_legitimacy_certificate_hash(
    block_hash: [u8; 32],
    authority_epoch: u64,
    constitution_root: [u8; 32],
    validator_authority_commitment: [u8; 32],
    transition_proof_root: [u8; 32],
    signers: &[ValidatorId],
) -> [u8; 32] {
    let authority_epoch_bytes = authority_epoch.to_le_bytes();
    let signer_len = (signers.len() as u64).to_le_bytes();

    let mut parts: Vec<&[u8]> = vec![
        &block_hash,
        &authority_epoch_bytes,
        &constitution_root,
        &validator_authority_commitment,
        &transition_proof_root,
        &signer_len,
    ];

    for signer in signers {
        parts.push(signer);
    }

    compute_hash(LEGITIMACY_CERTIFICATE_DOMAIN_V1, &parts)
}

fn compute_continuity_certificate_hash(
    block_hash: [u8; 32],
    height: u64,
    round: u64,
    epoch: u64,
    timeout_round: u64,
    observed_power: u64,
    signers: &[ValidatorId],
) -> [u8; 32] {
    let height_bytes = height.to_le_bytes();
    let round_bytes = round.to_le_bytes();
    let epoch_bytes = epoch.to_le_bytes();
    let timeout_round_bytes = timeout_round.to_le_bytes();
    let observed_power_bytes = observed_power.to_le_bytes();
    let signer_len = (signers.len() as u64).to_le_bytes();

    let mut parts: Vec<&[u8]> = vec![
        &block_hash,
        &height_bytes,
        &round_bytes,
        &epoch_bytes,
        &timeout_round_bytes,
        &observed_power_bytes,
        &signer_len,
    ];

    for signer in signers {
        parts.push(signer);
    }

    compute_hash(CONTINUITY_CERTIFICATE_DOMAIN_V1, &parts)
}

fn compute_constitutional_seal_hash(
    block_hash: [u8; 32],
    height: u64,
    round: u64,
    epoch: u64,
    execution_certificate_hash: [u8; 32],
    legitimacy_certificate_hash: [u8; 32],
    continuity_certificate_hash: [u8; 32],
) -> [u8; 32] {
    compute_hash(
        CONSTITUTIONAL_SEAL_DOMAIN_V1,
        &[
            &block_hash,
            &height.to_le_bytes(),
            &round.to_le_bytes(),
            &epoch.to_le_bytes(),
            &execution_certificate_hash,
            &legitimacy_certificate_hash,
            &continuity_certificate_hash,
        ],
    )
}

fn compute_hash(domain: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(domain);

    for part in parts {
        hasher.update(part);
    }

    hasher.finalize().into()
}
