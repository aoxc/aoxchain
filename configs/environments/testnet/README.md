# AOXC Testnet Environment

This directory contains the canonical public testnet environment bundle for AOXC.

## Canonical Identity

- Chain name: `AOXC PUSULA`
- Environment: `testnet`
- Network class: `public_testnet`
- Network serial: `2626-002`
- Chain ID: `2626010001`
- Network ID: `aoxc-testnet-2626-002`

## Authoritative Files

### Stable identity anchor
- `manifest.v1.json`

### Genesis anchor
- `genesis.v1.json`
- `genesis.v1.sha256`

### Operational network data
- `validators.json`
- `bootnodes.json`
- `certificate.json`

### Policy controls
- `profile.toml`
- `release-policy.toml`

## Rule

The testnet bundle must remain consistent with:
- `configs/registry/network-registry.toml`
- `configs/registry/binary-compatibility.toml`

Identity mismatches across these files must be treated as validation blockers.
