# AOXChain Production-Grade Testnet and PQ-Resilient Mainnet Roadmap

This roadmap defines the active execution path for AOXChain.

Guiding statement:

> **Deterministic by design, crypto-agile by governance, proven by evidence.**

## Strategic Target

AOXChain follows a strict promotion model:

1. establish and operate a production-grade testnet,
2. activate mainnet only after measurable readiness gates are met,
3. maintain continuous hardening across protocol, networking, kernel, and operations.

## Phase 1 — Documentation and Control Surface Reset

Objective: establish one clear source of truth for direction and readiness.

Deliverables:
- consolidated top-level documentation (`README.md`, `READ.md`, `ROADMAP.md`),
- retired legacy roadmap/checklist documents,
- explicit language eliminating unverifiable security claims.

Exit criteria:
- repository root docs are aligned and non-duplicative,
- roadmap and readiness language is consistent across governance documents.

## Phase 2 — Production-Grade Testnet Baseline

Objective: run testnet with production discipline.

Scope:
- deterministic build/test/quality/audit gate enforcement,
- validator/sentry/RPC role separation and admission controls,
- host and kernel hardening baseline for node and gateway classes,
- retained evidence for gate outcomes and operational drills.

Exit criteria:
- testnet gates pass consistently,
- rollback and incident drill artifacts are reproducible,
- compatibility-sensitive changes include migration notes.

## Phase 3 — Cryptographic Agility Activation (Hybrid)

Objective: enforce versioned cryptographic profile controls without consensus ambiguity.

Scope:
- profile-tagged consensus-visible structures,
- hybrid transition controls where required,
- strict fail-closed behavior for unsupported cryptographic profiles,
- downgrade detection and rejection telemetry.

Exit criteria:
- mixed-profile simulations converge deterministically,
- downgrade paths are rejected and measured,
- profile transition evidence is retained.

## Quantum-First Fast-Track (Governed Hard Cutover Option)

Objective: define the only acceptable path for "direct quantum" activation requests without weakening determinism, safety, or operator recoverability.

This track allows a hard cutover posture only when the following controls are all satisfied:

1. **Kernel-first profile authority:** post-quantum profile selection and rejection rules are enforced in kernel-owned consensus-visible types before any network or VM activation.
2. **No hidden classical fallback:** any classical-only acceptance path is either explicitly dual-profile by policy window or removed.
3. **State and signature migration plan:** existing validator/operator key material and persisted consensus artifacts have deterministic migration or controlled reset rules.
4. **Network handshake enforcement:** peer negotiation fails closed on profile mismatch and downgrade attempts are surfaced in telemetry.
5. **Rollback boundedness:** rollback is explicit, rehearsed, and version-bounded; "implicit rollback by config drift" is prohibited.
6. **Cutover evidence package:** pre-cutover simulation matrix, mixed-version rejection proofs, and incident drills are retained under `artifacts/`.

Hard cutover is therefore permitted only as a governed protocol migration, not as an ad-hoc replacement of running architecture.

## Phase 4 — PQ-Resilient Mainnet Readiness Gate

Objective: permit mainnet activation only with verified operational and cryptographic posture.

Gate requirements:
- deterministic validation evidence across required matrices,
- clear activation/deprecation/rollback governance controls,
- operator runbooks validated by rehearsal,
- residual risk statement updated with current assumptions.

Exit criteria:
- readiness package approved by engineering and operations review,
- promotion decision recorded with reproducible evidence references.

## Phase 5 — Continuous Hardening Program

Objective: sustain resilience after activation.

Scope:
- continuous kernel and network hardening,
- runtime control refinements under measured risk,
- periodic cryptographic assumption review,
- regular adversarial validation and recovery drills.

Exit criteria:
- hardening backlog is continuously prioritized by measured impact,
- periodic revalidation artifacts remain current.

## Product Surface Track (Parallel to Phase Gates)

The following capability track defines operator and ecosystem surfaces expected to mature alongside phase execution:

### Near-term
- repository-level API reference hardening and schema stabilization,
- full-node onboarding and deterministic bootstrap documentation,
- CI evidence visibility and release-gate artifact publication consistency.

### Mid-term
- operator wallet ergonomics (CLI-first, deterministic key workflow preservation),
- explorer-oriented indexed query surfaces for block/tx/address observability,
- stronger integration-network simulations with adversarial topology permutations.

### Long-term
- contract platform maturity under AOX VM governance (policy-constrained, deterministic),
- broader compatibility shims where they do not violate kernel trust boundaries,
- formalized ecosystem governance hooks for operator and contributor decision flow.

## Non-Negotiable Program Rules

1. No unverifiable “absolute security” claims.
2. No hidden downgrade fallback paths.
3. No readiness claim without evidence.
4. No compatibility-sensitive change without explicit documentation and rollback context.
5. No "direct quantum" production activation without kernel-level policy enforcement, deterministic migration controls, and retained cutover evidence.
