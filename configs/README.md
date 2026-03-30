# AOXC Configurations (Single-System Runtime Source)

This directory is the canonical configuration surface for AOXC runtime source
material. The operational model is **single-system runtime code** with
**network identity selected by configuration**.

## Goals

- Keep one runtime lifecycle path in scripts/Make targets.
- Select network identity (`mainnet`/`testnet`/`devnet`) via config, not via
  divergent operational scripts.
- Keep source material auditable and release-ready.

## Canonical Structure

- `environments/<network-kind>/` — canonical runtime source bundle.
- `network-matrix.toml` — environment matrix with security/runtime expectations.
- `mainnet.toml`, `testnet.toml`, `devnet.toml` — legacy compatibility presets.

## Single-System Selection

Use one code path and set:

```bash
AOXC_NETWORK_KIND=mainnet   # or testnet/devnet
```

`Makefile` and `scripts/runtime_daemon.sh` resolve runtime source from:

```text
configs/environments/${AOXC_NETWORK_KIND}/
```

Genesis metadata remains the source of truth for chain type (for example,
`environment = "mainnet"` in `genesis.v1.json`).

## Single-System Selection

Use one code path and set:

```bash
AOXC_NETWORK_KIND=mainnet   # or testnet/devnet/localnet/validation
```

`Makefile` and `scripts/runtime_daemon.sh` resolve runtime source from:

```text
configs/environments/${AOXC_NETWORK_KIND}/
```

Genesis metadata remains the source of truth for chain type (for example,
`environment = "mainnet"` in `genesis.v1.json`).

## Environment Compatibility Model

A selected runtime bundle is treated as **100% complete** when all are true:

1. Required files exist under `configs/environments/${AOXC_NETWORK_KIND}/`.
2. The selected bundle has chain identity, P2P, RPC, and security profile fields.
3. Security mode is explicit (no implicit defaults in production).
4. AOXHub mapping (if present for that network kind) is aligned with the selected canonical bundle root.

## Recommended Validation Commands

```bash
python - <<'PY'
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
for path in [
    'configs/mainnet.toml',
    'configs/testnet.toml',
    'configs/devnet.toml',
    'configs/network-matrix.toml',
]:
    with open(path, 'rb') as f:
        tomllib.load(f)
print('OK')
PY
python3 scripts/validate_environment_bundle.py
```

## Operational Notes

- Keep environment differences intentional and documented in `network-matrix.toml`.
- Apply stricter security posture on mainnet than test/dev environments.
- Treat this folder as release-critical; all modifications should be reviewed.
