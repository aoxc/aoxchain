# AOXC Mainnet Environment

This directory contains the canonical public mainnet environment bundle for AOXC.

## Canonical Identity

- Chain name: `AOXC AKDENIZ`
- Environment: `mainnet`
- Network class: `public_mainnet`
- Network serial: `2626-001`
- Chain ID: `2626000001`
- Network ID: `aoxc-mainnet-2626-001`

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

The mainnet bundle must remain consistent with:
- `configs/registry/network-registry.toml`
- `configs/registry/binary-compatibility.toml`

Identity mismatches across these files must be treated as release blockers.
