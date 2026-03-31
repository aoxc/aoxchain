# README.md

> Scope: `configs/environments/testnet`

## Purpose
Contains genesis, validator, profile, release-policy, and operational metadata files for testnet.

## Contents at a glance
- Files in this directory fix testnet identity (`chain_id`, `network_id`, genesis hash).
- `validators.json` references a minimum three-validator topology.
- `bootnodes.json` defines seed/bootnode peer discovery entry points.
- `network-metadata.json` publishes user-facing metadata such as public RPC, explorer, and faucet endpoints.
- Any change should be evaluated together with its testing and compatibility impact.

## Quick validation

```bash
scripts/validation/persistent_testnet_gate.sh
```

## CI-equivalent readiness gate

Use the repository single-command gate before requesting testnet promotion:

```bash
make testnet-readiness-gate
```

This command executes formatting, clippy, `aoxcai` tests, integration tests package (`tests`), and the persistent testnet bundle checks.

## Minimal controlled-use operator validation

Before controlled testnet use, operators should verify:

1. `make testnet-readiness-gate` is green on the exact branch/commit to deploy.
2. `make runtime-source-check AOXC_NETWORK_KIND=testnet` passes on the deployment host.
3. `make runtime-verify AOXC_NETWORK_KIND=testnet` passes after installation.
4. `make runtime-fingerprint AOXC_NETWORK_KIND=testnet` matches expected rollout evidence.
