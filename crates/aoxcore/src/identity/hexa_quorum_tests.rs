#[cfg(test)]
mod tests {
    use super::*;

    fn proof(
        actor_id: &str,
        lane: ApprovalLane,
        weight: u64,
        stake: u128,
        with_sig: bool,
    ) -> ApprovalProof {
        ApprovalProof {
            actor_id: actor_id.to_string(),
            lane,
            signature: if with_sig {
                Some("ABCDEF12".to_string())
            } else {
                None
            },
            timestamp: 1_700_000_000,
            weight,
            stake,
        }
    }

    #[test]
    fn duplicate_actor_same_lane_is_rejected() {
        let mut quorum = HexaQuorum::new();

        quorum
            .add_proof(proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true))
            .expect("first proof must succeed");

        let result = quorum.add_proof(proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true));

        assert_eq!(result, Err(HexaQuorumError::DuplicateActorLaneProof));
    }

    #[test]
    fn missing_signature_for_signed_lane_is_rejected() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proof(proof("actor-1", ApprovalLane::IdentitySig, 5, 100, false));

        assert_eq!(result, Err(HexaQuorumError::MissingSignatureForSignedLane));
    }

    #[test]
    fn invalid_signature_format_is_rejected() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proof(ApprovalProof {
            actor_id: "actor-1".to_string(),
            lane: ApprovalLane::IdentitySig,
            signature: Some("NOT_HEX!".to_string()),
            timestamp: 1_700_000_000,
            weight: 5,
            stake: 100,
        });

        assert_eq!(result, Err(HexaQuorumError::InvalidSignatureFormat));
    }

    #[test]
    fn zero_timestamp_is_rejected() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proof(ApprovalProof {
            actor_id: "actor-1".to_string(),
            lane: ApprovalLane::StakeLock,
            signature: None,
            timestamp: 0,
            weight: 5,
            stake: 100,
        });

        assert_eq!(result, Err(HexaQuorumError::InvalidTimestamp));
    }

    #[test]
    fn zero_stake_for_stake_lane_is_rejected() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proof(ApprovalProof {
            actor_id: "actor-1".to_string(),
            lane: ApprovalLane::StakeLock,
            signature: None,
            timestamp: 1_700_000_000,
            weight: 5,
            stake: 0,
        });

        assert_eq!(result, Err(HexaQuorumError::InvalidStake));
    }

    #[test]
    fn add_proofs_is_atomic() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proofs([
            proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true),
            proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true),
        ]);

        assert_eq!(result, Err(HexaQuorumError::DuplicateActorLaneProof));
        assert_eq!(quorum.len(), 0);
    }

    #[test]
    fn valid_policy_passes_with_sufficient_multiaxis_proofs() {
        let mut quorum = HexaQuorum::new();
        let policy = QuorumPolicy::strict_default();

        quorum
            .add_proofs([
                proof("actor-1", ApprovalLane::IdentitySig, 3, 100, true),
                proof("actor-2", ApprovalLane::IdentitySig, 3, 200, true),
                proof("actor-1", ApprovalLane::DeviceSig, 1, 100, true),
                proof("actor-2", ApprovalLane::TimeLockSig, 1, 200, true),
                proof("actor-1", ApprovalLane::StakeLock, 2, 100, false),
                proof("actor-2", ApprovalLane::RoleProof, 2, 200, false),
                proof("dao-1", ApprovalLane::DaoCosign, 4, 0, true),
            ])
            .expect("proof basket must be valid");

        let result = quorum.evaluate(&policy);

        assert!(result.passed);
        assert!(result.rejection_reasons.is_empty());
        assert_eq!(result.distinct_actors, 3);
        assert_eq!(result.total_stake, 300);
    }

    #[test]
    fn policy_fails_when_lanes_are_missing() {
        let mut quorum = HexaQuorum::new();
        let policy = QuorumPolicy::strict_default();

        quorum
            .add_proof(proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true))
            .expect("proof must succeed");

        let result = quorum.evaluate(&policy);

        assert!(!result.passed);
        assert!(!result.missing_lanes.is_empty());
        assert!(!result.rejection_reasons.is_empty());
    }

    #[test]
    fn stake_is_counted_once_per_actor_using_max_observed_value() {
        let mut quorum = HexaQuorum::new();
        let policy = QuorumPolicy {
            min_identity: 1,
            min_device: 1,
            min_timelock: 0,
            min_stake_lock: 1,
            min_role: 0,
            min_dao: 0,
            min_distinct_actors: 1,
            min_total_stake: 100,
            min_total_score: 3,
        };

        quorum
            .add_proofs([
                proof("actor-1", ApprovalLane::IdentitySig, 1, 50, true),
                proof("actor-1", ApprovalLane::DeviceSig, 1, 80, true),
                proof("actor-1", ApprovalLane::StakeLock, 1, 120, false),
            ])
            .expect("proof basket must be valid");

        let result = quorum.evaluate(&policy);

        assert!(result.passed);
        assert_eq!(result.total_stake, 120);
    }

    #[test]
    fn invalid_policy_is_reported_as_rejection() {
        let quorum = HexaQuorum::new();
        let policy = QuorumPolicy {
            min_identity: 0,
            min_device: 0,
            min_timelock: 0,
            min_stake_lock: 0,
            min_role: 0,
            min_dao: 0,
            min_distinct_actors: 0,
            min_total_stake: 0,
            min_total_score: 0,
        };

        let result = quorum.evaluate(&policy);

        assert!(!result.passed);
        assert_eq!(
            result.rejection_reasons,
            vec!["policy invalid: HEXA_QUORUM_INVALID_POLICY".to_string()]
        );
    }

    #[test]
    fn lane_counts_are_recorded_correctly() {
        let mut quorum = HexaQuorum::new();
        let policy = QuorumPolicy {
            min_identity: 1,
            min_device: 1,
            min_timelock: 0,
            min_stake_lock: 0,
            min_role: 0,
            min_dao: 0,
            min_distinct_actors: 1,
            min_total_stake: 0,
            min_total_score: 2,
        };

        quorum
            .add_proofs([
                proof("actor-1", ApprovalLane::IdentitySig, 1, 0, true),
                proof("actor-1", ApprovalLane::DeviceSig, 1, 0, true),
            ])
            .expect("proof basket must be valid");

        let result = quorum.evaluate(&policy);

        assert!(result.passed);
        assert_eq!(result.lane_counts.identity, 1);
        assert_eq!(result.lane_counts.device, 1);
        assert_eq!(result.lane_counts.timelock, 0);
    }
}
