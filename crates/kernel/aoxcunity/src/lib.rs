// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

pub mod block;
pub mod constitutional;
pub mod error;
pub mod fork_choice;
pub mod kernel;
pub mod messages;
pub mod proposer;
pub mod quorum;
pub mod rotation;
pub mod round;
pub mod safety;
pub mod seal;
pub mod state;
pub mod store;
pub mod validator;
pub mod version;
pub mod vote;
pub mod vote_pool;

pub use block::{
    AiSection, Block, BlockBody, BlockBuildError, BlockBuilder, BlockHeader, BlockSection,
    ConstitutionalSection, ExecutionLaneRecord, ExecutionSection, ExternalNetwork,
    ExternalProofRecord, ExternalProofSection, ExternalProofType, ExternalSettlementRecord,
    ExternalSettlementSection, IdentitySection, LaneCommitment, LaneCommitmentSection, LaneType,
    PQ_MANDATORY_START_EPOCH, PostQuantumSection, SignaturePolicy, TimeSealSection,
    enforce_signature_policy_migration, resolve_signature_policy,
};
pub use constitutional::{
    ConstitutionalSeal, ContinuityCertificate, ExecutionCertificate, LegitimacyCertificate,
};
pub use error::ConsensusError;
pub use fork_choice::{BlockMeta, ForkChoice};
pub use kernel::{
    ConsensusEngine, ConsensusEvent, InvariantStatus, KernelCertificate, KernelEffect,
    KernelOperationalSnapshot, KernelRejection, PruningAction, TimeoutVote, TransitionResult,
    VerifiedTimeoutVote, VerifiedVote,
};
pub use messages::ConsensusMessage;
pub use proposer::Proposer;
pub use quorum::QuorumThreshold;
pub use rotation::ValidatorRotation;
pub use round::{PacemakerStep, RoundChangeReason, RoundState};
pub use safety::{JustificationRef, LockState, SafeToVote, SafetyViolation, evaluate_safe_to_vote};
pub use seal::AuthenticatedQuorumCertificate;
pub use seal::{BlockSeal, QuorumCertificate};
pub use state::ConsensusState;
pub use store::{
    ConsensusEvidence, ConsensusJournal, FileConsensusStore, FinalityStore, KernelSnapshot,
    PersistedConsensusEvent, RecoveryState, SnapshotStore, hash_consensus_event, recover_state,
};
pub use validator::{SlashFault, Validator, ValidatorId, ValidatorLifecycle, ValidatorRole};
pub use version::{
    AOXC_CERTIFICATE_FORMAT_LINE, AOXC_COVENANT_KERNEL_LINE, AOXC_COVENANT_KERNEL_NAME,
    AOXC_VOTE_FORMAT_LINE, KernelIdentity, kernel_identity,
};

pub use vote::{AuthenticatedVote, VerifiedAuthenticatedVote, VoteAuthenticationContext};
pub use vote::{SignedVote, Vote, VoteAuthenticationError, VoteKind};
pub use vote_pool::VotePool;
