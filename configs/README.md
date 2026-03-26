# AOXC Configuration System

This directory contains the canonical configuration, identity, and release-control surfaces for AOXC environments.

The structure is intentionally designed for a **single-binary, multi-network** operating model. In this model, the node binary remains environment-agnostic, while network identity is derived from registry policy, environment manifests, and genesis bundles.

## Directory Roles

### `registry/`
This directory is the authority layer for network identity and binary compatibility policy.

It defines:
- canonical AOXC family identity rules,
- `chain_id` derivation policy,
- `network_serial` policy,
- `network_id` naming policy,
- binary compatibility requirements,
- cross-environment governance constraints.

The files under `registry/` must be treated as long-lived policy artifacts. They are not routine deployment toggles.

### `environments/`
This directory contains environment-scoped network bundles.

Each environment directory defines the canonical files required to bootstrap and validate a specific AOXC network line, including:
- manifest,
- genesis,
- genesis hash,
- validators,
- bootnodes,
- profile,
- release policy,
- certificate material.

### `aoxhub/`
This directory contains application-facing environment mapping files used by the AOXC hub and control-plane surfaces.

These files must remain consistent with the authoritative records under `registry/` and `environments/`.

## Identity Model

AOXC identity is governed through the following layered model:

1. `registry/network-registry.toml` defines canonical identity rules.
2. `registry/binary-compatibility.toml` defines which binaries may operate on which environment bundles.
3. `environments/*/manifest.v1.json` defines the canonical identity of each environment.
4. `environments/*/genesis.v1.json` defines the genesis configuration/state input.
5. `genesis.v1.sha256` binds the manifest to a specific genesis artifact.

## Canonical Public Baseline

For the current baseline:

- Public mainnet name: `AOXC AKDENIZ`
- Public testnet name: `AOXC PUSULA`
- Validation network name: `AOXC MIZAN`
- Local deterministic operator network name: `AOXC LOCALNET ATLAS`

## Stability Model

### Long-lived policy files
These files should change rarely and only with explicit governance and release evidence:

- `registry/network-registry.toml`
- `registry/binary-compatibility.toml`
- `environments/*/manifest.v1.json`

### Controlled configuration files
These files may change when the environment policy evolves:

- `environments/*/profile.toml`
- `environments/*/release-policy.toml`
- `aoxhub/*.toml`

### Operational data files
These files may change as validators, bootnodes, certificates, or finalized genesis inputs evolve:

- `environments/*/validators.json`
- `environments/*/bootnodes.json`
- `environments/*/certificate.json`
- `environments/*/genesis.v1.sha256`
- in some cases `environments/*/genesis.v1.json`

### Local-only or fixture-oriented files
These files are not governance anchors and must not be treated as canonical registry inputs:

- `environments/localnet/homes/**`
- `environments/localnet/**/test-node-seed.hex`
- local launch helper artifacts

## Release Rule

Any release-impacting change to:
- registry policy,
- binary compatibility policy,
- environment manifest identity,
- genesis identity assumptions,
- release policy files,

must be reflected in the relevant audit and release documentation before promotion.

## Security Rule

Private, sensitive, or non-public identity material must not be committed unless it is explicitly intended as deterministic public fixture data and clearly documented as such.
