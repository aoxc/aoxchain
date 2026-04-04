/// In-memory consensus state container.
///
/// # Responsibilities
/// - stores blocks admitted into the local consensus view,
/// - maintains fork-choice metadata,
/// - admits and validates votes,
/// - evaluates quorum attainment,
/// - builds deterministic finality artifacts when quorum conditions are satisfied.
///
/// # Security Note
/// This structure is intentionally deterministic and validation-oriented.
/// It does not provide durable persistence in this phase and therefore must not
/// be treated as a crash-recovery authority source.
#[derive(Debug, Clone)]
pub struct ConsensusState {
    pub fork_choice: ForkChoice,
    pub vote_pool: VotePool,
    pub round: RoundState,
    pub rotation: ValidatorRotation,
    pub quorum: QuorumThreshold,
    pub blocks: HashMap<[u8; 32], Block>,
}

impl ConsensusState {
    #[must_use]
    pub fn new(rotation: ValidatorRotation, quorum: QuorumThreshold) -> Self {
        Self {
            fork_choice: ForkChoice::new(),
            vote_pool: VotePool::new(),
            round: RoundState::new(),
            rotation,
            quorum,
            blocks: HashMap::new(),
        }
    }

    /// Admits a block into local consensus state.
    ///
    /// # Genesis Policy
    /// Only height `0` or `1` may use the zero parent hash under the explicit
    /// genesis rule. Every non-genesis block must reference an existing parent
    /// and must advance height by exactly one.
    ///
    /// # Security Semantics
    /// This method rejects:
    /// - duplicate block hashes,
    /// - invalid zero-parent usage,
    /// - unknown parents,
    /// - parent/child height discontinuity,
    /// - local head regressions that violate current fork-choice expectations.
    pub fn admit_block(&mut self, block: Block) -> Result<(), ConsensusError> {
        self.validate_block_integrity(&block)?;

        if self.fork_choice.contains(block.hash) || self.blocks.contains_key(&block.hash) {
            return Err(ConsensusError::DuplicateBlock);
        }

        if let Some(finalized_hash) = self.fork_choice.finalized_head()
            && let Some(finalized_meta) = self.fork_choice.get(finalized_hash)
            && block.header.height <= finalized_meta.height
            && block.hash != finalized_hash
        {
            return Err(ConsensusError::HeightRegression);
        }

        let is_genesis =
            matches!(block.header.height, 0 | 1) && block.header.parent_hash == [0u8; 32];

        if is_genesis {
            if !matches!(block.header.height, 0 | 1) {
                return Err(ConsensusError::InvalidGenesisParent);
            }
        } else {
            if block.header.parent_hash == [0u8; 32] {
                return Err(ConsensusError::InvalidGenesisParent);
            }

            let parent = self
                .blocks
                .get(&block.header.parent_hash)
                .ok_or(ConsensusError::UnknownParent)?;

            if block.header.height != parent.header.height + 1 {
                return Err(ConsensusError::InvalidParentHeight);
            }
        }

        if let Some(head) = self.fork_choice.get_head()
            && let Some(head_meta) = self.fork_choice.get(head)
            && block.header.height < head_meta.height
            && block.header.parent_hash == head_meta.parent
        {
            return Err(ConsensusError::HeightRegression);
        }

        self.fork_choice.insert_block(BlockMeta {
            hash: block.hash,
            parent: block.header.parent_hash,
            height: block.header.height,
            seal: None,
        });

        self.blocks.insert(block.hash, block);
        Ok(())
    }

    fn validate_block_integrity(&self, block: &Block) -> Result<(), ConsensusError> {
        let canonical_hash = compute_block_hash(&block.header);
        if block.hash != canonical_hash {
            return Err(ConsensusError::InvalidBlockHash);
        }

        let computed_roots = compute_body_roots(&block.body);
        let header = &block.header;
        if header.body_root != computed_roots.body_root
            || header.finality_root != computed_roots.finality_root
            || header.authority_root != computed_roots.authority_root
            || header.lane_root != computed_roots.lane_root
            || header.proof_root != computed_roots.proof_root
            || header.identity_root != computed_roots.identity_root
            || header.ai_root != computed_roots.ai_root
            || header.pq_root != computed_roots.pq_root
            || header.external_settlement_root != computed_roots.external_settlement_root
            || header.policy_root != computed_roots.policy_root
            || header.time_seal_root != computed_roots.time_seal_root
            || header.capability_flags != computed_roots.capability_flags
        {
            return Err(ConsensusError::InvalidBlockBodyCommitments);
        }

        validate_block_semantics(
            block.header.timestamp,
            block.header.crypto_epoch,
            &block.body,
        )
        .map_err(|_| ConsensusError::InvalidBlockSemantics)?;

        validate_capability_section_alignment(&block.body, block.header.capability_flags)
            .map_err(|_| ConsensusError::InvalidBlockBodyCommitments)?;

        let empty_roots = compute_body_roots(&BlockBody::default());
        validate_root_semantic_bindings(&block.body, &computed_roots, &empty_roots)
            .map_err(|_| ConsensusError::InvalidBlockBodyCommitments)?;

        Ok(())
    }

}
