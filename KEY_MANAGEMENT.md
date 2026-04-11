# AOXChain Key Management Architecture

This document defines repository-level key architecture for wallets, validators, governance actors, recovery authorities, and node operations.

## 1. Objectives


The key system must provide:

- quantum-resistant primary authorization,
- strict domain separation across authority classes,
- deterministic and auditable key lifecycle controls,
- migration-safe cryptographic agility through policy-governed profiles.

## 2. Key Domains (Non-Overlapping)

1. **Wallet Transaction Domain**: signs user transaction intents.
2. **Validator Consensus Domain**: signs consensus and validator-auth messages.
3. **Governance Authority Domain**: signs policy and protocol-governance actions.
4. **Recovery Authority Domain**: signs emergency recovery and break-glass flows.
5. **Node Transport Domain**: secures P2P session establishment and node channels.

A key from one domain must never be reused in another.

## 3. Algorithm Profile Governance

Algorithm usage is policy controlled:

- primary signature profile: ML-DSA,
- secondary/hybrid profile: SLH-DSA (only where policy permits),
- transport/session profile: policy-approved KEM/signature pairing.

Unsupported or deprecated profile usage must fail closed.

## 4. Wallet Key Construction Model

Wallets are policy-aware authority objects, not static keypairs.

Required fields:

- `wallet_id`,
- `scheme_id`,
- `policy_root`,
- `recovery_root`,
- `keyset_version`,
- `replay_domain`.

Required properties:

- deterministic authority encoding,
- explicit activation/deprecation states,
- bounded hybrid migration windows,
- auditable transcripts for rotation and recovery.

## 5. Node Key Architecture

Node keys are responsibility separated:

- consensus signing key,
- P2P identity/session key,
- RPC/operator control-plane key.

Compromise of one class must not automatically authorize another class.

## 6. Key Lifecycle Requirements

All key domains require governed lifecycle transitions:

1. generation with approved entropy,
2. activation through policy-validated transaction,
3. rotation with replay-safe intent and version bump,
4. deprecation through explicit policy transition,
5. emergency recovery through independent recovery roots.

Silent key replacement is prohibited.

## 7. Address and Identity Binding

Addresses and actor identities must bind canonical, profile-tagged public keys with:

- scheme/profile identifier,
- domain identifier,
- version metadata,
- anti-malleability normalization.

## 8. Threat Controls

Mandatory controls:

- domain-separated signing contexts,
- downgrade rejection for deprecated profiles,
- replay controls across migration/recovery intents,
- threshold controls for high-impact authorities,
- explicit metering for large PQ signatures/proofs.

## 9. Validation and Evidence

Any key-behavior change must include:

- deterministic encoding/signature vectors,
- adversarial replay/downgrade/malformed-proof tests,
- migration and rollback drills with artifacts linked to commit SHA.

Without this evidence, readiness state is `NOT_READY`.
