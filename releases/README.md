# Release Artifact Layout

## Purpose

This directory is the repository-governed release surface for operator-consumable binaries and verification metadata.

Its goals are:

- deterministic binary distribution,
- explicit version-to-network compatibility controls,
- reproducible verification and rollback posture.

## Directory Contract

Each shipped release must use one versioned directory:

- `releases/v<workspace-version>/`

Example:

- `releases/v0.2.0-aoxcq/`

A release directory should contain:

- `manifest.json` (machine-readable release index)
- `checksums.sha256` (hashes for every published artifact)
- `sbom.spdx.json` (SBOM for supply-chain review)
- `provenance.intoto.jsonl` (build provenance/attestation)
- `compatibility.toml` (declared chain/profile compatibility)
- `binaries/` (target-specific binaries)
- `signatures/` (detached signatures for manifest + binaries)

## Compatibility Rules

Release metadata must stay aligned with:

- `Cargo.toml` workspace version,
- `configs/version-policy.toml`,
- `configs/environments/<env>/release-policy.toml`,
- `configs/registry/network-registry.toml` identity values.

A release must not claim compatibility unless `compatibility.toml` explicitly maps:

- allowed `network_id` set,
- allowed `chain_id` set,
- required crypto/profile baseline,
- minimum/maximum manifest and certificate schema versions.

## Operator Consumption Guidance

Normal operators should install from this release surface (or mirrored immutable object storage) rather than building from source.

Minimum validation before install:

1. verify signature of `manifest.json`,
2. verify binary checksum against `checksums.sha256`,
3. verify `compatibility.toml` matches target network/profile,
4. run `aoxc version` and compare with manifest metadata.

## Governance Notes

- Do not rewrite an existing version directory after publication.
- Publish fix-only respins as a new version.
- Keep release evidence immutable and reviewable.
