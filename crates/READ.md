# Workspace Crates - Production Audit Guide

**Current Documentation Version:** `aoxc.v.0.0.0-alpha.3`
**Cargo-Compatible Release Version:** `0.0.0-alpha.3`

## Executive Summary
Unified roadmap for the Rust crate portfolio and its production release obligations.

## What This Directory Contains
The crates directory is the product surface of AOX Chain. Each crate owns a bounded architectural responsibility and is expected to preserve determinism, typed interfaces, and explicit error handling.

### Key Files and Subsystems
- `crates/README.md`
- `crates/aoxcore`
- `crates/aoxcmd`
- `crates/aoxcunity`
- `crates/aoxcvm`

## Architectural Overview
This directory is part of the wider AOX Chain architecture and must be understood together with the workspace release baseline. Reviewers should expect deterministic Rust control flow, explicit error propagation, bounded resource handling, and evidence that all externally visible behavior is traceable to versioned source code.

## Primary Responsibilities
- Separate concerns between core protocol modeling, operational tooling, networking, execution, and integration libraries.
- Allow audits to focus on the crate that owns a given risk instead of a monolithic codebase.
- Enforce shared version governance so crate releases cannot drift away from the workspace release train.

## Security Boundary and Audit Focus
The crate portfolio must prevent accidental coupling that would hide security assumptions. Interfaces between crates should remain explicit, versioned, and testable in isolation.

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
- `aoxc.v.0.0.0-alpha.3`: replaced lightweight roadmap text with a folder-specific production audit guide, aligned the directory with the alpha.3 release baseline, and declared the mandatory evidence expected for future changes.
- `aoxc.v.0.0.0-alpha.2`: initial folder-specific audit guide and directory-level version tracking baseline.

## Audit Checklist
- [ ] Memory ownership and lifecycle assumptions are understood.
- [ ] Error propagation remains explicit and reviewable.
- [ ] Security-sensitive inputs are bounded and validated.
- [ ] Version metadata and release notes are synchronized.
- [ ] Tests or other evidence exist for the changed behavior.
