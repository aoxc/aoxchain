impl HexaQuorum {
    /// Creates an empty HexaQuorum accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self { proofs: Vec::new() }
    }

    /// Returns the number of stored proofs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.proofs.len()
    }

    /// Returns true if no proofs have been added.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.proofs.is_empty()
    }

    /// Returns all stored proofs.
    #[must_use]
    pub fn proofs(&self) -> &[ApprovalProof] {
        &self.proofs
    }

    /// Adds a proof after validating its local invariants.
    ///
    /// Duplicate proofs from the same actor in the same lane are rejected.
    pub fn add_proof(&mut self, proof: ApprovalProof) -> Result<(), HexaQuorumError> {
        validate_proof(&proof)?;

        if self
            .proofs
            .iter()
            .any(|existing| existing.actor_id == proof.actor_id && existing.lane == proof.lane)
        {
            return Err(HexaQuorumError::DuplicateActorLaneProof);
        }

        self.proofs.push(proof);
        Ok(())
    }

    /// Adds multiple proofs atomically.
    ///
    /// Atomicity policy:
    /// - if any proof is invalid, no proof is added,
    /// - if any duplicate `(actor_id, lane)` exists either against the current
    ///   basket or inside the supplied batch, no proof is added.
    pub fn add_proofs<I>(&mut self, proofs: I) -> Result<(), HexaQuorumError>
    where
        I: IntoIterator<Item = ApprovalProof>,
    {
        let incoming: Vec<ApprovalProof> = proofs.into_iter().collect();

        let mut seen: BTreeSet<(String, ApprovalLane)> = self
            .proofs
            .iter()
            .map(|proof| (proof.actor_id.clone(), proof.lane))
            .collect();

        for proof in &incoming {
            validate_proof(proof)?;

            let key = (proof.actor_id.clone(), proof.lane);
            if !seen.insert(key) {
                return Err(HexaQuorumError::DuplicateActorLaneProof);
            }
        }

        self.proofs.extend(incoming);
        Ok(())
    }

    /// Clears all accumulated proofs.
    pub fn clear(&mut self) {
        self.proofs.clear();
    }

    /// Evaluates the current proof basket against the supplied policy.
    #[must_use]
    pub fn evaluate(&self, policy: &QuorumPolicy) -> QuorumResult {
        let mut lane_counts = LaneCounts::default();
        let mut missing_lanes = Vec::new();
        let mut rejection_reasons = Vec::new();

        let mut distinct_actors: BTreeSet<&str> = BTreeSet::new();
        let mut unique_stake_actors: BTreeMap<&str, u128> = BTreeMap::new();

        for proof in &self.proofs {
            distinct_actors.insert(proof.actor_id.as_str());
            lane_counts.increment(proof.lane);

            unique_stake_actors
                .entry(proof.actor_id.as_str())
                .and_modify(|current| {
                    if proof.stake > *current {
                        *current = proof.stake;
                    }
                })
                .or_insert(proof.stake);
        }

        let total_score = self
            .proofs
            .iter()
            .fold(0u64, |acc, proof| acc.saturating_add(proof.weight));

        let total_stake = unique_stake_actors
            .values()
            .fold(0u128, |acc, stake| acc.saturating_add(*stake));

        if let Err(error) = policy.validate() {
            rejection_reasons.push(format!("policy invalid: {}", error.code()));

            return QuorumResult {
                passed: false,
                distinct_actors: distinct_actors.len(),
                total_score,
                total_stake,
                lane_counts,
                missing_lanes,
                rejection_reasons,
            };
        }

        for lane in ApprovalLane::all() {
            let observed = lane_counts.get(lane);
            let required = policy.min_for_lane(lane);

            if observed < required {
                missing_lanes.push(lane);
                rejection_reasons.push(format!(
                    "lane {} below threshold: observed {} required {}",
                    lane.code(),
                    observed,
                    required
                ));
            }
        }

        if distinct_actors.len() < policy.min_distinct_actors as usize {
            rejection_reasons.push(format!(
                "distinct actor threshold not met: observed {} required {}",
                distinct_actors.len(),
                policy.min_distinct_actors
            ));
        }

        if total_stake < policy.min_total_stake {
            rejection_reasons.push(format!(
                "total stake threshold not met: observed {} required {}",
                total_stake, policy.min_total_stake
            ));
        }

        if total_score < policy.min_total_score {
            rejection_reasons.push(format!(
                "total score threshold not met: observed {} required {}",
                total_score, policy.min_total_score
            ));
        }

        let passed = rejection_reasons.is_empty();

        QuorumResult {
            passed,
            distinct_actors: distinct_actors.len(),
            total_score,
            total_stake,
            lane_counts,
            missing_lanes,
            rejection_reasons,
        }
    }
}

/// Validates a single proof before it is added to the accumulator.
fn validate_proof(proof: &ApprovalProof) -> Result<(), HexaQuorumError> {
    validate_actor_id(&proof.actor_id)?;

    if proof.timestamp == 0 {
        return Err(HexaQuorumError::InvalidTimestamp);
    }

    if proof.weight == 0 {
        return Err(HexaQuorumError::InvalidWeight);
    }

    if lane_requires_signature(proof.lane) {
        let signature = proof
            .signature
            .as_ref()
            .ok_or(HexaQuorumError::MissingSignatureForSignedLane)?;

        validate_signature_hex(signature)?;
    }

    if proof.lane == ApprovalLane::StakeLock && proof.stake == 0 {
        return Err(HexaQuorumError::InvalidStake);
    }

    Ok(())
}

/// Validates the actor identifier accepted by the quorum engine.
///
/// Compatibility note:
/// This module preserves a bounded string-based actor identifier contract
/// rather than importing a stricter actor-id parser directly.
fn validate_actor_id(actor_id: &str) -> Result<(), HexaQuorumError> {
    if actor_id.is_empty() || actor_id.trim().is_empty() {
        return Err(HexaQuorumError::EmptyActorId);
    }

    if actor_id != actor_id.trim() {
        return Err(HexaQuorumError::InvalidActorId);
    }

    if actor_id.len() > MAX_ACTOR_ID_LEN {
        return Err(HexaQuorumError::InvalidActorId);
    }

    if !actor_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(HexaQuorumError::InvalidActorId);
    }

    Ok(())
}

/// Validates a signature payload as bounded hexadecimal text.
fn validate_signature_hex(signature: &str) -> Result<(), HexaQuorumError> {
    if signature.is_empty() || signature.trim().is_empty() {
        return Err(HexaQuorumError::MissingSignatureForSignedLane);
    }

    if signature != signature.trim() {
        return Err(HexaQuorumError::InvalidSignatureFormat);
    }

    if signature.len() > MAX_SIGNATURE_HEX_LEN || !signature.len().is_multiple_of(2) {
        return Err(HexaQuorumError::InvalidSignatureFormat);
    }

    if !signature.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(HexaQuorumError::InvalidSignatureFormat);
    }

    Ok(())
}

/// Returns true when a lane requires a cryptographic signature payload.
#[must_use]
const fn lane_requires_signature(lane: ApprovalLane) -> bool {
    matches!(
        lane,
        ApprovalLane::IdentitySig
            | ApprovalLane::DeviceSig
            | ApprovalLane::TimeLockSig
            | ApprovalLane::DaoCosign
    )
}

