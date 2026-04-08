// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::constitutional::{
    ConstitutionalSeal, ConstitutionalValidationError, ContinuityCertificate, ExecutionCertificate,
    LegitimacyCertificate,
};
use crate::error::ConsensusError;
use crate::safety::{
    JustificationRef, LockState, SafeToVote, SafetyViolation, evaluate_safe_to_vote,
};
use crate::seal::AuthenticatedQuorumCertificate;
use crate::state::ConsensusState;
use crate::store::{
    ConsensusEvidence, ConsensusJournal, EvidenceStore, FinalityStore, KernelSnapshot,
    PersistedConsensusEvent, RecoveryState, SnapshotStore, hash_consensus_event,
};
use crate::validator::{SlashFault, ValidatorId};
use crate::vote::{VerifiedAuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

include!("kernel_types.rs");
include!("kernel_engine.rs");
include!("kernel_helpers.rs");
include!("kernel_tests.rs");
