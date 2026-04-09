# AOXChain Architecture Reset Roadmap

This document replaces prior roadmap drafts with one execution plan that is explicit, testable, and release-gated.


## Planning Cadence (Date-Free)

This roadmap is intentionally milestone-driven rather than calendar-driven.

- phase closure is based on evidence-backed exit criteria,
- checklist completion requires attributable artifacts,
- promotion decisions are governance approvals, not timeline assumptions.

---

## Program Goal

Build a deterministic Layer-1 architecture that is:

- classical-secure in the current operating window,
- post-quantum-primary by default,
- migration-safe by protocol design,
- evidence-gated at every promotion boundary.

No roadmap item is considered complete without checklist evidence and reproducible commands.

---

## Hard Constraints (Non-Negotiable)

1. No claim of absolute or permanent cryptographic security.
2. No hidden fallback that bypasses policy-governed crypto profile selection.
3. No architecture-sensitive merge without synchronized spec + tests + ops docs.
4. No readiness declaration without retained evidence artifacts.

---

## Phase 0 — Governance Reset and Repository Cleanup

Objective: remove stale planning ambiguity and establish one canonical planning surface.

Deliverables:

- retire superseded planning text and conflicting draft language,
- align `README.md`, `READ.md`, `ARCHITECTURE.md`, `SCOPE.md`, `TESTING.md`, `SECURITY.md` with this roadmap,
- declare authoritative terminology for actor, scheme, profile, migration, recovery, and replay.

Checklist:

- [ ] all root governance docs reference this roadmap as canonical planning source
- [ ] deprecated planning sections removed or marked superseded
- [ ] terminology glossary stabilized and reused across docs
- [ ] scope and liability language preserved and explicit

Exit criteria:

- one roadmap source of truth exists,
- no root-level policy conflict remains.

---

## Phase 1 — Canonical PQ Authority Specification

Objective: define the protocol-authority model before implementation.

Deliverables:

- canonical authority model for `AccountObject`, `ValidatorObject`, `GovernanceAuthorityObject`,
- profile-aware `scheme_id` model and policy constraints,
- `policy_root` and `recovery_root` separation rules,
- replay-domain model and intent-domain boundaries,
- migration rulebook for key rotation, scheme migration, policy rotation, and recovery invocation.

Checklist:

- [ ] actor taxonomy finalized and versioned
- [ ] proof bundle schema finalized
- [ ] replay semantics defined by intent/domain class
- [ ] validation transcript format specified for deterministic auditability
- [ ] profile activation/deprecation state machine documented

Exit criteria:

- spec is complete enough for independent implementation by two teams with convergent outputs.

---

## Phase 2 — State Model and Serialization Contract

Objective: lock storage and wire contracts for PQ-scale key/signature material.

Deliverables:

- deterministic state schemas for authority objects and replay state,
- serialization and versioning policy,
- transaction envelope support for variable-size keys/signatures/proof bundles,
- migration-safe state transition contracts.

Checklist:

- [ ] state object schema defined with compatibility notes
- [ ] tx envelope supports variable-size proof payloads
- [ ] deterministic serialization test vectors published
- [ ] migration transaction types defined and validated
- [ ] storage compatibility and rollback behavior specified

Exit criteria:

- state and wire formats are fixed for Phase 3 kernel implementation.

---

## Phase 3 — Validation Kernel (PQ-Primary, Hybrid-Capable)

Objective: implement deterministic pre-execution validation with profile dispatch.

Deliverables:

- verifier registry with profile-aware dispatch,
- ML-DSA primary verification path,
- SLH-DSA fallback path where policy permits,
- hybrid validation mode for controlled migration windows,
- deterministic error taxonomy and fail-closed rejection behavior.

Checklist:

- [ ] verify-first boundary enforced (no execution before acceptance)
- [ ] profile mismatch rejection is deterministic
- [ ] downgrade attempts are rejected and metered
- [ ] scheme-specific cost accounting implemented
- [ ] adversarial validation corpus expanded for malformed proofs and replay abuse

Exit criteria:

- kernel validation behaves deterministically across replay, node class, and profile matrix.

---

## Phase 4 — Consensus, Validator, and Governance Integration

Objective: integrate authority kernel into consensus and operations-critical paths.

Deliverables:

- validator identity and consensus signing integration,
- governance execution authorization via policy model,
- emergency rotation and bounded recovery flows,
- node-to-node profile negotiation hardening and telemetry.

Checklist:

- [ ] validator auth uses profile-governed policy
- [ ] governance auth path fully policy-based
- [ ] emergency rotation playbook tested and artifacted
- [ ] handshake/profile mismatch telemetry emitted and reviewable
- [ ] fail-closed network admission enforced

Exit criteria:

- testnet candidate can run with policy-governed validator and governance authority model.

---

## Phase 5 — Testnet Hardening and Promotion Gate

Objective: operate testnet under production discipline and evidence retention.

Deliverables:

- deterministic testnet gate suite and no-skip policy,
- incident and rollback rehearsal artifacts,
- operator runbooks for profile migration,
- compatibility notes for integration surfaces.

Checklist:

- [ ] required testnet gates pass consecutively across defined window
- [ ] rollback rehearsal evidence retained
- [ ] migration drills run and published as artifacts
- [ ] unresolved critical findings tracked with owner and deadline
- [ ] release candidate includes explicit residual-risk statement

Exit criteria:

- promotion recommendation can be made with complete and reproducible evidence.

---

## Phase 6 — Mainnet Activation Decision (Governed)

Objective: permit activation only when all gate classes pass and residual risk is accepted explicitly.

Deliverables:

- activation dossier with command logs, artifact references, and review sign-offs,
- rollback-bounded activation plan,
- post-activation hardening backlog with scheduled revalidation cadence.

Checklist:

- [ ] activation prerequisites satisfied with evidence
- [ ] emergency rollback constraints tested and documented
- [ ] operations and engineering sign-off recorded
- [ ] known limitations declared with mitigation owner

Exit criteria:

- mainnet activation is a governed decision, not an implied default.

---

## Program-Level Checklist (Always-On)

- [ ] deterministic behavior is preserved for identical canonical inputs
- [ ] policy and recovery roots remain independent and auditable
- [ ] replay protection is domain-separated and migration-safe
- [ ] profile activation/deprecation is explicit and versioned
- [ ] documentation and implementation remain synchronized
- [ ] readiness evidence is retained and attributable to commits

---


## Ownership and Review

- Architecture and protocol ownership: kernel maintainers.
- Validation and readiness ownership: test and release maintainers.
- Operational safety ownership: operator tooling and incident response maintainers.

A phase is not complete unless all three ownership classes approve closure with evidence.
