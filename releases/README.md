# Release Artifact Layout

## Purpose

`releases/` is the repository-governed publication surface for operator-consumable AOXC binaries.

This surface exists to provide:

- deterministic binary publication,
- explicit version and network compatibility metadata,
- reproducible verification for operators and reviewers.

## Versioned Directory Contract

Each published release uses an immutable version directory:

- `releases/v<workspace-version>/`

Example:

- `releases/v0.2.0-aoxcq/`

Required entries:

- `manifest.json` — machine-readable release metadata index,
- `checksums.sha256` — SHA-256 map for published artifacts,
- `compatibility.toml` — chain/network/profile compatibility contract,
- `binaries/` — versioned binary payloads grouped by target label,
- `signatures/` — detached signatures for binaries and manifest.

Recommended entries:

- `sbom.spdx.json` — software bill of materials,
- `provenance.intoto.jsonl` — build provenance and attestation.

## Mandatory Alignment Rules

Release metadata must remain aligned with repository policy surfaces:

- `Cargo.toml` (`[workspace.package].version`),
- `configs/version-policy.toml` (`[workspace].current` and schema tracks),
- `configs/registry/network-registry.toml` canonical network identity,
- target environment `configs/environments/<env>/release-policy.toml`.

A release is invalid if these sources disagree.

## Repository Automation

Use the release automation scripts instead of hand-editing files.

### 1) Build signed release bundle (recommended)

```bash
make repo-release-keygen
make repo-release-signed
make repo-release-signed-verify
```

`repo-release-signed` builds and publishes a platform bundle containing:

- `bin/aoxc`
- `bin/aoxchub`
- `bin/aoxckit`
- `SHA256SUMS`
- `manifest.json`
- `signatures/manifest.json.sig`
- `signatures/SHA256SUMS.sig`

Release signatures are produced using `RELEASE_SIGNING_KEY` and verified with
`RELEASE_SIGNING_CERT`.

### 2) Prepare a minimal versioned release directory

```bash
python3 scripts/release/prepare_repo_release.py \
  --binary target/release/aoxc \
  --network mainnet \
  --target-label linux-amd64 \
  --release-line AOXC-Q-v0.2.0 \
  --crypto-profile aoxcq-v1
```

This command creates and populates:

- `releases/v<workspace-version>/manifest.json`
- `releases/v<workspace-version>/checksums.sha256`
- `releases/v<workspace-version>/compatibility.toml`
- `releases/v<workspace-version>/binaries/<target-label>/aoxc`
- `releases/v<workspace-version>/signatures/`

### 3) Validate release integrity before publish

```bash
python3 scripts/release/validate_repo_release.py releases/v<workspace-version>
```

Validation checks:

- required files/directories exist,
- manifest artifact paths resolve,
- SHA-256 in `manifest.json` equals actual binary hash,
- SHA-256 in `checksums.sha256` equals actual binary hash.

## Operator Install Minimum

Before installation, operators should verify:

1. signature of `manifest.json`,
2. artifact hash against `checksums.sha256`,
3. `compatibility.toml` against target `network_id` and `chain_id`,
4. installed binary version via `aoxc version`.

## Fail-Closed Runtime Policy (Recommended)

For mainnet/testnet operators, node startup should be blocked unless all of the
following are true:

- signature verification passed (`repo-release-signed-verify` equivalent),
- checksum verification passed for `aoxc`, `aoxchub`, and `aoxckit`,
- release version matches approved network release policy.

When release version changes, publish a new manifest, new checksums, and new
signatures together. Do not reuse old signatures across versions.

## Immutability and Governance

- Do not rewrite an existing published version directory.
- Publish any fix as a new version directory.
- Keep release evidence immutable and reviewable.
