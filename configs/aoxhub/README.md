# AOXC Hub Environment Mapping

This directory contains AOXC Hub environment mapping files.

These files are application-facing configuration surfaces used by the AOXC desktop and control-plane layers to resolve, display, validate, and launch AOXC environment bundles.

## Purpose

The files under this directory do not define canonical network identity on their own.

Instead, they map AOXC Hub runtime behavior to the authoritative configuration layers located under:

- `configs/registry/`
- `configs/environments/`

## Authoritative Rule

The canonical source of truth for AOXC network identity remains:

1. `configs/registry/network-registry.toml`
2. `configs/registry/binary-compatibility.toml`
3. `configs/environments/*/manifest.v1.json`

The files under `configs/aoxhub/` must remain consistent with those authoritative records.

## Files

### `mainnet.toml`
Maps AOXC Hub to the canonical public mainnet environment:
- `AOXC AKDENIZ`
- `2626-001`
- `2626000001`
- `aoxc-mainnet-2626-001`

### `testnet.toml`
Maps AOXC Hub to the canonical public testnet environment:
- `AOXC PUSULA`
- `2626-002`
- `2626010001`
- `aoxc-testnet-2626-002`

### `validation.toml`
Maps AOXC Hub to the canonical validation environment:
- `AOXC MIZAN`
- `2626-004`
- `2626030001`
- `aoxc-validation-2626-004`

### `localnet.toml`
Maps AOXC Hub to the canonical deterministic local multi-node environment:
- `AOXC LOCALNET ATLAS`
- `2626-900`
- `2626900001`
- `aoxc-localnet-2626-900`

## Stability Model

These files are controlled configuration surfaces.

They may change when:
- AOXC Hub launch requirements evolve,
- path mappings change,
- runtime validation expectations are tightened,
- UI or operator metadata is improved.

They must not silently diverge from registry or manifest identity.

## Security Rule

AOXC Hub mapping files must not override canonical identity values by policy.

If an identity mismatch is detected between:
- hub mapping,
- registry policy,
- environment manifest,

the mismatch must be treated as an operator-visible error and a launch blocker.
