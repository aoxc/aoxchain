# AOXChain Single-Stage Infrastructure Completion Program

## Document Control

- **Document status:** Active baseline program
- **Program baseline:** `v0.1.1-alpha`
- **Program model:** Single-stage completion
- **Primary objective:** Infrastructure completeness at 100%
- **Audience:** Engineering, protocol, DevOps, security, release management, audit reviewers
- **Authoritative use:** This document defines what must be completed before AOXChain may claim a fully aligned testnet/mainnet infrastructure baseline.

---

## 1. Executive Summary

This document replaces the prior Turkish single-stage planning draft with an **English, audit-ready program** that can be read by external reviewers, engineers, operators, and security partners.

The governing decision is simple:

- AOXChain will treat the current baseline as **`v0.1.1-alpha`**.
- AOXChain will not proceed with stage-based ambiguity, partial placeholders, or undocumented infrastructure assumptions.
- AOXChain will close all foundational documentation, repository structure, versioning, infrastructure-file mapping, launch controls, and readiness expectations inside **one completion stage**.

The target condition is:

> **Infrastructure completeness at 100% for the documented baseline.**

For this program, “100%” does **not** mean that every future production feature is finished. It means that, for the declared baseline:

1. no critical documentation gaps remain,
2. no critical mdBook navigation gaps remain,
3. no critical versioning ambiguity remains,
4. no critical infrastructure-file ownership or placement ambiguity remains,
5. no critical readiness evidence category is left undocumented,
6. no launch-governance responsibility is left undefined.

---

## 2. Why This Document Exists

The previous approach suffered from a structural communication problem:

- strategic intent was documented in one place,
- execution expectations were scattered elsewhere,
- some navigation and document names were not suitable for an international engineering or audit audience,
- and the repository lacked a single English baseline document that explained what the one-stage completion program actually delivers.

This document corrects that problem by establishing one authoritative English program for the baseline.

---

## 3. Program Scope

This single-stage program covers the full infrastructure baseline required to support aligned testnet and mainnet readiness planning.

### In scope

1. documentation integrity,
2. mdBook structure and navigation integrity,
3. baseline versioning and release naming,
4. repository infrastructure-file mapping,
5. configuration, fixture, artifact, and output directory definitions,
6. node bootstrap and service-operation documentation,
7. network-validation expectations,
8. consensus hardening dependency mapping,
9. snapshot/recovery/rejoin expectations,
10. RPC and public-surface security expectations,
11. observability and soak-test evidence expectations,
12. release, provenance, rollback, and upgrade governance,
13. owner matrix and approval authority,
14. audit-facing evidence package structure,
15. launch go/no-go controls.

### Out of scope

This document does not claim that the entire mainnet implementation is complete today. Instead, it defines the exact baseline package that must be complete before the program can declare the infrastructure foundation fully closed for `v0.1.1-alpha`.

---

## 4. Baseline Versioning Policy

### 4.1 Baseline identifier

The official baseline for this program is:

- **`v0.1.1-alpha`**

### 4.2 Meaning of `v0.1.1-alpha`

For this repository, `v0.1.1-alpha` means:

- the baseline scope is frozen,
- the minimum required infrastructure documentation is frozen,
- the expected repository structure is frozen,
- required owner assignments are frozen,
- and completion can be audited against a stable reference.

### 4.3 Advancement rule

No subsequent planning version should be declared complete until the exit criteria in this document are met and evidenced.

---

## 5. Stage Objective

The single stage has one objective:

> Deliver a complete, English, audit-ready infrastructure baseline that clearly defines what exists, where it lives, how it is operated, what version it belongs to, who owns it, and what evidence is required to advance readiness claims.

---

## 6. What Will Be Done in the Single Stage

This section directly answers the user-level question, “What exactly will be done in Stage 1?”

### 6.1 Documentation normalization

The repository documentation set will be normalized so that:

- English becomes the default language for the core completion program,
- strategic, operational, readiness, and technical documents are clearly separated,
- cross-references are explicit,
- naming is suitable for external reviewers,
- and no critical baseline topic is documented only through implicit tribal knowledge.

### 6.2 mdBook correction

The documentation navigation will be corrected so that:

- the sidebar points to the authoritative program document,
- labels are understandable to external readers,
- obsolete or misleading labels are removed,
- and the book structure reflects the actual information architecture.

### 6.3 Versioning formalization

The baseline version will be formalized so that:

- `v0.1.1-alpha` has a defined meaning,
- its closure criteria are documented,
- and future reviewers can determine whether the baseline was truly completed.

### 6.4 Infrastructure-file mapping

The repository will be documented from an infrastructure perspective so that reviewers can identify:

- where configuration files are expected,
- where fixtures are expected,
- where generated artifacts are expected,
- where network-validation outputs are expected,
- and which directories are authoritative versus temporary.

### 6.5 Operational path clarification

The documentation will define the intended operational path, including:

- what qualifies as a local smoke command,
- what qualifies as a persistent node/service flow,
- what evidence must be collected for real network validation,
- and how operators should interpret readiness boundaries.

### 6.6 Control and accountability definition

The program will define:

- who owns protocol sign-off,
- who owns infrastructure sign-off,
- who owns security sign-off,
- who owns release sign-off,
- and what evidence each owner must review before program closure.

### 6.7 Audit evidence packaging

The program will specify the evidence pack required for closure, including:

- version declaration,
- owner approvals,
- repository structure references,
- readiness checklists,
- residual risk logs,
- and required decision records.

---

## 7. Required Deliverables

The single stage is not complete unless all deliverables below exist and are internally consistent.

### 7.1 Governance deliverables

- Program baseline declaration for `v0.1.1-alpha`
- Owner matrix
- Approval matrix
- Go/no-go decision template
- Residual risk register format

### 7.2 Documentation deliverables

- Authoritative single-stage completion program in English
- Updated mdBook summary/navigation entry
- Cross-reference alignment to readiness and operational documents
- Clear classification of strategy vs. checklist vs. runbook vs. technical analysis

### 7.3 Infrastructure deliverables

- Repository directory dictionary for infrastructure-relevant paths
- Defined expectations for config, fixture, artifact, and output placement
- Declared evidence locations for network, recovery, and soak validation outputs
- Explicit statement of any remaining placeholder content and its owner

### 7.4 Operational deliverables

- Defined distinction between local smoke and real-network validation
- Defined operator expectations for node lifecycle flows
- Defined evidence expectations for health/readiness visibility
- Defined relationship between testnet claims and mainnet blockers

### 7.5 Security and release deliverables

- Defined baseline expectations for RPC/public-surface controls
- Defined upgrade, rollback, and provenance governance expectations
- Defined required sign-off roles before readiness claims may be elevated

---

## 8. Repository Structure Expectations

The program requires every infrastructure-relevant area to be documented with purpose and expected contents.

### 8.1 Minimum repository mapping categories

The repository mapping should cover, at minimum:

- `docs/` for published documentation,
- `scripts/` for operational and validation scripts,
- `models/` for readiness evidence models and structured planning data,
- configuration-related paths,
- fixture-related paths,
- artifact/output paths,
- and any path used by testnet or mainnet operational flows.

### 8.2 Required directory-level statements

For each critical path, the program should define:

- purpose,
- owner,
- expected inputs,
- expected outputs,
- retention expectations,
- and whether the path is authoritative, generated, or temporary.

---

## 9. Owner Matrix

The single stage must assign accountable roles. One individual may hold multiple roles, but the responsibilities must be explicit.

| Role | Responsibility | Required sign-off |
| --- | --- | --- |
| Program Owner | Overall closure decision and program integrity | Yes |
| Protocol Owner | Consensus, validator, recovery dependency review | Yes |
| Infrastructure Owner | Repository structure, config/artifact layout, operator path review | Yes |
| Security Owner | RPC/public-surface, transport, and residual risk review | Yes |
| Release Owner | Version baseline, release notes, rollback/upgrade governance | Yes |
| Documentation Owner | mdBook integrity, naming consistency, cross-reference quality | Yes |
| Audit Liaison | Evidence-pack completeness and reviewer readiness | Recommended |

---

## 10. Exit Criteria

The stage is complete only when every criterion below is satisfied.

### 10.1 Documentation closure

- The authoritative single-stage completion document exists in English.
- The document is suitable for external engineering and audit review.
- No core program dependency is described only in Turkish inside the completion baseline.

### 10.2 Navigation closure

- `docs/src/SUMMARY.md` points to the authoritative English completion document.
- Navigation labels are understandable and professional.
- The navigation does not imply an outdated multi-stage or non-English-only baseline.

### 10.3 Versioning closure

- `v0.1.1-alpha` is explicitly defined.
- The baseline meaning is documented.
- Stage completion is tied to versioned evidence rather than informal statements.

### 10.4 Infrastructure closure

- The program defines the expected repository mapping categories.
- Infrastructure-relevant file families are described in operationally meaningful terms.
- The location and purpose of evidence outputs are defined.

### 10.5 Governance closure

- Owner roles are explicit.
- Approval responsibilities are explicit.
- The go/no-go review input set is defined.
- Residual risk recording is required.

### 10.6 Audit closure

- A reviewer can determine what the baseline claims.
- A reviewer can determine what the baseline does not claim.
- A reviewer can determine what evidence is required to advance beyond the baseline.

---

## 11. Evidence Package Required for Closure

The closure package for the single stage must contain, at minimum:

1. the authoritative completion document,
2. updated documentation navigation,
3. version-baseline declaration,
4. owner/sign-off matrix,
5. infrastructure path dictionary,
6. readiness cross-reference set,
7. residual risk register,
8. go/no-go decision record.

---

## 12. Go/No-Go Rules

The stage must be marked **No-Go** if any of the following is true:

- the authoritative completion document is not in English,
- navigation still points to obsolete or misleading stage labels,
- the baseline version is not defined,
- owner accountability is missing,
- required evidence categories are not described,
- or the repository structure remains materially ambiguous for operators or auditors.

The stage may be marked **Go** only if all exit criteria are satisfied and the sign-off owners accept the evidence package.

---

## 13. Expected Outcome After Completion

When this single stage is complete, AOXChain should be able to answer the following questions unambiguously:

- What is the baseline version?
- What exactly does the baseline claim?
- Which documents are authoritative?
- Where should operators look for infrastructure-relevant assets?
- Which readiness areas remain implementation blockers beyond documentation?
- Who is responsible for advancing the next readiness gate?

If those questions cannot be answered from repository documentation, the program is not complete.

---

## 14. Mandatory Language Policy for the Baseline Program

For this completion program and future revisions of this baseline artifact:

- **English is mandatory**,
- reviewer-facing naming must be professional and internationally understandable,
- and core program intent must not depend on Turkish-only terminology.

Localized companion documents may exist, but the authoritative baseline completion program must remain English-first.

---

## 15. Immediate Implementation in This Change

This repository change implements the first mandatory baseline correction by:

1. replacing the prior Turkish one-stage planning document with an English audit-ready version,
2. moving the navigation to an English file name and English-facing title,
3. and preserving the single-stage completion model centered on `v0.1.1-alpha` and infrastructure completeness.

