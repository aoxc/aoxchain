# AOXC Validation Environment

This directory contains the canonical validation environment bundle for AOXC.

## Canonical Identity

- Chain name: `AOXC MIZAN`
- Environment: `validation`
- Network class: `validation`
- Network serial: `2626-004`
- Chain ID: `2626030001`
- Network ID: `aoxc-validation-2626-004`

## Purpose

This environment exists to support pre-production verification, reproducibility checks, and controlled promotion readiness exercises.

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

The validation bundle must remain consistent with:
- `configs/registry/network-registry.toml`
- `configs/registry/binary-compatibility.toml`

This environment must be suitable for measurable release-readiness validation.
