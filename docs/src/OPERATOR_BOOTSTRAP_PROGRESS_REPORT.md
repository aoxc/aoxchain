# AOXC Operator Bootstrap Progress Report

## Executive Summary

This report captures the operator-plane bootstrap, validation, and environment-isolation work completed for AOXC across the `devnet`, `testnet`, and `mainnet` profiles.

The objective of this work was to establish a deterministic, profile-aligned, operator bootstrap surface with clear separation between environments and verifiable persistence across configuration, genesis, key material, runtime state, telemetry, and ledger state.

At the conclusion of this work:

- all previously failing `aoxcmd` tests were remediated,
- profile-aware bootstrap behavior was validated,
- `devnet`, `testnet`, and `mainnet` were isolated into separate operator homes,
- key / config / genesis / runtime alignment was confirmed per profile,
- runtime node state was enriched with operator key material,
- deterministic local block production succeeded for all three profiles,
- file-backed ledger state was initialized and persisted successfully for all three profiles.

---

## Initial Problems Identified

The work began with multiple failures and inconsistencies in the AOXC operator-plane surface.

### 1. Missing readiness and profile artifacts
The repository initially exhibited failures related to missing or unresolved readiness/profile artifacts, especially around:

- embedded network profile comparisons,
- AOXHub profile comparisons,
- readiness score expectations,
- readiness markdown report generation.

### 2. Profile initialization inconsistency
A profile-aware configuration initialization test indicated that `devnet` initialization was falling back to `validation` under certain test conditions. The root cause was determined to be test-environment state leakage rather than canonical profile parsing failure.

### 3. `--json-logs` flag ineffective
`config-init --profile <profile> --json-logs` completed successfully, but the resulting persisted `settings.json` still contained:

```json
"logging": {
  "json": false
}
