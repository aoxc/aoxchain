# AOXChain Advanced Key System (Wallet + Node)

This document defines the repository-level key architecture for wallets, validators, governance actors, and node operations.

## 1) Objectives

The key system must provide:

- quantum-resistant primary authorization,
- strict domain separation between wallet, consensus, governance, and recovery keys,
- deterministic and auditable key lifecycle controls,
- migration-safe cryptographic agility through policy-governed profiles.

## 2) Key Domains (Non-Overlapping)

AOXChain uses separate key domains. A key from one domain must never be reused in another domain.

1. **Wallet Transaction Key Domain**
   - signs user transaction intents.
2. **Validator Consensus Key Domain**
   - signs consensus and validator-auth messages.
3. **Governance Authority Key Domain**
   - signs policy, profile, and protocol-governance actions.
4. **Recovery Authority Key Domain**
   - signs emergency recovery and controlled break-glass flows.
5. **Node Transport Key Domain**
   - secures P2P session establishment and authenticated node channels.

## 3) Algorithm Profiles

Algorithm usage is controlled by on-chain policy profiles.

- **Primary signature profile:** ML-DSA (policy versioned).
- **Secondary/hybrid profile:** SLH-DSA (only when profile policy allows).
- **Transport/session profile:** policy-approved KEM/signature pairing for node channels.

No hidden fallback is allowed. Unsupported or deprecated profile usage must fail closed.

## 4) Wallet Key Construction Model

Wallets are built as policy-aware authority objects, not single static keypairs.

### Required structure

- `wallet_id`: canonical identifier.
- `scheme_id`: active signature scheme/profile.
- `policy_root`: authorization and threshold policy commitment.
- `recovery_root`: independent recovery authority commitment.
- `keyset_version`: monotonic version.
- `replay_domain`: domain-separated replay protection namespace.

### Required properties

- deterministic encoding of authority payload,
- explicit activation/deprecation states per scheme,
- bounded hybrid windows for migrations,
- auditable transition transcripts for key rotation and recovery.

## 5) Node Key Architecture

Node keys are split by responsibility:

- **Consensus signing key** (consensus-critical, policy-governed),
- **P2P identity key** (network identity and authenticated routing),
- **RPC operator key** (administrative and privileged control plane).

Operational rule: compromise of one node key class must not automatically authorize actions in another class.

## 6) Key Lifecycle Requirements

All domains follow governed lifecycle controls:

1. key generation using approved entropy source,
2. registration/activation through policy-validated transaction,
3. rotation with replay-safe intent and version bump,
4. deprecation with explicit policy state transition,
5. emergency recovery via independent recovery roots.

Silent key replacement is prohibited.

## 7) Address and Identity Binding

Addresses and actor identities must be bound to canonical, deterministic encodings of profile-tagged public key material.

Binding rules must include:

- scheme/profile identifier,
- domain identifier,
- versioning metadata,
- anti-malleability normalization.

## 8) Threat Controls

Mandatory controls:

- domain-separated signing contexts,
- downgrade rejection for deprecated profiles,
- replay-domain enforcement across migration/recovery intents,
- threshold/multi-party policy controls for high-impact authorities,
- explicit metering for large PQ signatures/proofs.

## 9) Validation and Evidence

Any change to wallet/node key behavior must include:

- deterministic test vectors for key encoding and signature verification,
- adversarial tests for replay, downgrade, malformed proof, and cross-domain misuse,
- migration and rollback drills with retained artifacts tied to commit SHA.

Without these artifacts, readiness status is `NOT_READY`.
