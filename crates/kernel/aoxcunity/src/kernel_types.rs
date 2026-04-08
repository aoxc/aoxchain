#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedVote {
    pub authenticated_vote: VerifiedAuthenticatedVote,
    pub verification_tag: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutVote {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub timeout_round: u64,
    pub voter: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedTimeoutVote {
    pub timeout_vote: TimeoutVote,
    pub verification_tag: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum ConsensusEvent {
    AdmitBlock(Block),
    AdmitVerifiedVote(VerifiedVote),
    AdmitTimeoutVote(VerifiedTimeoutVote),
    ObserveLegitimacy(LegitimacyCertificate),
    ReportLeaderFailure {
        height: u64,
        round: u64,
        leader: ValidatorId,
    },
    AdvanceRound {
        height: u64,
        round: u64,
    },
    EvaluateFinality {
        block_hash: [u8; 32],
    },
    PruneFinalizedState {
        finalized_height: u64,
    },
    RecoverPersistedEvent {
        event_hash: [u8; 32],
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelCertificate {
    Execution(AuthenticatedQuorumCertificate),
    Legitimacy(LegitimacyCertificate),
    Continuity(ContinuityCertificate),
    Constitutional(ConstitutionalSeal),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelEffect {
    BlockAccepted([u8; 32]),
    VoteAccepted([u8; 32]),
    TimeoutAccepted([u8; 32]),
    BlockFinalized([u8; 32]),
    RoundAdvanced { height: u64, round: u64 },
    StateRecovered([u8; 32]),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelRejection {
    UnknownParent,
    DuplicateArtifact,
    InvalidSignature,
    StaleArtifact,
    FinalityConflict,
    InvariantViolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruningAction {
    pub pruned_blocks: usize,
    pub pruned_votes: usize,
    pub pruned_timeouts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantStatus {
    pub conflicting_finality_detected: bool,
    pub stale_branch_reactivated: bool,
    pub replay_diverged: bool,
}

impl InvariantStatus {
    #[must_use]
    pub const fn healthy() -> Self {
        Self {
            conflicting_finality_detected: false,
            stale_branch_reactivated: false,
            replay_diverged: false,
        }
    }

    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        !self.conflicting_finality_detected && !self.stale_branch_reactivated && !self.replay_diverged
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelOperationalSnapshot {
    pub current_epoch: u64,
    pub current_height: u64,
    pub current_round: u64,
    pub known_block_count: usize,
    pub vote_record_count: usize,
    pub timeout_vote_bucket_count: usize,
    pub legitimacy_certificate_count: usize,
    pub continuity_certificate_count: usize,
    pub evidence_record_count: usize,
    pub replay_marker_count: usize,
    pub active_validator_count: usize,
    pub total_voting_power: u64,
    pub quorum_numerator: u64,
    pub quorum_denominator: u64,
    pub fork_head: Option<[u8; 32]>,
    pub finalized_head: Option<[u8; 32]>,
    pub invariant_status: InvariantStatus,
}

impl KernelOperationalSnapshot {
    #[must_use]
    pub fn production_ready_hint(&self) -> bool {
        self.invariant_status.is_healthy()
            && self.finalized_head.is_some()
            && self.active_validator_count > 0
            && self.total_voting_power > 0
            && self.known_block_count > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionResult {
    pub accepted_effects: Vec<KernelEffect>,
    pub rejected_reason: Option<KernelRejection>,
    pub emitted_certificates: Vec<KernelCertificate>,
    pub pruning_actions: Vec<PruningAction>,
    pub invariant_status: InvariantStatus,
}

impl TransitionResult {
    #[must_use]
    pub fn accepted(effect: KernelEffect) -> Self {
        Self {
            accepted_effects: vec![effect],
            rejected_reason: None,
            emitted_certificates: Vec::new(),
            pruning_actions: Vec::new(),
            invariant_status: InvariantStatus::healthy(),
        }
    }

    #[must_use]
    pub fn rejected(reason: KernelRejection) -> Self {
        Self {
            accepted_effects: Vec::new(),
            rejected_reason: Some(reason),
            emitted_certificates: Vec::new(),
            pruning_actions: Vec::new(),
            invariant_status: InvariantStatus::healthy(),
        }
    }

    #[must_use]
    fn with_conflicting_finality_detected(mut self) -> Self {
        self.invariant_status.conflicting_finality_detected = true;
        self
    }

    #[must_use]
    fn with_stale_branch_reactivated(mut self) -> Self {
        self.invariant_status.stale_branch_reactivated = true;
        self
    }

    #[must_use]
    fn with_replay_diverged(mut self) -> Self {
        self.invariant_status.replay_diverged = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TimeoutVoteKey {
    block_hash: [u8; 32],
    height: u64,
    round: u64,
    epoch: u64,
    timeout_round: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TimeoutConflictKey {
    voter: ValidatorId,
    height: u64,
    round: u64,
    epoch: u64,
    timeout_round: u64,
}

#[derive(Debug, Clone)]
pub struct ConsensusEngine {
    pub state: ConsensusState,
    pub network_id: u32,
    pub signature_scheme: u16,
    pub lock_state: LockState,
    pub current_epoch: u64,
    pub current_height: u64,
    pub legitimacy_by_block: BTreeMap<[u8; 32], LegitimacyCertificate>,
    pub continuity_by_block: BTreeMap<[u8; 32], ContinuityCertificate>,
    timeout_votes: BTreeMap<TimeoutVoteKey, BTreeMap<ValidatorId, VerifiedTimeoutVote>>,
    timeout_conflicts: BTreeMap<TimeoutConflictKey, [u8; 32]>,
    pub evidence_buffer: Vec<ConsensusEvidence>,
    pub replayed_event_hashes: BTreeSet<[u8; 32]>,
}
