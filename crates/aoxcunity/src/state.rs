// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::{BTreeSet, HashMap};

use crate::block::hash::{compute_block_hash, compute_body_roots};
use crate::block::semantic::{
    validate_block_semantics, validate_capability_section_alignment,
    validate_root_semantic_bindings,
};
use crate::block::{Block, BlockBody};
use crate::error::ConsensusError;
use crate::fork_choice::{BlockMeta, ForkChoice};
use crate::quorum::QuorumThreshold;
use crate::rotation::ValidatorRotation;
use crate::round::RoundState;
use crate::seal::{AuthenticatedQuorumCertificate, BlockSeal, QuorumCertificate};
use crate::validator::{SlashFault, ValidatorId};
use crate::vote::{
    SignedVote, VerifiedAuthenticatedVote, VerifiedVote, Vote, VoteAuthenticationContext,
    VoteAuthenticationError, VoteKind,
};
use crate::vote_pool::VotePool;

include!("state_core.rs");
include!("state_voting.rs");
include!("state_finalization.rs");
include!("state_tests/tests.rs");
