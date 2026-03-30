# AOXHub Environment Profiles

> Scope: `configs/aoxhub`

## Purpose
This directory contains canonical AOXHub environment profile mappings for the
single-system AOXC runtime model.

## Profiles
- `mainnet.toml`
- `testnet.toml`
- `localnet.toml`
- `validation.toml`

Each profile binds AOXHub to one canonical runtime bundle under
`configs/environments/<network-kind>/`.

## Operational Contract
- AOXHub must read identity from the mapped manifest/genesis bundle.
- AOXHub must validate genesis checksum before runtime launch.
- AOXHub must fail closed on missing bundle files.
- AOXHub profile identity fields must match canonical environment identity
  fields (`environment`, `network_id`, `chain_id`).

## Change Policy
- Treat these files as release-critical configuration surfaces.
- Validate updates with `python3 scripts/validate_environment_bundle.py`.
