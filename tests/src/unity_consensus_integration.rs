use std::{
    env, fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use aoxcmd::{
    data_home::ensure_layout,
    keys::manager::bootstrap_operator_key,
    node::{engine::produce_once, lifecycle::bootstrap_state},
};
use aoxcnet::{
    config::{ExternalDomainKind, NetworkConfig},
    gossip::{
        consensus_gossip::GossipEngine,
        peer::{NodeCertificate, Peer, PeerRole},
    },
};
use aoxcunity::{
    AuthenticatedVote, BlockBody, ConsensusMessage, ConsensusState, LaneCommitment,
    LaneCommitmentSection, LaneType, Proposer, QuorumThreshold, Validator, ValidatorRole,
    ValidatorRotation, Vote, VoteAuthenticationContext, VoteKind,
};
use ed25519_dalek::{Signer, SigningKey};

#[test]
fn unity_consensus_flow_integrates_cmd_network_and_finality() {
    let _lock = env_lock().lock().expect("env mutex must not be poisoned");
    let home = unique_test_home("unity-consensus");
    let previous_home = env::var_os("AOXC_HOME");

    ensure_layout(&home).expect("test AOXC home layout should be created");
    unsafe {
        env::set_var("AOXC_HOME", &home);
    }

    let test_result = {
        let key_material = bootstrap_operator_key("validator-01", "testnet", "Test#2026!")
            .expect("operator key bootstrap should succeed");
        let key_summary = key_material
            .summary()
            .expect("operator key summary should stay derivable");

        let bootstrapped = bootstrap_state().expect("node state bootstrap should succeed");
        assert_eq!(bootstrapped.consensus.network_id, 2626);

        let produced =
            produce_once("integration-lifecycle").expect("single block production should succeed");
        assert_eq!(produced.current_height, 1);
        assert_eq!(produced.produced_blocks, 1);
        assert_eq!(produced.consensus.last_message_kind, "block_proposal");
        assert_eq!(
            produced.key_material.consensus_public_key_hex,
            key_summary.consensus_public_key
        );

        let proposer_key =
            decode_hex32(&key_summary.consensus_public_key, "consensus proposer key");
        let auth_context = VoteAuthenticationContext {
            network_id: 2626,
            epoch: 1,
            validator_set_root: [9u8; 32],
            signature_scheme: 1,
        };

        let validator_keys = [[1u8; 32], [2u8; 32], [3u8; 32]]
            .into_iter()
            .map(|secret| SigningKey::from_bytes(&secret))
            .collect::<Vec<SigningKey>>();
        let validators = validator_keys
            .iter()
            .map(|key: &SigningKey| {
                Validator::new(key.verifying_key().to_bytes(), 1, ValidatorRole::Validator)
            })
            .collect::<Vec<_>>();

        let rotation =
            ValidatorRotation::new(validators).expect("validator rotation should be valid");
        let mut consensus = ConsensusState::new(rotation, QuorumThreshold::two_thirds());

        let proposal = Proposer::new(2626, proposer_key)
            .propose(
                [0u8; 32],
                1,
                0,
                1,
                1_800_000_000,
                BlockBody {
                    sections: vec![aoxcunity::BlockSection::LaneCommitment(
                        LaneCommitmentSection {
                            lanes: vec![LaneCommitment {
                                lane_id: 7,
                                lane_type: LaneType::Native,
                                tx_count: 1,
                                input_root: [1u8; 32],
                                output_root: [2u8; 32],
                                receipt_root: [3u8; 32],
                                state_commitment: [4u8; 32],
                                proof_commitment: [5u8; 32],
                            }],
                        },
                    )],
                },
            )
            .expect("canonical block proposal should be built");
        consensus
            .admit_block(proposal.clone())
            .expect("proposal should be admitted");

        let mut gossip = GossipEngine::new(NetworkConfig::default());
        gossip
            .register_peer(consensus_peer())
            .expect("peer should register");
        gossip
            .establish_session("validator-1")
            .expect("peer session should establish");

        let proposal_envelope = gossip
            .broadcast_from_peer(
                "validator-1",
                ConsensusMessage::BlockProposal {
                    block: proposal.clone(),
                },
            )
            .expect("proposal should broadcast");
        assert_eq!(
            proposal_envelope.payload.canonical_bytes().first(),
            Some(&0)
        );

        for signing_key in validator_keys.iter().take(2) {
            let vote = Vote {
                voter: signing_key.verifying_key().to_bytes(),
                block_hash: proposal.hash,
                height: proposal.header.height,
                round: proposal.header.round,
                kind: VoteKind::Commit,
            };
            let signature = signing_key
                .sign(
                    &AuthenticatedVote {
                        vote: vote.clone(),
                        context: auth_context,
                        signature: Vec::new(),
                    }
                    .signing_bytes(),
                )
                .to_bytes()
                .to_vec();
            let authenticated_vote = AuthenticatedVote {
                vote,
                context: auth_context,
                signature,
            };

            let verified = authenticated_vote
                .verify()
                .expect("authenticated vote should verify");
            consensus
                .add_authenticated_vote(verified, auth_context)
                .expect("verified vote should enter consensus state");
            gossip
                .broadcast_from_peer("validator-1", ConsensusMessage::Vote(authenticated_vote))
                .expect("vote should broadcast");
        }

        assert!(consensus.has_quorum(proposal.hash, VoteKind::Commit));
        let seal = consensus
            .try_finalize(proposal.hash, proposal.header.round)
            .expect("commit quorum should finalize block");
        let certificate = consensus
            .authenticated_quorum_certificate(proposal.hash, proposal.header.round, auth_context)
            .expect("finalized block should produce authenticated QC");

        let finalize_envelope = gossip
            .broadcast_from_peer(
                "validator-1",
                ConsensusMessage::Finalize {
                    seal: seal.clone(),
                    certificate: certificate.clone(),
                },
            )
            .expect("finalize should broadcast");

        assert_eq!(
            finalize_envelope.payload.canonical_bytes().first(),
            Some(&2)
        );
        assert!(gossip.receive().is_some());
        assert!(gossip.receive().is_some());
        assert!(gossip.receive().is_some());
        assert!(matches!(
            gossip.receive(),
            Some(ConsensusMessage::Finalize { .. })
        ));
        assert_eq!(
            consensus
                .fork_choice
                .finalized_head()
                .expect("finalized head should exist"),
            proposal.hash
        );

        Ok::<(), Box<dyn std::error::Error>>(())
    };

    if let Some(previous) = previous_home {
        unsafe {
            env::set_var("AOXC_HOME", previous);
        }
    } else {
        unsafe {
            env::remove_var("AOXC_HOME");
        }
    }
    let _ = fs::remove_dir_all(&home);

    test_result.expect("integration flow should succeed");
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn unique_test_home(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be monotonic")
        .as_nanos();
    env::temp_dir().join(format!("aoxchain-{prefix}-{nanos}"))
}

fn decode_hex32(value: &str, label: &str) -> [u8; 32] {
    let bytes = hex::decode(value).unwrap_or_else(|_| panic!("{label} must decode from hex"));
    assert_eq!(bytes.len(), 32, "{label} must be exactly 32 bytes");
    let mut output = [0u8; 32];
    output.copy_from_slice(&bytes);
    output
}

fn consensus_peer() -> Peer {
    Peer::new(
        "validator-1",
        "10.0.0.1:2727",
        "AOXC-MAINNET",
        ExternalDomainKind::Native,
        PeerRole::Validator,
        3,
        true,
        NodeCertificate {
            subject: "validator-1".to_string(),
            issuer: "AOXC-ROOT".to_string(),
            valid_from_unix: 1,
            valid_until_unix: u64::MAX,
            serial: "serial-1".to_string(),
            domain_attestation_hash: "attestation-1".to_string(),
        },
    )
}
