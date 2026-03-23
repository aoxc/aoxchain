# AOX Chain Release and Provenance Runbook

**Release Line:** `aoxc.v.0.1.0-testnet.1`

## Purpose
This runbook defines the non-code release controls required before promoting AOX Chain from internal validation toward public testnet operation.

## Required Release Artifacts
Every release candidate must produce and archive the following:
- compiled binary artifacts,
- checksums,
- SBOM output,
- dependency-audit results,
- `cargo deny` results,
- test and lint logs,
- signed release manifest,
- provenance / attestation record,
- rollback reference to the previous trusted version.

## Mandatory Release Steps
1. Freeze the target version and tag candidate inputs.
2. Run `make quality-release` and archive the logs.
3. Build the release binary with `make build-release`.
4. Package the binary with `make package-bin`.
5. Run `./scripts/release_artifact_certify.sh target/release/aoxc`.
6. Generate an SBOM and attach it to the release record.
7. Sign the manifest, checksums, and release notes.
8. Record provenance data for the exact source revision, toolchain, and artifact digest.
9. Obtain security, runtime, and release-owner approval before publication.

## Evidence Retention
Release evidence must be retained in a location accessible to auditors and on-call responders. At minimum, retain:
- git commit hash,
- release version,
- build host/toolchain metadata,
- test logs,
- audit logs,
- signatures and checksum files,
- SBOM,
- rollback target.

## Failure Policy
A release candidate must not be promoted if any of the following are missing:
- reproducible artifact digest,
- security audit evidence,
- signed manifest,
- rollback plan,
- explicit release owner approval.
