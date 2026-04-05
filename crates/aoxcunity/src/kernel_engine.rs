impl ConsensusEngine {
    #[must_use]
    pub fn new(state: ConsensusState) -> Self {
        Self::with_crypto_profile(state, 2626, 1)
    }

    #[must_use]
    pub fn with_network_id(state: ConsensusState, network_id: u32) -> Self {
        Self::with_crypto_profile(state, network_id, 1)
    }

    #[must_use]
    pub fn with_crypto_profile(
        state: ConsensusState,
        network_id: u32,
        signature_scheme: u16,
    ) -> Self {
        Self {
            state,
            network_id,
            signature_scheme,
            lock_state: LockState::default(),
            current_epoch: 0,
            current_height: 0,
            legitimacy_by_block: BTreeMap::new(),
            continuity_by_block: BTreeMap::new(),
            timeout_votes: BTreeMap::new(),
            timeout_conflicts: BTreeMap::new(),
            evidence_buffer: Vec::new(),
            replayed_event_hashes: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn apply_event(&mut self, event: ConsensusEvent) -> TransitionResult {
        match event {
            ConsensusEvent::AdmitBlock(block) => self.apply_admit_block(block),
            ConsensusEvent::AdmitVerifiedVote(verified_vote) => {
                self.apply_admit_verified_vote(verified_vote)
            }
            ConsensusEvent::AdmitTimeoutVote(timeout_vote) => self.apply_timeout_vote(timeout_vote),
            ConsensusEvent::ObserveLegitimacy(certificate) => {
                self.apply_legitimacy_certificate(certificate)
            }
            ConsensusEvent::ReportLeaderFailure {
                height,
                round,
                leader,
            } => self.apply_leader_failure(height, round, leader),
            ConsensusEvent::AdvanceRound { height, round } => {
                self.apply_advance_round(height, round)
            }
            ConsensusEvent::EvaluateFinality { block_hash } => {
                self.apply_evaluate_finality(block_hash)
            }
            ConsensusEvent::PruneFinalizedState { finalized_height } => {
                self.apply_prune_finalized_state(finalized_height)
            }
            ConsensusEvent::RecoverPersistedEvent { event_hash } => {
                self.apply_recover_persisted_event(event_hash)
            }
        }
    }

    pub fn apply_event_with_persistence<J, S, E, F>(
        &mut self,
        event: ConsensusEvent,
        sequence: u64,
        journal: &mut J,
        snapshots: &mut S,
        evidence: &mut E,
        finality: &mut F,
    ) -> Result<TransitionResult, String>
    where
        J: ConsensusJournal,
        S: SnapshotStore,
        E: EvidenceStore,
        F: FinalityStore,
    {
        let event_hash = hash_consensus_event(&event)?;
        journal.append(PersistedConsensusEvent {
            sequence,
            event_hash,
            event: event.clone(),
        })?;

        let evidence_start = self.evidence_buffer.len();
        let result = self.apply_event(event);

        for item in self.evidence_buffer.iter().skip(evidence_start) {
            evidence.append_evidence(item.clone())?;
        }

        for cert in &result.emitted_certificates {
            if let KernelCertificate::Constitutional(seal) = cert {
                finality.store_finalized_seal(seal.clone())?;
            }
        }

        snapshots.store_snapshot(KernelSnapshot {
            snapshot_height: self.current_height,
            snapshot_round: self.state.round.round,
            lock_state: self.lock_state.clone(),
            finalized_seal: finality.load_finalized_seal()?,
        })?;

        Ok(result)
    }

    pub fn recover_from_state(&mut self, recovery: &RecoveryState) -> Result<(), String> {
        if let Some(snapshot) = &recovery.snapshot {
            self.current_height = self.current_height.max(snapshot.snapshot_height);
            self.state.round.advance_to(snapshot.snapshot_round);
            self.lock_state = snapshot.lock_state.clone();
        }

        self.evidence_buffer.extend(recovery.evidence.clone());

        let mut journal = recovery.journal.clone();
        journal.sort_by_key(|entry| entry.sequence);
        for entry in journal {
            let expected_hash = hash_consensus_event(&entry.event)?;
            if expected_hash != entry.event_hash {
                return Err("persisted event hash mismatch".to_string());
            }

            let marker = self.apply_event(ConsensusEvent::RecoverPersistedEvent {
                event_hash: entry.event_hash,
            });
            if marker.rejected_reason.is_some() {
                return Err("recovery marker rejected".to_string());
            }

            let result = self.apply_event(entry.event);
            if result.rejected_reason.is_some() {
                return Err("persisted event replay rejected".to_string());
            }
        }

        Ok(())
    }

    fn apply_admit_block(&mut self, block: Block) -> TransitionResult {
        let block_hash = block.hash;
        let block_height = block.header.height;

        match self.state.admit_block(block) {
            Ok(()) => {
                self.current_height = self.current_height.max(block_height);
                TransitionResult::accepted(KernelEffect::BlockAccepted(block_hash))
            }
            Err(error) => {
                let result = TransitionResult::rejected(map_consensus_error(&error));
                if matches!(error, ConsensusError::HeightRegression) {
                    return result.with_stale_branch_reactivated();
                }
                result
            }
        }
    }

    fn apply_admit_verified_vote(&mut self, verified_vote: VerifiedVote) -> TransitionResult {
        let candidate = justification_from_vote(
            &verified_vote.authenticated_vote.vote,
            verified_vote.authenticated_vote.context.epoch,
        );

        if let SafeToVote::No(violation) = evaluate_safe_to_vote(&self.lock_state, &candidate) {
            return TransitionResult::rejected(map_safety_violation(violation));
        }

        let expected_context = self.vote_authentication_context();
        match self
            .state
            .add_authenticated_vote(verified_vote.authenticated_vote.clone(), expected_context)
        {
            Ok(()) => {
                if matches!(verified_vote.authenticated_vote.vote.kind, VoteKind::Commit) {
                    self.lock_state.advance_to(candidate);
                }

                self.current_epoch = self
                    .current_epoch
                    .max(verified_vote.authenticated_vote.context.epoch);
                self.current_height = self
                    .current_height
                    .max(verified_vote.authenticated_vote.vote.height);

                TransitionResult::accepted(KernelEffect::VoteAccepted(
                    verified_vote.authenticated_vote.vote.block_hash,
                ))
            }
            Err(error) => {
                if matches!(error, ConsensusError::EquivocatingVote) {
                    let _ = self.state.slash_validator(
                        verified_vote.authenticated_vote.vote.voter,
                        5,
                        100,
                        SlashFault::Equivocation,
                    );
                    self.evidence_buffer.push(equivocation_evidence(
                        verified_vote.authenticated_vote.vote.block_hash,
                        "vote",
                    ));
                }

                TransitionResult::rejected(map_consensus_error(&error))
            }
        }
    }

    fn apply_timeout_vote(&mut self, timeout_vote: VerifiedTimeoutVote) -> TransitionResult {
        let vote = timeout_vote.timeout_vote.clone();

        if !self.state.blocks.contains_key(&vote.block_hash) {
            return TransitionResult::rejected(KernelRejection::StaleArtifact);
        }

        if !self
            .state
            .rotation
            .contains_active_vote_eligible_validator(vote.voter)
        {
            return TransitionResult::rejected(KernelRejection::InvalidSignature);
        }

        let conflict_key = TimeoutConflictKey {
            voter: vote.voter,
            height: vote.height,
            round: vote.round,
            epoch: vote.epoch,
            timeout_round: vote.timeout_round,
        };

        if let Some(existing_block_hash) = self.timeout_conflicts.get(&conflict_key) {
            if *existing_block_hash == vote.block_hash {
                return TransitionResult::rejected(KernelRejection::DuplicateArtifact);
            }

            self.evidence_buffer
                .push(equivocation_evidence(vote.block_hash, "timeout"));

            return TransitionResult::rejected(KernelRejection::InvariantViolation)
                .with_conflicting_finality_detected();
        }

        let key = TimeoutVoteKey {
            block_hash: vote.block_hash,
            height: vote.height,
            round: vote.round,
            epoch: vote.epoch,
            timeout_round: vote.timeout_round,
        };

        self.timeout_conflicts.insert(conflict_key, vote.block_hash);
        self.timeout_votes
            .entry(key)
            .or_default()
            .insert(vote.voter, timeout_vote);

        let mut result = TransitionResult::accepted(KernelEffect::TimeoutAccepted(vote.block_hash));

        if let Some(certificate) = self.maybe_build_continuity_certificate(key) {
            self.lock_state.advance_to(JustificationRef {
                block_hash: certificate.block_hash,
                height: certificate.height,
                round: certificate.timeout_round,
                epoch: certificate.epoch,
                certificate_hash: certificate.certificate_hash,
            });

            self.state
                .round
                .advance_to(certificate.timeout_round.saturating_add(1));
            self.current_epoch = self.current_epoch.max(certificate.epoch);
            self.current_height = self.current_height.max(certificate.height);

            self.continuity_by_block
                .insert(certificate.block_hash, certificate.clone());

            result.accepted_effects.push(KernelEffect::RoundAdvanced {
                height: certificate.height,
                round: self.state.round.round,
            });
            result
                .emitted_certificates
                .push(KernelCertificate::Continuity(certificate));
        }

        result
    }

    fn apply_leader_failure(
        &mut self,
        height: u64,
        round: u64,
        leader: ValidatorId,
    ) -> TransitionResult {
        self.current_height = self.current_height.max(height);
        let step = self.state.round.on_leader_failure();
        if step.next_round < round {
            self.state.round.advance_to(round);
        }

        if let Some(validator) = self.state.rotation.validator_mut(leader) {
            validator.register_liveness_miss(3, self.state.round.round.saturating_add(3));
            if !validator.active {
                let _ = self
                    .state
                    .slash_validator(leader, 1, 100, SlashFault::Liveness);
            }
        }

        TransitionResult::accepted(KernelEffect::RoundAdvanced {
            height,
            round: self.state.round.round,
        })
    }

    fn apply_legitimacy_certificate(
        &mut self,
        certificate: LegitimacyCertificate,
    ) -> TransitionResult {
        if !self.state.blocks.contains_key(&certificate.block_hash) {
            return TransitionResult::rejected(KernelRejection::StaleArtifact);
        }

        if certificate.validate().is_err() {
            self.evidence_buffer.push(ConsensusEvidence {
                evidence_hash: certificate.certificate_hash,
                related_block_hash: certificate.block_hash,
                reason: "invalid_legitimacy_certificate".to_string(),
            });

            return TransitionResult::rejected(KernelRejection::InvariantViolation)
                .with_conflicting_finality_detected();
        }

        self.current_epoch = self.current_epoch.max(certificate.authority_epoch);
        self.legitimacy_by_block
            .insert(certificate.block_hash, certificate.clone());

        TransitionResult {
            accepted_effects: Vec::new(),
            rejected_reason: None,
            emitted_certificates: vec![KernelCertificate::Legitimacy(certificate)],
            pruning_actions: Vec::new(),
            invariant_status: InvariantStatus::healthy(),
        }
    }

    fn apply_advance_round(&mut self, height: u64, round: u64) -> TransitionResult {
        if round < self.state.round.round {
            return TransitionResult::rejected(KernelRejection::StaleArtifact);
        }

        self.current_height = self.current_height.max(height);
        self.state.round.advance_to(round);

        TransitionResult::accepted(KernelEffect::RoundAdvanced { height, round })
    }

    fn apply_evaluate_finality(&mut self, block_hash: [u8; 32]) -> TransitionResult {
        let Some(finalized_round) = self.state.finalizable_round(block_hash) else {
            return TransitionResult::rejected(KernelRejection::StaleArtifact);
        };

        let authenticated_certificate = self.state.authenticated_quorum_certificate(
            block_hash,
            finalized_round,
            self.vote_authentication_context(),
        );

        let Some(seal) = self.state.try_finalize(block_hash, finalized_round) else {
            return TransitionResult::rejected(KernelRejection::FinalityConflict)
                .with_conflicting_finality_detected();
        };

        let execution = ExecutionCertificate::new(
            self.current_epoch,
            self.state.rotation.validator_set_hash(),
            seal.certificate.clone(),
        );

        let mut result = TransitionResult::accepted(KernelEffect::BlockFinalized(block_hash));

        if let Some(certificate) = authenticated_certificate {
            result
                .emitted_certificates
                .push(KernelCertificate::Execution(certificate));
        }

        match self.try_compose_constitutional_seal(&execution) {
            Ok(Some(constitutional)) => {
                result
                    .emitted_certificates
                    .push(KernelCertificate::Constitutional(constitutional));
            }
            Ok(None) => {}
            Err(error) => {
                self.evidence_buffer.push(ConsensusEvidence {
                    evidence_hash: execution.certificate_hash,
                    related_block_hash: block_hash,
                    reason: constitutional_error_reason(error),
                });
                result = result.with_conflicting_finality_detected();
            }
        }

        result
    }

    fn apply_prune_finalized_state(&mut self, finalized_height: u64) -> TransitionResult {
        let before_blocks = self.state.blocks.len();
        let before_timeouts = self.timeout_votes.len();

        let pruned_blocks = prune_state_to_height(&mut self.state, finalized_height);

        self.timeout_votes
            .retain(|key, _| key.height >= finalized_height);
        self.timeout_conflicts
            .retain(|key, _| key.height >= finalized_height);

        let pruned_timeouts = before_timeouts.saturating_sub(self.timeout_votes.len());

        TransitionResult {
            accepted_effects: Vec::new(),
            rejected_reason: None,
            emitted_certificates: Vec::new(),
            pruning_actions: vec![PruningAction {
                pruned_blocks,
                pruned_votes: before_blocks
                    .saturating_sub(self.state.blocks.len())
                    .saturating_sub(pruned_blocks),
                pruned_timeouts,
            }],
            invariant_status: InvariantStatus::healthy(),
        }
    }

    fn apply_recover_persisted_event(&mut self, event_hash: [u8; 32]) -> TransitionResult {
        if !self.replayed_event_hashes.insert(event_hash) {
            return TransitionResult::rejected(KernelRejection::DuplicateArtifact)
                .with_replay_diverged();
        }

        TransitionResult::accepted(KernelEffect::StateRecovered(event_hash))
    }

    fn maybe_build_continuity_certificate(
        &self,
        key: TimeoutVoteKey,
    ) -> Option<ContinuityCertificate> {
        let votes = self.timeout_votes.get(&key)?;
        let signers: Vec<ValidatorId> = votes.keys().copied().collect();

        let observed_power: u64 = signers
            .iter()
            .filter_map(|validator| self.state.rotation.eligible_voting_power_of(*validator))
            .sum();

        let total_power = self.state.rotation.total_voting_power();
        if !self.state.quorum.is_reached(observed_power, total_power) {
            return None;
        }

        let certificate = ContinuityCertificate::new(
            key.block_hash,
            key.height,
            key.round,
            key.epoch,
            key.timeout_round,
            observed_power,
            signers,
        );

        certificate.validate().ok()?;
        Some(certificate)
    }

    fn try_compose_constitutional_seal(
        &self,
        execution: &ExecutionCertificate,
    ) -> Result<Option<ConstitutionalSeal>, ConstitutionalValidationError> {
        let Some(legitimacy) = self.legitimacy_by_block.get(&execution.block_hash) else {
            return Ok(None);
        };

        let Some(continuity) = self.continuity_by_block.get(&execution.block_hash) else {
            return Ok(None);
        };

        let seal = ConstitutionalSeal::compose_strict(execution, legitimacy, continuity)?;
        Ok(Some(seal))
    }

    fn vote_authentication_context(&self) -> VoteAuthenticationContext {
        VoteAuthenticationContext {
            network_id: self.network_id,
            epoch: self.current_epoch,
            validator_set_root: self.state.rotation.validator_set_hash(),
            pq_attestation_root: self.state.rotation.pq_attestation_root(),
            signature_scheme: self.signature_scheme,
        }
    }
}

fn prune_state_to_height(state: &mut ConsensusState, finalized_height: u64) -> usize {
    let before_blocks = state.blocks.len();

    state.blocks.retain(|hash, block| {
        block.header.height >= finalized_height
            || state
                .fork_choice
                .finalized_head()
                .is_some_and(|finalized| finalized == *hash)
    });

    state.vote_pool.prune_blocks(|hash| {
        state
            .blocks
            .get(&hash)
            .is_some_and(|block| block.header.height >= finalized_height)
    });

    before_blocks.saturating_sub(state.blocks.len())
}

