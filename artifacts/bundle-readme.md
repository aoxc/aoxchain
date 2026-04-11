# AOXC Artifacts — Advanced Production Readiness Bundle

This directory is the canonical, machine-auditable evidence bundle for AOXC releases.  
It is designed for **always-on operational reading**, deterministic validation, and governance-grade review across the following release surfaces:

- **Mainnet**
- **Testnet**
- **Devnet**
- **Desktop interface / wallet rollout**

---

## 1) Purpose

The `artifacts/` bundle serves as the single source of truth for release evidence and production readiness validation.

Its purpose is to consolidate the full evidence chain required for operational, governance, and audit review, including:

- release integrity evidence (checksum, signature status, provenance)
- build and supply-chain attestations (build manifest, SBOM, toolchain traceability)
- environment compatibility evidence (mainnet, testnet, devnet)
- production closure evidence (runtime, telemetry, drills, audits, rollback readiness)
- desktop interface and wallet rollout compatibility evidence
- machine-readable release indexing for CI/CD and governance gates
- bundle-wide verification status and release-readiness reporting

This bundle is not intended to function as informal documentation alone.  
It is a structured release control surface intended to support deterministic review, release gating, and post-release accountability.

---

## 2) Directory Structure

The bundle is intentionally divided into **root-level governance surfaces** and **domain-specific evidence sets**.

### Root-level files

- `bundle-index.json`
  - Machine-readable artifact index for CI/CD gates, automated verification, and governance review.
- `bundle-toolchain-manifest-*.json`
  - Bundle-wide build environment and toolchain traceability manifest.
- `bundle-verification-report-*.json`
  - Bundle-wide verification outcome, completion status, blocking findings, and release approval posture.
- `bundle-readme.md`
  - Canonical operator and reviewer guidance for interpreting this bundle.

### Evidence subdirectories

- `release-evidence/`
  - Release-specific evidence including release notes, build manifest, SBOM, provenance, compatibility matrix, checksum evidence, signature status, and release audit surfaces.
- `network-production-closure/`
  - Runtime status, telemetry snapshots, production audit, security drill, soak plan, AOXHub rollout, desktop wallet compatibility, and alert rules.
- `quantum-gate/`
  - Structural quantum-readiness gate summary for required transition and closure surfaces.

This structure reflects a single-bundle model:  
root-level files describe the bundle as a whole, while subdirectories contain specialized evidence domains.

---

## 3) Mandatory Reading Order (Always)

The following reading order is mandatory for governance, release approval, and audit review:

1. `artifacts/bundle-index.json`
2. `artifacts/bundle-verification-report-*.json`
3. `artifacts/bundle-toolchain-manifest-*.json`
4. `artifacts/release-evidence/release-notes-*.md`
5. `artifacts/release-evidence/release-build-manifest-*.json`
6. `artifacts/release-evidence/release-provenance-*.json`
7. `artifacts/release-evidence/release-sbom-*.json`
8. `artifacts/release-evidence/release-compatibility-matrix-*.json`
9. `artifacts/network-production-closure/closure-runtime-status.json`
10. `artifacts/network-production-closure/closure-telemetry-snapshot.json`
11. `artifacts/network-production-closure/closure-desktop-wallet-compat.json`
12. `artifacts/network-production-closure/closure-production-audit.json`
13. `artifacts/network-production-closure/closure-security-drill.json`
14. `artifacts/quantum-gate/summary.json`

This reading order is designed to ensure that bundle-wide trust posture is evaluated before individual evidence fragments are reviewed in isolation.

---

## 4) Compatibility Completion Criteria (100%)

A release may be considered **100% complete** only if all conditions below are satisfied and the bundle-wide verification report confirms a release-approvable state:

- Mainnet compatibility evidence is present and marked pass.
- Testnet compatibility evidence is present and marked pass.
- Devnet compatibility evidence is present and marked pass.
- Desktop interface compatibility evidence is present and marked pass.
- Provenance evidence is present, verifiable, and not represented by a placeholder or failure marker.
- Checksum evidence is present and matches the built binary or canonical output.
- Signature evidence is present and verifiable.
- Production closure evidence is present and internally consistent.
- Toolchain traceability evidence is present and aligned with the release record.
- Bundle-wide verification status reports no blocking findings for release approval.
- Quantum-readiness gate summary exists and reports no missing required surfaces.

File presence alone does not satisfy completion criteria.  
Completion requires verifiable evidence content, not merely path existence.

---

## 5) Validation Checklist

At minimum, all validation reviewers and CI/CD gates should confirm the following:

- JSON artifacts parse without error.
- Required files listed in `bundle-index.json` all exist at their declared locations.
- Root-level bundle metadata is consistent with release-specific evidence.
- Release evidence timestamp families are internally consistent across build manifest, SBOM, provenance, verification report, and toolchain manifest.
- Runtime and telemetry snapshots align with the declared release window.
- Desktop compatibility evidence references the same release target and compatibility line.
- Verification status and blocking findings are consistent with underlying evidence content.
- No artifact required for release approval is represented solely by a placeholder, stub, or failure marker.
- Quantum gate output confirms required transition surfaces are present and structurally valid.

Where any mismatch exists between `bundle-index.json`, the verification report, and underlying evidence, the bundle must be treated as **provisional** or **incomplete** until resolved.

---

## 6) Recommended Automation Pipeline

### Pre-release

The pre-release phase should produce a deterministic evidence set covering:

- build execution
- test execution
- build manifest generation
- toolchain manifest generation
- SBOM generation
- provenance generation
- compatibility matrix generation
- checksum generation
- initial verification report generation

### Release gate

The release gate should verify:

- checksum integrity
- signature integrity
- provenance validity
- compatibility status across mainnet, testnet, devnet, and desktop wallet surfaces
- consistency between `bundle-index.json`, toolchain manifest, verification report, and release evidence

The release gate must fail closed if any mandatory evidence surface is missing, unverifiable, or represented only by a failure marker.

### Post-release closure

The post-release closure phase should collect and lock:

- runtime status
- telemetry snapshot
- production audit evidence
- security drill evidence
- soak validation evidence
- rollout readiness evidence
- final bundle verification status
- retention-ready bundle state for audit and governance review

---

## 7) Placeholder and Failure Marker Policy

Artifacts that exist only as placeholders, bootstrap fixtures, incomplete stubs, or explicit failure markers do **not** satisfy completion criteria.

Examples include, but are not limited to:

- provenance artifacts reporting values such as `missing-generator`
- signature status artifacts reporting values such as `MISSING_SIGNATURE`
- manifests containing unresolved critical fields such as `unknown` or `unavailable`
- bootstrap-only SBOM content that does not represent the release inventory with sufficient fidelity

Such artifacts may remain in the bundle for operator visibility and debugging context, but they must cause the bundle status to remain **provisional**, **conditional**, or **incomplete** until replaced by verifiable evidence.

Under no circumstance should placeholder presence be interpreted as release approval evidence.

---

## 8) Governance and Audit Interpretation

This bundle is intended to support three distinct review modes:

### Operational review
Used by operators to confirm runtime, telemetry, compatibility, and rollout readiness.

### Release review
Used by release managers and governance signers to determine whether a release is approvable.

### Audit review
Used by internal or external reviewers to validate that release integrity, supply-chain traceability, and production closure evidence are complete and internally consistent.

Reviewers must interpret the bundle as a connected evidence system.  
No single artifact should be treated as sufficient proof of readiness in isolation.

---

## 9) Security Note

Artifacts are evidence surfaces, not standalone proof of authorship, authority, or release legitimacy.

Final trust requires coordinated verification of at least the following:

- signature validity
- provenance validity
- checksum integrity
- compatibility status
- production closure consistency
- bundle-wide verification status

A release must not be treated as fully trusted unless these surfaces agree and the verification report indicates a release-approvable outcome.
