# AOXC Configurations (Production-Grade)

This directory is the root configuration surface for AOXC runtime deployments.
It provides complete, environment-specific configuration coverage for:

- **Mainnet**
- **Testnet**
- **Devnet**

## Goals

- Maintain one clear, auditable source of environment configuration.
- Keep all environment files production-oriented and operationally readable.
- Ensure release teams can validate compatibility before rollout.

## Core Files

- `mainnet.toml` — canonical public mainnet node defaults.
- `testnet.toml` — canonical public testnet node defaults.
- `devnet.toml` — canonical development network defaults.
- `network-matrix.toml` — environment matrix with security/runtime expectations.

## Environment Compatibility Model

A configuration set is treated as **100% complete** when all are true:

1. Mainnet/Testnet/Devnet files exist and parse as valid TOML.
2. Each environment has chain identity, P2P, RPC, and security profile fields.
3. Security mode is explicit (no implicit defaults in production).
4. Peer lists and endpoint values are non-empty and environment-correct.

## Recommended Validation Commands

```bash
python - <<'PY'
import tomllib
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
```

## Operational Notes

- Keep environment differences intentional and documented in `network-matrix.toml`.
- Apply stricter security posture on mainnet than test/dev environments.
- Treat this folder as release-critical; all modifications should be reviewed.
