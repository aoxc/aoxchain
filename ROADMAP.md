# AOXC v0.01 Foundation Roadmap (Checklist + Analysis)

> Scope: **Infrastructure first, then protocol hardening, then production readiness**

This roadmap is intentionally execution-oriented. Every item is checklist-driven and measurable.

---

## A. Current Analysis (Where We Are)

### Strengths
- Modular workspace architecture is already established.
- Consensus, VM, network, RPC, and integration-test surfaces exist.
- CI quality gates are defined.

### Gaps
- Documentation was inconsistent and not operationally prescriptive.
- Cross-platform and Docker workflow was not fully codified in one place.
- Production readiness criteria needs stricter acceptance gates.

### Primary Risk
- Building too many features before finishing deterministic and operational foundations.

---

## B. Phase Plan (Checklist)

## Phase 0 — Documentation + Foundation Reset (Week 1)
- [ ] Establish root-level authoritative docs (`README.md`, `READ.md`, `ROADMAP.md`).
- [ ] Normalize per-directory docs to reference root standards.
- [ ] Define support matrix (Linux/macOS/Windows/Docker).
- [ ] Freeze non-foundation feature additions.

**Exit Criteria:** Single source of truth + clear operating baseline.

## Phase 1 — Build & Environment Unification (Weeks 2–3)
- [ ] Ensure all mandatory commands run clean on Linux/macOS/Windows.
- [ ] Add/update Docker dev workflow for parity with local checks.
- [ ] Provide one-command validation script for contributors.
- [ ] Verify deterministic behavior of key integration tests.

**Exit Criteria:** Reproducible environment and consistent onboarding.

## Phase 2 — Protocol Hardening (Weeks 4–7)
- [ ] Expand consensus safety/finality regression suites.
- [ ] Add deterministic multi-lane execution conformance tests.
- [ ] Expand network resilience tests (reorder/drop/dup/partition scenarios).
- [ ] Formalize compatibility constraints across RPC/SDK surfaces.

**Exit Criteria:** Safety-critical paths covered by repeatable tests.

## Phase 3 — Operations & Security (Weeks 8–10)
- [ ] Define SLI/SLO metrics and telemetry baseline.
- [ ] Ship operator runbook and incident response guide.
- [ ] Add release evidence checklist and artifact integrity workflow.
- [ ] Run security review pass for core protocol boundaries.

**Exit Criteria:** Operationally manageable pre-production posture.

## Phase 4 — Release Candidate Readiness (Weeks 11–12)
- [ ] Freeze protocol-critical interfaces for RC window.
- [ ] Execute full regression matrix in CI + Docker.
- [ ] Publish release notes + known limitations + rollback policy.
- [ ] Tag `v0.01-rc` only if all hard gates pass.

**Exit Criteria:** Controlled, auditable release candidate.

---

## C. Hard Quality Gates (Must Pass)

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --exclude aoxchub --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --exclude aoxchub --all-targets`
- [ ] `cargo check -p aoxchub --all-targets`
- [ ] Cross-platform smoke checks documented and green
- [ ] Docker parity checks documented and green

---

## D. Definition of Done (v0.01 Foundation)

v0.01 foundation is complete only when:

1. Docs are coherent and operationally actionable.
2. Core build/test runs reproducibly across supported platforms.
3. Determinism + consensus safety + resilience tests are reliable.
4. Release and incident workflows are documented and rehearsed.
