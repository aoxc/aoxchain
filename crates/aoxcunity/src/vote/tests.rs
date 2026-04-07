use crate::block::PQ_MANDATORY_START_EPOCH;
use ed25519_dalek::{Signer, SigningKey};
use pqcrypto_mldsa::mldsa65::{keypair as dilithium_keypair, sign as dilithium_sign};
use pqcrypto_traits::sign::{PublicKey as _, SignedMessage as _};

use super::{
    AuthenticatedVote, ConsensusIdentityProfile, SIGNATURE_SCHEME_DILITHIUM3,
    SIGNATURE_SCHEME_ED25519, SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3, SignedVote, Vote,
    VoteAuthenticationContext, VoteAuthenticationError, VoteKind,
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
    let a = make_vote([1u8; 32], 2, VoteKind::Prepare);
    let b = make_vote([1u8; 32], 2, VoteKind::Prepare);

    assert_eq!(a.signing_bytes(), b.signing_bytes());
}

#[test]
fn signing_bytes_change_with_block_hash() {
    let a = make_vote([1u8; 32], 2, VoteKind::Prepare);
    let b = make_vote([2u8; 32], 2, VoteKind::Prepare);

    assert_ne!(a.signing_bytes(), b.signing_bytes());
}

#[test]
fn signing_bytes_change_with_round() {
    let a = make_vote([1u8; 32], 2, VoteKind::Prepare);
    let b = make_vote([1u8; 32], 3, VoteKind::Prepare);

    assert_ne!(a.signing_bytes(), b.signing_bytes());
}

#[test]
fn signing_bytes_change_with_kind() {
    let a = make_vote([1u8; 32], 2, VoteKind::Prepare);
    let b = make_vote([1u8; 32], 2, VoteKind::Commit);

    assert_ne!(a.signing_bytes(), b.signing_bytes());
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

    let verified = SignedVote { vote, signature }.verify();
    assert!(verified.is_ok());
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

    let verified = SignedVote { vote, signature }.verify();
    assert_eq!(verified, Err(VoteAuthenticationError::InvalidSignature));
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
    let mut authenticated = AuthenticatedVote {
        vote,
        context,
        signature: Vec::new(),
        pq_public_key: None,
        pq_signature: None,
    };
    authenticated.signature = signing_key
        .sign(&authenticated.signing_bytes())
        .to_bytes()
        .to_vec();

    assert!(authenticated.verify().is_ok());

    authenticated.context.epoch = 5;
    assert_eq!(
        authenticated.verify(),
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
    let mut authenticated = AuthenticatedVote {
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
    authenticated.signature = signing_key
        .sign(&authenticated.signing_bytes())
        .to_bytes()
        .to_vec();

    assert_eq!(
        authenticated.verify(),
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
    let mut authenticated = AuthenticatedVote {
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
    authenticated.signature = signing_key
        .sign(&authenticated.signing_bytes())
        .to_bytes()
        .to_vec();

    assert_eq!(
        authenticated.verify(),
        Err(VoteAuthenticationError::PostQuantumPolicyRequired)
    );
}

#[test]
fn authenticated_vote_accepts_hybrid_scheme_after_cutover_epoch() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let (pq_public_key, pq_secret_key) = dilithium_keypair();
    let vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };
    let mut authenticated = AuthenticatedVote {
        vote,
        context: VoteAuthenticationContext {
            network_id: 2626,
            epoch: PQ_MANDATORY_START_EPOCH,
            validator_set_root: [5u8; 32],
            pq_attestation_root: [11u8; 32],
            signature_scheme: SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3,
        },
        signature: Vec::new(),
        pq_public_key: Some(pq_public_key.as_bytes().to_vec()),
        pq_signature: None,
    };
    let pq_signature = dilithium_sign(&authenticated.signing_bytes(), &pq_secret_key);
    authenticated.pq_signature = Some(pq_signature.as_bytes().to_vec());
    authenticated.signature = signing_key
        .sign(&authenticated.signing_bytes())
        .to_bytes()
        .to_vec();

    assert!(authenticated.verify().is_ok());
}

#[test]
fn authenticated_vote_rejects_hybrid_without_pq_signature() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let vote = Vote {
        voter: signing_key.verifying_key().to_bytes(),
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };
    let mut authenticated = AuthenticatedVote {
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
    authenticated.signature = signing_key
        .sign(&authenticated.signing_bytes())
        .to_bytes()
        .to_vec();

    assert_eq!(
        authenticated.verify(),
        Err(VoteAuthenticationError::MissingPostQuantumPublicKey)
    );
}

#[test]
fn authenticated_vote_accepts_dilithium_only_scheme() {
    let (pq_public_key, pq_secret_key) = dilithium_keypair();
    let vote = Vote {
        voter: [7u8; 32],
        block_hash: [1u8; 32],
        height: 9,
        round: 2,
        kind: VoteKind::Commit,
    };
    let mut authenticated = AuthenticatedVote {
        vote,
        context: VoteAuthenticationContext {
            network_id: 2626,
            epoch: PQ_MANDATORY_START_EPOCH,
            validator_set_root: [5u8; 32],
            pq_attestation_root: [11u8; 32],
            signature_scheme: SIGNATURE_SCHEME_DILITHIUM3,
        },
        signature: Vec::new(),
        pq_public_key: Some(pq_public_key.as_bytes().to_vec()),
        pq_signature: None,
    };

    let pq_signature = dilithium_sign(&authenticated.signing_bytes(), &pq_secret_key);
    authenticated.signature = pq_signature.as_bytes().to_vec();

    assert!(authenticated.verify().is_ok());
}
#[test]
fn vote_auth_context_maps_scheme_to_identity_profile() {
    let classical = VoteAuthenticationContext {
        network_id: 2626,
        epoch: 0,
        validator_set_root: [1u8; 32],
        pq_attestation_root: [2u8; 32],
        signature_scheme: SIGNATURE_SCHEME_ED25519,
    };
    let hybrid = VoteAuthenticationContext {
        signature_scheme: SIGNATURE_SCHEME_HYBRID_ED25519_DILITHIUM3,
        ..classical
    };
    let post_quantum = VoteAuthenticationContext {
        signature_scheme: SIGNATURE_SCHEME_DILITHIUM3,
        ..classical
    };

    assert!(matches!(
        classical.identity_profile().expect("classical profile"),
        ConsensusIdentityProfile::Classical
    ));
    assert!(matches!(
        hybrid.identity_profile().expect("hybrid profile"),
        ConsensusIdentityProfile::Hybrid
    ));
    assert!(matches!(
        post_quantum.identity_profile().expect("pq profile"),
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

    let mut authenticated = AuthenticatedVote {
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

    let signature = signing_key.sign(&authenticated.signing_bytes());
    authenticated.signature = signature.to_bytes().to_vec();

    let error = authenticated
        .verify()
        .expect_err("missing pq attestation root must fail");
    assert_eq!(
        error,
        VoteAuthenticationError::MissingPostQuantumAttestationRoot
    );
}
