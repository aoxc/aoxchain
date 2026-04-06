# AOXChain Testnet Release Runbook (Repository-Independent)

## Purpose

This runbook defines how to launch and operate the canonical AOXChain testnet release from a versioned release bundle **without requiring a source checkout**.

It is intended for operators who consume `releases/v<workspace-version>/` artifacts and run `aoxc` directly.

## Scope

This document covers:

- release bundle verification,
- host preparation,
- initial testnet bootstrap,
- persistent runtime control,
- identity/gate validation,
- CLI command surfaces for daily operations,
- promotion-ready evidence collection.

It does not define mainnet activation governance.

## Release Artifact Contract

A valid operator release directory must include at least:

- `manifest.json`
- `checksums.sha256`
- `compatibility.toml`
- `binaries/<target>/aoxc`
- `signatures/` (for signed bundles)

Use the repository release validator before installation:

```bash
python3 scripts/release/validate_repo_release.py releases/v<workspace-version>
```

## Operator Host Preparation

### 1) Install AOXC binary from release bundle

```bash
install -m 0755 releases/v<workspace-version>/binaries/<target>/aoxc /usr/local/bin/aoxc
```

### 2) Define operator home and network profile

```bash
export AOXC_HOME=/var/lib/aoxc
export AOXC_NETWORK_KIND=testnet
mkdir -p "${AOXC_HOME}"
```

### 3) Verify binary and build metadata

```bash
aoxc --help
aoxc version
aoxc build-manifest
```

## Canonical Testnet Bootstrap Flow

### 1) Initialize runtime configuration

```bash
aoxc config-init --profile testnet
```

### 2) Bootstrap validator/operator keys

```bash
aoxc key-bootstrap --profile testnet --password '<strong-password>'
```

### 3) Initialize and validate genesis bundle

```bash
aoxc genesis-init --profile testnet
aoxc genesis-validate --strict
aoxc genesis-production-gate
```

### 4) Run network identity gate (CLI)

Single environment:

```bash
aoxc network-identity-gate --enforce --env testnet --format json
```

All canonical environments:

```bash
aoxc network-identity-gate --full --enforce --format json
```

## Start and Operate Testnet Runtime

### Start node runtime

```bash
aoxc node start
```

### Runtime and network health

```bash
aoxc node status
aoxc network status
aoxc network verify
aoxc diagnostics-doctor
```

### API and chain queries

```bash
aoxc query chain status
aoxc query network peers
aoxc api status
aoxc rpc-status
```

## Full CLI Surface (Operator-Oriented)

### Core bootstrap and identity

- `aoxc config-init`
- `aoxc key-bootstrap`
- `aoxc key-rotate`
- `aoxc keys-inspect`
- `aoxc genesis-init`
- `aoxc genesis-validate --strict`
- `aoxc genesis-production-gate`
- `aoxc network-identity-gate [--full|--env <name>] [--enforce]`

### Runtime control

- `aoxc node start`
- `aoxc node status`
- `aoxc node doctor`
- `aoxc diagnostics-doctor`
- `aoxc diagnostics-bundle`

### Network/consensus visibility

- `aoxc network status`
- `aoxc network verify`
- `aoxc consensus-status`
- `aoxc consensus-validators`
- `aoxc consensus-finality`

### VM, state, and transaction surfaces

- `aoxc vm-status`
- `aoxc vm-call`
- `aoxc vm-simulate`
- `aoxc state-root`
- `aoxc tx-get --hash <tx-hash>`
- `aoxc tx-receipt --hash <tx-hash>`

### Readiness and release controls

- `aoxc testnet-readiness --enforce`
- `aoxc full-surface-readiness --enforce`
- `aoxc full-surface-gate --enforce`
- `aoxc production-audit`

## Testnet Release Go/No-Go (Operator)

Before publishing a testnet release announcement, require all of the following:

1. `aoxc network-identity-gate --enforce --env testnet` passes.
2. `aoxc genesis-validate --strict` and `aoxc genesis-production-gate` pass.
3. `aoxc testnet-readiness --enforce` passes.
4. `aoxc diagnostics-doctor` reports no blocking findings.
5. Release checksum/signature verification is recorded in evidence logs.

If any control fails, status remains `NOT_READY` and publication is blocked.

## Repository-Independent Operations Notes

- Runtime identity must come from manifest/profile/genesis files, not binary hardcoding.
- Do not override `chain_id`, `network_id`, or manifest identity in release policies.
- Keep release artifacts immutable after publication.
- Prefer signed bundles for any externally consumed testnet rollout.

## Minimal Daily Operator Checklist

```bash
aoxc node status
aoxc network status
aoxc network-identity-gate --enforce --env testnet --format json
aoxc testnet-readiness --enforce --format json
```

This is the minimum deterministic control loop for a continuously running testnet release line.
