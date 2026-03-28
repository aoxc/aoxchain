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
