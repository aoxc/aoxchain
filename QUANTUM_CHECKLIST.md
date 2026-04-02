# QUANTUM_CHECKLIST.md

Execution checklist for the AOXChain quantum-resilience transformation program.

Use this checklist as a release-gating surface. Items should only be marked complete when backed by reproducible evidence artifacts and tests.

## 1) Program Governance and Threat Modeling

- [ ] Define and approve quantum threat model document.
- [ ] Assign ownership for each cryptographic surface (consensus, networking, wallet, RPC, storage).
- [ ] Publish migration risk matrix with severity/likelihood scoring.
- [ ] Define compatibility policy for profile versioning and deprecation windows.
- [ ] Define emergency rollback authority and constraints.

## 2) Cryptographic Surface Inventory

- [ ] Inventory all signatures, key exchange primitives, hash functions, and randomness sources.
- [ ] Map each primitive to concrete code paths and configuration controls.
- [ ] Mark consensus-critical vs non-consensus cryptographic paths.
- [ ] Document external dependencies and upstream security posture.
- [ ] Add artifact export (`artifacts/crypto-surface-inventory.json`).

## 3) Block Structure Upgrades (First Implementation Priority)

- [ ] Add block/header version fields for crypto profile declaration.
- [ ] Add explicit algorithm IDs to witness structures.
- [ ] Add canonical serialization vectors for all new structures.
- [ ] Enforce unknown-profile reject behavior (fail closed).
- [ ] Add replay-domain separation tags for profile variants.
- [ ] Add block size and witness size guardrails for PQ payloads.
- [ ] Add mixed-profile chain simulation tests.
- [ ] Add backward-compatibility tests for legacy decode windows.

## 4) Transaction and Account Layer

- [ ] Introduce hybrid signature container format (classical + PQ).
- [ ] Support deterministic key-type discovery in account metadata.
- [ ] Define account migration flow from legacy keys to hybrid/PQ keys.
- [ ] Add anti-replay controls across profile boundaries.
- [ ] Add mempool validation rules for profile-specific witness constraints.

## 5) Deterministic VM (Special-Purpose Hardening)

- [ ] Define deterministic cryptographic syscall API.
- [ ] Add profile-gated syscall dispatch controlled by finalized on-chain state.
- [ ] Update gas/meter schedule for PQ verification costs.
- [ ] Add bytecode admission checks: opcode allowlist and static safety rules.
- [ ] Add memory, stack, and recursion limits with deterministic failures.
- [ ] Add regression tests for deterministic outputs across hardware targets.
- [ ] Add adversarial tests for syscall abuse and resource exhaustion.

## 6) P2P Networking and Handshake

- [ ] Implement hybrid key establishment for node-to-node sessions.
- [ ] Add downgrade-protection checks and telemetry.
- [ ] Add key rotation and session expiration policy.
- [ ] Add mixed-version compatibility matrix tests.
- [ ] Add operator alerts for handshake negotiation anomalies.

## 7) Consensus and Governance Controls

- [ ] Define on-chain `CryptoProfile` object schema.
- [ ] Define activation and deprecation state machine.
- [ ] Require validator conformance evidence for active profiles.
- [ ] Add governance simulation tests for profile transitions.
- [ ] Add deterministic emergency rollback path.

## 8) Data, Storage, and Sync Safety

- [ ] Estimate storage amplification under PQ witness growth.
- [ ] Add pruning/archive strategy for profile-tagged artifacts.
- [ ] Ensure snapshot and state-sync compatibility across profiles.
- [ ] Add corruption and malformed-witness recovery tests.

## 9) Wallet, SDK, and API Surfaces

- [ ] Add key generation and signing support for selected PQ/hybrid modes.
- [ ] Expose profile metadata in RPC and SDK responses.
- [ ] Add API versioning notes for breaking and non-breaking changes.
- [ ] Publish migration guides for application developers and operators.

## 10) AI-Assisted Security Layer (Non-Consensus)

- [ ] Define model scope: anomaly detection only, no consensus authority.
- [ ] Add dataset lineage and model version recording.
- [ ] Define alert thresholds and escalation paths.
- [ ] Add explainability fields for every generated alert.
- [ ] Validate false-positive and false-negative bounds.

## 11) Verification and Audit

- [ ] Add fuzz targets for profile-tagged serialization and parsing.
- [ ] Add differential tests across profile combinations.
- [ ] Add long-horizon replay and reorg adversarial scenarios.
- [ ] Commission independent cryptography and protocol audits.
- [ ] Integrate audit findings with closure criteria and remediation tracking.

## 12) Release and Production Closure

- [ ] Publish PQ readiness scorecard in `artifacts/`.
- [ ] Publish known residual risks and unsupported scenarios.
- [ ] Add release gating checks that fail on profile non-conformance.
- [ ] Run staged devnet/testnet/mainnet rollout rehearsals.
- [ ] Sign and archive production-closure evidence bundle.

## Recommended Completion Rules

An item may be marked complete only when:
1. Code and documentation are merged in the same change stream.
2. Test evidence exists and is reproducible by command.
3. Operational impact and rollback notes are documented.
4. Compatibility impact is explicitly stated.

## Suggested Artifact Set

- `artifacts/crypto-surface-inventory.json`
- `artifacts/pq-profile-compatibility-matrix.json`
- `artifacts/pq-performance-budget-report.json`
- `artifacts/pq-downgrade-attempt-telemetry.json`
- `artifacts/pq-readiness-scorecard.json`


## 13) Ready-to-Start Sprint Breakdown

### Sprint A (Block structure first)

- [ ] Define `crypto_profile_id` and `witness_schema_id` semantics and reserved ranges.
- [ ] Produce at least 20 canonical block/witness vectors (valid + invalid).
- [ ] Add replay-domain test fixtures for all active profiles.
- [ ] Add validator memory-pressure tests for max witness bounds.

### Sprint B (Special deterministic VM)

- [ ] Implement `verify_sig` syscall with profile gating and deterministic errors.
- [ ] Implement cryptographic gas schedule v2 with p50/p99 benchmark report.
- [ ] Add adversarial syscall flood tests under constrained resources.
- [ ] Publish VM migration notes for contract developers.

### Sprint C (Network + governance)

- [ ] Implement hybrid handshake negotiation with downgrade lockout.
- [ ] Add mixed-version interoperability matrix for at least three node versions.
- [ ] Add governance simulation for activate/deprecate/rollback flow.
- [ ] Run staged rollout rehearsal and publish signed evidence bundle.

## 14) Done Definition for Quantum Program Claims

Before any claim such as "quantum-ready" is published, all of the following must be true:

- [ ] Threat model document is approved and current.
- [ ] Active crypto profile set and deprecation schedule are on-chain and auditable.
- [ ] Determinism, replay, and downgrade tests are green in CI and archived in artifacts.
- [ ] External audit findings are resolved or formally accepted with mitigation plans.
- [ ] Public status statement includes residual risks and unsupported scenarios.
