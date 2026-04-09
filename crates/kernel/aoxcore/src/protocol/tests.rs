use super::core::canonicalize_payload_type;
use super::quantum::{
    QuantumAdmissionError, QuantumHandshakeError, QuantumKernelProfile, QuantumProfileError,
    SignatureScheme,
};
use super::{
    ChainFamily, FeeClass, MessageEnvelope, MessageEnvelopeError, ModuleId, SovereignRoot,
    canonical_chain_families, canonical_message_envelope_fields, canonical_modules,
    canonical_sovereign_roots,
};
use crate::block::{Capability, TargetOutpost};
use crate::identity::pq_keys;
use crate::transaction::quantum::QuantumTransaction;

fn sample_envelope() -> MessageEnvelope {
    MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        7,
        "bridge.commitment",
        [0x11; 32],
        Some([0x22; 32]),
        FeeClass::Priority,
        Some(42),
        [0x33; 16],
    )
    .expect("sample envelope must be valid")
}

#[test]
fn canonical_module_list_matches_six_domain_model() {
    assert_eq!(canonical_modules().len(), 6);
    assert_eq!(canonical_modules()[0], ModuleId::RelayCore);
    assert_eq!(canonical_modules()[5], ModuleId::Proof);
}

#[test]
fn canonical_chain_family_list_matches_five_target_model() {
    let families = canonical_chain_families();
    assert_eq!(families.len(), 5);
    assert!(families.contains(&ChainFamily::Evm));
    assert!(families.contains(&ChainFamily::Solana));
    assert!(families.contains(&ChainFamily::Utxo));
    assert!(families.contains(&ChainFamily::Ibc));
    assert!(families.contains(&ChainFamily::Object));
}

#[test]
fn canonical_message_field_count_remains_stable() {
    let fields = canonical_message_envelope_fields();
    assert_eq!(fields.len(), 11);
    assert_eq!(fields[0], "sourceModule");
    assert_eq!(fields[10], "replayProtectionTag");
}

#[test]
fn canonical_sovereign_roots_match_local_constitutional_model() {
    let roots = canonical_sovereign_roots();
    assert_eq!(roots.len(), 7);
    assert_eq!(roots[0], SovereignRoot::Identity);
    assert_eq!(roots[6], SovereignRoot::Treasury);
}

#[test]
fn constructor_canonicalizes_payload_type() {
    let envelope = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        1,
        "Bridge.Commitment",
        [0x11; 32],
        None,
        FeeClass::Standard,
        None,
        [0x44; 16],
    )
    .expect("constructor must canonicalize payload type");

    assert_eq!(envelope.payload_type(), "bridge.commitment");
}

#[test]
fn validation_rejects_same_source_and_destination_module() {
    let result = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Interop,
        ChainFamily::Evm,
        ChainFamily::Relay,
        1,
        "bridge.commitment",
        [0x11; 32],
        None,
        FeeClass::Standard,
        None,
        [0x44; 16],
    );

    assert_eq!(
        result.expect_err("must reject identical modules"),
        MessageEnvelopeError::SameSourceAndDestinationModule
    );
}

#[test]
fn validation_rejects_zero_nonce() {
    let result = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        0,
        "bridge.commitment",
        [0x11; 32],
        None,
        FeeClass::Standard,
        None,
        [0x44; 16],
    );

    assert_eq!(
        result.expect_err("must reject zero nonce"),
        MessageEnvelopeError::ZeroNonce
    );
}

#[test]
fn validation_rejects_zero_payload_hash() {
    let result = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        1,
        "bridge.commitment",
        [0u8; 32],
        None,
        FeeClass::Standard,
        None,
        [0x44; 16],
    );

    assert_eq!(
        result.expect_err("must reject zero payload hash"),
        MessageEnvelopeError::ZeroPayloadHash
    );
}

#[test]
fn validation_rejects_zero_proof_reference_when_present() {
    let result = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        1,
        "bridge.commitment",
        [0x11; 32],
        Some([0u8; 32]),
        FeeClass::Standard,
        None,
        [0x44; 16],
    );

    assert_eq!(
        result.expect_err("must reject zero proof reference"),
        MessageEnvelopeError::ZeroProofReference
    );
}

#[test]
fn validation_rejects_zero_replay_protection_tag() {
    let result = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        1,
        "bridge.commitment",
        [0x11; 32],
        None,
        FeeClass::Standard,
        None,
        [0u8; 16],
    );

    assert_eq!(
        result.expect_err("must reject zero replay tag"),
        MessageEnvelopeError::ZeroReplayProtectionTag
    );
}

#[test]
fn validation_rejects_zero_expiry_when_present() {
    let result = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        1,
        "bridge.commitment",
        [0x11; 32],
        None,
        FeeClass::Standard,
        Some(0),
        [0x44; 16],
    );

    assert_eq!(
        result.expect_err("must reject zero expiry"),
        MessageEnvelopeError::ExpiryMustBeNonZeroWhenPresent
    );
}

#[test]
fn canonicalizer_rejects_whitespace_and_empty_segments() {
    assert!(canonicalize_payload_type("bridge. commitment").is_err());
    assert!(canonicalize_payload_type("bridge..commitment").is_err());
    assert!(canonicalize_payload_type(".bridge").is_err());
    assert!(canonicalize_payload_type("bridge.").is_err());
}

#[test]
fn canonicalizer_rejects_non_ascii_and_invalid_symbols() {
    assert!(canonicalize_payload_type("köprü.commitment").is_err());
    assert!(canonicalize_payload_type("bridge/commitment").is_err());
    assert!(canonicalize_payload_type("bridge-commitment").is_err());
}

#[test]
fn message_envelope_hash_is_deterministic_and_sensitive() {
    let envelope = sample_envelope();
    let mut changed = sample_envelope();
    changed.nonce += 1;

    assert_eq!(envelope.hash(), sample_envelope().hash());
    assert_ne!(envelope.hash(), changed.hash());
}

#[test]
fn hash_distinguishes_optional_field_presence() {
    let without_proof = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        7,
        "bridge.commitment",
        [0x11; 32],
        None,
        FeeClass::Priority,
        Some(42),
        [0x33; 16],
    )
    .expect("envelope must be valid");

    let with_proof = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        7,
        "bridge.commitment",
        [0x11; 32],
        Some([0x22; 32]),
        FeeClass::Priority,
        Some(42),
        [0x33; 16],
    )
    .expect("envelope must be valid");

    assert_ne!(without_proof.hash(), with_proof.hash());
}

#[test]
fn strict_quantum_profile_is_valid_and_disables_legacy_support() {
    let profile = QuantumKernelProfile::strict_default();
    assert!(profile.validate().is_ok());
    assert!(!profile.legacy_signature_support);
    assert_eq!(profile.profile_version, 2);
    assert_eq!(profile.allowed_signatures, vec![SignatureScheme::MlDsa65]);
    assert_eq!(profile.fallback_signature, None);
}

#[test]
fn quantum_profile_rejects_default_signature_outside_allowed_set() {
    let mut profile = QuantumKernelProfile::strict_default();
    profile.default_signature = SignatureScheme::SphincsSha2128f;

    assert_eq!(
        profile
            .validate()
            .expect_err("default signature outside allowed set must fail"),
        QuantumProfileError::DefaultSignatureNotAllowed
    );
}

#[test]
fn quantum_profile_rejects_fallback_signature_outside_allowed_set() {
    let mut profile = QuantumKernelProfile::strict_default();
    profile.fallback_signature = Some(SignatureScheme::SphincsSha2128f);

    assert_eq!(
        profile
            .validate()
            .expect_err("fallback signature outside allowed set must fail"),
        QuantumProfileError::FallbackSignatureNotAllowed
    );
}

#[test]
fn quantum_profile_rejects_legacy_support_flag() {
    let mut profile = QuantumKernelProfile::strict_default();
    profile.legacy_signature_support = true;

    assert_eq!(
        profile
            .validate()
            .expect_err("legacy support must remain disabled"),
        QuantumProfileError::LegacySupportMustRemainDisabled
    );
}

#[test]
fn quantum_profile_upgrade_compatibility_requires_monotonic_version_and_default_support() {
    let current = QuantumKernelProfile::strict_default();

    let mut next = QuantumKernelProfile::strict_default();
    next.profile_version = 2;
    next.default_signature = SignatureScheme::SphincsSha2128f;
    next.allowed_signatures = vec![
        SignatureScheme::MlDsa65,
        SignatureScheme::SphincsSha2128f,
        SignatureScheme::Dilithium3,
    ];
    assert!(
        current
            .is_upgrade_compatible_with(&next)
            .expect("compatibility check must succeed")
    );

    let mut downgraded = next.clone();
    downgraded.profile_version = 0;
    assert_eq!(
        current
            .is_upgrade_compatible_with(&downgraded)
            .expect_err("invalid profile version must fail"),
        QuantumProfileError::InvalidProfileVersion
    );

    let mut incompatible = QuantumKernelProfile::strict_default();
    incompatible.profile_version = 2;
    incompatible.allowed_signatures = vec![SignatureScheme::SphincsSha2128f];
    incompatible.default_signature = SignatureScheme::SphincsSha2128f;
    assert!(
        !current
            .is_upgrade_compatible_with(&incompatible)
            .expect("compatibility check must return false")
    );
}

#[test]
fn is_expired_at_respects_optional_expiry() {
    let envelope = sample_envelope();
    assert!(!envelope.is_expired_at(42));
    assert!(envelope.is_expired_at(43));

    let no_expiry = MessageEnvelope::new(
        ModuleId::Interop,
        ModuleId::Proof,
        ChainFamily::Evm,
        ChainFamily::Relay,
        9,
        "bridge.commitment",
        [0x11; 32],
        None,
        FeeClass::Priority,
        None,
        [0x33; 16],
    )
    .expect("envelope must be valid");

    assert!(!no_expiry.is_expired_at(u64::MAX));
}

#[test]
fn handshake_negotiation_accepts_matching_or_higher_peer_profile() {
    let local = QuantumKernelProfile::strict_default();
    let mut peer = QuantumKernelProfile::strict_default();
    peer.profile_version = local.profile_version + 1;

    let result = local
        .negotiate_peer_profile(&peer)
        .expect("peer with compatible profile must be admitted");

    assert_eq!(result.negotiated_profile_version, local.profile_version);
    assert_eq!(result.selected_signature, local.default_signature);
}

#[test]
fn handshake_negotiation_rejects_profile_downgrade() {
    let local = QuantumKernelProfile::strict_default();
    let mut peer = QuantumKernelProfile::strict_default();
    peer.profile_version = local.profile_version - 1;

    assert_eq!(
        local
            .negotiate_peer_profile(&peer)
            .expect_err("downgraded peer profile must fail"),
        QuantumHandshakeError::ProfileDowngradeRejected
    );
}

#[test]
fn handshake_negotiation_rejects_peer_without_local_default_signature() {
    let local = QuantumKernelProfile::strict_default();
    let mut peer = QuantumKernelProfile::strict_default();
    peer.allowed_signatures = vec![SignatureScheme::SphincsSha2128f];
    peer.default_signature = SignatureScheme::SphincsSha2128f;

    assert_eq!(
        local
            .negotiate_peer_profile(&peer)
            .expect_err("peer missing local default signature support must fail"),
        QuantumHandshakeError::PeerDoesNotSupportLocalDefaultSignature
    );
}

#[test]
fn handshake_negotiation_rejects_invalid_peer_profile() {
    let local = QuantumKernelProfile::strict_default();
    let mut peer = QuantumKernelProfile::strict_default();
    peer.legacy_signature_support = true;

    assert_eq!(
        local
            .negotiate_peer_profile(&peer)
            .expect_err("invalid peer profile must fail"),
        QuantumHandshakeError::InvalidPeerProfile(
            QuantumProfileError::LegacySupportMustRemainDisabled
        )
    );
}

#[test]
fn strict_profile_admits_valid_quantum_transaction() {
    let profile = QuantumKernelProfile::strict_default();
    let (pk, sk) = pq_keys::generate_keypair();
    let payload = vec![7, 8, 9];
    let nonce = 9;
    let message = QuantumTransaction::canonical_signing_message(
        nonce,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        &payload,
    )
    .expect("message must be buildable");

    let signed_payload = pq_keys::sign_message_domain_separated(&message, &sk);
    let tx = QuantumTransaction::new(
        pq_keys::serialize_public_key(&pk),
        nonce,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        payload,
        signed_payload,
    )
    .expect("transaction must be valid");

    assert!(profile.admit_quantum_transaction(&tx).is_ok());
}

#[test]
fn profile_rejects_invalid_quantum_transaction_during_admission() {
    let profile = QuantumKernelProfile::strict_default();
    let (pk, sk) = pq_keys::generate_keypair();
    let payload = vec![1, 2, 3];
    let nonce = 3;
    let message = QuantumTransaction::canonical_signing_message(
        nonce,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        &payload,
    )
    .expect("message must be buildable");
    let mut signed_payload = pq_keys::sign_message_domain_separated(&message, &sk);
    signed_payload.push(0);

    let tx = QuantumTransaction {
        sender_public_key: pq_keys::serialize_public_key(&pk),
        nonce,
        capability: Capability::UserSigned,
        target: TargetOutpost::EthMainnetGateway,
        payload,
        signed_payload,
    };

    assert_eq!(
        profile
            .admit_quantum_transaction(&tx)
            .expect_err("tampered signature must be rejected"),
        QuantumAdmissionError::InvalidTransactionPayload
    );
}
