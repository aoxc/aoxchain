# Genesis and Identity Consistency Checklist

## Purpose

Define a deterministic operator checklist to ensure network identity is consistent from policy registry to genesis/runtime artifacts.

## Required Identity Tuple

The tuple below must remain consistent for a target environment:

- `chain_id`
- `network_id`
- `network_serial`

## Verification Order

For each environment (`mainnet`, `testnet`, `validation`, `localnet`, etc.), verify in this order:

1. `configs/registry/network-registry.toml`
2. `configs/environments/<env>/release-policy.toml`
3. `configs/environments/<env>/profile.toml`
4. `configs/environments/<env>/genesis.v1.json`
5. operator bundle (`configs/aoxhub/<env>.toml` if applicable)

If any tuple field mismatches, treat the environment as invalid and fail closed.

## Mandatory Controls

- `allow_chain_id_override = false`
- `allow_network_id_override = false`
- `allow_manifest_identity_override = false`
- registry duplicate protections enabled

## Quick Command Workflow

```bash
# 1) Parse critical policy files
python - <<'PY'
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
paths = [
    'configs/registry/network-registry.toml',
    'configs/version-policy.toml',
    'configs/environments/mainnet/release-policy.toml',
    'configs/environments/testnet/release-policy.toml',
]
for path in paths:
    with open(path, 'rb') as f:
        tomllib.load(f)
print('OK: policy files parsed')
PY

# 2) Run bundle-level validation
python3 scripts/validate_environment_bundle.py

# 3) Enforce fail-closed tuple/hash consistency gate
python3 scripts/validation/network_identity_gate.py
```

## Pre-Promotion Gate

Before promotion to externally consumed testnet/mainnet bundles:

1. confirm tuple equality across registry/release-policy/profile/genesis;
2. confirm genesis hash and manifest references are synchronized;
3. confirm no override flag has been relaxed;
4. retain evidence under `artifacts/` with reproducible command logs.

## Layer and Role Expansion Check

If adding a new role/layer/plane, also confirm:

- topology files include explicit activation policy,
- chain identity remains unchanged unless intentional migration is approved,
- migration notes define compatibility impact and rollback steps.
