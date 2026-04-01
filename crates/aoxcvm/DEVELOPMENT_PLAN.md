# DEVELOPMENT_PLAN.md

> Scope: `crates/aoxcvm`
> Model: AOXCLang Kernel

## Goal
Evolve AOXCLang from policy-ready language families to a production-grade, relay-capable interoperability kernel that is:
- language-first,
- proof-gated,
- deterministic under replay,
- operationally auditable.

## Phase 1 — Kernel contract freeze (short horizon)
1. Freeze `LanguageInteropProfile` schema and versioning policy.
2. Add strict validation for profile fields (ABI IDs, state-model IDs, proof requirements).
3. Define compatibility promises for `LanguageKind` additions and deprecations.

### Exit criteria
- policy schema has explicit semver rules,
- invalid profile metadata cannot pass construction-time checks,
- compatibility matrix is test-verified.

## Phase 2 — Language adapter interface hardening
1. Add adapter trait boundaries per language family.
2. Require adapter-declared finality proof type and replay domain format.
3. Add conformance tests that replay the same intent across adapters and compare deterministic outcomes.

### Exit criteria
- each language family has a typed adapter contract,
- all adapters declare proof and replay contracts,
- deterministic replay tests pass for all registered families.

## Phase 3 — Relay safety core
1. Introduce canonical relay intent envelope (domain-separated IDs, nonce lanes, source finality slot).
2. Add kernel replay ledger and idempotent settlement guards.
3. Enforce `no-proof => no-settlement` at kernel boundary.

### Exit criteria
- replay attempts are rejected deterministically,
- settlement requires explicit verified finality proof,
- all failure paths emit auditable evidence artifacts.

## Phase 4 — Adversarial and chaos validation
1. Build scenario matrix: reorg, delayed finality, equivocation, malformed proofs, duplicated intents.
2. Add cross-language fuzzing of relay envelopes.
3. Add deterministic gas/resource regression baselines per language family.

### Exit criteria
- adversarial matrix runs in CI,
- regressions block release,
- failure triage links to reproducible evidence.

## Phase 5 — Productionization gates
1. Wire release gate checks to policy + replay + proof invariants.
2. Add operator-grade observability: per-family relay metrics and proof-latency dashboards.
3. Publish runtime readiness scorecards by language family.

### Exit criteria
- release process blocks on kernel safety gates,
- runtime telemetry is available for all enabled families,
- readiness scorecard is generated automatically.

## Non-classic differentiators (must preserve)
- Language-first policy ownership in kernel (not chain-first sprawl).
- Deterministic replay guarantees as a first-class invariant.
- Proof-gated settlement for every relay path.
- Adapter extensibility without weakening kernel invariants.

## Recommended next implementation task
Start with **Phase 2 / Step 1**: define a typed `LanguageAdapter` trait and a conformance test harness that every current family must pass before execution is admitted.
