use crate::block::PQ_MANDATORY_START_EPOCH;
use ed25519_dalek::{Signer, SigningKey};
use libcrux_ml_dsa::ml_dsa_65::{generate_key_pair, sign as mldsa_sign};
use rand::random;

use super::{
    AuthenticatedVote, ConsensusIdentityProfile, SignedVote, Vote, VoteAuthenticationContext,
    VoteAuthenticationError, VoteKind, SIGNATURE_SCHEME_DILITHIUM3, SIGNATURE_SCHEME_ED25519,
    SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3,
};

fn make_vote(block_hash: [u8; 32], round: u64, kind: VoteKind) -> Vote {
    Vote {
        voter: [7u8; 32],
        block_hash,
        height: 9,
        round,
        kind,
    }
}

#[test]
fn signing_bytes_are_deterministic_for_identical_votes() {
    let first = make_vote([1u8; 32], 2, VoteKind::Prepare);
    let second = make_vote([1u8; 32], 2, VoteKind::Prepare);

    assert_eq!(first.signing_bytes(), second.signing_bytes());
}

#[test]
fn signing_bytes_change_with_block_hash() {
    let first = make_vote([1u8; 32], 2, VoteKind::Prepare);
    let second = make_vote([2u8; 32], 2, VoteKind::Prepare);

    assert_ne!(first.signing_bytes(), second.signing_bytes());
}

#[test]
fn signing_bytes_change_with_round() {
    let first = make_vote([1u8; 32], 2, VoteKind::Prepare);
    let second = make_vote([1u8; 32], 3, VoteKind::Prepare);

    assert_ne!(first.signing_bytes(), second.signing_bytes());
}

#[test]
fn signing_bytes_change_with_kind() {
    let first = make_vote([1u8; 32], 2, VoteKind::Prepare);
    let second = make_vote([1u8; 32], 2, VoteKind::Commit);

    assert_ne!(first.signing_bytes(), second.signing_bytes());
}

#[test]
fn signed_vote_verifies_with_matching_public_key() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);

    let vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };

    let signature = signing_key.sign(&vote.signing_bytes()).to_bytes().to_vec();

    let verification_result = SignedVote { vote, signature }.verify();
    assert!(verification_result.is_ok());
}

#[test]
fn modified_vote_payload_breaks_signature() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);

    let mut vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };

    let signature = signing_key.sign(&vote.signing_bytes()).to_bytes().to_vec();
    vote.round = 3;

    let verification_result = SignedVote { vote, signature }.verify();
    assert_eq!(
        verification_result,
        Err(VoteAuthenticationError::InvalidSignature)
    );
}

#[test]
fn authenticated_vote_binds_context_into_signature() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);

    let vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };

    let context = VoteAuthenticationContext {
        network_id: 2626,
        epoch: 4,
        validator_set_root: [5u8; 32],
        pq_attestation_root: [11u8; 32],
        signature_scheme: SIGNATURE_SCHEME_ED25519,
    };

    let mut authenticated_vote = AuthenticatedVote {
        vote,
        context,
        signature: Vec::new(),
        pq_public_key: None,
        pq_signature: None,
    };

    authenticated_vote.signature = signing_key
        .sign(&authenticated_vote.signing_bytes())
        .to_bytes()
        .to_vec();

    assert!(authenticated_vote.verify().is_ok());

    authenticated_vote.context.epoch = 5;

    assert_eq!(
        authenticated_vote.verify(),
        Err(VoteAuthenticationError::InvalidSignature)
    );
}

#[test]
fn authenticated_vote_rejects_unknown_signature_scheme() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);

    let vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };

    let mut authenticated_vote = AuthenticatedVote {
        vote,
        context: VoteAuthenticationContext {
            network_id: 2626,
            epoch: 4,
            validator_set_root: [5u8; 32],
            pq_attestation_root: [11u8; 32],
            signature_scheme: 999,
        },
        signature: Vec::new(),
        pq_public_key: None,
        pq_signature: None,
    };

    authenticated_vote.signature = signing_key
        .sign(&authenticated_vote.signing_bytes())
        .to_bytes()
        .to_vec();

    assert_eq!(
        authenticated_vote.verify(),
        Err(VoteAuthenticationError::UnknownSignatureScheme)
    );
}

#[test]
fn authenticated_vote_requires_pq_hardened_scheme_after_cutover_epoch() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);

    let vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };

    let mut authenticated_vote = AuthenticatedVote {
        vote,
        context: VoteAuthenticationContext {
            network_id: 2626,
            epoch: PQ_MANDATORY_START_EPOCH,
            validator_set_root: [5u8; 32],
            pq_attestation_root: [11u8; 32],
            signature_scheme: SIGNATURE_SCHEME_ED25519,
        },
        signature: Vec::new(),
        pq_public_key: None,
        pq_signature: None,
    };

    authenticated_vote.signature = signing_key
        .sign(&authenticated_vote.signing_bytes())
        .to_bytes()
        .to_vec();

    assert_eq!(
        authenticated_vote.verify(),
        Err(VoteAuthenticationError::PostQuantumPolicyRequired)
    );
}

#[test]
fn authenticated_vote_accepts_hybrid_scheme_after_cutover_epoch() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let pq_key_pair = generate_key_pair(random());

    let vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };

    let mut authenticated_vote = AuthenticatedVote {
        vote,
        context: VoteAuthenticationContext {
            network_id: 2626,
            epoch: PQ_MANDATORY_START_EPOCH,
            validator_set_root: [5u8; 32],
            pq_attestation_root: [11u8; 32],
            signature_scheme: SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3,
        },
        signature: Vec::new(),
        pq_public_key: Some(pq_key_pair.verification_key.as_ref().to_vec()),
        pq_signature: None,
    };

    let pq_signature = mldsa_sign(
        &pq_key_pair.signing_key,
        &authenticated_vote.signing_bytes(),
        b"",
        random(),
    )
    .expect("ML-DSA signing must succeed for a valid key pair and message");

    authenticated_vote.pq_signature = Some(pq_signature.as_ref().to_vec());
    authenticated_vote.signature = signing_key
        .sign(&authenticated_vote.signing_bytes())
        .to_bytes()
        .to_vec();

    assert!(authenticated_vote.verify().is_ok());
}

#[test]
fn authenticated_vote_rejects_hybrid_without_pq_public_key() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);

    let vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };

    let mut authenticated_vote = AuthenticatedVote {
        vote,
        context: VoteAuthenticationContext {
            network_id: 2626,
            epoch: PQ_MANDATORY_START_EPOCH,
            validator_set_root: [5u8; 32],
            pq_attestation_root: [11u8; 32],
            signature_scheme: SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3,
        },
        signature: Vec::new(),
        pq_public_key: None,
        pq_signature: None,
    };

    authenticated_vote.signature = signing_key
        .sign(&authenticated_vote.signing_bytes())
        .to_bytes()
        .to_vec();

    assert_eq!(
        authenticated_vote.verify(),
        Err(VoteAuthenticationError::MissingPostQuantumPublicKey)
    );
}

#[test]
fn authenticated_vote_accepts_post_quantum_only_scheme() {
    let pq_key_pair = generate_key_pair(random());

    let vote = Vote {
        voter: [7u8; 32],
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };

    let mut authenticated_vote = AuthenticatedVote {
        vote,
        context: VoteAuthenticationContext {
            network_id: 2626,
            epoch: PQ_MANDATORY_START_EPOCH,
            validator_set_root: [5u8; 32],
            pq_attestation_root: [11u8; 32],
            signature_scheme: SIGNATURE_SCHEME_DILITHIUM3,
        },
        signature: Vec::new(),
        pq_public_key: Some(pq_key_pair.verification_key.as_ref().to_vec()),
        pq_signature: None,
    };

    let pq_signature = mldsa_sign(
        &pq_key_pair.signing_key,
        &authenticated_vote.signing_bytes(),
        b"",
        random(),
    )
    .expect("ML-DSA signing must succeed for a valid key pair and message");

    authenticated_vote.signature = pq_signature.as_ref().to_vec();

    assert!(authenticated_vote.verify().is_ok());
}

#[test]
fn vote_authentication_context_maps_scheme_to_identity_profile() {
    let classical_context = VoteAuthenticationContext {
        network_id: 2626,
        epoch: 0,
        validator_set_root: [1u8; 32],
        pq_attestation_root: [2u8; 32],
        signature_scheme: SIGNATURE_SCHEME_ED25519,
    };

    let hybrid_context = VoteAuthenticationContext {
        signature_scheme: SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3,
        ..classical_context
    };

    let post_quantum_context = VoteAuthenticationContext {
        signature_scheme: SIGNATURE_SCHEME_DILITHIUM3,
        ..classical_context
    };

    assert!(matches!(
        classical_context
            .identity_profile()
            .expect("classical profile resolution must succeed"),
        ConsensusIdentityProfile::Classical
    ));

    assert!(matches!(
        hybrid_context
            .identity_profile()
            .expect("hybrid profile resolution must succeed"),
        ConsensusIdentityProfile::Hybrid
    ));

    assert!(matches!(
        post_quantum_context
            .identity_profile()
            .expect("post-quantum profile resolution must succeed"),
        ConsensusIdentityProfile::PostQuantum
    ));
}

#[test]
fn authenticated_vote_rejects_hybrid_profile_without_pq_attestation_root() {
    let signing_key = SigningKey::from_bytes(&[17u8; 32]);

    let vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [4u8; 32],
        height: 77,
        round: 2,
        kind: VoteKind::Prepare,
    };

    let mut authenticated_vote = AuthenticatedVote {
        vote,
        context: VoteAuthenticationContext {
            network_id: 2626,
            epoch: 0,
            validator_set_root: [5u8; 32],
            pq_attestation_root: [0u8; 32],
            signature_scheme: SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3,
        },
        signature: Vec::new(),
        pq_public_key: None,
        pq_signature: None,
    };

    authenticated_vote.signature = signing_key
        .sign(&authenticated_vote.signing_bytes())
        .to_bytes()
        .to_vec();

    let verification_error = authenticated_vote
        .verify()
        .expect_err("hybrid profile without PQ attestation root must fail closed");

    assert_eq!(
        verification_error,
        VoteAuthenticationError::MissingPostQuantumAttestationRoot
    );
}
