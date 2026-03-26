# AOXC Environment Bundles

This directory contains the canonical environment bundles for AOXC.

Each environment bundle defines the files required to identify, validate, and operate a specific AOXC network line.

## Environment Classes

The current AOXC environment structure includes:

- `mainnet/`
- `testnet/`
- `validation/`
- `localnet/`
- `sovereign/template/`

## Bundle Model

Each canonical AOXC environment bundle is expected to contain:

- `manifest.v1.json`
- `genesis.v1.json`
- `genesis.v1.sha256`
- `validators.json`
- `bootnodes.json`
- `profile.toml`
- `release-policy.toml`
- `certificate.json`

Localnet additionally includes local orchestration and deterministic multi-node support files.

## Authority Model

The authoritative policy hierarchy is:

1. `configs/registry/network-registry.toml`
2. `configs/registry/binary-compatibility.toml`
3. `configs/environments/*/manifest.v1.json`

The remaining files inside each environment bundle must remain consistent with those layers.

## Stability Expectations

### Long-lived identity anchors
- `manifest.v1.json`

### Long-lived but revisable policy files
- `profile.toml`
- `release-policy.toml`

### Operationally variable files
- `validators.json`
- `bootnodes.json`
- `certificate.json`
- `genesis.v1.sha256`
- in some cases `genesis.v1.json`

## Security Rule

Environment bundles must not be partially updated in a way that breaks identity consistency.

If any of the following drift apart:
- `chain_id`
- `network_id`
- `network_serial`
- manifest references
- genesis hash bindings

the environment must be treated as invalid until reconciled.
