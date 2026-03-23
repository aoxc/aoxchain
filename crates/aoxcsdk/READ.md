# AOXCSDK - Production Audit Guide

**Current Documentation Version:** `aoxc.v.0.0.0-alpha.2`
**Cargo-Compatible Release Version:** `0.0.0-alpha.2`

## Executive Summary
Developer integration safety documentation.

## What This Directory Contains
AOXCSDK offers contract builder and client-side helper abstractions for integrators.

### Key Files and Subsystems
- `src/contracts`

## Architectural Overview
This directory is part of the wider AOX Chain architecture and must be understood together with the workspace release baseline. Reviewers should expect deterministic Rust control flow, explicit error propagation, bounded resource handling, and evidence that all externally visible behavior is traceable to versioned source code.

## Primary Responsibilities
- Present ergonomic but deterministic integration helpers.
- Keep builder defaults explicit.
- Help downstream developers avoid incompatible contract metadata.

## Security Boundary and Audit Focus
SDK defaults must not encourage insecure or ambiguous integration patterns.

Additional review expectations for this directory are listed below.
- Validate that all input boundaries reject malformed or stale values before state mutation.
- Confirm that operational assumptions are documented and do not depend on tribal knowledge.
- Confirm that test evidence exists for both nominal behavior and hostile scenarios.
- Confirm that version changes are recorded before release promotion.

## Production-Readiness Target
The target state for this directory is a 99.99% production-grade posture measured by deterministic build repeatability, explicit failure handling, bounded runtime behavior, and evidence-backed operational readiness. Alpha status does not reduce the documentation standard; it only indicates that promotion gates remain active and incomplete items must still be tracked.

## Verification and Evidence Expectations
The following evidence is expected whenever this directory changes.
1. Formatting and lint validation for all touched Rust surfaces.
2. Unit and integration coverage for changed logic paths.
3. Adversarial, fuzz, or property-style evidence for high-risk logic.
4. Documentation updates that explain what changed and why.
5. Version updates across manifests, binaries, and release notes whenever compatibility changes.

## Strict Change-Control Rule
A change under this directory is not considered release-ready until the corresponding documentation version, semantic version, and verification evidence are updated together. If a contributor adds a new file or subsystem here, this document must be expanded so that a reviewer can understand the purpose, trust boundary, and release impact without reading the entire repository first.

## Current Release Ledger
- `aoxc.v.0.0.0-alpha.2`: replaced lightweight roadmap text with a folder-specific production audit guide, aligned the directory with the alpha.2 release baseline, and declared the mandatory evidence expected for future changes.
- `aoxc.v.0.0.0-alpha.1`: initial alpha roadmap introduction for directory-level audit tracking.

## Audit Checklist
- [ ] Memory ownership and lifecycle assumptions are understood.
- [ ] Error propagation remains explicit and reviewable.
- [ ] Security-sensitive inputs are bounded and validated.
- [ ] Version metadata and release notes are synchronized.
- [ ] Tests or other evidence exist for the changed behavior.
