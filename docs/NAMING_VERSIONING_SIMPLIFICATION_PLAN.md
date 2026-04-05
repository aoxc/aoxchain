# AOXChain Naming and Versioning Simplification Plan

## Objective

Reduce naming and versioning ambiguity across repository docs, runtime configs, and operator workflows while preserving deterministic identity controls.

## Problem Summary

Current surfaces mix multiple identity forms and version axes:

- repository/workspace versioning (`configs/version-policy.toml`);
- consensus/profile line naming (`AOXC-Q-*` surfaces);
- network identity (`chain_id`, `network_id`, `network_serial`);
- environment-local node fixture identifiers.

The model is technically strong but operationally noisy when these are discussed with one shared term (for example, "version").

## Recommended Canonical Model

Treat these as separate, non-interchangeable dimensions:

1. **Brand/Asset Identity**
   - Chain brand: `AOXChain`
   - Native asset ticker: `AOXC`
   - Never encode release line or cryptographic profile into the brand or ticker.

2. **Protocol/Release Line**
   - Use a formal line such as `AOXC-QTR-V1` as a release-line label only.
   - Bind this label to explicit compatibility metadata in `configs/version-policy.toml` and release notes.
   - Do not use release-line labels as `chain_id` or `network_id` values.

3. **Network Identity**
   - Keep `chain_id` numeric and registry-derived.
   - Keep `network_id` human-readable and class/serial based.
   - Enforce immutability through release policy and registry checks.

4. **Execution/Crypto Profile Version**
   - Track cryptographic profile activation through profile policy files and consensus topology overlays.
   - Keep profile transitions auditable and independent from brand/network naming.

## Mainnet/Testnet Identity Guidance

Prefer one deterministic naming grammar for all environments:

- `network_id`: `aoxc-<network-class>-<family-serial>`
- `chain_id`: registry-derived numeric ID (`FFFFCCNNNN`)
- `display_name`: governance/branding string (human-facing only)

Suggested operator-facing naming examples:

- Mainnet display: `AOXC Mainnet (AKDENIZ)`
- Testnet display: `AOXC Testnet (PUSULA)`
- Validation display: `AOXC Validation (MIZAN)`

This keeps machine identity stable while preserving brand flexibility.

## Node Naming Guidance

Use role-first, environment-scoped node naming to reduce ambiguity:

- Pattern: `<env>-<role>-<ordinal>`
- Examples: `mainnet-validator-01`, `testnet-rpc-02`, `validation-sentry-01`

Avoid mixing fixture names, mythology names, and role names in the same environment unless documented as non-production fixtures.

## Repository Documentation Cleanup Proposal

Normalize root and config surfaces around one explicit map:

1. `READ.md`: strict invariant language (non-negotiable rules).
2. `ARCHITECTURE.md`: boundary and dependency model only.
3. `configs/registry/network-registry.toml`: canonical identity derivation authority.
4. `configs/version-policy.toml`: compatibility and release-line authority.
5. `configs/environments/<env>/release-policy.toml`: per-environment enforcement profile.

Add one short glossary section to root `README.md` covering:

- brand vs ticker,
- release-line vs workspace version,
- chain ID vs network ID,
- profile version vs software version.

## Migration Strategy (Low Risk)

1. **Freeze identity policy**
   - No manual overrides for `chain_id` and `network_id`.
2. **Introduce glossary and naming policy document**
   - Keep behavior unchanged; improve interpretation first.
3. **Standardize node naming in non-production fixtures**
   - Migrate fixture files to role-first names.
4. **Gate enforcement**
   - Add CI checks for naming grammar and identity consistency.

## Decision Rule for Your Specific Question

If you want to use `AOXC-QTR-V1` and keep versioning mostly on GitHub:

- **Yes** for release communication and tag strategy.
- **No** as the only versioning system.

You still need in-repo machine-readable version and identity policy files for deterministic runtime validation, operator safety, and auditability.
