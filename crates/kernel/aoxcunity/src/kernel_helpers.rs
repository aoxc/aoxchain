fn map_consensus_error(error: &ConsensusError) -> KernelRejection {
    match error {
        ConsensusError::UnknownParent => KernelRejection::UnknownParent,
        ConsensusError::DuplicateBlock | ConsensusError::DuplicateVote => {
            KernelRejection::DuplicateArtifact
        }
        ConsensusError::EquivocatingVote => KernelRejection::InvariantViolation,
        ConsensusError::VoteForUnknownBlock
        | ConsensusError::StaleVote
        | ConsensusError::InvalidAuthenticatedContext
        | ConsensusError::HeightRegression
        | ConsensusError::InvalidParentHeight
        | ConsensusError::InvalidGenesisParent => KernelRejection::StaleArtifact,
        ConsensusError::InvalidBlockHash
        | ConsensusError::InvalidBlockBodyCommitments
        | ConsensusError::InvalidBlockSemantics => KernelRejection::InvalidSignature,
        ConsensusError::ValidatorNotFound
        | ConsensusError::InactiveValidator
        | ConsensusError::NonVotingValidator
        | ConsensusError::InvalidQuorumThreshold
        | ConsensusError::EmptyValidatorSet
        | ConsensusError::DuplicateValidator
        | ConsensusError::BlockBuild(_) => KernelRejection::InvalidSignature,
    }
}

fn map_safety_violation(violation: SafetyViolation) -> KernelRejection {
    match violation {
        SafetyViolation::LockRegression
        | SafetyViolation::EpochRegression
        | SafetyViolation::RoundRegression => KernelRejection::InvariantViolation,
    }
}

fn constitutional_error_reason(error: ConstitutionalValidationError) -> String {
    match error {
        ConstitutionalValidationError::EmptySignerSet => {
            "constitutional_empty_signer_set".to_string()
        }
        ConstitutionalValidationError::ZeroObservedPower => {
            "constitutional_zero_observed_power".to_string()
        }
        ConstitutionalValidationError::InvalidTimeoutRound => {
            "constitutional_invalid_timeout_round".to_string()
        }
        ConstitutionalValidationError::ExecutionLegitimacyBlockMismatch => {
            "constitutional_execution_legitimacy_block_mismatch".to_string()
        }
        ConstitutionalValidationError::ExecutionContinuityBlockMismatch => {
            "constitutional_execution_continuity_block_mismatch".to_string()
        }
        ConstitutionalValidationError::ExecutionContinuityHeightMismatch => {
            "constitutional_execution_continuity_height_mismatch".to_string()
        }
        ConstitutionalValidationError::ExecutionContinuityRoundMismatch => {
            "constitutional_execution_continuity_round_mismatch".to_string()
        }
        ConstitutionalValidationError::ExecutionLegitimacyEpochMismatch => {
            "constitutional_execution_legitimacy_epoch_mismatch".to_string()
        }
        ConstitutionalValidationError::ExecutionContinuityEpochMismatch => {
            "constitutional_execution_continuity_epoch_mismatch".to_string()
        }
    }
}

fn justification_from_vote(vote: &Vote, epoch: u64) -> JustificationRef {
    JustificationRef {
        block_hash: vote.block_hash,
        height: vote.height,
        round: vote.round,
        epoch,
        certificate_hash: vote.block_hash,
    }
}

fn equivocation_evidence(block_hash: [u8; 32], reason: &str) -> ConsensusEvidence {
    ConsensusEvidence {
        evidence_hash: block_hash,
        related_block_hash: block_hash,
        reason: format!("{reason}_equivocation"),
    }
}

