impl ConsensusState {
    /// Attempts to finalize a block by constructing a quorum certificate and
    /// marking the block finalized in fork choice.
    ///
    /// # Security Semantics
    /// Finalization succeeds only when a valid quorum certificate can be built
    /// from eligible commit votes matching the exact block, height, and round.
    ///
    /// # Determinism
    /// The returned seal is fully derived from canonical block identity,
    /// deterministic signer normalization, and the fixed quorum policy.
    pub fn try_finalize(
        &mut self,
        block_hash: [u8; 32],
        finalized_round: u64,
    ) -> Option<BlockSeal> {
        let certificate = self.build_quorum_certificate(block_hash, finalized_round)?;
        let seal = BlockSeal {
            block_hash,
            finalized_round,
            attestation_root: certificate.certificate_hash,
            certificate,
        };

        if self.fork_choice.mark_finalized(block_hash, seal.clone()) {
            self.prune_to_finalized_branch(block_hash);
            Some(seal)
        } else {
            None
        }
    }

    /// Builds an authenticated quorum certificate by binding deterministic quorum
    /// evidence to the supplied authentication context.
    pub fn authenticated_quorum_certificate(
        &self,
        block_hash: [u8; 32],
        finalized_round: u64,
        context: VoteAuthenticationContext,
    ) -> Option<AuthenticatedQuorumCertificate> {
        let certificate = self.build_quorum_certificate(block_hash, finalized_round)?;
        Some(AuthenticatedQuorumCertificate::new(
            certificate,
            context.network_id,
            context.epoch,
            context.validator_set_root,
            context.signature_scheme,
        ))
    }

    /// Returns the proposer selected for the supplied height under the current
    /// deterministic rotation policy.
    #[must_use]
    pub fn proposer_for_height(&self, height: u64) -> Option<ValidatorId> {
        self.rotation.proposer(height)
    }

    /// Returns the proposer selected for `(height, round)` using deterministic
    /// weight-aware lottery and caller-supplied entropy.
    #[must_use]
    pub fn proposer_for_round(
        &self,
        height: u64,
        round: u64,
        entropy: [u8; 32],
    ) -> Option<ValidatorId> {
        self.rotation.proposer_with_round(height, round, entropy)
    }

    /// Bonds additional self stake for the validator and increases its voting power.
    pub fn bond_validator(
        &mut self,
        validator_id: ValidatorId,
        amount: u64,
    ) -> Result<(), ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        validator.bond(amount);
        Ok(())
    }

    /// Starts unbonding and removes stake from immediate voting power.
    pub fn unbond_validator(
        &mut self,
        validator_id: ValidatorId,
        amount: u64,
    ) -> Result<u64, ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        Ok(validator.unbond(amount))
    }

    /// Adds delegated stake to validator effective voting power.
    pub fn delegate_to_validator(
        &mut self,
        validator_id: ValidatorId,
        amount: u64,
    ) -> Result<(), ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        validator.delegate(amount);
        Ok(())
    }

    /// Removes delegated stake from validator effective voting power.
    pub fn undelegate_from_validator(
        &mut self,
        validator_id: ValidatorId,
        amount: u64,
    ) -> Result<u64, ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        Ok(validator.undelegate(amount))
    }

    /// Applies slashing to a validator under a supplied fault model.
    pub fn slash_validator(
        &mut self,
        validator_id: ValidatorId,
        numerator: u64,
        denominator: u64,
        fault: SlashFault,
    ) -> Result<u64, ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        Ok(validator.slash(numerator, denominator, fault))
    }

    /// Returns the highest round for which the target block currently satisfies
    /// deterministic finalization requirements.
    #[must_use]
    pub fn finalizable_round(&self, block_hash: [u8; 32]) -> Option<u64> {
        let mut rounds: Vec<u64> = self
            .vote_pool
            .votes_for_block_kind(block_hash, VoteKind::Commit)
            .into_iter()
            .map(|vote| vote.round)
            .collect();

        rounds.sort_unstable();
        rounds.dedup();

        rounds
            .into_iter()
            .rev()
            .find(|round| self.build_quorum_certificate(block_hash, *round).is_some())
    }

    /// Builds a deterministic quorum certificate from eligible commit votes.
    ///
    /// # Certificate Rules
    /// - only `Commit` votes participate,
    /// - vote height must match the target block height,
    /// - vote round must match the requested finalization round,
    /// - only eligible validator voting power contributes,
    /// - signer ordering is canonicalized.
    fn build_quorum_certificate(
        &self,
        block_hash: [u8; 32],
        finalized_round: u64,
    ) -> Option<QuorumCertificate> {
        let block = self.blocks.get(&block_hash)?;

        let mut signer_set = BTreeSet::new();
        let mut signers = Vec::new();
        let mut observed_voting_power = 0u64;

        for vote in self
            .vote_pool
            .votes_for_block_kind(block_hash, VoteKind::Commit)
        {
            if vote.height != block.header.height || vote.round != finalized_round {
                continue;
            }

            if !signer_set.insert(vote.voter) {
                continue;
            }

            let voting_power = self.rotation.eligible_voting_power_of(vote.voter)?;
            signers.push(vote.voter);
            observed_voting_power = observed_voting_power.saturating_add(voting_power);
        }

        let total_voting_power = self.rotation.total_voting_power();
        if !self
            .quorum
            .is_reached(observed_voting_power, total_voting_power)
        {
            return None;
        }

        Some(QuorumCertificate::new(
            block_hash,
            block.header.height,
            finalized_round,
            signers,
            observed_voting_power,
            total_voting_power,
            self.quorum.numerator,
            self.quorum.denominator,
        ))
    }

    /// Enforces that the candidate vote remains on the live finalized branch.
    ///
    /// This policy is intentionally strict. Once a finalized head exists, votes
    /// must not target lower-height historical artifacts or conflicting branches.
    fn ensure_vote_targets_live_branch(
        &self,
        block_hash: [u8; 32],
        vote_height: u64,
    ) -> Result<(), ConsensusError> {
        let Some(finalized_hash) = self.fork_choice.finalized_head() else {
            return Ok(());
        };

        let finalized_height = self
            .fork_choice
            .get(finalized_hash)
            .map(|meta| meta.height)
            .unwrap_or_default();

        if vote_height < finalized_height {
            return Err(ConsensusError::StaleVote);
        }

        if block_hash != finalized_hash && !self.fork_choice.is_ancestor(finalized_hash, block_hash)
        {
            return Err(ConsensusError::StaleVote);
        }

        Ok(())
    }

    /// Removes blocks and votes that are no longer on the finalized branch.
    ///
    /// This is a memory-bounding and safety-preserving step. Once finality is
    /// established, conflicting non-finalized branches should not remain vote-active
    /// inside the canonical local state view.
    fn prune_to_finalized_branch(&mut self, finalized_hash: [u8; 32]) {
        self.blocks.retain(|hash, _| {
            *hash == finalized_hash || self.fork_choice.is_ancestor(finalized_hash, *hash)
        });

        self.vote_pool
            .prune_blocks(|hash| self.blocks.contains_key(&hash));
    }
}
