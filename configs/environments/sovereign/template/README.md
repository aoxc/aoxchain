# AOXC Sovereign Template Environment

This directory contains the canonical sovereign private network template bundle for AOXC.

## Canonical Template Identity

- Chain name: `AOXC SOVEREIGN PRIVATE TEMPLATE`
- Environment: `sovereign-template`
- Network class: `sovereign_private`
- Network serial: `2626-150`
- Chain ID: `2626100001`
- Network ID: `aoxc-sovereign-private-2626-150`

## Purpose

This template exists to support future sovereign AOXC private network deployments using the same binary and identity policy model as the public AOXC family.

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

This template must remain consistent with:
- `configs/registry/network-registry.toml`
- `configs/registry/binary-compatibility.toml`

Template reuse for real deployments must regenerate environment identity rather than copying template identity unchanged.
