impl ConsensusState {
    /// Verifies the supplied signed vote and admits the resulting canonical vote.
    pub fn add_signed_vote(
        &mut self,
        signed_vote: SignedVote,
    ) -> Result<(), VoteAuthenticationError> {
        let verified = signed_vote.verify()?;
        self.add_verified_vote(verified)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)
    }

    /// Admits an already-verified vote into state.
    pub fn add_verified_vote(&mut self, verified_vote: VerifiedVote) -> Result<(), ConsensusError> {
        self.add_vote(verified_vote.into_vote())
    }

    /// Admits an authenticated vote only if the authentication context matches
    /// the exact local expectation.
    ///
    /// This prevents replay or cross-context contamination across network,
    /// epoch, validator-set, or signature-scheme boundaries.
    pub fn add_authenticated_vote(
        &mut self,
        verified_vote: VerifiedAuthenticatedVote,
        expected_context: VoteAuthenticationContext,
    ) -> Result<(), ConsensusError> {
        if verified_vote.context != expected_context {
            return Err(ConsensusError::InvalidAuthenticatedContext);
        }

        self.add_vote(verified_vote.vote)
    }

    /// Admits a vote into local consensus state after eligibility validation.
    ///
    /// # Security Semantics
    /// Votes are rejected when:
    /// - the target block is unknown,
    /// - the vote targets stale or conflicting finalized ancestry,
    /// - the voter is unknown,
    /// - the voter is inactive,
    /// - the voter is not eligible to vote.
    ///
    /// Duplicate and equivocating votes are further rejected by `VotePool`.
    pub fn add_vote(&mut self, vote: Vote) -> Result<(), ConsensusError> {
        let target = self
            .blocks
            .get(&vote.block_hash)
            .ok_or(ConsensusError::VoteForUnknownBlock)?;

        if vote.height != target.header.height {
            return Err(ConsensusError::VoteForUnknownBlock);
        }

        self.ensure_vote_targets_live_branch(vote.block_hash, vote.height)?;

        let validator = self
            .rotation
            .validator(vote.voter)
            .ok_or(ConsensusError::ValidatorNotFound)?;

        if !validator.is_eligible_for_vote() {
            if !validator.active {
                return Err(ConsensusError::InactiveValidator);
            }
            return Err(ConsensusError::NonVotingValidator);
        }

        self.vote_pool.add_vote(vote)
    }

    /// Returns observed voting power for a block and vote kind using only
    /// active, eligible voting participants.
    #[must_use]
    pub fn observed_voting_power(&self, block_hash: [u8; 32], kind: VoteKind) -> u64 {
        self.vote_pool
            .votes_for_block_kind(block_hash, kind)
            .into_iter()
            .filter_map(|vote| self.rotation.eligible_voting_power_of(vote.voter))
            .sum()
    }

    /// Returns `true` if quorum is reached for the specified block and vote kind.
    #[must_use]
    pub fn has_quorum(&self, block_hash: [u8; 32], kind: VoteKind) -> bool {
        let observed = self.observed_voting_power(block_hash, kind);
        let total = self.rotation.total_voting_power();
        self.quorum.is_reached(observed, total)
    }
}
