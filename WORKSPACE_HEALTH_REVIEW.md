# AOXChain Workspace and Runtime Closure Plan (2026-04-12)

## Purpose

This document provides a single operational baseline for driving AOXChain toward an auditable, production-grade runtime posture across **dev**, **testnet**, and **mainnet**.

> Engineering note: an absolute "bug-free" guarantee is not technically provable. The practical target is a fail-closed, evidence-backed, continuously validated operating model.

---

## 1) Current Verified Baseline

### 1.1 Workspace integrity

- The workspace currently declares 19 members.
- Local verification confirms that all 19 member manifests exist on disk and are tracked in the workspace.
- No orphan crate directory or missing workspace member path was observed.

### 1.2 Build/compile surface

- `cargo check --workspace --exclude aoxchub --all-targets` succeeded.
- `cargo check -p aoxchub --all-targets` succeeded.
- At check-level validation, the full workspace compiles successfully.

### 1.3 Runtime model

- Runtime support is present.
- The operator surface follows a single-runtime path contract.
- Structural layout is fixed; runtime source material is selected in a controlled manner by network/profile inputs.

---

## 2) Runtime Contract: Fixed vs Configurable

### 2.1 Structural contract (fixed)

The following runtime surfaces are treated as structurally fixed:

- single runtime root,
- canonical runtime directories (`identity`, `state`, `config`, `db`, `operator`, `snapshots`),
- canonical audit/evidence file paths,
- lifecycle sequence (`source-check`, `install`, `verify`, `activate`, `status`, `doctor`).

### 2.2 Environment contract (configurable)

The following are environment-governed inputs:

- `AOXC_NETWORK_KIND` selection (`dev`, `testnet`, `mainnet`),
- profile and release-policy material,
- validator and topology parameters,
- rollout strategy (canary, staged, progressive).

In short: **the runtime contract is structurally fixed, while environment activation is controlled and configurable**.

---

## 3) Full Runtime Targets by Environment

### 3.1 Dev runtime (fast feedback, fail-fast)

Required controls:

1. Full quality gate on every PR (`fmt`, `clippy`, `test`, `check`).
2. Automated runtime doctor and smoke command verification.
3. Snapshot creation and restoration checks.
4. Schema compatibility checks for consensus/ledger telemetry fields.
5. Deterministic replay mini-suite using fixed seeds.

Success criteria:

- all merge gates are green before integration,
- latest evidence bundle is retrievable,
- rollback command path is documented and tested.

### 3.2 Testnet runtime (soak + migration proof)

Required controls:

1. Multi-node endurance/soak validation.
2. Genesis/validator/bootnode/certificate validation chain.
3. Upgrade/downgrade/migration drills under policy constraints.
4. Fault-injection scenarios (partition, stale state, replay attempts).
5. Archived readiness and remediation outputs.

Success criteria:

- testnet gate + readiness gate pass per release candidate,
- no deterministic convergence violation,
- recovery drill report is current.

### 3.3 Mainnet runtime (change control + safety first)

Required controls:

1. Signed release artifacts with checksum/provenance.
2. Two-stage activation (`preflight verify -> controlled activate`).
3. Canary validator rollout before wider promotion.
4. SLO/SLI-driven runtime observability.
5. Mandatory incident playbook and postmortem flow.

Success criteria:

- release promotion only from evidence-complete bundles,
- policy-root/scheme/replay controls remain compliant,
- rollback timing is measured and periodically exercised.

---

## 4) Material Gaps Before Claiming "Full"

The following gaps must be closed before asserting full runtime completeness:

1. **Environment-specific runtime stability policy**
   - Explicitly define immutable vs mutable runtime files/fields.

2. **Mandatory evidence publication discipline**
   - Persist gate outputs to immutable artifact storage.

3. **Deterministic replay regression matrix**
   - Maintain fixed-seed golden outputs for critical transaction classes.

4. **Snapshot compatibility policy**
   - Enforce versioned compatibility rules for runtime/state snapshots.

5. **Operational SLO ownership**
   - Assign explicit thresholds and accountability for dev/testnet/mainnet.

---

## 5) Implementation Roadmap

### Phase A — Immediate (0-2 weeks)

- Enforce gate chain in CI:
  - `make quality`
  - `make audit`
  - `make testnet-gate`
  - `make testnet-readiness-gate`
- Define evidence bundle standard (manifest, checksum, retention).
- Add automated smoke stage for runtime doctor surfaces.

### Phase B — Hardening (2-6 weeks)

- Add deterministic replay and snapshot compatibility suites.
- Run testnet soak and fault-injection in nightly CI.
- Bind release signing + verification to promotion gates.

### Phase C — Mainnet-grade operations (6+ weeks)

- Formalize canary rollout with stop-the-line conditions.
- Productionize SLO/SLI dashboards and alert/runbook linkage.
- Schedule and enforce recurring recovery/rollback drills.

### 5.1 Executable closure commands (implemented)

This plan is backed by runnable orchestration targets:

- `make dev-full`
- `make testnet-full`
- `make mainnet-full`
- `make full-runtime-all`

Command chain:

- `dev-full`: quality + dev runtime source/activate + runtime doctor + phase1 determinism suite
- `testnet-full`: dev-full + testnet gate + testnet readiness gate
- `mainnet-full`: testnet-full + network identity gate + production-full
- `full-runtime-all`: dev -> testnet -> mainnet closure chain

---

## 6) Direct Answers

- **Is development "fully complete"?**
  - The foundation is strong, but production-grade completeness requires closure of the Phase A/B/C controls.

- **Is runtime present?**
  - Yes. Runtime lifecycle and runtime telemetry/persistence surfaces are implemented.

- **Is runtime fixed?**
  - Structural runtime contract is fixed; environment/profile activation is controlled and configurable.

- **Are there remaining deficiencies?**
  - No immediate compile-level blocker was observed; primary remaining gaps are evidence enforcement, replay/snapshot policy rigor, and environment-specific stability governance.

- **Is dev/testnet/mainnet fully closed?**
  - Closure targets are implemented; full completion requires enforcing them as merge-blocking CI policy.

---

## 7) Single Highest-Impact Next Action

Make evidence-complete runtime closure **mandatory** in CI for promotion/merge:

- quality + audit + readiness gates,
- signed artifact verification,
- retained immutable evidence bundle.

This single change materially improves auditability, safety, and operational confidence.
