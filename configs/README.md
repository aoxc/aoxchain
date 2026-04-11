# AOXC Configurations (Single-System Runtime Source)

This directory is the canonical configuration surface for AOXC runtime source
material. The operational model is **single-system runtime code** with
**network identity selected by configuration**.

## Goals

- Keep one runtime lifecycle path in scripts/Make targets.
- Select network identity (`mainnet`/`testnet`/`devnet`/`validation`/`localnet`) via config, not via divergent operational scripts.
- Keep source material auditable and release-ready.

## Canonical Structure

- `environments/<network-kind>/` — canonical runtime source bundle.
- `registry/network-registry.toml` — canonical identity derivation policy.
- `version-policy.toml` — repository-level version governance contract.
- `network-matrix.toml` — environment matrix with security/runtime expectations.
- `quantum-resilience-policy.toml` — shared cryptographic transition and post-quantum policy baseline.
- `mainnet.toml`, `testnet.toml`, `devnet.toml` — canonical compatibility presets with advanced structured sections.

## Single-System Selection

Use one code path and set:

```bash
AOXC_NETWORK_KIND=mainnet   # or testnet/devnet/localnet/validation
```

`Makefile` and `scripts/runtime_daemon.sh` resolve runtime source from:

```text
configs/environments/${AOXC_NETWORK_KIND}/
```

Genesis metadata remains the source of truth for environment class (for example,
`environment = "mainnet"` in `genesis.v1.json`).

## Identity Source of Truth

Identity tuple fields are policy-controlled and must stay synchronized:

- `chain_id` (machine identifier),
- `network_id` (human-readable canonical identifier),
- `network_serial` (registry serial key).

Required authority order:

1. `configs/registry/network-registry.toml`
2. `configs/environments/<env>/release-policy.toml`
3. `configs/environments/<env>/profile.toml`
4. `configs/environments/<env>/genesis.v1.json`

Manual identity overrides are not acceptable for governed environments.

## Node Naming Baseline

Environment node labels should follow:

```text
<env>-<role>-<ordinal>
```

Examples:

- `mainnet-validator-01`
- `testnet-rpc-02`
- `validation-sentry-01`

If legacy fixture names are retained, document them explicitly as fixture-only and
non-authoritative for production naming policy.

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
    'configs/version-policy.toml',
    'configs/quantum-resilience-policy.toml',
    'configs/registry/network-registry.toml',
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

## Advanced Topology Templates

The `configs/topology/` folder defines a full-role, multi-plane blueprint for staged activation:

- `full-role-topology.toml` — complete role inventory with policy-gated activation flags.
- `socket-matrix.toml` — explicit role-to-role transport allowances on control/consensus/data/service planes.
- `consensus-policy.toml` — advanced consensus hardening, crypto-agility, and governance multisig policy template.

These templates are intentionally definition-first. They should be adapted per environment before activation in production-like deployments.
