# AOXC Node Identity v0.1.0-alpha

## 1. Objective

This document defines the recommended next-step identity architecture for AOXChain nodes before consensus-block and certificate-surface expansion.

The core design goal is to move from standalone identity primitives toward a canonical, role-separated, hybrid-signed node identity bundle that can support:

- deterministic node bootstrap,
- post-quantum readiness,
- cleaner validator and certificate semantics,
- explicit key rotation,
- and future multi-node transport and consensus hardening.

## 2. Why identity comes before block redesign

Current AOXChain already contains strong identity building blocks:

- deterministic seed and derivation flow in `aoxcore::identity::key_engine`,
- post-quantum signing helpers in `aoxcore::identity::pq_keys`,
- encrypted secret persistence in `aoxcore::identity::keyfile`,
- and actor passport metadata in `aoxcore::identity::passport`.

However, these surfaces are not yet unified into a single canonical node identity model.

That matters because future block and certificate formats will need stable answers for questions such as:

- Which public key identifies a validator?
- Which key signs consensus votes?
- Which key authenticates transport sessions?
- How does a node rotate keys without breaking validator identity continuity?
- What cryptographic profile does the network consider valid?

For this reason, AOXChain should freeze node identity semantics before expanding block-header or quorum-certificate semantics.

## 3. Current baseline in the repository

### 3.1 Deterministic key derivation

`KeyEngine` already provides deterministic seed-based derivation over canonical paths and is a suitable root for multi-role key separation.

### 3.2 Post-quantum primitive availability

`pq_keys` already exposes Dilithium3 generation, signing, verification, serialization, and fingerprinting. This is a strong base for quantum-readiness, but it currently behaves as a cryptographic utility rather than a fully modeled node-role key.

### 3.3 Secret-at-rest protection

`keyfile` already provides an encrypted, versioned Argon2id + AES-256-GCM envelope suitable for node secret persistence.

### 3.4 Identity artifact shape

`Passport` currently carries actor metadata plus a certificate string and validity timestamps. It is useful as a lightweight identity artifact, but it is not sufficient as the canonical node-identity bundle for long-term validator, transport, and rotation workflows.

## 4. Design principles

The AOXC node identity design should adopt the following principles.

### 4.1 Role separation

A node must not rely on a single long-lived key for every function. Consensus, transport, operator access, and recovery should be isolated.

### 4.2 Hybrid cryptography

AOXChain should not become PQ-only immediately. Instead, it should support a hybrid profile where a classical key and a post-quantum key coexist in the canonical identity bundle.

This provides:

- operational compatibility for current tooling and integrations,
- near-term deployability,
- and a cleaner path toward future PQ-first policy.

### 4.3 Deterministic derivation with explicit domains

All operational keys may derive from one protected root seed, but every derived key must have a domain-separated purpose.

### 4.4 Rotation without identity collapse

Key rotation must be a first-class feature. The system should support replacing operational keys while preserving the continuity of the node identity bundle and certificate chain.

### 4.5 Certificate-ready modeling

The identity bundle must be structured so future node certificates, validator attestation, revocation records, and quorum artifacts can reference it directly.

## 5. Recommended key taxonomy

The following key classes are recommended for AOXChain node architecture.

### 5.1 Root seed

A protected root seed is used only as derivation input. It should never be used directly for network signing.

Recommended derivation domains:

- `node_identity`
- `consensus`
- `transport`
- `operator`
- `recovery`
- `pq_attestation`

### 5.2 NodeIdentityKey

Purpose:

- long-lived node identity anchor,
- identity bundle continuity,
- certificate subject binding.

### 5.3 ConsensusVoteKey

Purpose:

- block proposal admission,
- vote signing,
- quorum and finality participation.

This key may rotate more frequently than the long-lived identity anchor.

### 5.4 TransportKey

Purpose:

- peer authentication,
- handshake binding,
- secure transport session identity.

### 5.5 OperatorKey

Purpose:

- privileged administrative actions,
- bootstrap approval flows,
- local operational control.

This key should be usable independently from consensus signing.

### 5.6 PqAttestationKey

Purpose:

- post-quantum identity proof,
- hybrid certificate signing support,
- future PQ-first migration.

### 5.7 RecoveryKey

Purpose:

- offline emergency recovery,
- rotation authorization,
- compromise response workflow.

This key should be held offline or in a cold-control path whenever possible.

## 6. Recommended cryptographic posture

## 6.1 Hybrid public-key set

AOXChain should define a canonical `HybridPublicKeySet` that can include at least:

- an Ed25519 public key for near-term interoperability,
- a Dilithium public key for post-quantum readiness.

The exact classical algorithm can remain profile-driven, but the bundle should make room for both classical and PQ material.

### 6.2 Crypto profile

The identity bundle should carry a machine-readable `CryptoProfile`, for example:

- `classic-ed25519`
- `hybrid-ed25519-dilithium3`
- `pq-dilithium3-preview`

This avoids ambiguous assumptions inside consensus, transport, or certificate verification code.

## 7. Canonical identity artifact

AOXChain should introduce a canonical identity artifact named `NodeIdentityBundleV1`.

Recommended fields:

- `bundle_version`
- `node_id`
- `actor_id`
- `role`
- `zone`
- `network_profile`
- `consensus_profile`
- `crypto_profile`
- `identity_public_keys`
- `transport_public_key`
- `operator_public_key`
- `validator_set_hash` (optional)
- `certificate_chain`
- `revocation_policy`
- `rotation_counter`
- `created_at`
- `expires_at`

### 7.1 Canonical identity semantics

The node identifier should be derived from a stable commitment to the canonical bundle content rather than from an incidental single-key encoding.

This gives AOXChain a stable identity anchor even when operational keys rotate under controlled policy.

## 8. Certificate model recommendation

AOXChain should add a structured node certificate surface, tentatively named `NodeCertificateV1`.

Recommended fields:

- `certificate_version`
- `subject_node_id`
- `subject_actor_id`
- `identity_public_keys`
- `transport_public_key`
- `capabilities`
- `roles`
- `zone`
- `valid_from`
- `valid_to`
- `issuer_id`
- `issuer_public_keys`
- `signature_suite`
- `signature_payload_hash`
- `issuer_signature`
- `issuer_pq_signature` (optional in hybrid mode)

This certificate model is intended to replace free-form certificate embedding with a typed, auditable structure.

## 9. Relationship to the existing passport

The current `Passport` type should remain usable as a lightweight runtime-facing artifact, but it should eventually become either:

1. a derived projection of `NodeIdentityBundleV1` plus `NodeCertificateV1`, or
2. a compatibility wrapper around the canonical identity artifacts.

In other words, passport data should be downstream of canonical identity, not the primary source of it.

## 10. Keyfile evolution recommendation

The current encrypted envelope is a strong primitive, but the next step should be a role-aware bundle format such as `NodeKeyfileBundleV1`.

Recommended contents:

- encrypted secret material for each role,
- declared key roles,
- crypto profile metadata,
- creation and rotation metadata,
- checksum or commitment fields for bundle integrity,
- optional certificate and bundle references.

This would let AOXChain persist node secrets as one coherent artifact instead of several unrelated plaintext-role assumptions.

## 11. Suggested implementation order

### Phase 0 — terminology freeze

Define and approve the canonical identity vocabulary:

- root seed,
- node identity key,
- consensus vote key,
- transport key,
- operator key,
- PQ attestation key,
- recovery key.

### Phase 1 — canonical Rust types

Add strongly typed identity models under `aoxcore::identity`, beginning with:

- `NodeKeyRole`
- `CryptoProfile`
- `HybridPublicKeySet`
- `NodeIdentityBundleV1`

### Phase 2 — certificate typing

Add `NodeCertificateV1` and validation helpers.

### Phase 3 — keyfile bundling

Add `NodeKeyfileBundleV1` on top of the existing keyfile envelope.

### Phase 4 — CLI integration

Update bootstrap and key-management flows so `aoxcmd` and `aoxckit` emit, inspect, and validate the canonical node identity artifacts.

### Phase 5 — consensus and transport binding

After the identity model is stable, update consensus headers, validator membership references, transport handshake metadata, and rotation rules to commit to the new identity surfaces.

#### Current implementation note

The repository now has an initial implementation of this phase:

- a typed `NodeKeyBundleV1` exists in `aoxcore::identity`,
- `aoxcmd` bootstrap persists the bundle,
- validator bootstrap can expose the canonical consensus public key,
- and block validation can enforce that a producer matches the bundle's consensus key.

Transport admission, revocation distribution, recovery authorization, and validator-set membership binding still need deeper runtime integration.

## 12. Risks and constraints

### 12.1 Payload size

Dilithium public keys and signatures are larger than classical equivalents. This affects:

- certificate size,
- handshake cost,
- storage cost,
- and network message overhead.

### 12.2 Premature PQ-only migration

A PQ-only switch may be too disruptive for current tooling and ecosystem integration. Hybrid deployment is the safer intermediate step.

### 12.3 Rotation complexity

Key rotation is not only a cryptographic problem. It also affects validator membership, certificate validity, audit history, and operational recovery.

## 13. Immediate outcome expected from this proposal

If AOXChain adopts this design direction, the project gains a cleaner foundation for:

- validator identity continuity,
- future quorum certificate semantics,
- transport authentication,
- certificate issuance and revocation,
- and post-quantum migration planning.

## 13.1 Core-first implementation note

The preferred implementation model is **core-first**: identity validation, certificate verification, signer authorization, revocation, and key-rotation continuity should ultimately be enforced inside `aoxcore` and the consensus/security kernel rather than delegated to CLI or application-layer policy. Operator tools should consume these rules, not redefine them.

## 14. Summary recommendation

AOXChain should prioritize node identity architecture before block-structure expansion.

The recommended target is a role-separated, hybrid-signed, certificate-ready identity model centered on:

- `NodeIdentityBundleV1`
- `NodeCertificateV1`
- `NodeKeyfileBundleV1`
- `CryptoProfile`
- `NodeKeyRole`
- `HybridPublicKeySet`

This sequence uses the repository’s existing identity primitives as a foundation while providing a safer path toward production consensus and long-term quantum readiness.
