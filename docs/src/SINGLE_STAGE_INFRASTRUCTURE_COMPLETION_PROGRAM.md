# AOXChain Infrastructure and Readiness Completion Program

## Document Control

- **Document status:** Active program baseline
- **Stage 1 baseline:** `v0.1.1-alpha`
- **Stage 2 target baseline:** `v0.2.0-alpha`
- **Program model:** Two-stage closure with Stage 1 completed and Stage 2 active
- **Primary objective:** Close the infrastructure baseline first, then complete the implementation-ready execution package
- **Audience:** Engineering, protocol, DevOps, security, release management, operators, audit reviewers
- **Authoritative use:** This document defines what Stage 1 delivered and what Stage 2 must fully complete before AOXChain can claim aligned testnet/mainnet operational readiness progression.

---

## 1. Executive Summary

This document records the transition from the completed documentation and infrastructure-baseline stage to the active implementation-completion stage.

The governing interpretation is now:

- **Stage 1 is complete** as the documentation and infrastructure-baseline closure for `v0.1.1-alpha`.
- **Stage 2 is now active** and is the full completion package for execution, validation, recovery, security hardening, observability, and release governance needed to move beyond a documentation-only posture.
- AOXChain will continue to use English as the mandatory language for reviewer-facing program artifacts.

The program target is therefore split into two controlled outcomes:

1. **Stage 1:** baseline clarity, versioning, structure, ownership, and evidence definitions,
2. **Stage 2:** implementation-facing readiness closure for real network operation and launch governance.

---

## 2. Program Status

### 2.1 Stage 1 status: Completed baseline

Stage 1 delivered the following closure outcomes:

- English authoritative baseline program documentation,
- mdBook navigation alignment,
- baseline version declaration for `v0.1.1-alpha`,
- owner and evidence expectations,
- repository structure and infrastructure mapping expectations,
- baseline go/no-go policy for documentation integrity.

### 2.2 Stage 2 status: Active completion stage

Stage 2 is the stage requested after Stage 1 completion. It is the stage where AOXChain must close the operational and implementation gaps that remain between a documented baseline and a credible testnet/mainnet readiness posture.

---

## 3. Versioning Policy

### 3.1 Stage 1 baseline

- **Stage 1 baseline version:** `v0.1.1-alpha`
- Meaning: documentation, repository structure, naming, ownership, and evidence expectations are stabilized.

### 3.2 Stage 2 target

- **Stage 2 target version:** `v0.2.0-alpha`
- Meaning: operational, validation, security, recovery, observability, and release-readiness deliverables are implemented or formally evidenced to the level defined in this document.

### 3.3 Advancement rule

Stage 2 may be declared complete only if all exit criteria in this document are satisfied and the required evidence package is approved by the designated owners.

---

## 4. What Stage 2 Must Fully Complete

Stage 2 is not a partial planning exercise. It is the execution stage that must close the remaining readiness package in full.

### 4.1 Node and service operation

Stage 2 must define and complete the operator-facing execution path for:

- node bootstrap,
- long-running node or service mode,
- health and readiness checks,
- operator startup/shutdown workflows,
- and service-level expectations for persistent execution.

### 4.2 Real network validation

Stage 2 must define and execute a credible real-network validation package that covers:

- multi-node validation beyond local-only smoke behavior,
- evidence expectations for propagation and convergence,
- operator-visible outputs and logs,
- and an auditable definition of what qualifies as real network proof.

### 4.3 Consensus and protocol hardening dependencies

Stage 2 must explicitly close or evidence the protocol-facing readiness dependencies required before stronger readiness claims may be made, including:

- consensus persistence expectations,
- validator or authority-set state expectations,
- replay and recovery assumptions,
- authenticated transport-envelope expectations,
- and the dependency relationship between protocol hardening and testnet confidence.

### 4.4 Recovery and rejoin capability

Stage 2 must fully define the recovery package, including:

- snapshot expectations,
- restore expectations,
- rejoin expectations,
- restart behavior expectations,
- and the evidence required to prove consistency after interruption.

### 4.5 Fault and resilience validation

Stage 2 must include a fault-oriented validation model that covers:

- restart scenarios,
- delay and partition scenarios,
- packet-loss or degraded-network assumptions,
- and the expected evidence for recovery, convergence, or residual-risk acceptance.

### 4.6 RPC and public-surface security

Stage 2 must define the minimum security posture for public-facing and operator-facing surfaces, including:

- transport security expectations,
- authentication or trust-boundary expectations,
- abuse-control expectations,
- exposure policy,
- and the distinction between what may be acceptable for testnet versus what remains a mainnet blocker.

### 4.7 Observability and soak readiness

Stage 2 must define the runtime evidence required to support longer-lived operation, including:

- telemetry expectations,
- sync-state visibility,
- peer-state visibility,
- error-counter visibility,
- performance or block-progress visibility,
- and soak-test evidence expectations.

### 4.8 Release and launch governance

Stage 2 must complete the release-governance package, including:

- release owner approval requirements,
- rollback and upgrade expectations,
- artifact and provenance expectations,
- residual risk review,
- and final go/no-go decision controls.

---

## 5. Full Stage 2 Work Packages

Stage 2 is complete only when all work packages below are complete.

### Work Package A — Service Runtime Completion

**Objective:** Convert operator guidance into a complete service/runtime operating model.

**Required outputs:**

- defined persistent node/service path,
- startup and shutdown procedure,
- health/readiness signal expectations,
- operator command boundary between demo flow and real service flow.

**Completion standard:**

A reviewer can determine how AOXChain is expected to run continuously, not only as a local smoke command.

### Work Package B — Multi-Host Network Validation Completion

**Objective:** Establish the evidence package for real network behavior.

**Required outputs:**

- documented validation topology expectations,
- required evidence types for propagation and convergence,
- output/log location expectations,
- pass/fail interpretation guidance.

**Completion standard:**

A reviewer can distinguish local deterministic smoke from credible multi-host validation and see what evidence is required.

### Work Package C — Consensus and Recovery Dependency Closure

**Objective:** Connect protocol hardening expectations to operational readiness claims.

**Required outputs:**

- protocol dependency map,
- persistence and replay expectations,
- recovery dependency statements,
- testnet-vs-mainnet blocker boundaries.

**Completion standard:**

The documentation no longer leaves ambiguity about which protocol gaps are informational versus blocking.

### Work Package D — Snapshot, Restore, and Rejoin Completion

**Objective:** Define the operational recovery lifecycle.

**Required outputs:**

- snapshot expectation statement,
- restore path expectation statement,
- rejoin behavior expectation statement,
- recovery-evidence requirements.

**Completion standard:**

Operators and reviewers can determine what recovery capability must exist and what evidence proves it.

### Work Package E — Fault and Resilience Completion

**Objective:** Make resilience a required evidence category, not an implied future task.

**Required outputs:**

- fault scenario inventory,
- partition/restart/degradation test expectations,
- residual-risk recording rules for incomplete resilience coverage.

**Completion standard:**

The program cannot hide resilience gaps behind generic readiness language.

### Work Package F — Security Surface Completion

**Objective:** Define the minimum operator and public-surface security posture.

**Required outputs:**

- security control expectation list,
- trust-boundary statement,
- testnet exception policy,
- mainnet blocker policy.

**Completion standard:**

A reviewer can see which security controls are mandatory before readiness claims can be upgraded.

### Work Package G — Observability and Soak Completion

**Objective:** Define the minimum runtime visibility package.

**Required outputs:**

- telemetry signal list,
- health and sync visibility list,
- error and peer-state visibility list,
- soak-evidence expectations.

**Completion standard:**

Long-running operation is measurable rather than assumed.

### Work Package H — Release Governance Completion

**Objective:** Close the final approval package for version advancement.

**Required outputs:**

- release checklist expectations,
- rollback and upgrade governance expectations,
- provenance expectations,
- approval sign-off requirements,
- final go/no-go evidence bundle.

**Completion standard:**

Version advancement beyond Stage 2 cannot occur without explicit release governance evidence.

---

## 6. Stage 2 Deliverables

The following deliverables must exist and be internally consistent for Stage 2 to be considered complete.

### 6.1 Operational deliverables

- service-runtime operating definition,
- operator lifecycle procedure,
- health/readiness evidence expectations,
- network-validation procedure and evidence expectations.

### 6.2 Protocol and recovery deliverables

- protocol dependency and blocker map,
- recovery lifecycle expectations,
- snapshot/restore/rejoin evidence expectations,
- restart and resilience requirement definitions.

### 6.3 Security deliverables

- public-surface security expectation set,
- trust-boundary statement,
- testnet exception statement,
- mainnet blocker statement.

### 6.4 Runtime-evidence deliverables

- telemetry expectation set,
- peer/sync/error visibility expectations,
- soak-evidence definition,
- evidence retention expectation.

### 6.5 Release deliverables

- release-governance expectation set,
- rollback/upgrade expectation set,
- approval matrix,
- residual-risk review record,
- go/no-go decision record.

---

## 7. Owner Matrix for Stage 2

| Role | Stage 2 responsibility | Required sign-off |
| --- | --- | --- |
| Program Owner | Final Stage 2 closure and evidence completeness | Yes |
| Protocol Owner | Consensus, validator, replay, recovery dependency approval | Yes |
| Infrastructure Owner | Runtime operation, topology, and evidence-path approval | Yes |
| Security Owner | Security posture and exposure-boundary approval | Yes |
| SRE / Operations Owner | Observability, soak, and operational-lifecycle approval | Yes |
| Release Owner | Version advancement, rollback, and provenance approval | Yes |
| Documentation Owner | Program consistency and reviewer readability | Yes |
| Audit Liaison | Audit evidence-pack review support | Recommended |

---

## 8. Stage 2 Exit Criteria

Stage 2 is complete only if all criteria below are satisfied.

### 8.1 Runtime closure

- The persistent runtime/service path is clearly documented.
- Operator lifecycle expectations are clearly documented.
- Health/readiness expectations are clearly documented.

### 8.2 Network-validation closure

- Real-network validation requirements are defined.
- Evidence categories for propagation and convergence are defined.
- Output and log expectations are defined.

### 8.3 Protocol-closure boundary

- Protocol hardening dependencies are explicitly mapped.
- Testnet-acceptable gaps and mainnet blockers are explicitly separated.
- Recovery and replay expectations are explicitly defined.

### 8.4 Recovery closure

- Snapshot, restore, rejoin, and restart expectations are defined.
- Recovery evidence requirements are defined.

### 8.5 Security closure

- Public-surface security expectations are defined.
- Trust-boundary assumptions are defined.
- Residual security risks are reviewable.

### 8.6 Observability closure

- Telemetry and runtime-visibility expectations are defined.
- Soak evidence expectations are defined.

### 8.7 Release closure

- Release-governance expectations are defined.
- Rollback and upgrade expectations are defined.
- Required sign-offs are complete.

---

## 9. Stage 2 Evidence Package

The evidence package for Stage 2 must contain, at minimum:

1. Stage 2 version declaration for `v0.2.0-alpha`,
2. service-runtime and operator-lifecycle references,
3. network-validation evidence specification,
4. protocol dependency and blocker map,
5. recovery expectation set,
6. security expectation and exception records,
7. telemetry/soak expectation set,
8. release approval package,
9. residual risk register,
10. final go/no-go decision record.

---

## 10. Go/No-Go Rules for Stage 2

Stage 2 must be marked **No-Go** if any of the following is true:

- the operator runtime path remains ambiguous,
- real-network validation remains undefined,
- recovery expectations remain undefined,
- security posture remains undefined,
- telemetry and soak expectations remain undefined,
- release-governance evidence remains incomplete,
- or owner sign-off remains incomplete.

Stage 2 may be marked **Go** only if every work package, deliverable, exit criterion, and evidence requirement in this document has been satisfied and approved.

---

## 11. Mandatory Language Policy

For this program and all future revisions of the baseline artifact:

- English is mandatory for the authoritative reviewer-facing program,
- naming must be suitable for an international engineering and audit audience,
- and localized companion material may exist only as secondary support material.

---

## 12. Immediate Implementation in This Change

This repository change advances the program from a Stage 1-only completion artifact to a Stage 1-complete / Stage 2-active program record by:

1. recording Stage 1 as completed,
2. defining the Stage 2 target as `v0.2.0-alpha`,
3. documenting the full Stage 2 work packages,
4. defining Stage 2 deliverables, exit criteria, evidence, and go/no-go rules,
5. and keeping the program authoritative and English-first for audit use.
