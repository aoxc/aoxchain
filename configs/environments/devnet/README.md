# AOXC Devnet Environment

This directory contains the canonical development environment bundle for AOXC.

## Canonical Identity

- Chain name: `AOXC KIVILCIM`
- Environment: `devnet`
- Network class: `devnet`
- Network serial: `2626-003`
- Chain ID: `2626020001`
- Network ID: `aoxc-devnet-2626-003`

## Purpose

This environment exists to support controlled AOXC development, experimental integration work, and non-production feature validation before promotion into stricter validation or public test environments.

## Expected Bundle Growth

At minimum, this environment should define:
- `manifest.v1.json`
- `profile.toml`
- `release-policy.toml`

As the environment becomes operationally active, it should also include:
- `genesis.v1.json`
- `genesis.v1.sha256`
- `validators.json`
- `bootnodes.json`
- `certificate.json`

## Rule

The devnet bundle must remain consistent with:
- `configs/registry/network-registry.toml`
- `configs/registry/binary-compatibility.toml`

Identity mismatches across these files must be treated as environment inconsistencies.
