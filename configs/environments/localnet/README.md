# AOXC Localnet Environment

This directory contains the deterministic local multi-node AOXC environment bundle.

## Canonical Identity

- Chain name: `AOXC LOCALNET ATLAS`
- Environment: `localnet`
- Network class: `localnet`
- Network serial: `2626-900`
- Chain ID: `2626900001`
- Network ID: `aoxc-localnet-2626-900`

## Purpose

This environment supports:
- local multi-node testing,
- deterministic operator workflows,
- launch validation,
- hub and CLI integration checks.

## Authoritative Files

### Stable identity anchor
- `manifest.v1.json`

### Genesis anchor
- `genesis.v1.json`
- `genesis.v1.sha256`

### Operational network data
- `accounts.json`
- `validators.json`
- `bootnodes.json`
- `certificate.json`

### Policy controls
- `profile.toml`
- `release-policy.toml`

### Local orchestration surfaces
- `nodes/`
- `homes/`
- `launch-localnet.sh`
- `hosts.txt.example`

## Rule

The localnet bundle must remain consistent with:
- `configs/registry/network-registry.toml`
- `configs/registry/binary-compatibility.toml`

Node-specific local files under `homes/` are not registry authority artifacts and must not be treated as canonical policy inputs.
