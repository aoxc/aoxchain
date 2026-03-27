# AOXC Foundation Roadmap (Advanced Execution Plan)

> Objective: deliver a production-candidate foundation through deterministic engineering, strict quality gates, and operator-ready infrastructure.

## Phase 0 — Foundation Reset (Week 1)
- [ ] Establish canonical top-level docs and governance language
- [ ] Define branch strategy and release branching conventions
- [ ] Freeze non-foundational feature expansion
- [ ] Publish acceptance criteria for every upcoming phase

## Phase 1 — Build and Toolchain Convergence (Weeks 2–3)
- [ ] Ensure Linux/macOS/Windows compatibility for mandatory workflows
- [ ] Standardize Docker validation path for all quality gates
- [ ] Enforce one-command verification policy for contributors
- [ ] Track and remove environment-specific drift

## Phase 2 — Determinism and Consensus Hardening (Weeks 4–7)
- [ ] Expand deterministic transaction and hashing regressions
- [ ] Expand consensus stale/fork/finality stress tests
- [ ] Add fault profile scenarios (reorder, duplicate, drop, delay)
- [ ] Promote invariant violations to release-blocking checks

## Phase 3 — Network and Runtime Reliability (Weeks 8–10)
- [ ] Define runtime SLOs (latency, finality, error budgets)
- [ ] Add telemetry and alerting baselines
- [ ] Harden RPC contracts and backward compatibility policy
- [ ] Verify recovery and rollback procedures under stress

## Phase 4 — Release Candidate Gate (Weeks 11–12)
- [ ] Full CI + Docker + cross-platform evidence bundle
- [ ] Security and dependency review report
- [ ] Release notes, migration notes, rollback guidance
- [ ] `v0.01-rc` tag only after all hard gates pass

---

## Hard Gates (Non-Negotiable)

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --exclude aoxchub --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --exclude aoxchub --all-targets`
- [ ] `cargo check -p aoxchub --all-targets`
- [ ] Cross-platform smoke validation report
- [ ] Docker parity report

---

## Definition of Done

AOXC v0.01 foundation is complete only if:

1. Determinism-critical paths are test-guarded.
2. Consensus and resilience regressions are stable.
3. Build/test pipelines are reproducible across target platforms.
4. Operator runbooks and release evidence are complete.
